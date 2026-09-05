//! Bounded `PyPI` JSON reads and content-addressed wheel downloads.
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use pep508_rs::pep440_rs::Version;
use serde_json::Value;

use super::{Artifact, invalid, sha256};

pub(super) const MAX_WHEEL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;

pub(super) struct Registry {
    cache: PathBuf,
    offline: bool,
    projects: BTreeMap<String, Value>,
}

impl Registry {
    pub(super) const fn new(cache: PathBuf, offline: bool) -> Self {
        Self {
            cache,
            offline,
            projects: BTreeMap::new(),
        }
    }

    pub(super) fn candidates(&mut self, name: &str) -> io::Result<Vec<Artifact>> {
        if !self.projects.contains_key(name) {
            if self.offline {
                return Err(invalid(
                    "dependency resolution needs PyPI; use sync --locked --offline with an existing lock",
                ));
            }
            let bytes = download(
                &format!("https://pypi.org/pypi/{name}/json"),
                MAX_INDEX_BYTES,
            )?;
            self.projects.insert(
                name.to_owned(),
                serde_json::from_slice(&bytes).map_err(invalid)?,
            );
        }
        let project = self
            .projects
            .get(name)
            .ok_or_else(|| invalid("missing PyPI project"))?;
        let releases = project
            .get("releases")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid("PyPI response has no releases object"))?;
        let mut candidates = Vec::new();
        for (version_text, files) in releases {
            let Ok(version) = version_text.parse::<Version>() else {
                continue;
            };
            let Some(files) = files.as_array() else {
                continue;
            };
            for file in files {
                let Some(filename) = file.get("filename").and_then(Value::as_str) else {
                    continue;
                };
                if !pure_wheel(filename)
                    || file.get("yanked").and_then(Value::as_bool).unwrap_or(false)
                {
                    continue;
                }
                if let Some(text) = file.get("requires_python").and_then(Value::as_str)
                    && !text.is_empty()
                {
                    let constraint = text
                        .parse::<pep508_rs::pep440_rs::VersionSpecifiers>()
                        .map_err(invalid)?;
                    if !constraint.contains(&"3.11.0".parse().map_err(invalid)?) {
                        continue;
                    }
                }
                let Some(url) = file.get("url").and_then(Value::as_str) else {
                    continue;
                };
                let Some(hash) = file
                    .get("digests")
                    .and_then(|value| value.get("sha256"))
                    .and_then(Value::as_str)
                else {
                    continue;
                };
                if !valid_hash(hash) || !artifact_url(url) {
                    continue;
                }
                if file
                    .get("size")
                    .and_then(Value::as_u64)
                    .is_some_and(|size| size > MAX_WHEEL_BYTES)
                {
                    continue;
                }
                candidates.push(Artifact {
                    name: name.to_owned(),
                    version: version.to_string(),
                    filename: filename.to_owned(),
                    url: url.to_owned(),
                    sha256: hash.to_owned(),
                    requires_dist: Vec::new(),
                });
            }
        }
        candidates.sort_by(|left, right| {
            let left_version = left.version.parse::<Version>().ok();
            let right_version = right.version.parse::<Version>().ok();
            right_version
                .cmp(&left_version)
                .then_with(|| left.filename.cmp(&right.filename))
        });
        Ok(candidates)
    }

    pub(super) fn wheel(&self, artifact: &Artifact) -> io::Result<Vec<u8>> {
        if !valid_hash(&artifact.sha256)
            || !artifact_url(&artifact.url)
            || !pure_wheel(&artifact.filename)
        {
            return Err(invalid(format!(
                "{}: invalid locked wheel identity",
                artifact.name
            )));
        }
        let path = self.cache.join(format!("{}.whl", artifact.sha256));
        match read_regular(&path, MAX_WHEEL_BYTES) {
            Ok(bytes) if sha256(&bytes) == artifact.sha256 => return Ok(bytes),
            Ok(_) => {
                return Err(invalid(format!(
                    "{}: cached wheel hash mismatch; remove {} and sync again",
                    artifact.name,
                    path.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if self.offline {
            return Err(invalid(format!(
                "{}=={}: wheel is missing from the offline cache; run sync --locked online once",
                artifact.name, artifact.version
            )));
        }
        let bytes = download(&artifact.url, MAX_WHEEL_BYTES)?;
        if sha256(&bytes) != artifact.sha256 {
            return Err(invalid(format!(
                "{}: downloaded wheel SHA-256 does not match PyPI/lock",
                artifact.name
            )));
        }
        crate::run_command::write_atomically(&path, &[&bytes], 0o600)?;
        Ok(bytes)
    }
}

pub(super) fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn artifact_url(url: &str) -> bool {
    url.starts_with("https://files.pythonhosted.org/packages/")
        && !url.chars().any(char::is_control)
}

pub(super) fn pure_wheel(filename: &str) -> bool {
    let Some(stem) = filename.strip_suffix(".whl") else {
        return false;
    };
    let parts: Vec<_> = stem.rsplitn(4, '-').collect();
    matches!(parts.as_slice(), ["any", "none", python, _] if python.split('.').any(|tag| tag == "py3"))
}

fn download(url: &str, maximum: u64) -> io::Result<Vec<u8>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(45)))
        .build()
        .into();
    agent
        .get(url)
        .header("User-Agent", concat!("kipferl/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|error| invalid(format!("cannot download {url}: {error}")))?
        .body_mut()
        .with_config()
        .limit(maximum)
        .read_to_vec()
        .map_err(invalid)
}

pub(super) fn read_regular(path: &Path, maximum: u64) -> io::Result<Vec<u8>> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(invalid(format!(
            "{}: expected a regular file of at most {maximum} bytes",
            path.display()
        )));
    }
    let mut bytes = Vec::new();
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > maximum {
        return Err(invalid("file exceeds size limit"));
    }
    Ok(bytes)
}

#[cfg(test)]
impl Registry {
    pub(super) const fn fixture(cache: PathBuf, projects: BTreeMap<String, Value>) -> Self {
        Self {
            cache,
            offline: true,
            projects,
        }
    }
}
