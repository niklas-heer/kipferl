//! Offline wheel fixtures exercise the public package workflow and portable output.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);

fn hash(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

impl Fixture {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "kipferl-dependencies-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        let fixture = Self(path.canonicalize()?);
        fixture.write(
            "kipferl.json",
            r#"{"entry":"app.py","dependencies":["demo-app==1.0"]}"#,
        )?;
        fixture.write("app.py", "global counter\ncounter = 10\nimport dephelper\nfrom demopkg import greet\ndephelper.bump()\nassert counter == 10\nassert dephelper.counter == 1\nprint(greet())\n")?;
        fixture.write(
            "tests/test_package.py",
            "from demopkg import greet\nassert greet() == 'hello locked world'\n",
        )?;
        fs::create_dir_all(fixture.0.join(".kipferl/cache"))?;
        let mut files = BTreeMap::new();
        let first = fixture.wheel("demo-app", &["demo-leaf>=1"], &[
            ("demopkg/__init__.py", "from .messages import greet\n"),
            ("demopkg/messages.py", "import os\nfrom dephelper import phrase\ndef greet():\n    with open(os.path.join(os.path.dirname(__file__), 'data.txt'), 'r') as resource:\n        return phrase + resource.read()\n"),
            ("demopkg/data.txt", "world"),
        ], &mut files)?;
        let second = fixture.wheel(
            "demo-leaf",
            &[],
            &[("dephelper.py", "global phrase\nphrase = 'hello locked '\ncounter = 0\ndef bump():\n    global counter\n    counter += 1\n")],
            &mut files,
        )?;
        let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let runtime = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/pocketpy-kipferl-{target}")),
        )?;
        let lock = json!({
            "schema": 1, "runtime_sha256": hash(&runtime), "target": target,
            "requirements": ["demo-app==1.0"], "allow_unverified": true,
            "packages": [first, second], "files": files
        });
        fixture.write("kipferl.lock", serde_json::to_vec_pretty(&lock)?)?;
        Ok(fixture)
    }

    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> io::Result<()> {
        let path = self.0.join(name);
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("fixture has no parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(path, bytes)
    }

    fn wheel(
        &self,
        name: &str,
        requirements: &[&str],
        sources: &[(&str, &str)],
        hashes: &mut BTreeMap<String, String>,
    ) -> io::Result<Value> {
        let normalized = name.replace('-', "_");
        let info = format!("{normalized}-1.0.dist-info");
        let mut metadata = format!("Metadata-Version: 2.1\nName: {name}\nVersion: 1.0\n");
        for dependency in requirements {
            writeln!(metadata, "Requires-Dist: {dependency}").map_err(io::Error::other)?;
        }
        let mut files: BTreeMap<String, String> = sources
            .iter()
            .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
            .collect();
        files.insert(format!("{info}/METADATA"), metadata);
        files.insert(
            format!("{info}/WHEEL"),
            "Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n".to_owned(),
        );
        files.insert(
            format!("{info}/LICENSE"),
            "Fixture written for the Kipferl test suite. MIT.\n".to_owned(),
        );
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (path, content) in files {
            archive.start_file(&path, SimpleFileOptions::default())?;
            archive.write_all(content.as_bytes())?;
            hashes.insert(path, hash(content.as_bytes()));
        }
        let bytes = archive.finish()?.into_inner();
        let digest = hash(&bytes);
        self.write(&format!(".kipferl/cache/{digest}.whl"), bytes)?;
        let filename = format!("{normalized}-1.0-py3-none-any.whl");
        Ok(json!({"name":name,"version":"1.0","filename":filename,
            "url":format!("https://files.pythonhosted.org/packages/fixtures/{filename}"),
            "sha256":digest,"requires_dist":requirements}))
    }

    fn cli(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_kipferl"))
            .args(arguments)
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("runtime-cache"))
            .output()
    }

    fn lock(&self) -> io::Result<Value> {
        Ok(serde_json::from_slice(&fs::read(
            self.0.join("kipferl.lock"),
        )?)?)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn failure(output: &Output, expected: &str) {
    assert!(
        !output.status.success(),
        "unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "expected {expected:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "I/O setup errors propagate while behavior assertions intentionally fail this integration test"
)]
fn offline_dependencies_run_test_and_bundle_resources_without_checkout() -> io::Result<()> {
    let fixture = Fixture::new()?;
    failure(&fixture.cli(&["run"])?, "run kipferl sync --locked");
    success(&fixture.cli(&["sync", "--locked", "--offline"])?);
    success(&fixture.cli(&["deps", "check"])?);
    let list = fixture.cli(&["deps", "list"])?;
    success(&list);
    assert!(String::from_utf8_lossy(&list.stdout).contains("demo-leaf==1.0"));
    let run = fixture.cli(&["run"])?;
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "hello locked world\n");
    success(&fixture.cli(&["test"])?);
    success(&fixture.cli(&["build", "-o", "portable"])?);
    fs::remove_dir_all(fixture.0.join(".kipferl"))?;
    fs::remove_file(fixture.0.join("app.py"))?;
    fs::remove_file(fixture.0.join("kipferl.json"))?;
    let output = Command::new(fixture.0.join("portable"))
        .current_dir(std::env::temp_dir())
        .env("KIPFERL_CACHE_DIR", fixture.0.join("loader-cache"))
        .output()?;
    success(&output);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "hello locked world\n"
    );
    Ok(())
}

#[test]
fn tampered_installation_is_rejected_and_locked_sync_restores_it() -> io::Result<()> {
    let fixture = Fixture::new()?;
    success(&fixture.cli(&["sync", "--locked", "--offline"])?);
    fixture.write(".kipferl/packages/dephelper.py", "phrase = 'tampered'\n")?;
    for command in [
        vec!["deps", "check"],
        vec!["run"],
        vec!["build", "--mode", "single"],
    ] {
        failure(&fixture.cli(&command)?, "installed dependency files differ");
    }
    success(&fixture.cli(&["sync", "--locked", "--offline"])?);
    success(&fixture.cli(&["run"])?);
    fixture.write(".kipferl/packages/untracked.py", "print('untracked')")?;
    failure(
        &fixture.cli(&["deps", "check"])?,
        "installed dependency files differ",
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "I/O setup errors propagate while behavior assertions intentionally fail this integration test"
)]
fn lock_mismatch_and_corrupt_cache_fail_without_changing_existing_install() -> io::Result<()> {
    let fixture = Fixture::new()?;
    success(&fixture.cli(&["sync", "--locked", "--offline"])?);
    let original_lock = fs::read(fixture.0.join("kipferl.lock"))?;
    let package = fs::read(fixture.0.join(".kipferl/packages/dephelper.py"))?;
    let mut lock = fixture.lock()?;
    let object = lock
        .as_object_mut()
        .ok_or_else(|| io::Error::other("fixture lock not an object"))?;
    object.insert("runtime_sha256".to_owned(), Value::String("0".repeat(64)));
    fixture.write("kipferl.lock", serde_json::to_vec(&lock)?)?;
    failure(
        &fixture.cli(&["sync", "--locked", "--offline"])?,
        "different runtime or target",
    );
    fixture.write("kipferl.lock", &original_lock)?;
    fixture.write(
        "kipferl.json",
        r#"{"entry":"app.py","dependencies":["demo-app==2.0"]}"#,
    )?;
    failure(&fixture.cli(&["run"])?, "dependencies differ");
    fixture.write(
        "kipferl.json",
        r#"{"entry":"app.py","dependencies":["demo-app==1.0"]}"#,
    )?;
    for entry in fs::read_dir(fixture.0.join(".kipferl/cache"))? {
        fs::write(entry?.path(), b"corrupt wheel")?;
    }
    failure(
        &fixture.cli(&["sync", "--locked", "--offline"])?,
        "hash mismatch",
    );
    assert_eq!(fs::read(fixture.0.join("kipferl.lock"))?, original_lock);
    assert_eq!(
        fs::read(fixture.0.join(".kipferl/packages/dephelper.py"))?,
        package
    );
    success(&fixture.cli(&["run"])?);
    Ok(())
}

#[test]
fn installed_symlinks_are_rejected_without_reading_their_target() -> io::Result<()> {
    let fixture = Fixture::new()?;
    success(&fixture.cli(&["sync", "--locked", "--offline"])?);
    let path = fixture.0.join(".kipferl/packages/dephelper.py");
    fs::remove_file(&path)?;
    std::os::unix::fs::symlink(fixture.0.join("app.py"), path)?;
    failure(&fixture.cli(&["deps", "check"])?, "symlink");
    Ok(())
}

#[test]
fn unverified_packages_require_explicit_recorded_opt_in() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let mut lock = fixture.lock()?;
    lock.as_object_mut()
        .ok_or_else(|| io::Error::other("fixture lock not an object"))?
        .insert("allow_unverified".to_owned(), Value::Bool(false));
    fixture.write("kipferl.lock", serde_json::to_vec(&lock)?)?;
    failure(
        &fixture.cli(&["sync", "--locked", "--offline"])?,
        "is unverified",
    );
    if fixture.0.join(".kipferl/packages").exists() {
        return Err(io::Error::other("rejected wheel was installed"));
    }
    Ok(())
}

#[test]
fn unsupported_requirements_fail_before_mutating_project() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let original = fs::read(fixture.0.join("kipferl.json"))?;
    for (requirement, expected) in [
        ("example[extra]", "extras are not supported"),
        (
            "example; python_version > '3'",
            "environment markers are not supported",
        ),
        ("example @ https://example.com/example.whl", "direct URLs"),
    ] {
        failure(&fixture.cli(&["add", requirement])?, expected);
    }
    if fs::read(fixture.0.join("kipferl.json"))? != original {
        return Err(io::Error::other("invalid requirement changed the config"));
    }
    Ok(())
}
