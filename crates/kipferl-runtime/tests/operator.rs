use std::process::{Command, Output};

#[test]
fn passes_the_zig_operator_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_operator.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 115 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_numeric_identity_sequence_and_call_helpers() {
    let output = run(concat!(
        "import operator\n",
        "assert operator.pos(-7) == -7\n",
        "assert operator.abs(-7.5) == 7.5\n",
        "assert operator.index(7) == 7\n",
        "assert operator.inv(0) == -1\n",
        "assert operator.is_none(None) and not operator.is_none(0)\n",
        "assert operator.is_not_none(0) and not operator.is_not_none(None)\n",
        "left = [1, 2]\n",
        "assert operator.concat(left, [3]) == [1, 2, 3]\n",
        "assert left == [1, 2]\n",
        "assert operator.countOf([1, 2, 2], 2) == 2\n",
        "assert operator.indexOf([1, 2, 2], 2) == 1\n",
        "assert operator.ipow(2, 8) == 256\n",
        "assert operator.iconcat(left, [3]) is left\n",
        "assert left == [1, 2, 3]\n",
        "self_concat = [1, 2]\n",
        "assert operator.iconcat(self_concat, self_concat) is self_concat\n",
        "assert self_concat == [1, 2, 1, 2]\n",
        "assert operator.length_hint([1, 2]) == 2\n",
        "assert operator.length_hint('界') == 3\n",
        "assert operator.length_hint(iter([]), 9) == 9\n",
        "assert operator.call(lambda a, b: a + b, 2, 3) == 5",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn roots_getter_results_and_supports_nested_and_keyword_calls() {
    let output = run(concat!(
        "import operator\n",
        "class Point:\n",
        "    def __init__(self, x, y): self.x = x; self.y = y\n",
        "    def shifted(self, amount=0): return self.x + amount\n",
        "class Holder:\n",
        "    def __init__(self, point): self.point = point\n",
        "point = Point(3, 4)\n",
        "holder = Holder(point)\n",
        "assert operator.itemgetter('a', 'b')({'a': [1], 'b': [2]}) == ([1], [2])\n",
        "assert operator.attrgetter('point.x', 'point.y')(holder) == (3, 4)\n",
        "assert operator.methodcaller('shifted', amount=5)(point) == 8\n",
        "getters = []\n",
        "for i in range(2000):\n",
        "    getters.append(operator.itemgetter(0, 1)([[i], [i + 1]]))\n",
        "assert getters[0] == ([0], [1])\n",
        "assert getters[-1] == ([1999], [2000])",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_extension_errors_and_limits() {
    for (source, expected) in [
        (
            "import operator; operator.index(1.5)",
            "TypeError: 'index' requires an integer",
        ),
        (
            "import operator; operator.concat(1, 2)",
            "TypeError: can only concatenate sequences",
        ),
        (
            "import operator; operator.countOf((1, 2), 1)",
            "TypeError: expected list",
        ),
        (
            "import operator; operator.indexOf([1], 2)",
            "ValueError: sequence.index(x): x not in sequence",
        ),
        (
            "import operator; operator.itemgetter()",
            "TypeError: itemgetter requires at least one argument",
        ),
        (
            "import operator; operator.attrgetter()",
            "TypeError: attrgetter requires at least one argument",
        ),
        (
            "import operator; operator.methodcaller(1)(object())",
            "TypeError: method name must be string",
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
