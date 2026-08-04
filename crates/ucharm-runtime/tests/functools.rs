use std::process::{Command, Output};

#[test]
fn passes_the_zig_functools_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_functools.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 40 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_reduce_and_partial_behavior_with_compatibility_aliases() {
    let output = run(concat!(
        "import functools\n",
        "assert functools.reduce(lambda x, y: x + y, (1, 2, 3), 10) == 16\n",
        "def combine(a, b, c=0): return a + b + c\n",
        "partial = functools.partial(combine, 1, c=4)\n",
        "assert partial(2) == 7\n",
        "assert partial.func is combine\n",
        "assert partial.args == (1,)\n",
        "assert partial.keywords == {'c': 4}\n",
        "nested = functools.partial(partial, 2)\n",
        "assert nested() == 7",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn caches_recursive_keyword_and_exception_paths_with_lru_eviction() {
    let output = run(concat!(
        "import functools\n",
        "calls = 0\n",
        "@functools.lru_cache(maxsize=2)\n",
        "def cached(a, b=0):\n",
        "    global calls\n",
        "    calls += 1\n",
        "    return [a + b]\n",
        "first = cached(1, b=2)\n",
        "assert cached(1, b=2) is first\n",
        "assert cached(1, b=2) is cached(1, **{'b': 2})\n",
        "cached(2); cached(1, b=2); cached(3); cached(2)\n",
        "assert calls == 4\n",
        "assert cached.cache_info() == (4, 4, 2, 2)\n",
        "cached.cache_clear()\n",
        "assert cached.cache_info() == (0, 0, 2, 0)\n",
        "fib_calls = 0\n",
        "@functools.cache\n",
        "def fib(n):\n",
        "    global fib_calls\n",
        "    fib_calls += 1\n",
        "    if n < 2: return n\n",
        "    return fib(n - 1) + fib(n - 2)\n",
        "assert fib(10) == 55 and fib_calls == 11\n",
        "key_calls = 0\n",
        "@functools.cache\n",
        "def capture(*args, **kwargs):\n",
        "    global key_calls\n",
        "    key_calls += 1\n",
        "    return key_calls\n",
        "assert capture((), (('x', 1),)) == 1\n",
        "assert capture(x=1) == 2\n",
        "assert capture((), (('x', 1),)) == 1\n",
        "typed_calls = 0\n",
        "@functools.lru_cache(typed=True)\n",
        "def typed_identity(value):\n",
        "    global typed_calls\n",
        "    typed_calls += 1\n",
        "    return typed_calls\n",
        "assert typed_identity(1) == 1\n",
        "assert typed_identity(True) == 2\n",
        "assert typed_identity(1) == 1\n",
        "failed_calls = 0\n",
        "@functools.cache\n",
        "def fails_once(value):\n",
        "    global failed_calls\n",
        "    failed_calls += 1\n",
        "    if failed_calls == 1: raise ValueError(7)\n",
        "    return value\n",
        "try:\n",
        "    fails_once(9)\n",
        "except ValueError:\n",
        "    pass\n",
        "assert fails_once(9) == 9 and failed_calls == 2",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn compares_keys_and_preserves_wrapper_metadata() {
    let output = run(concat!(
        "import functools\n",
        "def compare(a, b): return a - b\n",
        "factory = functools.cmp_to_key(compare)\n",
        "one = factory(1)\n",
        "two = factory(2)\n",
        "assert one < two and one <= two and one != two\n",
        "assert two > one and two >= one and one == one\n",
        "assert sorted([3, 1, 2], key=factory) == [1, 2, 3]\n",
        "def original(): pass\n",
        "original.__name__ = 'kept_name'\n",
        "original.__doc__ = 'kept docs'\n",
        "@functools.wraps(original)\n",
        "def wrapper(): return 7\n",
        "assert wrapper() == 7\n",
        "assert wrapper.__name__ == 'kept_name'\n",
        "assert wrapper.__doc__ == 'kept docs'",
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn propagates_reduce_partial_comparator_and_cache_key_errors() {
    for (source, expected) in [
        (
            "import functools; functools.reduce(lambda x, y: x + y, [])",
            "TypeError: reduce() of empty sequence with no initial value",
        ),
        (
            "import functools; functools.partial(lambda x: x)()",
            "TypeError:",
        ),
        (
            concat!(
                "import functools\n",
                "key = functools.cmp_to_key(lambda a, b: 'bad')\n",
                "key(1) < key(2)",
            ),
            "TypeError:",
        ),
        (
            concat!(
                "import functools\n",
                "@functools.cache\n",
                "def identity(value): return value\n",
                "identity([])",
            ),
            "TypeError:",
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
