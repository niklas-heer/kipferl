use std::process::{Command, Output};

#[test]
fn passes_all_crypto_compression_and_archive_fixtures() {
    for (module, source, summary) in [
        (
            "hashlib",
            include_str!("../../../tests/cpython/test_hashlib.py"),
            "Results: 29 passed, 0 failed, 0 skipped",
        ),
        (
            "hmac",
            include_str!("../../../tests/cpython/test_hmac.py"),
            "Results: 4 passed, 0 failed, 0 skipped",
        ),
        (
            "gzip",
            include_str!("../../../tests/cpython/test_gzip.py"),
            "Results: 6 passed, 0 failed, 0 skipped",
        ),
        (
            "zipfile",
            include_str!("../../../tests/cpython/test_zipfile.py"),
            "Results: 7 passed, 0 failed, 0 skipped",
        ),
        (
            "tarfile",
            include_str!("../../../tests/cpython/test_tarfile.py"),
            "Results: 7 passed, 0 failed, 0 skipped",
        ),
        (
            "io",
            include_str!("../../../tests/cpython/test_io.py"),
            "Results: 53 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run_runtime(source);
        assert!(
            output.status.success(),
            "{module} fixture failed:\n{}",
            diagnostic(&output)
        );
        assert!(
            text(&output.stdout).contains(summary),
            "{module} summary missing:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{module}");
    }
}

#[test]
fn matches_cpython_for_deterministic_crypto_compression_and_buffer_operations() {
    let source = concat!(
        "import gzip, hashlib, hmac, io\n",
        "data = b'crypto-compression-wave' * 64\n",
        "print(hashlib.md5(data).hexdigest())\n",
        "print(hashlib.sha1(data).hexdigest())\n",
        "print(hashlib.sha256(data).hexdigest())\n",
        "print(hashlib.sha512(data).hexdigest())\n",
        "print(hmac.new(b'key', data, 'sha256').hexdigest())\n",
        "compressed = gzip.compress(data)\n",
        "print(len(compressed) > 0)\n",
        "print(gzip.decompress(compressed) == data)\n",
        "buffer = io.BytesIO(b'abc')\n",
        "buffer.seek(5); buffer.write(b'z')\n",
        "print(len(buffer.getvalue()))\n",
        "text = io.StringIO('alpha\\nbeta')\n",
        "print(text.readline() == 'alpha\\n')\n",
    );
    let rust = run_runtime(source);
    assert!(rust.status.success(), "{}", diagnostic(&rust));

    let cpython = Command::new("python3")
        .args(["-c", source])
        .output()
        .expect("run CPython differential oracle");
    assert!(cpython.status.success(), "{}", diagnostic(&cpython));
    assert_eq!(rust.stdout, cpython.stdout);
    assert_eq!(text(&rust.stderr), "");
}

#[test]
fn preserves_crypto_buffer_state_limits_and_error_paths_under_stress() {
    let output = run_runtime(concat!(
        "import gzip, hashlib, hmac, io\n",
        "retained = []\n",
        "for i in range(50):\n",
        "    data = (str(i) + ':' + ('x' * 2048)).encode()\n",
        "    digest = hashlib.sha256(data)\n",
        "    first = digest.hexdigest()\n",
        "    digest.update(b'!')\n",
        "    assert first != digest.hexdigest()\n",
        "    assert len(hmac.new(b'key', data, 'sha512').digest()) == 64\n",
        "    packed = gzip.compress(data)\n",
        "    assert gzip.decompress(packed) == data\n",
        "    buffer = io.BytesIO(data)\n",
        "    buffer.seek(10); buffer.write(b'abc')\n",
        "    buffer.seek(0); assert len(buffer.read()) == len(data)\n",
        "    retained.append((first, packed))\n",
        "    if len(retained) == 32:\n",
        "        for value in retained:\n",
        "            assert len(value[0]) == 64 and len(value[1]) > 0\n",
        "        retained = []\n",
        "try:\n",
        "    gzip.decompress(b'not-gzip')\n",
        "    raise AssertionError('expected invalid gzip error')\n",
        "except ValueError:\n",
        "    pass\n",
        "try:\n",
        "    hashlib.new('not-a-hash')\n",
        "    raise AssertionError('expected unsupported hash error')\n",
        "except ValueError:\n",
        "    pass\n",
        "try:\n",
        "    io.BytesIO('not-bytes')\n",
        "    raise AssertionError('expected buffer type error')\n",
        "except TypeError:\n",
        "    pass\n",
        "closed = io.StringIO('x')\n",
        "closed.close()\n",
        "try:\n",
        "    closed.read()\n",
        "    raise AssertionError('expected closed buffer error')\n",
        "except ValueError:\n",
        "    pass\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

fn run_runtime(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm-rs"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
