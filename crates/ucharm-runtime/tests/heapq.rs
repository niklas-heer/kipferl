use std::process::{Command, Output};

#[test]
fn passes_the_zig_heapq_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_heapq.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 42 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_custom_comparison_identity_and_mutation() {
    let output = run(concat!(
        "import heapq\n",
        "class Item:\n",
        "    def __init__(self, priority, name):\n",
        "        self.priority = priority\n",
        "        self.name = name\n",
        "    def __lt__(self, other):\n",
        "        scratch = [self.priority, other.priority]\n",
        "        return scratch[0] < scratch[1]\n",
        "a = Item(2, 'a')\n",
        "b = Item(1, 'b')\n",
        "c = Item(3, 'c')\n",
        "heap = [a, b, c]\n",
        "assert heapq.heapify(heap) is None\n",
        "assert heapq.heappop(heap) is b\n",
        "assert heapq.heapreplace(heap, b) is a\n",
        "assert heapq.heappushpop(heap, a) is b\n",
        "assert heapq.nlargest(1, [a, b, c])[0] is c\n",
        "assert heapq.nsmallest(1, [a, b, c])[0] is b",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_heapq_errors_bounds_and_zero_shortcut() {
    let shortcut = run("import heapq\n\
assert heapq.nlargest(0, 42) == []\n\
assert heapq.nsmallest(0, 42) == []");
    assert!(shortcut.status.success(), "{}", diagnostic(&shortcut));

    for (source, expected) in [
        (
            "import heapq; heapq.heapify()",
            "TypeError: too few arguments",
        ),
        (
            "import heapq; heapq.heapify(())",
            "TypeError: expected list",
        ),
        (
            "import heapq; heapq.heappop([])",
            "IndexError: pop from empty heap",
        ),
        (
            "import heapq; heapq.heapreplace([], 1)",
            "IndexError: heap is empty",
        ),
        (
            "import heapq; heapq.nlargest('1', [])",
            "TypeError: expected int for n",
        ),
        (
            "import heapq; heapq.nlargest(-1, [])",
            "ValueError: n must be non-negative",
        ),
        (
            "import heapq; heapq.nsmallest(1, [0] * 257)",
            "ValueError: data too large",
        ),
        (
            "import heapq; heapq.heapify([1, 'x'])",
            "TypeError: unsupported operand type(s) for '<': 'str' and 'int'",
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
