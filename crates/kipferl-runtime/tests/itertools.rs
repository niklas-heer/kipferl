use std::process::{Command, Output};

#[test]
fn passes_the_zig_itertools_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_itertools.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 33 passed, 0 failed"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_iterator_state_identity_and_eager_legacy_results() {
    let output = run(concat!(
        "import itertools\n",
        "counter = itertools.count(10, 2)\n",
        "assert iter(counter) is counter\n",
        "assert itertools.islice(counter, 1, 5, 2) == [12, 16]\n",
        "assert next(counter) == 20\n",
        "first = []\n",
        "second = {}\n",
        "cycled = itertools.cycle([first, second])\n",
        "assert next(cycled) is first\n",
        "assert next(cycled) is second\n",
        "assert next(cycled) is first\n",
        "assert list(itertools.cycle([])) == []\n",
        "repeated = itertools.repeat(first, 2)\n",
        "assert iter(repeated) is repeated\n",
        "assert next(repeated) is first\n",
        "assert next(repeated) is first\n",
        "assert list(itertools.repeat(first, -2)) == []\n",
        "assert itertools.chain((1, 2), 'ab') == [1, 2, 'a', 'b']\n",
        "assert type(itertools.chain([1], [2])) is list\n",
        "assert itertools.takewhile(lambda x: x < 3, [1, 2, 4, 1]) == [1, 2]\n",
        "assert itertools.dropwhile(lambda x: x < 3, [1, 2, 4, 1]) == [4, 1]\n",
        "def nested(value):\n",
        "    assert itertools.takewhile(lambda item: item < 2, [1, 3]) == [1]\n",
        "    return value < 3\n",
        "assert itertools.takewhile(nested, [1, 2, 4]) == [1, 2]",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_itertools_binding_restriction_and_predicate_errors() {
    for (source, expected) in [
        (
            "import itertools; itertools.count(1, 2, 3)",
            "TypeError: too many arguments (count)",
        ),
        (
            "import itertools; itertools.cycle()",
            "TypeError: cycle() takes 1 positional arguments but 0 were given",
        ),
        (
            "import itertools; itertools.cycle((1, 2))",
            "TypeError: cycle() argument must be list or string",
        ),
        (
            "import itertools; itertools.repeat()",
            "TypeError: repeat() takes 1 positional arguments but 0 were given",
        ),
        (
            "import itertools; itertools.chain([1], 2)",
            "TypeError: chain() arguments must be iterable",
        ),
        (
            "import itertools; itertools.islice([1])",
            "TypeError: islice() requires 1 to 3 positional args after iterable",
        ),
        (
            "import itertools; itertools.islice([1], 0, 1, 0)",
            "ValueError: step must be >= 1",
        ),
        (
            "import itertools; itertools.takewhile(lambda x: x, (1, 2))",
            "TypeError: takewhile() iterable must be a list",
        ),
        (
            "import itertools; itertools.dropwhile(lambda x: x, (1, 2))",
            "TypeError: dropwhile() iterable must be a list",
        ),
        (
            "import itertools; itertools.takewhile(lambda x: 1 / 0, [1])",
            "ZeroDivisionError: float division by zero",
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
