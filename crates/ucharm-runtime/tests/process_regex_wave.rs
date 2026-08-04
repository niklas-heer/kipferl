use std::process::{Command, Output};

#[test]
fn passes_all_process_regex_wave_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "re",
            include_str!("../../../tests/cpython/test_re.py"),
            "Results: 79 passed, 0 failed, 0 skipped",
        ),
        (
            "logging",
            include_str!("../../../tests/cpython/test_logging.py"),
            "Results: 39 passed, 0 failed, 0 skipped",
        ),
        (
            "signal",
            include_str!("../../../tests/cpython/test_signal.py"),
            "Results: 15 passed, 0 failed, 0 skipped",
        ),
        (
            "subprocess",
            include_str!("../../../tests/cpython/test_subprocess.py"),
            "Results: 19 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run_runtime(source);
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
fn matches_cpython_for_deterministic_regex_and_process_operations() {
    let source = concat!(
        "import re, subprocess\n",
        "print(re.findall(r'(\\w)(\\d+)', 'a1 b22 c333'))\n",
        "found = re.search(r'(\\w+)=(\\d+)', 'prefix value=42 suffix')\n",
        "print(found.group(0), found.groups(), found.span(2))\n",
        "print(re.sub(r'(\\w+)', r'[\\1]', 'alpha beta', 1))\n",
        "print(re.split(r'\\s+', 'alpha beta gamma', 1))\n",
        "result = subprocess.run(['sh', '-c', 'printf result'], capture_output=True)\n",
        "if isinstance(result, dict):\n",
        "    print(result['returncode'], result['stdout'].decode())\n",
        "else:\n",
        "    print(result.returncode, result.stdout.decode())\n",
    );
    let rust = run_runtime(source);
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
fn preserves_process_capture_limits_state_and_errors_under_stress() {
    let output = run_runtime(concat!(
        "import logging, re, signal, subprocess\n",
        "pattern = re.compile(r'(item)-(\\d+)')\n",
        "assert re.sub('x', 'y', 'xx', -1) == 'xx'\n",
        "assert re.split('x', 'axb', -1) == ['axb']\n",
        "for i in range(300):\n",
        "    text = 'prefix item-' + str(i) + ' suffix'\n",
        "    found = pattern.search(text)\n",
        "    assert found.group(1) == 'item' and found.group(2) == str(i)\n",
        "    assert pattern.sub(r'entry-\\2', text) == 'prefix entry-' + str(i) + ' suffix'\n",
        "logger = logging.getLogger('wave.child')\n",
        "assert logger is logging.getLogger('wave.child')\n",
        "assert logger.parent is logging.getLogger('wave')\n",
        "handler = logging.StreamHandler(); handler.setLevel(logging.ERROR)\n",
        "logger.addHandler(handler); assert logger.handlers[0] is handler\n",
        "marker = lambda signum: signum\n",
        "signal.signal(signal.SIGUSR1, marker)\n",
        "assert signal.getsignal(signal.SIGUSR1) is marker\n",
        "command = '(yes o | head -c 1100000) & (yes e | head -c 1100000 >&2) & wait'\n",
        "result = subprocess.run(command, capture_output=True, shell=True)\n",
        "assert result['returncode'] == 0\n",
        "assert len(result['stdout']) == 1048576 and len(result['stderr']) == 1048576\n",
        "for operation in (\n",
        "    lambda: re.compile('['),\n",
        "    lambda: subprocess.run([], capture_output=True),\n",
        "    lambda: subprocess.run(['ucharm-command-that-does-not-exist'], capture_output=True),\n",
        "):\n",
        "    caught = False\n",
        "    try:\n",
        "        operation()\n",
        "    except Exception:\n",
        "        caught = True\n",
        "    assert caught\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn preserves_logging_thresholds_and_legacy_stderr_format() {
    let output = run_runtime(concat!(
        "import logging\n",
        "logger = logging.getLogger('output')\n",
        "logger.setLevel(logging.ERROR)\n",
        "logger.info('hidden')\n",
        "logger.error('visible')\n",
        "logging.warning('root')\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), "ERROR: visible\nWARNING: root\n");
}

fn run_runtime(source: &str) -> Output {
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
