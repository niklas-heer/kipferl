use std::process::{Command, Output};

#[test]
fn passes_the_zig_typing_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_typing.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 43 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_typevar_alias_sentinel_and_identity_behavior() {
    let output = run(concat!(
        "import typing\n",
        "T = typing.TypeVar('T')\n",
        "assert T.__name__ == 'T'\n",
        "assert repr(T) == '~T'\n",
        "assert type(T) is typing.TypeVar\n",
        "assert repr(typing.TypeVar('x' * 126)) == '~' + 'x' * 126\n",
        "assert repr(typing.TypeVar('x' * 127)) == '~T'\n",
        "assert repr(typing.TypeVar(1)) == '~T'\n",
        "assert typing.List.__name__ == 'List'\n",
        "assert typing.Any is not typing.Optional\n",
        "marker = []\n",
        "assert typing.cast(list, marker) is marker\n",
        "assert typing.overload(marker) is marker\n",
        "assert typing.final(marker) is marker\n",
        "assert typing.no_type_check(marker) is marker\n",
        "assert typing.runtime_checkable(marker) is marker\n",
        "assert typing.get_args(T) == ()\n",
        "assert typing.get_origin(T) is None\n",
        "assert typing.get_type_hints(marker) == {}\n",
        "assert typing.get_type_hints(marker, {}, {}) == {}",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_typing_argument_and_alias_errors() {
    for (source, expected) in [
        (
            "import typing; typing.TypeVar()",
            "TypeError: __new__() takes 2 positional arguments but 1 were given",
        ),
        (
            "import typing; typing.cast(int)",
            "TypeError: too few arguments",
        ),
        (
            "import typing; typing.cast(int, 1, 2)",
            "TypeError: too many arguments",
        ),
        (
            "import typing; typing.List()",
            "TypeError: object.__new__(List) is not safe, use List.__new__() instead",
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
