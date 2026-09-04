use std::process::{Command, Output};

#[test]
fn passes_the_zig_statistics_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_statistics.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 28 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_tuples_float_results_and_mode_identity() {
    let output = run("import statistics\n\
assert statistics.mean((1, 2, 3)) == 2.0\n\
assert type(statistics.mean((1, 2, 3))) is float\n\
assert statistics.median((3, 1, 2)) == 2.0\n\
assert statistics.mode([1, 2, 1, 2]) == 1\n\
assert statistics.mode(['a', 'b', 'a']) == 'a'\n\
marker = []\n\
assert statistics.mode([marker, marker]) is marker");

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_statistics_errors_and_legacy_bounds() {
    for (source, expected) in [
        (
            "import statistics; statistics.mean()",
            "TypeError: too few arguments",
        ),
        (
            "import statistics; statistics.mean([])",
            "TypeError: mean requires at least one data point",
        ),
        (
            "import statistics; statistics.mean([1, 'x'])",
            "TypeError: expected 'int' or 'float', got 'str'",
        ),
        (
            "import statistics; statistics.mean([True, False])",
            "TypeError: expected 'int' or 'float', got 'bool'",
        ),
        (
            "import statistics; statistics.median([1, 'x'])",
            "TypeError: data must be numeric",
        ),
        (
            "import statistics; statistics.median_low([])",
            "TypeError: median_low requires numeric data",
        ),
        (
            "import statistics; statistics.variance([1])",
            "TypeError: variance requires at least two data points",
        ),
        (
            "import statistics; statistics.median([0] * 257)",
            "ValueError: data too large",
        ),
        (
            "import statistics; statistics.mode([i for i in range(257)])",
            "ValueError: too many unique values",
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
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
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
