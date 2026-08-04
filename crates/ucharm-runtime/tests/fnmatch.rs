use std::process::{Command, Output};

#[test]
fn passes_the_zig_fnmatch_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_fnmatch.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 55 passed, 0 failed"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_filter_identity_and_skips_non_strings() {
    let output = run("import fnmatch\n\
marker = ['value']\n\
names = ['alpha', marker, 42, 'alpine', 'beta']\n\
matched = fnmatch.filter(names, 'al*')\n\
assert matched == ['alpha', 'alpine']\n\
all_strings = fnmatch.filter(names, '*')\n\
assert all_strings == ['alpha', 'alpine', 'beta']\n\
assert all_strings[0] is names[0]");

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_fnmatch_argument_errors() {
    for (source, expected) in [
        (
            "import fnmatch; fnmatch.fnmatch('name')",
            "TypeError: too few arguments",
        ),
        (
            "import fnmatch; fnmatch.fnmatch(1, '*')",
            "TypeError: name must be a string",
        ),
        (
            "import fnmatch; fnmatch.fnmatch('name', 1)",
            "TypeError: pattern must be a string",
        ),
        (
            "import fnmatch; fnmatch.filter(1, '*')",
            "TypeError: names must be a list",
        ),
        (
            "import fnmatch; fnmatch.translate(1)",
            "TypeError: pattern must be a string",
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
