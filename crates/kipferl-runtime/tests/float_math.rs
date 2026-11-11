//! Float math must round like the host C library, which `CPython` also uses.
//!
//! `PocketPy` 2.2 replaced libm with approximations (`sqrt(9)` was
//! `2.9999999999999996`). Float `repr` round-trips, so equal output is
//! bit-for-bit equality with `CPython` on the same host.
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
    let mut command = Command::new(program);
    if program == "python3" {
        // Instrument the runtime under test, not the system CPython oracle.
        for variable in [
            "LD_PRELOAD",
            "DYLD_INSERT_LIBRARIES",
            "ASAN_OPTIONS",
            "UBSAN_OPTIONS",
            "LSAN_OPTIONS",
        ] {
            command.env_remove(variable);
        }
    }
    command.args(["-c", source]).output()
}

fn matches_cpython(source: &str) -> io::Result<()> {
    let actual = run(env!("CARGO_BIN_EXE_pocketpy-kipferl"), source)?;
    let expected = run("python3", source)?;
    for (name, output) in [("Kipferl", &actual), ("CPython", &expected)] {
        check(
            output.status.success(),
            format!(
                "{name} failed:\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )?;
    }
    check(
        actual.stdout == expected.stdout && actual.stderr.is_empty(),
        format!(
            "Kipferl and CPython float results differ:\n{}\n---\n{}",
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout)
        ),
    )
}

#[test]
fn perfect_squares_have_exact_roots() -> io::Result<()> {
    matches_cpython(
        r"
import math
for n in range(1, 5001):
    square = n * n
    assert math.sqrt(square) == n, (square, math.sqrt(square))
    assert square ** 0.5 == n, (square, square ** 0.5)
    assert float(square) ** 0.5 == n, (square, float(square) ** 0.5)
    assert math.pow(square, 0.5) == n, (square, math.pow(square, 0.5))
print(math.sqrt(9), 9 ** 0.5, math.pow(9, 0.5), math.sqrt(0.25), math.sqrt(1e300 * 1e8))
print('perfect squares are exact')
",
    )
}

#[test]
fn elementary_functions_round_like_the_c_library() -> io::Result<()> {
    matches_cpython(
        r"
import math
values = [1e-300, 1e-8, 0.1, 0.5, 0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 7.0, 10.0, 123.456, 1e8, 1e22, 1e300]
for x in values:
    print(repr(math.sqrt(x)), repr(math.exp(-x if x > 700 else x)), repr(math.log(x)),
          repr(math.log2(x)), repr(math.log10(x)), repr(math.sin(x)), repr(math.cos(x)),
          repr(math.tan(x)), repr(math.atan(x)), repr(math.atan2(x, 3.0)))
for x in [-1.0, -0.75, -0.5, -0.1, 0.0, 0.1, 0.5, 0.75, 1.0]:
    print(repr(math.asin(x)), repr(math.acos(x)))
for base, exponent in [(2.0, 0.5), (10.0, -2.0), (1.1, 10.0), (2.0, 1023.0), (0.5, 1022.0),
                       (3.0, 1.0 / 3.0), (7.0, 2.5), (1e10, 0.1), (-2.0, 3.0)]:
    print(repr(base ** exponent), repr(math.pow(base, exponent)))
print(repr(2 ** -2), repr(10 ** -3), repr(1.1 ** 10))
print(repr(math.log(1000, 10)), repr(math.log(8, 2)), repr(math.log(100, 3)))
print(repr(math.exp(1)), repr(math.e), repr(math.pi), repr(0.1 + 0.2))
print('elementary functions')
",
    )
}

#[test]
fn rounding_floor_division_and_integral_results_match_python() -> io::Result<()> {
    matches_cpython(
        r"
import math
print(1e300 // 1.0, -1e300 // 3.0, 1e20 % 7.0, divmod(1e300, 7.0), divmod(-7.5, 2))
for x in [2.5, -2.5, 2.0, -0.5, 0.49999999999999994, 1e15 + 0.5, -9.2e18]:
    print(math.floor(x), math.ceil(x), math.trunc(x), type(math.floor(x)).__name__)
print(math.fmod(7.5, -2), math.modf(-3.25), math.copysign(2, -0.0), abs(-0.0), math.fabs(-3))
print(math.isinf(math.inf), math.isnan(math.nan), math.isfinite(1e308))
print('rounding')
",
    )
}

#[test]
fn integral_results_reject_undefined_conversions() -> io::Result<()> {
    let output = run(
        env!("CARGO_BIN_EXE_pocketpy-kipferl"),
        r"
import math
for expression in ['math.floor(math.nan)', 'math.ceil(math.inf)', 'math.trunc(-math.inf)',
                   'math.floor(1e19)', 'math.ceil(-1e19)']:
    try:
        eval(expression)
    except ValueError as error:
        print(expression, error)
print(math.floor(-9223372036854775808.0), math.trunc(9.2e18))
",
    )?;
    check(
        output.status.success() && output.stderr.is_empty(),
        format!("Kipferl failed: {output:?}"),
    )?;
    check(
        output.stdout
            == b"math.floor(math.nan) cannot convert float NaN to integer
math.ceil(math.inf) cannot convert float infinity to integer
math.trunc(-math.inf) cannot convert float infinity to integer
math.floor(1e19) float too large to convert to int64
math.ceil(-1e19) float too large to convert to int64
-9223372036854775808 9200000000000000000
",
        format!(
            "unexpected conversion results:\n{}",
            String::from_utf8_lossy(&output.stdout)
        ),
    )
}

// `int()` and `round()` cast floats to int64 directly, which is undefined for
// NaN, infinities and values outside int64 (and `round(x, n)` looped `n`
// times). They raise like `math.floor`, or keep `x` when no digit fits.
#[test]
fn int_and_round_reject_undefined_conversions() -> io::Result<()> {
    let output = run(
        env!("CARGO_BIN_EXE_pocketpy-kipferl"),
        r"
for expression in ['int(float(\'nan\'))', 'int(float(\'inf\'))', 'int(-1e300)', 'int(2.0**63)',
                   'round(float(\'nan\'))', 'round(float(\'-inf\'))', 'round(1e19)']:
    try:
        eval(expression)
    except ValueError as error:
        print(expression, error)
print(int(-2.0**63), int(9.2e18), int(-0.5), round(-9.2e18), round(2.4), round(-2.6))
print(round(1e17, 2), round(1.5, 400), round(1.5, 2**62), round(-1e300, 5), round(0.0, 400))
print(round(float('inf'), 2), round(float('nan'), 2), round(3.14159, 2))
",
    )?;
    check(
        output.status.success() && output.stderr.is_empty(),
        format!("Kipferl failed: {output:?}"),
    )?;
    check(
        output.stdout
            == b"int(float('nan')) cannot convert float NaN to integer
int(float('inf')) cannot convert float infinity to integer
int(-1e300) float too large to convert to int64
int(2.0**63) float too large to convert to int64
round(float('nan')) cannot convert float NaN to integer
round(float('-inf')) cannot convert float infinity to integer
round(1e19) float too large to convert to int64
-9223372036854775808 9200000000000000000 0 -9200000000000000000 2 -3
1e+17 1.5 1.5 -1e+300 0.0
inf nan 3.14
",
        format!(
            "unexpected conversion results:\n{}",
            String::from_utf8_lossy(&output.stdout)
        ),
    )
}
