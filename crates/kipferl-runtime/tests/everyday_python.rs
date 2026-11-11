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
