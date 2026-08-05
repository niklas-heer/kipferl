use std::process::{Command, Output};

#[test]
fn passes_the_zig_copy_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_copy.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 33 passed, 0 failed"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_shallow_identity_constructor_fallback_and_bytearray_copying() {
    let output = run(concat!(
        "import copy\n",
        "assert copy.Error is RuntimeError\n",
        "atomic = (None, True, 3, 4.5, 'x', (1, 2))\n",
        "for value in atomic:\n",
        "    assert copy.copy(value) is value\n",
        "nested = [[1], [2]]\n",
        "shallow = copy.copy(nested)\n",
        "assert shallow == nested and shallow is not nested\n",
        "assert shallow[0] is nested[0]\n",
        "data = bytearray(b'hello')\n",
        "data_copy = copy.copy(data)\n",
        "assert data_copy == data and data_copy is not data\n",
        "assert len(data_copy) == 5\n",
        "class ConstructorFirst:\n",
        "    def __copy__(self): return 41\n",
        "value = ConstructorFirst()\n",
        "assert copy.copy(value) is not value\n",
        "class HookFallback:\n",
        "    def __init__(self): pass\n",
        "    def __copy__(self): return 43\n",
        "assert copy.copy(HookFallback()) == 43",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn repairs_circular_and_tuple_deepcopy_while_preserving_shared_identity() {
    let output = run(concat!(
        "import copy\n",
        "cycle = []\n",
        "cycle.append(cycle)\n",
        "cycle_copy = copy.deepcopy(cycle)\n",
        "assert cycle_copy is not cycle and cycle_copy[0] is cycle_copy\n",
        "left = [1]\n",
        "right = [2, left]\n",
        "left.append(right)\n",
        "left_copy = copy.deepcopy(left)\n",
        "assert left_copy is not left\n",
        "assert left_copy[1] is not right\n",
        "assert left_copy[1][1] is left_copy\n",
        "inner = []\n",
        "wrapped = (inner,)\n",
        "inner.append(wrapped)\n",
        "wrapped_copy = copy.deepcopy(wrapped)\n",
        "assert wrapped_copy is not wrapped\n",
        "assert wrapped_copy[0] is not inner\n",
        "assert wrapped_copy[0][0] is wrapped_copy\n",
        "shared = [1, 2]\n",
        "shared_copy = copy.deepcopy([shared, shared])\n",
        "assert shared_copy[0] is shared_copy[1]\n",
        "assert shared_copy[0] is not shared",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn calls_custom_deepcopy_and_copy_hooks_and_propagates_errors() {
    let output = run(concat!(
        "import copy\n",
        "class DeepHook:\n",
        "    def __deepcopy__(self, memo):\n",
        "        assert isinstance(memo, dict)\n",
        "        return [42]\n",
        "value = DeepHook()\n",
        "result = copy.deepcopy([value, value])\n",
        "assert result == [[42], [42]] and result[0] is result[1]\n",
        "class CopyHook:\n",
        "    def __copy__(self): return 43\n",
        "assert copy.deepcopy(CopyHook()) == 43\n",
        "class Unknown: pass\n",
        "unknown = Unknown()\n",
        "assert copy.deepcopy(unknown) is unknown",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));

    for (source, expected) in [
        (
            "import copy; copy.copy()",
            "TypeError: copy() takes 1 positional arguments but 0 were given",
        ),
        (
            "import copy; copy.deepcopy()",
            "TypeError: deepcopy() takes 1 positional arguments but 0 were given",
        ),
        (
            concat!(
                "import copy\n",
                "class Unsupported:\n",
                "    def __init__(self): pass\n",
                "copy.copy(Unsupported())",
            ),
            "TypeError: object does not support copy",
        ),
        (
            concat!(
                "import copy\n",
                "class Broken:\n",
                "    def __deepcopy__(self, memo): raise ValueError(7)\n",
                "copy.deepcopy(Broken())",
            ),
            "ValueError: 7",
        ),
        (
            concat!(
                "import copy\n",
                "class BrokenLookup:\n",
                "    def __getattr__(self, name): raise ValueError(9)\n",
                "copy.deepcopy(BrokenLookup())",
            ),
            "ValueError: 9",
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
