//! Compile module source through the runtime's non-executing checker.
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::DirBuilderExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::run_command;

static COUNTER: AtomicU64 = AtomicU64::new(0);
const TIMEOUT: Duration = Duration::from_secs(30);
const BATCH_SIZE: usize = 32;

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_string())
}

/// Compile already-extracted sources, without invoking package code.
pub fn check_files(directory: &Path, names: &[&str]) -> io::Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let directory = directory.canonicalize()?;
    let paths: Vec<_> = names.iter().map(|name| directory.join(name)).collect();
    let workspace = Workspace::new()?;
    check_paths(&run_command::prepare_runtime()?, &workspace.0, &paths)
}

/// Check transformed module and bootstrap sources using isolated regular files.
pub fn check_sources<'a>(sources: impl IntoIterator<Item = (&'a Path, &'a str)>) -> io::Result<()> {
    let workspace = Workspace::new()?;
    let mut paths = Vec::new();
    for (index, (name, source)) in sources.into_iter().enumerate() {
        if name
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(invalid("syntax source filename must be a relative path"));
        }
        let path = workspace.0.join(index.to_string()).join(name);
        let parent = path
            .parent()
            .ok_or_else(|| invalid("missing syntax source directory"))?;
        fs::create_dir_all(parent)?;
        fs::write(&path, source)?;
        paths.push(path);
    }
    if paths.is_empty() {
        return Ok(());
    }
    check_paths(&run_command::prepare_runtime()?, &workspace.0, &paths)
}

fn check_paths(runtime: &Path, workspace: &Path, paths: &[PathBuf]) -> io::Result<()> {
    let started = Instant::now();
    for batch in paths.chunks(BATCH_SIZE) {
        if started.elapsed() >= TIMEOUT {
            return Err(invalid("Python syntax checking exceeded 30 seconds"));
        }
        check_batch(runtime, workspace, batch, started)?;
    }
    Ok(())
}

fn check_batch(
    runtime: &Path,
    workspace: &Path,
    paths: &[PathBuf],
    started: Instant,
) -> io::Result<()> {
    // Older runtime assets interpret an unknown flag as a filename. The private
    // workspace contains no --check-syntax file, so stale runtimes fail closed
    // instead of executing a wheel member that happens to have that filename.
    let mut child = Command::new(runtime)
        .args(["--check-syntax", "--"])
        .args(paths)
        .current_dir(workspace)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out = std::thread::spawn(move || bounded_output(stdout));
    let err = std::thread::spawn(move || bounded_output(stderr));
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Err(error) => break Err(error),
            Ok(None) if started.elapsed() >= TIMEOUT => {
                break Err(invalid("Python syntax checking exceeded 30 seconds"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    if status.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let stdout = out
        .join()
        .map_err(|_| invalid("compiler output reader failed"))??;
    let stderr = err
        .join()
        .map_err(|_| invalid("compiler diagnostic reader failed"))??;
    if !status?.success() {
        return Err(invalid(format!(
            "Python syntax check failed (no application code was executed):\n{}{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        )));
    }
    Ok(())
}

fn bounded_output(reader: Option<impl Read>) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if let Some(mut reader) = reader {
        let mut buffer = [0_u8; 4096];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            let keep = read.min(65_536_usize.saturating_sub(bytes.len()));
            bytes.extend_from_slice(
                buffer
                    .get(..keep)
                    .ok_or_else(|| invalid("invalid output read length"))?,
            );
        }
    }
    Ok(bytes)
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> io::Result<Self> {
        for _ in 0..16 {
            let path = std::env::temp_dir().join(format!(
                "kipferl-syntax-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Ok(Self(path.canonicalize()?)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "cannot create private syntax-check workspace",
        ))
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{Workspace, bounded_output, check_paths};
    use std::fs;
    use std::io::{self, Read};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
    )]
    fn stale_runtime_cannot_execute_a_wheel_member_named_like_the_checker_flag() -> io::Result<()> {
        let wheel = Workspace::new()?;
        let checker = Workspace::new()?;
        let runtime = checker.0.join("legacy-runtime");
        fs::write(&runtime, "#!/bin/sh\nexec /bin/sh \"$1\"\n")?;
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700))?;
        let marker = wheel.0.join("EXECUTED");
        fs::write(
            wheel.0.join("--check-syntax"),
            format!("touch '{}'\n", marker.display()),
        )?;
        let source = wheel.0.join("module.py");
        fs::write(&source, "pass\n")?;
        assert!(check_paths(&runtime, &checker.0, &[source]).is_err());
        assert!(!marker.exists());
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
    )]
    fn diagnostic_capture_drains_but_bounds_long_output() -> io::Result<()> {
        assert_eq!(
            bounded_output(Some(io::repeat(b'x').take(100_000)))?.len(),
            65_536
        );
        Ok(())
    }
}
