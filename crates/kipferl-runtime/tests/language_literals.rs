//! Syntax and behavior regressions for package-friendly literal/import grammar.
use std::io;
use std::process::{Command, Output};

fn check(condition: bool, message: impl std::fmt::Display) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message.to_string()))
    }
}

fn run(program: &str, source: &str) -> io::Result<Output> {
    Command::new(program).args(["-c", source]).output()
}

fn matches_cpython(source: &str) -> io::Result<()> {
    let actual = run(env!("CARGO_BIN_EXE_pocketpy-kipferl"), source)?;
    check(
        actual.status.success(),
        format!(
            "Kipferl failed:\n{}{}",
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&actual.stderr)
        ),
    )?;
    let expected = run("python3", source)?;
    check(
        expected.status.success(),
        format!(
            "CPython failed:\n{}{}",
            String::from_utf8_lossy(&expected.stdout),
            String::from_utf8_lossy(&expected.stderr)
        ),
    )?;
    check(
        actual.stdout == expected.stdout && actual.stderr.is_empty(),
        "Kipferl and CPython literal behavior differ",
    )
}

#[test]
fn parenthesized_imports_accept_trailing_commas_comments_and_newlines() -> io::Result<()> {
    matches_cpython(
        r"
from math import (sqrt as root, floor,)
from math import (
    ceil  # newline before the comma is valid inside parentheses
    , sin,
    # a comment after the final comma

)
assert root(9) == 3
assert floor(2.9) == 2
assert ceil(2.1) == 3
assert sin(0) == 0
print('imports preserve aliases and values')
",
    )
}

#[test]
fn function_and_lambda_parameter_terminators_preserve_binding() -> io::Result<()> {
    matches_cpython(
        r"
def positional(a, b,):
    return a + b
def optional(a, b=2,):
    return a + b
def starred(a, *rest,):
    return (a, rest)
def keywords(a, **options,):
    return (a, options)
def combined(a, *rest, flag=1, **options,):
    return (a, rest, flag, options)
def empty(
    # genuinely empty, not a missing parameter after a comma
):
    return 42
def multiline(a
    , b,
):
    return a * b
assert positional(2, 3) == 5
assert optional(2) == 4 and optional(2, b=5) == 7
assert starred(1, 2, 3) == (1, (2, 3))
assert keywords(1, name='ok') == (1, {'name': 'ok'})
assert combined(1, 2, 3, flag=4, name='ok') == (1, (2, 3), 4, {'name': 'ok'})
assert empty() == 42 and multiline(2, 3) == 6
assert (lambda x,: x + 1)(3) == 4
assert (lambda **kw,: kw)(name='ok') == {'name': 'ok'}
print('trailing parameter commas preserve argument semantics')
",
    )
}

#[test]
fn adjacent_literals_preserve_precedence_types_and_binary_lengths() -> io::Result<()> {
    matches_cpython(
        r"
assert 'a' 'b' * 2 == 'abab'
assert 'a' + 'b' 'c' == 'abc'
assert ['a' 'b', 'c' 'd'] == ['ab', 'cd']
assert {'a' 'b': 'c' 'd'} == {'ab': 'cd'}
assert r'\d' '\\w' == '\\d\\w'
assert 'é' '🍰' == 'é🍰'
assert len('a\x00' 'b') == 3
assert ('a\x00' 'b')[2] == 'b'
raw = b'\x00' b'\xff' b'abc'
assert len(raw) == 5 and raw[0] == 0 and raw[1] == 255
assert raw[4] == 99
print('literal concatenation preserves values')
",
    )
}

#[test]
fn adjacency_respects_statement_boundaries_and_docstrings() -> io::Result<()> {
    matches_cpython(
        r#"
value = 'left'
'right'
assert value == 'left'
joined = (
    'left'  # comment between adjacent literals

    "right"
)
assert joined == 'leftright'
sequence = [
    'one'
    'two',
    'three'
]
assert sequence == ['onetwo', 'three']
def documented():
    'joined ' 'docstring'
    return 'ok'
assert documented.__doc__ == 'joined docstring'
assert documented() == 'ok'
print('newlines and docstrings preserve semantics')
"#,
    )
}

#[test]
fn default_literals_support_adjacency_grouping_and_tuple_distinctions() -> io::Result<()> {
    matches_cpython(
        r"
def defaults(text=(
    'left'  # a grouped string is not a one-element tuple
    'right'
), raw=(b'\x00' b'\xff'), number=(7), empty=(), single=('a' 'b',), nested=(('x' 'y',), ()),):
    return (text, raw, number, empty, single, nested)
result = defaults()
assert result[0] == 'leftright'
assert len(result[1]) == 2 and result[1][1] == 255
assert result[2] == 7 and result[3] == ()
assert result[4] == ('ab',) and result[5] == (('xy',), ())
def nul(value='a\x00' 'b',):
    return value
assert len(nul()) == 3 and nul()[2] == 'b'
assert defaults(text='override')[0] == 'override'
print('default values preserve grouping and embedded NUL')
",
    )
}

#[test]
fn malformed_commas_and_mixed_literal_types_remain_syntax_errors() -> io::Result<()> {
    for source in [
        "from math import ()",
        "from math import (sqrt,,)",
        "from math import sqrt,",
        "def f(,): pass",
        "def f(x,,): pass",
        "def f(**kw, x): pass",
        "value = b'a' 'b'",
        "value = ('a'\n # mixed types\n b'b')",
        "def f(value=b'a' 'b'): pass",
        "def f(value=(,)): pass",
        "value = b'a' f'b'",
        "value = f'a' b'b'",
    ] {
        for program in [env!("CARGO_BIN_EXE_pocketpy-kipferl"), "python3"] {
            let output = run(program, source)?;
            check(
                output.status.code() == Some(1),
                format!("{program} accepted malformed source {source:?}"),
            )?;
            check(
                String::from_utf8_lossy(&output.stdout).contains("SyntaxError")
                    || String::from_utf8_lossy(&output.stderr).contains("SyntaxError"),
                format!("{program} failed without SyntaxError for {source:?}"),
            )?;
        }
    }
    Ok(())
}

#[test]
fn long_literal_runs_are_combined_without_losing_tokens_or_data() -> io::Result<()> {
    let source = format!(
        "value = {}\nassert len(value) == 10000\nassert value[9999] == 'x'\nprint('large literal run')",
        "'x' ".repeat(10000)
    );
    matches_cpython(&source)
}

#[test]
fn repeated_mixed_literal_failures_leave_the_compiler_usable() -> io::Result<()> {
    matches_cpython(
        r#"
for iteration in range(500):
    rejected = False
    try:
        compile("value = 'left' 'middle' b'wrong'", 'bad.py', 'exec')
    except SyntaxError:
        rejected = True
    assert rejected
    compile("value = 'left' 'right'", 'good.py', 'exec')
print('compiler recovers after mixed-literal errors')
"#,
    )
}
