use std::process::{Command, Output};

#[test]
fn passes_the_zig_binascii_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_binascii.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 55 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn exposes_aliases_crc_and_string_decoding() {
    let output = run("import binascii\n\
assert binascii.Error is ValueError\n\
assert binascii.Incomplete is ValueError\n\
assert binascii.unhexlify('DeAdBeEf') == b'\\xde\\xad\\xbe\\xef'\n\
assert binascii.a2b_base64('YQ==\\n') == b'a'\n\
assert binascii.b2a_base64(b'') == b'\\n'\n\
assert binascii.crc32(b'') == 0\n\
assert binascii.crc32(b'123456789') == 3421780262\n\
assert binascii.crc32(b'123456789', []) == 3421780262");

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_binascii_errors_and_buffer_limits() {
    for (source, expected) in [
        (
            "import binascii; binascii.hexlify('x')",
            "TypeError: a bytes-like object is required",
        ),
        (
            "import binascii; binascii.unhexlify(b'a')",
            "ValueError: Odd-length string",
        ),
        (
            "import binascii; binascii.unhexlify(b'gg')",
            "ValueError: Non-hexadecimal digit found",
        ),
        (
            "import binascii; binascii.a2b_base64(b'YR==')",
            "ValueError: Invalid base64-encoded string",
        ),
        (
            "import binascii; binascii.b2a_base64(bytes([0] * 6142))",
            "ValueError: data too large",
        ),
        (
            "import base64, binascii; binascii.a2b_base64(base64.b64encode(bytes([0] * 8193)))",
            "ValueError: data too large",
        ),
        (
            "import binascii; binascii.crc32('x')",
            "TypeError: a bytes-like object is required",
        ),
    ] {
        let output = run(source);
        assert_eq!(output.status.code(), Some(1), "{}", diagnostic(&output));
        assert!(
            text(&output.stdout).contains(expected),
            "{}",
            diagnostic(&output)
        );
        assert!(text(&output.stderr).contains("Python execution failed"));
    }
}

fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm"))
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
