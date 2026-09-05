//! Reproducible, project-local `PyPI` wheel dependencies for the embedded runtime.
mod audit;
mod registry;
mod resolver;
mod wheel;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::package_compat::{self, Status};

static COUNTER: AtomicU64 = AtomicU64::new(0);
const MAX_LOCK: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    name: String,
    version: String,
    filename: String,
    url: String,
    sha256: String,
    requires_dist: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Lock {
    schema: u32,
    runtime_sha256: String,
    target: String,
    requirements: Vec<String>,
    allow_unverified: bool,
    packages: Vec<Artifact>,
    files: BTreeMap<String, String>,
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_string())
}

fn sha256(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn extension(path: &str, suffix: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(suffix))
}

fn runtime_hash() -> io::Result<String> {
    Ok(sha256(crate::run_command::embedded_runtime()?))
}

/// Execute dependency commands without requiring a system Python installation.
pub fn execute(
    command: &str,
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    match execute_inner(command, arguments, current_directory, stdout) {
        Ok(()) => Ok(0),
        Err(error) => {
            writeln!(stderr, "Dependency error: {error}")?;
            Ok(1)
        }
    }
}

fn execute_inner(
    command: &str,
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    if command == "deps" && arguments.first().is_some_and(|value| value == "audit") {
        return audit::show(arguments, stdout);
    }
    if arguments
        .iter()
        .any(|value| value == "--help" || value == "-h")
    {
        writeln!(
            stdout,
            "Usage: kipferl add <requirement> [--allow-unverified]\n       kipferl sync --locked [--offline]\n       kipferl deps check|list|catalog|audit\n\nPure-Python PyPI wheels only. Extras, markers, URLs, native extensions and source builds are unsupported. Compatibility is tied to exact wheel and runtime hashes. Unverified packages require explicit opt-in; known blockers always fail."
        )?;
        return Ok(());
    }
    if command == "deps" && arguments.first().is_some_and(|value| value == "catalog") {
        return show_catalog(arguments, stdout);
    }
    let config = crate::project_config::discover(current_directory)?.ok_or_else(|| {
        invalid("no kipferl.json found; create a project with kipferl new <name> first")
    })?;
    let root = config.root;
    match command {
        "add" => add(arguments, &root, stdout),
        "sync" => sync(arguments, &root, stdout),
        "deps" => match arguments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["check"] => {
                let path = validate_installation(&root)?;
                writeln!(
                    stdout,
                    "{}",
                    if path.is_some() {
                        "Dependency lock, runtime identity and all installed file hashes verified."
                    } else {
                        "No dependencies configured."
                    }
                )?;
                Ok(())
            }
            ["list"] => {
                if read_requirements(&root)?.is_empty() {
                    writeln!(stdout, "No dependencies configured.")?;
                    return Ok(());
                }
                let lock = read_lock(&root)?;
                check_lock(&root, &lock)?;
                for artifact in &lock.packages {
                    let report = package_compat::lookup(
                        &artifact.name,
                        &artifact.version,
                        &artifact.sha256,
                        &lock.runtime_sha256,
                    )?;
                    writeln!(
                        stdout,
                        "{}=={}  {}  {}",
                        artifact.name,
                        artifact.version,
                        report.status.as_str(),
                        artifact.filename
                    )?;
                }
                Ok(())
            }
            _ => Err(invalid("usage: kipferl deps check|list|catalog|audit")),
        },
        _ => Err(invalid("unknown dependency command")),
    }
}

fn show_catalog(arguments: &[String], stdout: &mut dyn Write) -> io::Result<()> {
    let catalog = package_compat::catalog()?;
    match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["catalog", "--json"] => writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&catalog).map_err(invalid)?
        ),
        ["catalog"] => {
            writeln!(
                stdout,
                "Package compatibility catalog (evidence applies only to exact wheel, runtime and target):"
            )?;
            let records = catalog
                .get("records")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| invalid("catalog records missing"))?;
            let runtime = runtime_hash()?;
            for record in records {
                let field = |key| {
                    record
                        .get(key)
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("?")
                };
                let applies = field("runtime_sha256") == runtime
                    && field("target") == crate::run_command::embedded_runtime_target();
                writeln!(
                    stdout,
                    "{}=={}  {}  {}{}",
                    field("name"),
                    field("version"),
                    field("status"),
                    field("target"),
                    if applies {
                        " (this runtime)"
                    } else {
                        " (different runtime/target)"
                    }
                )?;
                if let Some(reason) = record.get("reason").and_then(serde_json::Value::as_str) {
                    writeln!(stdout, "  {reason}")?;
                }
            }
            writeln!(
                stdout,
                "Unlisted artifacts are unverified. Use --json for complete evidence and hashes."
            )
        }
        _ => Err(invalid("usage: kipferl deps catalog [--json]")),
    }
}

fn read_config(root: &Path) -> io::Result<serde_json::Value> {
    serde_json::from_slice(&registry::read_regular(
        &root.join("kipferl.json"),
        64 * 1024,
    )?)
    .map_err(invalid)
}

fn read_requirements(root: &Path) -> io::Result<Vec<String>> {
    let config = read_config(root)?;
    match config.get("dependencies") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .map(|item| {
                let text = item
                    .as_str()
                    .ok_or_else(|| invalid("dependencies must contain requirement strings"))?;
                resolver::requirement(text)?;
                Ok(text.to_owned())
            })
            .collect(),
        _ => Err(invalid(
            "dependencies must be an array of requirement strings",
        )),
    }
}

fn add(arguments: &[String], root: &Path, stdout: &mut dyn Write) -> io::Result<()> {
    let allow = arguments.iter().any(|value| value == "--allow-unverified");
    let positional: Vec<_> = arguments
        .iter()
        .filter(|value| value.as_str() != "--allow-unverified")
        .collect();
    let [text] = positional.as_slice() else {
        return Err(invalid(
            "usage: kipferl add '<requirement>' [--allow-unverified]",
        ));
    };
    let parsed = resolver::requirement(text)?;
    let _guard = Operation::new(root)?;
    let mut requirements = read_requirements(root)?;
    requirements
        .retain(|value| resolver::requirement(value).is_ok_and(|item| item.name != parsed.name));
    requirements.push(parsed.to_string());
    requirements.sort();
    let mut registry = registry::Registry::new(root.join(".kipferl/cache"), false);
    let packages = resolver::resolve(&requirements, &mut registry)?;
    let lock = Lock {
        schema: 1,
        runtime_sha256: runtime_hash()?,
        target: crate::run_command::embedded_runtime_target().to_owned(),
        requirements: requirements.clone(),
        allow_unverified: allow,
        packages,
        files: BTreeMap::new(),
    };
    let mut config = read_config(root)?;
    config
        .as_object_mut()
        .ok_or_else(|| invalid("expected project object"))?
        .insert(
            "dependencies".to_owned(),
            serde_json::to_value(requirements).map_err(invalid)?,
        );
    install(root, lock, &registry, Some(config), stdout)
}

fn sync(arguments: &[String], root: &Path, stdout: &mut dyn Write) -> io::Result<()> {
    if !arguments.iter().any(|value| value == "--locked")
        || arguments
            .iter()
            .any(|value| !matches!(value.as_str(), "--locked" | "--offline"))
    {
        return Err(invalid(
            "usage: kipferl sync --locked [--offline] (use add to change the lock)",
        ));
    }
    let _guard = Operation::new(root)?;
    let lock = read_lock(root)?;
    check_lock(root, &lock)?;
    let registry = registry::Registry::new(
        root.join(".kipferl/cache"),
        arguments.iter().any(|value| value == "--offline"),
    );
    install(root, lock, &registry, None, stdout)
}

fn read_lock(root: &Path) -> io::Result<Lock> {
    let bytes = registry::read_regular(&root.join("kipferl.lock"), MAX_LOCK).map_err(|error| {
        invalid(format!(
            "cannot read kipferl.lock: {error}; run kipferl add to resolve dependencies"
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(invalid)
}

fn check_lock(root: &Path, lock: &Lock) -> io::Result<()> {
    if lock.schema != 1 {
        return Err(invalid("unsupported dependency lock schema"));
    }
    if lock.runtime_sha256 != runtime_hash()?
        || lock.target != crate::run_command::embedded_runtime_target()
    {
        return Err(invalid(
            "dependency lock was checked against a different runtime or target; run kipferl add to re-resolve and recheck",
        ));
    }
    if lock.requirements != read_requirements(root)? {
        return Err(invalid(
            "kipferl.json dependencies differ from kipferl.lock; run kipferl add to update the lock",
        ));
    }
    resolver::validate_graph(&lock.requirements, &lock.packages)
}

/// Validate the entire installed tree before execution/bundling; never use stale packages silently.
pub fn validate_installation(root: &Path) -> io::Result<Option<PathBuf>> {
    if !root.join("kipferl.json").try_exists()? {
        return Ok(None);
    }
    let requirements = read_requirements(root)?;
    let path = root.join(".kipferl/packages");
    if requirements.is_empty() {
        if path.exists() {
            return Err(invalid(
                "installed packages exist but no dependencies are configured; remove .kipferl/packages or restore kipferl.json",
            ));
        }
        return Ok(None);
    }
    let lock = read_lock(root)?;
    check_lock(root, &lock)?;
    for artifact in &lock.packages {
        let report = package_compat::lookup(
            &artifact.name,
            &artifact.version,
            &artifact.sha256,
            &lock.runtime_sha256,
        )?;
        if report.status == Status::Incompatible
            || (report.status == Status::Unverified && !lock.allow_unverified)
        {
            return Err(invalid(format!(
                "{}=={} is {} in the current compatibility catalog; run kipferl add to recheck",
                artifact.name,
                artifact.version,
                report.status.as_str()
            )));
        }
    }
    for directory in [root.join(".kipferl"), path.clone()] {
        safe_directory(&directory, false).map_err(|error| {
            invalid(format!(
                "installed dependencies are unavailable at {}: {error}; run kipferl sync --locked",
                directory.display()
            ))
        })?;
    }
    if inventory(&path)? != lock.files {
        return Err(invalid(
            "installed dependency files differ from the lock; run kipferl sync --locked to restore them",
        ));
    }
    Ok(Some(path))
}

fn install(
    root: &Path,
    mut lock: Lock,
    registry: &registry::Registry,
    config: Option<serde_json::Value>,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    let mut stage = Stage::new(&root.join(".kipferl"))?;
    let destination = stage.0.join("packages");
    fs::create_dir(&destination)?;
    let extraction_root = stage.0.join("wheels");
    fs::create_dir(&extraction_root)?;
    let mut seen = BTreeMap::new();
    for artifact in &lock.packages {
        let bytes = registry.wheel(artifact)?;
        let wheel = wheel::inspect(&bytes, artifact)?;
        if wheel.requirements != artifact.requires_dist {
            return Err(invalid(format!(
                "{}: locked dependency metadata differs from the wheel",
                artifact.name
            )));
        }
        let wheel_root = extraction_root.join(&artifact.name);
        fs::create_dir(&wheel_root)?;
        wheel::extract(&wheel, &wheel_root)?;
        let report = package_compat::inspect(
            &artifact.name,
            &artifact.version,
            &artifact.sha256,
            &lock.runtime_sha256,
            &wheel_root,
        )?;
        for diagnostic in &report.diagnostics {
            writeln!(
                stdout,
                "{}=={}: {diagnostic}",
                artifact.name, artifact.version
            )?;
        }
        if report.status == Status::Incompatible {
            return Err(invalid(format!(
                "{}=={} has known compatibility blockers",
                artifact.name, artifact.version
            )));
        }
        if report.status == Status::Unverified && !lock.allow_unverified {
            return Err(invalid(format!(
                "{}=={} is unverified for this runtime; inspect kipferl deps catalog or explicitly opt in with add --allow-unverified",
                artifact.name, artifact.version
            )));
        }
        compile_sources(&wheel, &wheel_root)?;
        for name in wheel.files.keys() {
            let folded = name.to_lowercase();
            for ancestor in Path::new(&folded).ancestors() {
                if ancestor
                    .to_str()
                    .is_some_and(|part| seen.contains_key(part))
                {
                    return Err(invalid(format!("package path collision: {name}")));
                }
            }
            if seen
                .keys()
                .any(|existing: &String| existing.starts_with(&format!("{folded}/")))
            {
                return Err(invalid(format!("package file/directory collision: {name}")));
            }
            seen.insert(folded, artifact.name.clone());
        }
        wheel::extract(&wheel, &destination)?;
        writeln!(
            stdout,
            "Checked {}=={} ({})",
            artifact.name,
            artifact.version,
            report.status.as_str()
        )?;
    }
    let files = inventory(&destination)?;
    if config.is_none() && files != lock.files {
        return Err(invalid(
            "locked installed-file inventory does not match the wheel artifacts",
        ));
    }
    lock.files = files;
    let result = commit_install(root, &destination, &lock, config);
    if result.is_err() && stage.0.join("previous-packages").exists() {
        stage.1 = true;
        return Err(invalid(format!(
            "{}; recovery files retained in {}",
            result
                .err()
                .ok_or_else(|| invalid("missing installation error"))?,
            stage.0.display()
        )));
    }
    result?;
    writeln!(
        stdout,
        "Installed {} locked packages in .kipferl/packages",
        lock.packages.len()
    )?;
    Ok(())
}

fn compile_sources(wheel: &wheel::Wheel, directory: &Path) -> io::Result<()> {
    let mut names = Vec::new();
    for (name, source) in &wheel.files {
        if !extension(name, "py") {
            continue;
        }
        std::str::from_utf8(source)
            .map_err(|error| invalid(format!("{name}: Python source must use UTF-8: {error}")))?;
        names.push(name.as_str());
    }
    crate::syntax_check::check_files(directory, &names)
}

fn inventory(root: &Path) -> io::Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    let mut total = 0_u64;
    while let Some(directory) = pending.pop() {
        safe_directory(&directory, false)?;
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .map_err(invalid)?
                    .to_str()
                    .ok_or_else(|| invalid("non-UTF8 installed path"))?
                    .to_owned();
                wheel::safe_path(&relative)?;
                let bytes = registry::read_regular(
                    &path,
                    if extension(&relative, "py") {
                        1024 * 1024
                    } else {
                        4 * 1024 * 1024
                    },
                )?;
                total = total.saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
                files.insert(relative, sha256(&bytes));
                if files.len() > 1024 || total > 32 * 1024 * 1024 {
                    return Err(invalid(
                        "installed packages exceed bundle limits: 1024 files or 32 MiB",
                    ));
                }
            } else {
                return Err(invalid(
                    "installed dependency tree contains a symlink or special file",
                ));
            }
        }
    }
    Ok(files)
}

fn safe_directory(path: &Path, create: bool) -> io::Result<()> {
    if create {
        match fs::create_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(invalid(format!(
            "{} must be a real directory, not a symlink",
            path.display()
        )));
    }
    Ok(())
}

struct Operation(PathBuf);
impl Operation {
    fn new(root: &Path) -> io::Result<Self> {
        safe_directory(&root.join(".kipferl"), true)?;
        safe_directory(&root.join(".kipferl/cache"), true)?;
        let path = root.join(".kipferl/dependencies.operation");
        fs::create_dir(&path).map_err(|error| invalid(format!("cannot acquire package operation lock: {error}; if no package command is running, remove {}", path.display())))?;
        Ok(Self(path))
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

struct Stage(PathBuf, bool);
impl Stage {
    fn new(parent: &Path) -> io::Result<Self> {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!("stage-{}-{id}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(Self(path, false))
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        if !self.1 {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn commit_install(
    root: &Path,
    staged: &Path,
    lock: &Lock,
    config: Option<serde_json::Value>,
) -> io::Result<()> {
    let destination = root.join(".kipferl/packages");
    let backup = staged.with_file_name("previous-packages");
    let old_config = registry::read_regular(&root.join("kipferl.json"), 64 * 1024)?;
    let old_lock = match registry::read_regular(&root.join("kipferl.lock"), MAX_LOCK) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let lock_bytes = serde_json::to_vec_pretty(lock).map_err(invalid)?;
    let config_bytes = config
        .map(|value| serde_json::to_vec_pretty(&value))
        .transpose()
        .map_err(invalid)?;
    let had_packages = match fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            fs::rename(&destination, &backup)?;
            true
        }
        Ok(_) => return Err(invalid(".kipferl/packages must be a real directory")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    let result: io::Result<()> = (|| {
        fs::rename(staged, &destination)?;
        crate::run_command::write_atomically(&root.join("kipferl.lock"), &[&lock_bytes], 0o644)?;
        if let Some(bytes) = &config_bytes {
            crate::run_command::write_atomically(&root.join("kipferl.json"), &[bytes], 0o644)?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let rollback = (|| {
            if destination.exists() {
                fs::remove_dir_all(&destination)?;
            }
            if had_packages {
                fs::rename(&backup, &destination)?;
            }
            crate::run_command::write_atomically(
                &root.join("kipferl.json"),
                &[&old_config],
                0o644,
            )?;
            if let Some(bytes) = old_lock {
                crate::run_command::write_atomically(&root.join("kipferl.lock"), &[&bytes], 0o644)?;
            } else if root.join("kipferl.lock").exists() {
                fs::remove_file(root.join("kipferl.lock"))?;
            }
            Ok::<_, io::Error>(())
        })();
        return Err(invalid(format!(
            "cannot publish dependency installation: {error}; rollback: {rollback:?}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
