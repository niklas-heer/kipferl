use std::process::{Command, Output};

#[test]
fn passes_the_zig_base64_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_base64.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 18 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn extends_pocketpy_base64_and_preserves_limits() {
    let output = run("import base64\n\
assert base64.b64encode(b'Hello') == b'SGVsbG8='\n\
assert base64.b64decode('SGVsbG8=') == b'Hello'\n\
assert base64.urlsafe_b64decode('-_w=') == b'\\xfb\\xfc'\n\
assert base64.urlsafe_b64decode('+/w=') == b'\\xfb\\xfc'\n\
assert len(base64.urlsafe_b64encode(bytes([0] * 3072))) == 4096\n\
assert len(base64.urlsafe_b64decode(bytes([65] * 4096))) == 3072");

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_base64_errors_and_noncanonical_rejection() {
    for (source, expected) in [
        (
            "import base64; base64.urlsafe_b64encode('x')",
            "TypeError: expected bytes",
        ),
        (
            "import base64; base64.urlsafe_b64decode(1)",
            "TypeError: expected bytes or string",
        ),
        (
            "import base64; base64.urlsafe_b64decode(b'YR==')",
            "ValueError: invalid base64",
        ),
        (
            "import base64; base64.urlsafe_b64encode(bytes([0] * 3073))",
            "ValueError: data too large",
        ),
        (
            "import base64; base64.urlsafe_b64decode(bytes([65] * 4097))",
            "ValueError: data too large",
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
