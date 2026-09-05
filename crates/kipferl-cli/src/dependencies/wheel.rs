//! Wheel metadata and extraction, without installation hooks or Python execution.
use std::collections::BTreeMap;
use std::io::{self, Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};

use super::{Artifact, invalid};

const MAX_FILES: usize = 1024;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_UNPACKED_BYTES: u64 = 32 * 1024 * 1024;

pub(super) struct Wheel {
    pub(super) files: BTreeMap<String, Vec<u8>>,
    pub(super) requirements: Vec<String>,
}

pub(super) fn inspect(bytes: &[u8], artifact: &Artifact) -> io::Result<Wheel> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(invalid)?;
    if archive.len() > MAX_FILES {
        return Err(invalid("wheel has more than 1024 entries"));
    }
    let mut files = BTreeMap::new();
    let mut paths = std::collections::BTreeSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(invalid)?;
        let name = entry.name().trim_end_matches('/').to_owned();
        safe_path(&name)?;
        if !paths.insert(name.to_lowercase()) {
            return Err(invalid(format!(
                "wheel contains duplicate/case-colliding path '{name}'"
            )));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| !matches!(mode & 0o170_000, 0 | 0o040_000 | 0o100_000))
        {
            return Err(invalid(format!(
                "wheel contains a symlink or special file: {name}"
            )));
        }
        if entry.is_dir() {
            continue;
        }
        if native_file(&name) || super::extension(&name, "pth") || super::extension(&name, "pyc") {
            return Err(invalid(format!(
                "wheel contains unsupported native/installation code: {name}"
            )));
        }
        // Wheel .data schemes include scripts and external installation paths; never execute or relocate them implicitly.
        if name
            .split('/')
            .next()
            .is_some_and(|part| super::extension(part, "data"))
        {
            return Err(invalid(format!(
                "wheel .data installation schemes are not supported: {name}"
            )));
        }
        if entry.size() > MAX_FILE_BYTES
            || (super::extension(&name, "py") && entry.size() > 1024 * 1024)
        {
            return Err(invalid(
                "wheel exceeds extraction limits (4 MiB/file, 32 MiB total)",
            ));
        }
        let mut content = Vec::new();
        entry
            .by_ref()
            .take(MAX_FILE_BYTES.saturating_add(1))
            .read_to_end(&mut content)?;
        if u64::try_from(content.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES
            || (super::extension(&name, "py") && content.len() > 1024 * 1024)
        {
            return Err(invalid("inflated wheel entry exceeds file limit"));
        }
        total = total
            .checked_add(u64::try_from(content.len()).map_err(invalid)?)
            .ok_or_else(|| invalid("wheel size overflow"))?;
        if total > MAX_UNPACKED_BYTES {
            return Err(invalid("wheel exceeds 32 MiB inflated total"));
        }
        files.insert(name, content);
    }
    // Reject a/b alongside regular file a before attempting filesystem writes.
    for name in files.keys() {
        for ancestor in Path::new(name).ancestors().skip(1) {
            if ancestor
                .to_str()
                .is_some_and(|parent| files.contains_key(parent))
            {
                return Err(invalid(format!("wheel file/directory collision: {name}")));
            }
        }
    }
    metadata(files, artifact)
}

fn metadata(files: BTreeMap<String, Vec<u8>>, artifact: &Artifact) -> io::Result<Wheel> {
    let metadata: Vec<_> = files
        .iter()
        .filter(|(path, _)| path.ends_with(".dist-info/METADATA"))
        .collect();
    let [(metadata_path, metadata_bytes)] = metadata.as_slice() else {
        return Err(invalid("wheel must have exactly one .dist-info/METADATA"));
    };
    if metadata_path.matches('/').count() != 1 {
        return Err(invalid("wheel metadata must be at its root"));
    }
    validate_identity(metadata_path, artifact)?;
    let metadata = headers(metadata_bytes)?;
    let name = one_header(&metadata, "Name")?
        .parse::<pep508_rs::PackageName>()
        .map_err(invalid)?;
    let version = one_header(&metadata, "Version")?
        .parse::<pep508_rs::pep440_rs::Version>()
        .map_err(invalid)?;
    if name.as_ref() != artifact.name
        || version
            != artifact
                .version
                .parse::<pep508_rs::pep440_rs::Version>()
                .map_err(invalid)?
    {
        return Err(invalid(
            "wheel METADATA name/version does not match resolved artifact",
        ));
    }
    if let Some(required) = metadata.get("requires-python") {
        if required.len() != 1 {
            return Err(invalid("duplicate Requires-Python metadata"));
        }
        for value in required {
            let specifiers = value
                .parse::<pep508_rs::pep440_rs::VersionSpecifiers>()
                .map_err(invalid)?;
            let supported = "3.11.0".parse().map_err(invalid)?;
            if !specifiers.contains(&supported) {
                return Err(invalid(format!(
                    "{} requires Python {value}; Kipferl advertises Python 3.11.0",
                    artifact.name
                )));
            }
        }
    }
    let wheel_name = metadata_path
        .strip_suffix("METADATA")
        .ok_or_else(|| invalid("invalid metadata path"))?
        .to_owned()
        + "WHEEL";
    let wheel_metadata = headers(
        files
            .get(&wheel_name)
            .ok_or_else(|| invalid("wheel has no WHEEL metadata"))?,
    )?;
    if one_header(&wheel_metadata, "Wheel-Version")? != "1.0" {
        return Err(invalid("only Wheel-Version 1.0 is supported"));
    }
    if one_header(&wheel_metadata, "Root-Is-Purelib")? != "true" {
        return Err(invalid("wheel Root-Is-Purelib must be true"));
    }
    let tags = wheel_metadata
        .get("tag")
        .ok_or_else(|| invalid("wheel has no Tag metadata"))?;
    if !tags
        .iter()
        .any(|tag| super::registry::pure_wheel(&format!("x-0-{tag}.whl")))
    {
        return Err(invalid("WHEEL metadata has no py3-none-any tag"));
    }
    let requirements = metadata.get("requires-dist").cloned().unwrap_or_default();
    Ok(Wheel {
        files,
        requirements,
    })
}

fn validate_identity(metadata_path: &str, artifact: &Artifact) -> io::Result<()> {
    let info = metadata_path
        .strip_suffix(".dist-info/METADATA")
        .ok_or_else(|| invalid("invalid dist-info path"))?;
    let (name, version) = info
        .rsplit_once('-')
        .ok_or_else(|| invalid("dist-info directory has no version"))?;
    let name = name.parse::<pep508_rs::PackageName>().map_err(invalid)?;
    let version = version
        .parse::<pep508_rs::pep440_rs::Version>()
        .map_err(invalid)?;
    let mut components = artifact.filename.split('-');
    let file_name = components
        .next()
        .ok_or_else(|| invalid("wheel filename has no name"))?
        .parse::<pep508_rs::PackageName>()
        .map_err(invalid)?;
    let file_version = components
        .next()
        .ok_or_else(|| invalid("wheel filename has no version"))?
        .parse::<pep508_rs::pep440_rs::Version>()
        .map_err(invalid)?;
    if name.as_ref() != artifact.name
        || name != file_name
        || version
            != artifact
                .version
                .parse::<pep508_rs::pep440_rs::Version>()
                .map_err(invalid)?
        || version != file_version
    {
        return Err(invalid(
            "wheel filename/dist-info identity differs from resolved package",
        ));
    }
    Ok(())
}

fn headers(bytes: &[u8]) -> io::Result<BTreeMap<String, Vec<String>>> {
    let text = std::str::from_utf8(bytes).map_err(invalid)?;
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut last = String::new();
    for line in text.lines() {
        if line.is_empty() {
            break;
        }
        if line.starts_with([' ', '\t']) {
            let value = result
                .get_mut(&last)
                .and_then(|values| values.last_mut())
                .ok_or_else(|| invalid("invalid metadata continuation"))?;
            value.push(' ');
            value.push_str(line.trim());
        } else {
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| invalid("invalid wheel metadata header"))?;
            last = key.to_ascii_lowercase();
            result
                .entry(last.clone())
                .or_default()
                .push(value.trim().to_owned());
        }
    }
    Ok(result)
}

fn one_header<'a>(headers: &'a BTreeMap<String, Vec<String>>, name: &str) -> io::Result<&'a str> {
    match headers.get(&name.to_ascii_lowercase()).map(Vec::as_slice) {
        Some([value]) => Ok(value),
        _ => Err(invalid(format!(
            "wheel must have exactly one {name} header"
        ))),
    }
}

pub(super) fn safe_path(value: &str) -> io::Result<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > 1024
        || value.contains(['\\', ':'])
        || value.chars().any(char::is_control)
        || value.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.ends_with([' ', '.'])
        })
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(invalid(format!("unsafe wheel path: {value:?}")));
    }
    Ok(path.to_owned())
}

fn native_file(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    [".so", ".pyd", ".dll", ".dylib", ".exe", ".a", ".o"]
        .iter()
        .any(|suffix| path.ends_with(suffix))
        || path.contains(".so.")
}

pub(super) fn extract(wheel: &Wheel, destination: &Path) -> io::Result<()> {
    for (name, bytes) in &wheel.files {
        let path = destination.join(safe_path(name)?);
        let parent = path
            .parent()
            .ok_or_else(|| invalid("wheel path has no parent"))?;
        std::fs::create_dir_all(parent)?;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        output.write_all(bytes)?;
    }
    Ok(())
}
