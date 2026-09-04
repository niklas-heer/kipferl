use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use kipferl_format::Trailer;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn executes_an_embedded_runtime_and_forwards_arguments_from_a_path_with_spaces() {
    let temporary = TestDirectory::new();
    let universal = temporary.path.join("application with spaces");
    let loader = fs::read(env!("CARGO_BIN_EXE_kipferl-loader")).expect("read Rust loader");
    let runtime = b"#!/bin/sh\nexec /bin/sh \"$@\"\n";
    let python = b"printf 'loader:%s\\n' \"$1\"\n";
    write_bundle(&universal, &loader, runtime, python);

    let output = Command::new(&universal)
        .arg("argument with spaces")
        .env("KIPFERL_CACHE_DIR", &temporary.path)
        .output()
        .expect("execute universal bundle");

    assert!(
        output.status.success(),
        "status: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"loader:argument with spaces\n");
}

#[test]
fn executes_updated_code_when_bundles_share_a_sampled_cache_key() {
    let temporary = TestDirectory::new();
    let loader = fs::read(env!("CARGO_BIN_EXE_kipferl-loader")).expect("read Rust loader");
    let runtime = b"#!/bin/sh\nexec /bin/sh \"$@\"\n";
    // The frozen cache key sees the same first 1 KiB and equal payload sizes.
    let prefix = format!("#{}\n", "padding".repeat(200));
    let first = temporary.path.join("first application");
    let second = temporary.path.join("second application");
    write_bundle(
        &first,
        &loader,
        runtime,
        format!("{prefix}printf 'first\\n'\n").as_bytes(),
    );
    write_bundle(
        &second,
        &loader,
        runtime,
        format!("{prefix}printf 'other\\n'\n").as_bytes(),
    );

    for (application, expected) in [
        (&first, b"first\n"),
        (&second, b"other\n"),
        (&first, b"first\n"),
    ] {
        let output = Command::new(application)
            .env("KIPFERL_CACHE_DIR", &temporary.path)
            .output()
            .expect("execute universal bundle");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected);
    }
}

#[expect(
    clippy::expect_used,
    reason = "Test fixture construction must fail the test on invalid size or filesystem failure"
)]
fn write_bundle(path: &Path, loader: &[u8], runtime: &[u8], python: &[u8]) {
    let runtime_offset = u64::try_from(loader.len()).expect("loader size fits u64");
    let runtime_size = u64::try_from(runtime.len()).expect("runtime size fits u64");
    let trailer = Trailer {
        runtime_offset,
        runtime_size,
        python_offset: runtime_offset
            .checked_add(runtime_size)
            .expect("fixture offset fits u64"),
        python_size: u64::try_from(python.len()).expect("Python size fits u64"),
    };
    let bytes = [loader, runtime, python, &trailer.encode()].concat();
    fs::write(path, bytes).expect("write universal bundle");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make bundle executable");
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    #[expect(
        clippy::expect_used,
        reason = "A failed temporary-directory setup must fail the integration test"
    )]
    fn new() -> Self {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kipferl-loader-exec-test-{}-{counter} with spaces",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
