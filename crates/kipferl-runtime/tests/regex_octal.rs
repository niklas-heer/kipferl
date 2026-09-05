//! Differential checks for the supported Python octal regex escape forms.
use std::io;
use std::process::{Command, Output};

#[test]
fn octal_escapes_match_cpython_across_regex_operations() -> io::Result<()> {
    let source = include_str!("fixtures/re_octal.py");
    let runtime = run_runtime(source)?;
    let cpython = Command::new("python3")
        .args(["-I", "-S", "-c", source])
        .output()?;
    check(
        runtime.status.success() && runtime.stderr.is_empty(),
        &diagnostic(&runtime),
    )?;
    check(cpython.status.success(), &diagnostic(&cpython))?;
    check(
        runtime.stdout == cpython.stdout,
        &format!(
            "runtime and CPython differ:\n{}\n{}",
            diagnostic(&runtime),
            diagnostic(&cpython)
        ),
    )?;
    Ok(())
}

#[test]
fn unsupported_numeric_backreferences_and_surrogate_patterns_remain_errors() -> io::Result<()> {
    let output = run_runtime(
        r"import re
for pattern in [r'(a)\1', r'(a)\11', r'[\1]', r'[\11]', r'\uD800', r'[\uD800-\uDBFF]']:
    rejected = False
    try:
        re.compile(pattern)
    except Exception:
        rejected = True
    assert rejected
assert re.search(r'\\1', r'\1') is not None
print('unsupported constructs remain rejected')
",
    )?;
    check(
        output.status.success() && output.stderr.is_empty(),
        &diagnostic(&output),
    )?;
    Ok(())
}

fn check(condition: bool, detail: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(detail))
    }
}

fn run_runtime(source: &str) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}
