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
