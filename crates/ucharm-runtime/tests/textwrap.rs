use std::process::{Command, Output};

#[test]
fn passes_the_zig_textwrap_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_textwrap.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 24 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_legacy_textwrap_edges() {
    let output = run("import textwrap\n\
assert textwrap.wrap('a bb ccc dddd', 1) == ['a', 'ccc']\n\
assert textwrap.wrap('  a\\tb\\nc\\r\\v d\\f', 3) == ['a b', 'd']\n\
assert textwrap.wrap('abcdef gh', 3) == ['abcdef']\n\
assert textwrap.wrap('a bb', 0) == ['a bb']\n\
assert textwrap.fill('a bb ccc', 0) == 'a\\nbb\\nccc'\n\
assert textwrap.dedent('  a\\n\\tb\\n    c\\n') == ' a\\nb\\n   c\\n'\n\
assert textwrap.indent('a\\n\\nb\\n', '>') == '>a\\n>\\n>b\\n>'\n\
assert textwrap.shorten('a bb ccc', 0) == ''\n\
assert textwrap.shorten('a bb ccc', 1) == '.'\n\
assert textwrap.shorten('a bb ccc', 4) == 'a...'\n\
assert textwrap.shorten('  a\\tb\\n c ', 20) == 'a b c'\n\
many = textwrap.wrap('a ' * 3000, 1)\n\
assert len(many) == 1024\n\
assert all([line == 'a' for line in many])");

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_textwrap_argument_errors_and_width_fallbacks() {
    let fallback = run("import textwrap\n\
assert textwrap.wrap('a bb', 'invalid') == ['a bb']\n\
assert textwrap.fill('a bb', 'invalid') == 'a bb'");
    assert!(fallback.status.success(), "{}", diagnostic(&fallback));

    for (source, expected) in [
        (
            "import textwrap; textwrap.wrap()",
            "TypeError: wrap() takes 1 positional arguments but 0 were given",
        ),
        (
            "import textwrap; textwrap.wrap(1)",
            "TypeError: text must be a string",
        ),
        (
            "import textwrap; textwrap.dedent()",
            "TypeError: too few arguments",
        ),
        (
            "import textwrap; textwrap.indent('a', 1)",
            "TypeError: prefix must be a string",
        ),
        (
            "import textwrap; textwrap.shorten('a', 'x')",
            "TypeError: width must be an int",
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
