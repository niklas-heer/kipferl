use std::process::{Command, Output};

#[test]
fn passes_all_core_runtime_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "math",
            include_str!("../../../tests/cpython/test_math.py"),
            "Results: 82 passed, 0 failed, 0 skipped",
        ),
        (
            "time",
            include_str!("../../../tests/cpython/test_time.py"),
            "Results: 42 passed, 0 failed, 0 skipped",
        ),
        (
            "sys",
            include_str!("../../../tests/cpython/test_sys.py"),
            "Results: 58 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run(source);
        assert!(
            output.status.success(),
            "{module} fixture failed:\n{}",
            diagnostic(&output)
        );
        assert!(
            text(&output.stdout).contains(summary),
            "{module} summary missing:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{module}");
    }
}

#[test]
fn matches_cpython_for_deterministic_math_time_and_sys_operations() {
    let source = concat!(
        "import math, sys, time\n",
        "print(round(math.sinh(1), 10), round(math.cosh(1), 10), round(math.tanh(1), 10))\n",
        "print(math.frexp(8.0), math.ldexp(0.5, 4), round(math.log1p(2), 12))\n",
        "epoch = time.gmtime(0)\n",
        "print(tuple(epoch), time.strftime('%Y-%m-%d %H:%M:%S', epoch))\n",
        "parsed = time.strptime('2024-02-29 12:34:56', '%Y-%m-%d %H:%M:%S')\n",
        "print(tuple(parsed))\n",
        "print(sys.version_info[0], sys.byteorder, sys.maxsize > 2**30, sys.implementation.name in ('cpython', 'pocketpy'))\n",
        "print('ABC123'.isalnum(), '123'.isdigit(), 'Hello World'.istitle())\n",
    );
    let rust = run(source);
    assert!(rust.status.success(), "{}", diagnostic(&rust));
    let cpython = Command::new("python3")
        .args(["-c", source])
        .output()
        .expect("run CPython differential oracle");
    assert!(cpython.status.success(), "{}", diagnostic(&cpython));
    assert_eq!(rust.stdout, cpython.stdout);
    assert_eq!(text(&rust.stderr), "");
}

#[test]
fn preserves_clock_calendar_interning_and_string_state_under_stress() {
    let output = run(concat!(
        "import math, sys, time\n",
        "previous = time.monotonic()\n",
        "for i in range(1000):\n",
        "    value = (i - 500) / 100.0\n",
        "    assert abs(math.ldexp(math.frexp(value)[0], math.frexp(value)[1]) - value) < 1e-12\n",
        "    assert abs(math.tanh(value)) <= 1.0\n",
        "    name = sys.intern('item-' + str(i % 25))\n",
        "    assert name is sys.intern('item-' + str(i % 25))\n",
        "    assert str(i).isdigit() and ('item_' + str(i)).isidentifier()\n",
        "current = time.monotonic()\n",
        "assert current >= previous\n",
        "for year in range(2000, 2031):\n",
        "    parsed = time.strptime(str(year) + '-06-15', '%Y-%m-%d')\n",
        "    assert parsed[0] == year and parsed[1] == 6 and parsed[2] == 15\n",
        "    assert time.strftime('%Y-%m-%d', parsed) == str(year) + '-06-15'\n",
        "original = sys.getrecursionlimit()\n",
        "sys.setrecursionlimit(750)\n",
        "assert sys.getrecursionlimit() == 750\n",
        "sys.setrecursionlimit(original)\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_domain_type_and_calendar_errors() {
    let output = run(concat!(
        "import math, sys, time\n",
        "def must_fail(expected, operation):\n",
        "    try:\n",
        "        operation()\n",
        "    except expected:\n",
        "        return\n",
        "    raise AssertionError('operation unexpectedly succeeded')\n",
        "must_fail(ValueError, lambda: math.acosh(0.5))\n",
        "must_fail(ValueError, lambda: math.atanh(1.0))\n",
        "must_fail(ValueError, lambda: math.log1p(-1.0))\n",
        "must_fail(TypeError, lambda: math.ldexp(1.0, 1.5))\n",
        "must_fail(TypeError, lambda: time.mktime((2024, 1)))\n",
        "must_fail(ValueError, lambda: time.strptime('not-a-date', '%Y-%m-%d'))\n",
        "must_fail(TypeError, lambda: sys.intern(42))\n",
        "must_fail(ValueError, lambda: sys.setrecursionlimit(0))\n",
        "assert ''.isdigit() is False and ''.isprintable() is True\n",
        "assert 'abc-123'.isascii() and not 'μ'.isascii()\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn resolves_iana_and_posix_time_zones_with_dst() {
    let source = concat!(
        "import time\n",
        "assert time.localtime(0) == (1969, 12, 31, 19, 0, 0, 2, 365, 0)\n",
        "summer = time.localtime(1719792000)\n",
        "assert summer == (2024, 6, 30, 20, 0, 0, 6, 182, 1)\n",
        "assert time.mktime((1970, 1, 1, 0, 0, 0, 3, 1, -1)) == 18000.0\n",
        "assert time.strftime('%z %Z', time.localtime(0)) == '-0500 EST'\n",
        "assert time.strftime('%z %Z', summer) == '-0400 EDT'\n",
    );
    for time_zone in ["America/New_York", "EST5EDT,M3.2.0,M11.1.0"] {
        let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm-rs"))
            .env("TZ", time_zone)
            .args(["-c", source])
            .output()
            .expect("run Rust PocketPy runtime with time zone");
        assert!(
            output.status.success(),
            "{time_zone}: {}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{time_zone}");
    }
}

#[test]
fn writes_sys_streams_and_returns_byte_lengths() {
    let output = run(concat!(
        "import sys\n",
        "assert sys.stdout.write('output') == 6\n",
        "assert sys.stderr.write('error') == 5\n",
        "sys.stdout.flush(); sys.stderr.flush()\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "output");
    assert_eq!(text(&output.stderr), "error");
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
