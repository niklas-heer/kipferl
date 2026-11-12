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

/// An exit from a finally clause supersedes a pending return, even inside loops and `with`.
#[test]
fn exits_from_finally_override_a_pending_return() -> io::Result<()> {
    matches_cpython(
        r#"
def a():
    for x in [1,2]:
        try:
            return 1
        finally:
            break
    return "after"
print(a())
class M:
    def __enter__(self): return self
    def __exit__(self, *a): print("exit"); return False
def b():
    with M():
        try:
            return 1
        finally:
            return 2
print(b())
def c():
    for i in range(3):
        try:
            return i
        finally:
            continue
    return "c-end"
print(c())
def d():
    for i in range(3):
        with M():
            try:
                try:
                    return "inner"
                finally:
                    for j in range(2):
                        try:
                            return "x"
                        finally:
                            if j == 0:
                                continue
                            break
            finally:
                print("outer finally", i)
                if i < 2:
                    continue
    return "d-end"
print(d())
def e():
    while True:
        try:
            try:
                return 1
            finally:
                return 2
        finally:
            break
    return 3
print(e())
def g():
    total = 0
    for i in range(100000):
        try:
            return i
        finally:
            total += 1
            continue
    return total
print(g())
def h():
    with M():
        with M():
            try:
                return 1
            finally:
                with M():
                    return 5
print(h())
def gen():
    for i in range(3):
        try:
            yield i
            return i
        finally:
            continue
print(list(gen()))
"#,
    )
}

/// A pending return that an exception or an exit from finally discards leaves no value behind.
#[test]
fn superseded_returns_do_not_accumulate_on_the_stack() -> io::Result<()> {
    matches_cpython(
        r#"
def a():
    for i in range(50000):
        try:
            try:
                return 1
            finally:
                raise ValueError
        except ValueError:
            pass
    return "ok"
print(a())
def b():
    for i in range(3):
        try:
            raise KeyError
        finally:
            break
    return "b"
print(b())
def c():
    for k in range(20000):
        for i in range(2):
            try:
                return i
            finally:
                break
    return "c"
print(c())
"#,
    )
}

/// An exit compiled after a sibling exit still sees only the pending returns
/// that no enclosing exit has discarded, so it never pops the loop iterator.
#[test]
fn later_exits_do_not_discard_superseded_returns_again() -> io::Result<()> {
    matches_cpython(
        r#"
def f():
    for i in range(3):
        try:
            try:
                return 3
            finally:
                return 4
        finally:
            if i == 2:
                break
            continue
    return "end"
print(f())
def g():
    for i in range(3):
        try:
            try:
                return 3
            finally:
                if i == 0:
                    continue
                return 4
        finally:
            if i == 1:
                continue
            if i == 5:
                break
            return i
    return "end"
print(g())
def h(flag):
    try:
        try:
            return 1
        finally:
            return 2
    finally:
        if flag:
            return 3
        return 4
print(h(True), h(False), h(0))
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

#[test]
fn dicts_accept_mappings_pairs_and_keywords() -> io::Result<()> {
    matches_cpython(
        r#"
from collections import defaultdict, Counter
print(dict(zip("ab", range(2))), dict({"a": 1}), dict([("a", 1)], b=2), dict(a=1, b=2), dict())
print(dict((k, v) for k, v in [("x", 1)]), dict([["k", "v"]]), dict(["ab", "cd"]))
d = defaultdict(list)
d["a"].append(1)
print(dict(d), dict(Counter("aab")))
d = {"a": 1}
d.update(b=2)
d.update([("c", 3)])
d.update({"d": 4}, e=5)
print(d, d.setdefault("a", 9), d.setdefault("z", []), d)
print(dict.fromkeys("ab"), dict.fromkeys(["x", "y"], 0))
x = {"a": 1, "b": 2}
y = {"b": 3, "c": 4}
print(x | y, y | x, x)
x |= y
print(x)
class Mapping:
    def keys(self):
        return ["p", "q"]
    def __getitem__(self, key):
        return key.upper()
print(dict(Mapping()))
for bad in [lambda: dict([(1, 2, 3)]), lambda: dict(1, 2), lambda: {}.update({}, {})]:
    try:
        bad()
    except (TypeError, ValueError) as e:
        print(type(e).__name__, e)
"#,
    )
}

#[test]
fn typing_forms_work_in_aliases_and_classes() -> io::Result<()> {
    matches_cpython(
        r#"
from typing import (Any, Callable, ClassVar, Dict, Final, List, Literal, NamedTuple, NewType,
                    Optional, Tuple, TypedDict, Union)
MaybeInt = Optional[int]
Handler = Callable[[int], Optional[str]]
Pair = Tuple[int, ...]
Mode = Literal["r", "w"]
Rows = Optional[List[Dict[str, Any]]]
print(Any, Optional)
class Point(NamedTuple):
    x: int
    y: int = 0
p = Point(1)
print(p, p.x, p[1], list(p), p == Point(1, 0), p._asdict(), p._replace(y=5), len(p))
print(Point(y=2, x=3), p._fields, hash(Point(1)) == hash(Point(1)))
for bad in [lambda: Point(), lambda: Point(1, 2, 3), lambda: Point(1, q=2)]:
    try:
        bad()
    except TypeError:
        print("TypeError")
class Movie(TypedDict):
    name: str
    year: int
m = Movie(name="Up", year=2009)
print(m["name"], m, isinstance(m, dict))
UserId = NewType("UserId", int)
print(UserId(5))
def f(x: Optional[int] = None, y: Union[int, str] = 1) -> Optional[str]:
    return None if x is None else str(x)
print(f(), f(3))
try:
    Optional()
except TypeError as e:
    print("TypeError", e)
"#,
    )
}

#[test]
fn any_iterable_unpacks_into_nested_and_starred_targets() -> io::Result<()> {
    matches_cpython(
        r#"
a, b = map(int, "3 4".split())
print(a + b)
x, y = (i * 2 for i in range(2))
print(x, y)
first, *middle, last = [1, 2, 3, 4, 5]
print(first, middle, last)
*init, tail = "abc"
print(init, tail)
head, *rest = range(3)
print(head, rest)
a, *b, c = (1, 2)
print(a, b, c)
p, q = "xy"
print(p, q)
for i, (key, value) in enumerate([("a", 1), ("b", 2)]):
    print(i, key, value)
for (a, b), c in [((1, 2), 3)]:
    print(a, b, c)
print([n for n, (s, t) in [(1, (2, 3))]])
from typing import NamedTuple
class Point(NamedTuple):
    x: int
    y: int
px, py = Point(5, 6)
print(px, py)
for bad in ["a, b = [1, 2, 3]", "a, b, c = [1, 2]", "a, *b, c = [1]"]:
    try:
        exec(bad)
    except ValueError as e:
        print("ValueError", e)
"#,
    )
}

#[test]
fn exceptions_carry_attributes_arguments_and_causes() -> io::Result<()> {
    matches_cpython(
        r#"
class AppError(Exception):
    def __init__(self, message, code=1):
        super().__init__(message)
        self.code = code
try:
    raise AppError("boom", code=3)
except AppError as e:
    print(e, repr(e), e.code, e.args)
e = ValueError("a", 1)
print(e.args, str(e), repr(e))
print(repr(ValueError()), str(ValueError()), ValueError("x").args, repr(KeyError("k")), str(KeyError("k")))
v = ValueError("v")
v.detail = "extra"
print(v.detail, v.__cause__, v.__suppress_context__)
def parse(text):
    try:
        return int(text)
    except ValueError as err:
        raise RuntimeError("bad number") from err
try:
    parse("x")
except RuntimeError as e:
    print(type(e.__cause__).__name__, e.__suppress_context__)
try:
    raise KeyError("k") from None
except KeyError as e:
    print(e.__cause__, e.__suppress_context__)
try:
    raise ValueError from KeyError
except ValueError as e:
    print(repr(e.__cause__))
class Wrapped(Exception):
    pass
w = Wrapped("a", "b")
print(w.args, str(w))
"#,
    )
}

#[test]
fn print_writes_to_files_and_ne_defaults_to_not_eq() -> io::Result<()> {
    matches_cpython(
        r#"
import sys, io
print("to stdout", file=sys.stdout)
print("a", "b", sep=None, end=None)
buffer = io.StringIO()
print("x", 1, sep="-", file=buffer)
print("y", file=buffer, end="")
print(repr(buffer.getvalue()))
print("flushed", file=sys.stdout, flush=True)
class P:
    def __init__(self, x):
        self.x = x
    def __eq__(self, other):
        if not isinstance(other, P):
            return NotImplemented
        return self.x == other.x
    def __hash__(self):
        return hash(self.x)
print(P(1) == P(1), P(1) != P(1), P(1) != P(2), P(1) != 5, len({P(1), P(1)}))
class Q(P):
    def __eq__(self, other):
        return True
print(Q(1) != Q(2))
print("a"); sys.stdout.write("b\n"); print("c", end=""); sys.stdout.write("-d\n"); print("e")
"#,
    )?;
    let output = run(
        env!("CARGO_BIN_EXE_pocketpy-kipferl"),
        "import sys\nprint('out')\nprint('err', 'line', sep='-', file=sys.stderr)\n",
    )?;
    check(
        output.stdout == b"out\n" && output.stderr == b"err-line\n",
        format!("print(file=sys.stderr) wrote {output:?}"),
    )
}

#[test]
fn with_items_parameter_markers_and_digit_separators_compile() -> io::Result<()> {
    matches_cpython(
        r#"
print(1_000_000, 0x_ff, 0b1010_1010, 1_000.5, 1e1_0, 1_0j if False else 0, int("1_000"), int(" 42 "), int("\t-7\n"), int("ff_ff", 16))
for bad in ["int('1__0')", "int('_1')", "int('1_')", "int('')", "int('1 2')"]:
    try:
        eval(bad)
    except ValueError as e:
        print("ValueError", e)
class CM:
    def __init__(self, name):
        self.name = name
    def __enter__(self):
        print("enter", self.name)
        return self.name
    def __exit__(self, *exc):
        print("exit", self.name, exc[0])
        return False
with CM("a") as a, CM("b") as b:
    print(a, b)
with CM("x"), CM("y"), CM("z") as z:
    print(z)
try:
    with CM("p"), CM("q"):
        raise ValueError("boom")
except ValueError:
    print("caught")
def kw(a, *, sep=", ", end="."):
    return sep.join(a) + end
print(kw(["x", "y"]), kw(["x"], end="!"))
def required(*, key):
    return key
print(required(key=3))
def pos(a, b, /, c=1):
    return a + b + c
print(pos(1, 2), pos(1, 2, c=3))
def mixed(a, /, b, *, c=0, **rest):
    return a, b, c, rest
print(mixed(1, 2, c=3, d=4))
print((lambda x, /, y=2, *, z=3: x + y + z)(1))
def star_then(*args, key=None, flag):
    return args, key, flag
print(star_then(1, 2, flag=True))
"#,
    )?;
    for source in [
        "x = 1__0",
        "x = 1_",
        "def f(a=1, b): pass",
        "def f(*a, *b): pass",
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
fn none_is_hashable_and_usable_as_a_key() -> io::Result<()> {
    matches_cpython(
        r#"
groups = {}
for name, team in [("a", None), ("b", "red"), ("c", None)]:
    groups.setdefault(team, []).append(name)
print(groups, groups[None], None in {None}, len({None, None}), {(None, 1): 3})
print(hash(None) == hash(None), None == None, None != None, None == 0)
print(NotImplemented in {NotImplemented: 1})
"#,
    )
}

#[test]
fn everyday_builtins_and_methods_match_cpython() -> io::Result<()> {
    matches_cpython(
        r#"
print(pow(2, 10), pow(2, 10, 1000), pow(3, 200, 1000000007), pow(2.0, 0.5), pow(7, 0, 1))
print(sum([1, 2, 3]), sum([[1], [2]], []), sum([0.5, 0.25], 1), sum(x for x in range(4)))
print(min([3, 1, 2]), max(3, 1, 2), min([], default=None), max([], default=-1), max("apple", "fig", key=len), min([(2, "b"), (1, "z")]))
print(max([1, 3, 2], key=lambda v: -v), min("cab"), max(range(5), default=9))
print(list(zip([1, 2], "ab")), list(zip()), list(zip([1, 2, 3], "ab")), list(zip([1, 2], [3, 4], strict=True)))
for args in [([1, 2, 3], "ab"), ([1], "ab"), ([1, 2], [3, 4], [5])]:
    try:
        list(zip(*args, strict=True))
        print("ok")
    except ValueError as e:
        print("ValueError", e)
print(round(0.5), round(1.5), round(2.5), round(-2.5), round(-0.5), round(3.7), round(-3.2))
print(round(2.675, 2), round(1.005, 2), round(0.125, 2), round(3.14159, 3), round(1234.5, -2), round(1250, -2), round(1350, -2), round(17, 0), round(-1250, -2))
print("foobar".removeprefix("foo"), "foobar".removesuffix("bar"), "foo".removeprefix(""), "a,b,c".partition(","), "a,b,c".rpartition(","), "abc".partition("x"), "abc".rpartition("x"))
print("hello world".title(), "they're bill's".title(), "aBc".swapcase(), "x".center(5, "*"), "ab".center(5), "abc".center(6), "hELLO wORLD".capitalize(), "a\tb".expandtabs(4))
print("a b  c ".split(), " a b c ".split(None, 1), "a,b,,c".split(","), "a,b,c".split(",", 1), "a b c".rsplit(None, 1), "  a  b  ".rsplit(), "a,b,c".rsplit(",", 1), "".split(","), "".split())
print("aaa".replace("a", "b", 2), "aaa".replace("a", "b"), "ab".replace("", "-"), "ab".replace("", "-", 2))
print({1} <= {1, 2}, {1, 2} < {1, 2}, {1, 2} >= {1}, {1, 2} > {1, 2}, {1} <= {2})
s = {1, 2, 3}
s.difference_update({1})
s.intersection_update({2, 3, 4})
s.symmetric_difference_update({3, 5})
print(sorted(s), len({7}), {8}.pop())
print((2.0).is_integer(), (2.5).is_integer(), (5).to_bytes(2, "big"), (5).to_bytes(2, "little"), (-1).to_bytes(2, "big", signed=True), int.from_bytes(b"\x01\x00", "big"), int.from_bytes(b"\xff\xfe", "big", signed=True), (255).bit_count())
print(b"\x00\xffab".hex(), b"\x01\x02".hex(":"))
"#,
    )
}

#[test]
fn percent_and_str_format_match_cpython() -> io::Result<()> {
    matches_cpython(
        r#"
print('%5.1f|%-5s|%05d|%x|%+d|%3s|%.2s|%r|%%' % (3.14159, 'ab', 42, 255, 7, 'x', 'abc', 'q'))
print('%s' % 'one', '%s and %s' % ('a', 'b'), '%d items' % 3, '%i' % 7.9, '%.3f' % 2, '%e' % 12345.678, '%g' % 0.0001)
print('%(name)s is %(age)d years' % {'name': 'Ann', 'age': 30}, '%s' % {'k': 1}, '%(x)05.1f' % {'x': 2.25})
print('%*d|%-*d|%.*f' % (5, 42, 4, 7, 2, 3.14159), '%#x %#o %X' % (255, 8, 255), '%c%c' % (72, 'i'), '% d' % 5)
print('%-8s|%8s|' % ('left', 'right'), '%3d%%' % 50, '%s' % None, '%r' % 'it\'s', '%s %s' % (True, [1, 2]))
for bad in ["'%d' % 'x'", "'%s %s' % ('a',)", "'%s' % ('a', 'b')", "'%z' % 1", "'%(k)s' % {}"]:
    try:
        eval(bad)
    except (TypeError, ValueError, KeyError) as e:
        print(type(e).__name__)
print('%05d|%-6d|%+06.1f|%08.3e|%5x|%-5o|%05s' % (-42, -3, -2.5, -1234.5, 255, 8, 'ab'))
class P:
    def __init__(self):
        self.x = 3
        self.items = {"k": [10, 20]}
p = P()
print("{name}:{0:>4}".format(7, name="n"), "{} {}".format("a", "b"), "{1}{0}".format("x", "y"))
print("{0.x} {0.items[k][1]} {d[a]} {lst[0]}".format(p, d={"a": 1}, lst=[5]))
print("{!r} {!s:>5} {x!r}".format("q", "w", x=[1]), "{:{w}.{p}f}|{:>{}}".format(3.14159, "z", 4, w=8, p=2))
print("{{literal}} {0}".format(1), "{:,}".format(1234567), "{name:*^9}".format(name="mid"))
print("{a} and {b}".format_map({"a": 1, "b": 2}))
class Default(dict):
    def __missing__(self, key):
        return "?"
print("{a} {b}".format_map(Default(a=1)))
for bad in ["'{}{1}'.format(1, 2)", "'{0}{}'.format(1, 2)", "'{x}'.format()", "'{2}'.format(1)", "'{'.format()", "'}'.format()", "'{!z}'.format(1)"]:
    try:
        eval(bad)
    except (ValueError, KeyError, IndexError) as e:
        print(type(e).__name__, e)
print(repr("it's"), repr('a"b'), ["x", "y'"])
"#,
    )
}

#[test]
fn displays_unpack_starred_items_and_mappings() -> io::Result<()> {
    matches_cpython(
        r#"
a = [1, 2]
b = (3,)
print([*a, *b, 4], (*a, 5), {*a, *b}, [*range(3)], [*"ab", *{"k": 1}])
d1 = {"a": 1, "b": 2}
d2 = {"b": 3}
print({**d1, **d2}, {**d1, "c": 4}, {"z": 0, **d2}, {**{}}, {**dict(x=1)})
def f(*args, **kwargs):
    return args, kwargs
print(f(*a, *b), f(**d1, c=5))
first, *rest = [*a, *b]
print(first, rest)
print([*(x * 2 for x in a)], (*[],))
"#,
    )
}

#[test]
fn json_dumps_indents_with_sorted_keys_and_any_indent() -> io::Result<()> {
    matches_cpython(
        r#"
import json
data = {"b": [1, 2.5, None, True], "a": {"nested": {"x": []}}, "c": {}, "d": "e"}
print(json.dumps(data, indent=2))
print(json.dumps(data, indent=2, sort_keys=True))
print(json.dumps(data, indent="\t", sort_keys=True))
print(json.dumps(data, sort_keys=True), json.dumps(data, separators=(",", ":")))
print(json.dumps({1: 2, "x": {"y": 1}, None: 0}), json.dumps([1, {"k": [2]}], indent=0))
"#,
    )
}

/// Building a dict hashes keys whose `__hash__` may change the source or the return register.
#[test]
fn dict_construction_copies_keys_and_values_before_hashing() -> io::Result<()> {
    matches_cpython(
        r#"
print(dict.fromkeys([(1, 2), (3, 4)]))
print(dict.fromkeys(["a", (5, 6)], 0))
class Box:
    def __init__(self, d): self.d = d
    def keys(self): return list(self.d.keys())
    def __getitem__(self, k): return self.d[k]
m = Box({(1, 2): "x", "k": (3, 4)})
print(dict(m))
print({**m})
print(dict([((1, 2), (3, 4))]))
class Evil:
    def __init__(self, src, n): self.src = src; self.n = n
    def __hash__(self):
        self.src.clear()
        for i in range(100):
            self.src[("filler", i)] = i
        return self.n
    def __eq__(self, other): return self is other
src = {}
for i in range(20):
    src[Evil(src, i)] = list(range(i))
try:
    d = dict(src)
    print("dict len", type(d).__name__)
except RuntimeError as e:
    print("RuntimeError")
class PairMutator:
    def __init__(self, pair): self.pair = pair
    def __hash__(self):
        self.pair.clear()
        self.pair.extend([object() for _ in range(50)])
        return 1
    def __eq__(self, other): return self is other
pair = []
k = PairMutator(pair)
pair.extend([k, "value"])
d = dict([pair])
print(list(d.values()))
src2 = {}
for i in range(20):
    src2[Evil(src2, i)] = i
print(len(src2 | {}) >= 0, len({**src2}) >= 0)
"#,
    )
}

/// Booleans format as the integers they are, and precision is not capped.
#[test]
fn percent_and_format_handle_bools_and_long_precisions() -> io::Result<()> {
    matches_cpython(
        r#"
print(format(1.0, '.201f'))
print(format(1e300, '.300f'))
print(format(-2.5, '.250e'))
print(format(2.5, '.250g'), format(2.5, '.250'), format(1/3, '.300'))
print(format(0.125, '.400%'))
print(f"{1e308:,.220f}")
print(len(format(1.5, '.100000f')))
print("%.300f" % 0.1)
print("%f %.1f %c %e" % (True, False, True, True))
print("%5.2f|%-6.1e|%c" % (True, True, False) == "%5.2f|%-6.1e|%c" % (1, 1, 0))
print("%f %d %x %e %g %s %r %i %o %X %c" % (True, True, True, False, True, True, False, True, True, True, True))
print("%.2f" % False, "%5.1e" % True)
print(float(True), "{:f}".format(True), format(True, ".1f"), format(True, "d"), format(True, "x"))
"#,
    )
}

/// Ranges, bit lengths, gcd and bool arithmetic stay exact up to the 64-bit limits.
#[test]
fn integer_edges_stay_exact_at_the_64_bit_limits() -> io::Result<()> {
    matches_cpython(
        r#"
import sys, math
M = sys.maxsize
N = -M - 1
print(list(range(M - 1, M, 2)), list(range(N + 1, N, -2)), list(range(M - 2, M, 1)))
print(list(range(N, N + 3)), list(range(M, M - 3, -1)), list(range(M - 5, M, 4)))
print(N.bit_length(), M.bit_length(), (-1).bit_length(), (0).bit_length())
print(math.gcd(N, M), math.gcd(N, 2), math.gcd(-12, 18), math.gcd(0, 0), math.gcd(N, 6))
print(True + 1, True - M, True * M, False * N, M - True, N + True, sum([M - 1, True]))
print(issubclass(OverflowError, ArithmeticError), issubclass(ZeroDivisionError, ArithmeticError))
print(issubclass(ArithmeticError, Exception), ArithmeticError.__name__)
for f in (lambda: 1 // 0, lambda: 1 % 0, lambda: 1.0 / 0):
    try:
        f()
    except ArithmeticError as e:
        print("arithmetic", type(e).__name__)
try:
    raise ArithmeticError("x")
except Exception as e:
    print(type(e).__name__, e)
print(M == 2**62 - 1 + 2**62, N == -2**62 - 2**62)
"#,
    )
}

/// Modular `pow` and `round` with negative digits stay exact when only intermediates exceed 64 bits.
#[test]
fn modular_pow_and_negative_rounding_avoid_intermediate_overflow() -> io::Result<()> {
    matches_cpython(
        r"
import sys
M = sys.maxsize
print(pow(2**32, 1, 2**40), pow(2**32, 2, 2**40), pow(3, 10**18, M), pow(M - 1, M - 1, M))
print(pow(2**62, 3, M), pow(-7, 5, 2**61 - 1), pow(5, 3, -7), pow(5, 0, -7), pow(6, 2, -4))
print(pow(7, 0, 1), pow(0, 0, 5), pow(12345678901, 98765, 10**18 + 9), pow(-3, 3, 10))
print(round(1, -20), round(-M, -20), round(M // 2, -19), round(5 * 10**18, -19), round(-5 * 10**18, -19))
print(round(-M - 1, -25), round(4_999_999_999_999_999_999, -19), round(1234, -2), round(1250, -2))
",
    )
}

/// Positional parameters take keywords, `/` and `*` restrict them, and binding errors match the reference interpreter.
#[test]
fn arguments_bind_by_position_and_keyword_like_cpython() -> io::Result<()> {
    matches_cpython(
        r#"
def f(a, b): return (a, b)
def g(a, b=1): return (a, b)
def h(a, /, b, *, c, d=4): return (a, b, c, d)
def k(*, c): return c
def m(a): return a
def n(a, b, c, /): return (a, b, c)
def q(a, b=2, /, c=3, *args, d, e=5, **kw): return (a, b, c, args, d, e, kw)
def r(a, b=2, *args): return (a, b, args)
calls = ["f(1,2,3)", "g(1,2,3)", "m(1,2)", "k(1)", "h(1, 2)", "h(a=1, b=2, c=3)", "f(1, a=2)",
         "f()", "n(1)", "h(1)", "h(1, b=2)", "n(a=1,b=2,c=3)", "g()", "k()", "m(1, a=1)",
         "f(1, b=2)", "f(b=2, a=1)", "g(a=5)", "h(1, b=2, c=3)", "h(1, 2, c=3, d=0)", "k(c=9)",
         "n(1, 2, 3)", "q(1, d=4)", "q(1, 2, 3, 4, 5, d=6, z=7)", "q(1, a=2, b=3, d=4)",
         "r(1)", "r(1, 2, 3, 4)", "r(1, b=5)", "m(a=3)", "q(1, 2, c=3, d=4, e=6)",
         "(lambda x, /, y=2, *, z: (x, y, z))(1, z=3)", "(lambda *, z: z)(z=1)", "f(*[1, 2])",
         "f(**{'a': 1, 'b': 2})", "g(1, **{'b': 7})", "q(1, *[2, 3, 4], d=1)"]
for call in calls:
    try:
        print(call, "=", eval(call))
    except TypeError as e:
        print(call, "->", e)
def p(a, /, **kw): return a, kw
print(p(1, a=2))
class C:
    def method(self, a, *, key=None): return (a, key)
    @staticmethod
    def st(x, /): return x
c = C()
print(c.method(1, key=2), c.method(a=3), C.st(4))
def gen(a, *, b):
    yield a
    yield b
print(list(gen(1, b=2)))
try:
    list(gen(1, 2))
except TypeError as e:
    print(e)
import inspect
for fn in (f, g, h, k, n, q, r, p):
    print(fn.__name__, inspect.signature(fn))
"#,
    )
}

/// Misplaced `/` markers are rejected with the reference interpreter's messages.
#[test]
fn misplaced_parameter_markers_are_syntax_errors() -> io::Result<()> {
    for source in [
        "def f(a, /, /, b): pass",
        "def f(*, a, /): pass",
        "def f(/, a): pass",
        "def f(a, *, b=1, /): pass",
    ] {
        raises_like_cpython(source)?;
    }
    Ok(())
}

/// Augmented assignment calls `__iadd__`/`__ior__` first, so containers change in place.
#[test]
fn augmented_assignment_mutates_containers_in_place() -> io::Result<()> {
    matches_cpython(
        r#"
def add(d):
    d |= {"b": 2}
def extend(items):
    items += [3, 4]
def grow(s):
    s |= {9}
    s &= {9, 1}
    s -= {1}
    s ^= {5}
d = {"a": 1}
add(d)
items = [1, 2]
extend(items)
s = {1, 2, 9}
grow(s)
print(d, items, sorted(s))
alias = items
items += "xy"
print(alias is items, alias)
t = frozen = "ab"
t += "c"
print(t, frozen)
n = 5
n += 2
text = "a"
text += "b"
print(n, text)
class Acc:
    def __init__(self): self.total = 0
    def __iadd__(self, other):
        self.total += other
        return self
    def __add__(self, other):
        return "fresh"
a = Acc()
b = a
a += 5
print(a is b, a.total)
class OnlyAdd:
    def __add__(self, other): return "sum"
o = OnlyAdd()
o += 1
print(o)
class NotImpl:
    def __iadd__(self, other): return NotImplemented
    def __add__(self, other): return "fallback"
x = NotImpl()
x += 1
print(x)
class Holder:
    def __init__(self): self.items = []
h = Holder()
ref = h.items
h.items += [1]
grid = {"k": [0]}
cell = grid["k"]
grid["k"] += [1]
print(ref, cell, h.items is ref)
m = {"x": 1}
m |= [("y", 2)]
print(m)
try:
    items += 5
except TypeError as e:
    print("TypeError", e)
"#,
    )
}

/// A bare `raise` nested in a handler or a `finally` clause re-raises the
/// exception being handled, which a `try` entered since may still catch.
#[test]
fn bare_raise_finds_the_exception_from_nested_blocks() -> io::Result<()> {
    matches_cpython(
        r#"
class CM:
    def __enter__(self): return self
    def __exit__(self, *exc): return False
def with_in_finally():
    try:
        raise ValueError(2)
    finally:
        with CM():
            raise
def finally_in_handler():
    try:
        raise KeyError(1)
    except KeyError:
        try:
            raise ValueError(3)
        finally:
            raise
def nested_in_handler(mode):
    try:
        raise KeyError(1)
    except KeyError:
        try:
            if mode == 0:
                for _ in range(2):
                    raise
            elif mode == 1:
                try:
                    raise ValueError(5)
                except ValueError:
                    pass
                raise
            elif mode == 2:
                with CM():
                    raise
            else:
                try:
                    raise
                except KeyError as e:
                    return ("caught inside", e.args)
        except KeyError as e:
            return ("caught", e.args)
for function in (with_in_finally, finally_in_handler):
    try:
        function()
    except Exception as e:
        print(type(e).__name__, e)
for mode in range(4):
    print(nested_in_handler(mode))
try:
    raise
except RuntimeError as e:
    print("RuntimeError", e)
"#,
    )
}

/// A bare `raise` in `finally`, unpacking that stops at the first excess item, and a missing `flush`.
#[test]
fn bare_raise_unpacking_and_print_flush_follow_cpython() -> io::Result<()> {
    matches_cpython(
        r#"
def f():
    try:
        raise ValueError("x")
    finally:
        raise
try:
    f()
except ValueError as e:
    print("ValueError", e)
import itertools
try:
    a, b = itertools.count()
except ValueError as e:
    print(e)
class W:
    def write(self, s): pass
try:
    print("x", file=W(), flush=True)
except AttributeError as e:
    print("AttributeError")
def g():
    try:
        pass
    finally:
        raise
try:
    g()
except RuntimeError as e:
    print("RuntimeError", e)
def h():
    try:
        raise KeyError("k")
    except KeyError:
        try:
            pass
        finally:
            raise
try:
    h()
except KeyError as e:
    print("KeyError", e)
def loop_in_finally():
    try:
        raise IndexError("i")
    finally:
        for _ in range(1):
            raise
try:
    loop_in_finally()
except IndexError as e:
    print("IndexError", e)
a, b = (x for x in [1, 2])
c, = iter([3])
print(a, b, c)
try:
    a, b, c = map(int, "12")
except ValueError as e:
    print(e)
def gen():
    n = 0
    while True:
        n += 1
        yield n
try:
    x, y, z = gen()
except ValueError as e:
    print(e)
k = [10, 20]
p, q = reversed(k)
print(p, q)
class F:
    def __init__(self): self.log = []
    def write(self, s): self.log.append(s)
    def flush(self): self.log.append("flushed")
f = F()
print("y", file=f, flush=True)
print("".join(f.log))
"#,
    )
}
