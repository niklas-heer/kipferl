use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ucharm_format::Trailer;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn executes_an_embedded_runtime_and_forwards_arguments_from_a_path_with_spaces() {
    let temporary = TestDirectory::new();
    let universal = temporary.path.join("application with spaces");
    let loader = fs::read(env!("CARGO_BIN_EXE_ucharm-loader")).expect("read Rust loader");
    let runtime = b"#!/bin/sh\nexec /bin/sh \"$@\"\n";
    let python = b"printf 'loader:%s\\n' \"$1\"\n";
    write_bundle(&universal, &loader, runtime, python);

    let output = Command::new(&universal)
        .arg("argument with spaces")
        .env("UCHARM_CACHE_DIR", &temporary.path)
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

fn write_bundle(path: &Path, loader: &[u8], runtime: &[u8], python: &[u8]) {
    let trailer = Trailer {
        runtime_offset: loader.len() as u64,
        runtime_size: runtime.len() as u64,
        python_offset: (loader.len() + runtime.len()) as u64,
        python_size: python.len() as u64,
    };
    let bytes = [loader, runtime, python, &trailer.encode()].concat();
    fs::write(path, bytes).expect("write universal bundle");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make bundle executable");
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ucharm-loader-exec-test-{}-{counter} with spaces",
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
