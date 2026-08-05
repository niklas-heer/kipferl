use std::process::{Command, Output};

#[test]
fn passes_all_binary_container_wave_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "array",
            include_str!("../../../tests/cpython/test_array.py"),
            "Results: 69 passed, 0 failed, 0 skipped",
        ),
        (
            "struct",
            include_str!("../../../tests/cpython/test_struct.py"),
            "Results: 68 passed, 0 failed, 0 skipped",
        ),
        (
            "secrets",
            include_str!("../../../tests/cpython/test_secrets.py"),
            "Results: 8 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run(source);
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
fn preserves_binary_roundtrips_mutable_buffers_entropy_and_errors() {
    let output = run(concat!(
        "from array import array\n",
        "import secrets, struct\n",
        "numbers = array('I', [0x12345678, 0x90ABCDEF])\n",
        "encoded = numbers.tobytes()\n",
        "assert encoded == struct.pack('<2I', 0x12345678, 0x90ABCDEF)\n",
        "assert struct.unpack('<2I', encoded) == (0x12345678, 0x90ABCDEF)\n",
        "record = struct.Struct('>hIf')\n",
        "packed = record.pack(-123, 0x10203040, 1.5)\n",
        "unpacked = record.unpack(packed)\n",
        "assert unpacked[0] == -123 and unpacked[1] == 0x10203040\n",
        "assert abs(unpacked[2] - 1.5) < 0.0001\n",
        "buffer = bytearray(12)\n",
        "struct.pack_into('>I', buffer, 4, 0xAABBCCDD)\n",
        "assert buffer[4:8] == b'\\xaa\\xbb\\xcc\\xdd'\n",
        "assert struct.unpack_from('>I', buffer, 4) == (0xAABBCCDD,)\n",
        "tokens = [secrets.token_bytes(16) for _ in range(16)]\n",
        "assert all([len(token) == 16 for token in tokens])\n",
        "assert len(set(tokens)) == 16\n",
        "assert len(secrets.token_hex(7)) == 14\n",
        "assert len(secrets.token_urlsafe(7)) > 0\n",
        "for operation in (\n",
        "    lambda: struct.pack('b', 128),\n",
        "    lambda: struct.unpack('I', b'\\x00'),\n",
        "    lambda: struct.calcsize('z'),\n",
        "):\n",
        "    try:\n",
        "        operation()\n",
        "        raise AssertionError('expected struct.error')\n",
        "    except struct.error:\n",
        "        pass\n",
        "try:\n",
        "    secrets.randbelow(0)\n",
        "    raise AssertionError('expected ValueError')\n",
        "except ValueError:\n",
        "    pass",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
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
