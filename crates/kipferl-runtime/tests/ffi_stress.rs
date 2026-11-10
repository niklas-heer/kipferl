use std::process::{Command, Output};

#[test]
fn keeps_callback_values_rooted_across_allocation_and_exception_stress() {
    let output = run(
        include_str!("fixtures/ffi_allocation_stress.py"),
        &["--name=alice", "--count", "42", "--verbose", "tail"],
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), "");
}

// PocketPy records locals/globals for each traceback frame only while its
// debugger runs, but the collector marks them for every caught exception. The
// uninitialized values crashed Linux (SIGSEGV) and fail UBSan's bool check.
#[test]
fn collects_while_caught_exceptions_keep_their_tracebacks() {
    let output = run(
        concat!(
            "import gc
",
            "def fail(depth):
",
            "    if depth == 0:
",
            "        raise ValueError('deep')
",
            "    fail(depth - 1)
",
            "kept = []
",
            "for i in range(200):
",
            "    try:
",
            "        fail(i % 9)
",
            "    except ValueError as error:
",
            "        kept.append(error)
",
            "    try:
",
            "        import missing_module_for_traceback_stress
",
            "    except ImportError as error:
",
            "        kept.append(error)
",
            "    if i % 20 == 0:
",
            "        gc.collect()
",
            "        junk = [[i] * 4 for _ in range(200)]
",
            "gc.collect()
",
            "assert len(kept) == 400 and str(kept[0]) == 'deep'",
        ),
        &[],
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn repeatedly_initializes_executes_and_finalizes_the_runtime_process() {
    for cycle in 0..24 {
        let output = run(
            concat!(
                "import args, tui\n",
                "for i in range(100):\n",
                "    assert args.parse({'--value': (int, 7)}) == {'_': [], 'value': 9}\n",
                "    assert tui.spinner_frame(i) == tui.spinner_frame(i + 10)",
            ),
            &["--value=9"],
        );
        assert!(
            output.status.success(),
            "runtime cycle {cycle} failed:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stdout), "");
        assert_eq!(text(&output.stderr), "");
    }
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn run(source: &str, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .args(arguments)
        // The fixture asserts tui's default styling.
        .env_remove("NO_COLOR")
        .output()
        .expect("run Rust PocketPy runtime")
}

fn text(output: &impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(output.as_ref()).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
