//! Everyday Python constructs, checked against the reference interpreter's output.
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
        actual.status.success() && actual.stderr.is_empty(),
        format!(
            "Kipferl failed:\n{}{}",
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&actual.stderr)
        ),
    )?;
    check(
        actual.stdout == expected.stdout,
        format!(
            "output differs\n--- CPython\n{}--- Kipferl\n{}",
            String::from_utf8_lossy(&expected.stdout),
            String::from_utf8_lossy(&actual.stdout)
        ),
    )
}

/// Both interpreters reject `source` with the same exception type and message.
fn raises_like_cpython(source: &str) -> io::Result<()> {
    let last_line = |output: &Output| {
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .rfind(|line| !line.starts_with("error: "))
            .unwrap_or_default()
            .to_owned()
    };
    let actual = run(env!("CARGO_BIN_EXE_pocketpy-kipferl"), source)?;
    let expected = run("python3", source)?;
    check(
        !expected.status.success() && !actual.status.success(),
        format!("{source:?} did not fail in both interpreters"),
    )?;
    check(
        last_line(&actual) == last_line(&expected),
        format!(
            "{source:?} raised differently\n--- CPython\n{}\n--- Kipferl\n{}",
            last_line(&expected),
            last_line(&actual)
        ),
    )
}

#[test]
fn try_statements_run_else_and_finally_clauses() -> io::Result<()> {
    matches_cpython(
        r#"
log = []
def f(mode):
    try:
        log.append("body")
        if mode == "raise":
            raise ValueError("v")
        if mode == "other":
            raise KeyError("k")
        if mode == "return":
            return "from-body"
    except ValueError as e:
        log.append("except " + str(e))
        if mode == "raise":
            return "from-except"
    else:
        log.append("else")
        return "from-else"
    finally:
        log.append("finally")
    return "end"

for mode in ["ok", "raise", "return"]:
    log.clear()
    print(mode, f(mode), log)
log.clear()
try:
    f("other")
except KeyError as e:
    print("propagated", repr(e), log)

def only_finally():
    try:
        x = 1
    finally:
        x += 1
    return x
print(only_finally())

try:
    pass
except Exception:
    print("never")
else:
    print("module else")
finally:
    print("module finally")

try: x = 1
except: pass
else: print("inline else")
finally: print("inline finally")

def else_raises():
    try:
        pass
    except ValueError:
        return "not this handler"
    else:
        raise ValueError("from else")
try:
    else_raises()
except ValueError as e:
    print("else exceptions skip the handlers:", e)
"#,
    )
}

#[test]
fn finally_runs_when_break_continue_or_return_leave_it() -> io::Result<()> {
    matches_cpython(
        r#"
def loops():
    out = []
    for i in range(4):
        try:
            if i == 1:
                continue
            if i == 3:
                break
            out.append(i)
        finally:
            out.append("f%d" % i)
    else:
        out.append("no-break")
    n = 0
    while True:
        n += 1
        try:
            if n > 2:
                break
        finally:
            out.append("w%d" % n)
    return out
print(loops())

def nested():
    out = []
    try:
        try:
            return "inner"
        finally:
            out.append("inner-finally")
    finally:
        out.append("outer-finally")
        print(out)
print(nested())

def deep(n):
    for i in range(2):
        for j in range(2):
            try:
                if j == 1:
                    return (i, j, n)
            finally:
                print("deep finally", i, j)
print(deep(5))

def finally_raises():
    try:
        return 1
    finally:
        raise RuntimeError("from finally")
try:
    finally_raises()
except RuntimeError as e:
    print("caught", e)

def try_in_finally():
    try:
        raise ValueError("a")
    finally:
        try:
            raise KeyError("b")
        except KeyError:
            print("inner handled")
try:
    try_in_finally()
except ValueError as e:
    print("outer", e)

def gen():
    try:
        yield 1
        yield 2
    finally:
        print("gen finally")
print(list(gen()))
"#,
    )
}

#[test]
fn leaving_a_with_block_early_calls_exit() -> io::Result<()> {
    matches_cpython(
        r#"
class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self
    def __exit__(self, exc_type, exc, tb):
        print("exit", self.name, exc_type, exc, tb)
        return False

def with_return():
    with CM("a"):
        for i in range(3):
            with CM("b"):
                return i
print(with_return())
for i in range(3):
    with CM("loop"):
        if i == 0:
            continue
        break
def with_finally():
    try:
        with CM("c"):
            return "w"
    finally:
        print("after with")
print(with_finally())
"#,
    )
}

#[test]
fn exit_receives_the_exception_type_value_and_traceback() -> io::Result<()> {
    matches_cpython(
        r#"
class Suppress:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc, tb):
        print("exit", exc_type, exc, tb is None or tb is not None)
        return exc_type is ValueError

with Suppress():
    print("clean body")
with Suppress():
    raise ValueError("suppressed")
try:
    with Suppress():
        raise KeyError("kept")
except KeyError as e:
    print("propagated", repr(e))

from contextlib import suppress
with suppress(ZeroDivisionError):
    1 / 0
print("suppress works")
"#,
    )
}

#[test]
fn malformed_try_statements_remain_syntax_errors() -> io::Result<()> {
    for source in [
        "try:\n    pass\n",
        "try:\n    pass\nelse:\n    pass\n",
        "try:\n    pass\nfinally:\n    pass\nexcept:\n    pass\n",
        "try:\n    pass\nelse:\n    pass\nfinally:\n    pass\n",
    ] {
        for program in [env!("CARGO_BIN_EXE_pocketpy-kipferl"), "python3"] {
            let output = run(program, source)?;
            check(
                !output.status.success()
                    && String::from_utf8_lossy(&output.stderr).contains("SyntaxError"),
                format!("{program} accepted malformed source {source:?}"),
            )?;
        }
    }
    Ok(())
}

#[test]
fn generator_expressions_are_lazy_and_scoped_like_cpython() -> io::Result<()> {
    matches_cpython(
        r#"
xs = [1, 2, 3]
print(sum(x * x for x in xs))
print(any(x > 2 for x in range(5)), all(x < 9 for x in range(5)))
print("".join(c.upper() for c in "abc"))
print(max(len(w) for w in ["a", "bbb"]), min(x for x in [5, 3]))
print(tuple(i for i in range(3)), sorted(set(i % 2 for i in range(4))))
print(next((x for x in range(10) if x > 3), None))
g = (x * 2 for x in range(3))
print(type(g).__name__, list(g), list(g))
import itertools
print(next(n for n in itertools.count(10) if n % 7 == 0))
def f(scale):
    data = [3, 1, 2]
    return sorted(v * scale for v in data if v != 2)
print(f(10))
class C:
    items = [1, 2, 3]
    total = sum(i for i in items)
print(C.total)
x = "outer"
print(list(x for x in "ab"), x)
def lazy():
    for i in range(3):
        print("produce", i)
        yield i
g = (v + 1 for v in lazy())
print("created")
print(next(g))
try:
    (y for y in 5)
except TypeError as e:
    print("TypeError", e)
print(sum(
    v
    for v in range(4)
))
"#,
    )
}

#[test]
fn comprehensions_accept_several_for_and_if_clauses() -> io::Result<()> {
    matches_cpython(
        r#"
print([(a, b) for a in range(3) for b in range(3) if a != b if a < b])
print({(a, b) for a in "xy" for b in "zz"} == {("x", "z"), ("y", "z")})
print({a + b: len(a + b) for a in ["p", "qq"] for b in ["r", "ss"]})
print(list((a, b) for a in range(2) for b in "ab"))
print([y for x in [[1, 2], [3]] for y in x])
print(list(i * j for i in range(3) if i for j in range(i)))
def matrix(n):
    return [[r * n + c for c in range(n)] for r in range(n)]
print(matrix(3), [v for row in matrix(2) for v in row])
"#,
    )
}

#[test]
fn malformed_generator_expressions_remain_syntax_errors() -> io::Result<()> {
    for source in [
        "f(x for x in y, 1)",
        "f(1, x for x in y)",
        "(x for x)",
        "[x for x in]",
    ] {
        for program in [env!("CARGO_BIN_EXE_pocketpy-kipferl"), "python3"] {
            let output = run(program, source)?;
            check(
                !output.status.success()
                    && String::from_utf8_lossy(&output.stderr).contains("SyntaxError"),
                format!("{program} accepted malformed source {source:?}"),
            )?;
        }
    }
    Ok(())
}

#[test]
fn format_specs_follow_the_cpython_mini_language() -> io::Result<()> {
    matches_cpython(
        r##"
ints = [0, 7, -7, 1234567, -1234567, 255, 2**40, True, False]
int_specs = ["", ",", "_", "d", ">10,", "<10", "^11,", "=+12,", "+", " ", "010,", "08,",
             "#x", "#X", "#o", "#b", "x", "_x", "_b", "#010_x", "e", ".2f", ",.2f", "%",
             ".1%", "*^9", "n", "g", "-^7d", "0>5", "05"]
for v in ints:
    print(v, [format(v, spec) for spec in int_specs])
print([format(v, "c") for v in [65, 233, 0x1F600]])
floats = [0.0, -0.0, 1.5, -2.25, 1234567.891, 1e16, 1e-7, 3.14159, 100.0, 12.0, 2.5,
          0.125, float("inf"), float("-inf"), float("nan"), 1e300]
float_specs = ["", ",", "_", ".2f", ",.2f", "10.3f", "<10.1f", "^12,.1f", "=+12.2f",
               "+.1f", " .1f", "010.2f", "010,.1f", "e", ".3e", "E", "g", ".3g", "G",
               "#.3g", "%", ".1%", ".3", ".1", ".2", "10", "*>12", "08", ".0f", ".0e",
               "#.0f", "F"]
for v in floats:
    print(v, [format(v, spec) for spec in float_specs])
for spec in ["", "10", "<10", ">10", "^10", "*^10", ".2", "10.2", "s", "05", "é^7"]:
    print(repr(format("héllo", spec)))
n = 1234567.891
print(f"{n:,.2f} {n:_.1f} {1234:>10,} {-1234567:,} {0:,} {42:+d} {3.5:08.3f}")
print(f"{'a':<5}|{'b':^5}|{'c':>5}| {0.256:.1%} {12345.678:e} {255:#x} {255:08b} {255:o}")
print(f"{'x'!r:>10}| {'y'!s:3}| {7!s:>3}|")
print("{:,} {:_} {:>8.2f}".format(1234567, 7654321, 3.14159))
class Money:
    def __init__(self, cents):
        self.cents = cents
    def __format__(self, spec):
        return format(self.cents / 100, spec or ".2f") + " EUR"
print(f"{Money(5):>10.1f}", format(Money(250)), "{:.0f}".format(Money(99)))
"##,
    )
}

#[test]
fn invalid_format_specs_raise_cpython_errors() -> io::Result<()> {
    for source in [
        "format(1, '.2d')",
        "format(1, ',x')",
        "format('a', '+')",
        "format('a', ',')",
        "format('a', '=')",
        "format(1.5, 'd')",
        "format('a', 'd')",
        "format(1, 'abc')",
        "format(1.5, '.')",
    ] {
        raises_like_cpython(source)?;
    }
    Ok(())
}

#[test]
fn errors_in_except_clauses_propagate_instead_of_hanging() -> io::Result<()> {
    raises_like_cpython(
        "try:\n    raise ValueError('x')\nexcept (KeyError, Undefined):\n    pass\n",
    )?;
    matches_cpython(
        r#"
def kinds():
    raise RuntimeError("no kinds")
def check():
    try:
        try:
            raise ValueError("inner")
        except kinds():
            print("never")
    except RuntimeError as e:
        print("outer caught", e)
    try:
        raise KeyError("k")
    except KeyError:
        print("still handles")
check()
"#,
    )
}

#[test]
fn integers_that_fit_64_bits_match_cpython_and_larger_ones_raise() -> io::Result<()> {
    matches_cpython(
        r#"
m = -(2**62) * 2
print(2**62, 10**18, 3**39, (-2)**63, m, m // 2, m // 3, m % 7, divmod(m, 7), m // -2)
print(-1 << 63, 1 << 62, int("-9223372036854775808"), int("000000000000000000000001"))
print(int("7fffffffffffffff", 16), int(-2.0**63), 9223372036854775807 - 1)
import math
print(math.factorial(20), sum([2**61, 2**61]))
"#,
    )?;
    let output = run(
        env!("CARGO_BIN_EXE_pocketpy-kipferl"),
        r#"
for expression in ["2**63", "2**62 * 2", "9223372036854775807 + 1", "-(-(2**62) * 2)",
                   "int('9223372036854775808')", "1 << 64", "__import__('math').factorial(21)",
                   "int(1e19)", "int(float('inf'))"]:
    try:
        eval(expression)
        print(expression, "returned")
    except OverflowError as error:
        print(expression, "OverflowError", error)
"#,
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    check(
        output.status.success() && !stdout.contains("returned") && stdout.lines().count() == 9,
        format!("integer overflow did not raise:\n{stdout}"),
    )
}
