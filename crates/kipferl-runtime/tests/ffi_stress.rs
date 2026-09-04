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
