use std::process::{Command, Output};

#[test]
fn passes_the_zig_errno_compatibility_suite() {
    let output = run(include_str!("../../../tests/cpython/test_errno.py"));

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(text(&output.stdout).contains("Results: 38 passed, 0 failed, 0 skipped"));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn exposes_platform_constants_reverse_mapping_and_ascii_isupper() {
    let output = run(&format!(
        concat!(
            "import errno\n",
            "expected = {{\n",
            "    'EPERM': {}, 'ENOENT': {}, 'ESRCH': {}, 'EINTR': {},\n",
            "    'EIO': {}, 'EBADF': {}, 'ECHILD': {}, 'EAGAIN': {},\n",
            "    'ENOMEM': {}, 'EACCES': {}, 'EEXIST': {}, 'ENOTDIR': {},\n",
            "    'EISDIR': {}, 'EINVAL': {}, 'ENOSPC': {}, 'EPIPE': {},\n",
            "}}\n",
            "for name, value in expected.items():\n",
            "    assert getattr(errno, name) == value\n",
            "    assert errno.errorcode[value] == name\n",
            "assert len(errno.errorcode) == 16\n",
            "assert 'ABC'.isupper() is True\n",
            "assert 'A1'.isupper() is True\n",
            "assert 'Ab'.isupper() is False\n",
            "assert ''.isupper() is False\n",
            "assert '123'.isupper() is False",
        ),
        libc::EPERM,
        libc::ENOENT,
        libc::ESRCH,
        libc::EINTR,
        libc::EIO,
        libc::EBADF,
        libc::ECHILD,
        libc::EAGAIN,
        libc::ENOMEM,
        libc::EACCES,
        libc::EEXIST,
        libc::ENOTDIR,
        libc::EISDIR,
        libc::EINVAL,
        libc::ENOSPC,
        libc::EPIPE,
    ));

    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn preserves_oserror_compatibility_and_repairs_the_one_argument_crash() {
    let output = run(concat!(
        "import errno\n",
        "assert OSError().args == ()\n",
        "assert OSError(5).args == (5,)\n",
        "error = OSError(errno.ENOENT, 'ignored legacy message')\n",
        "assert error.args == (errno.ENOENT,)\n",
        "assert str(error) == '2'",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));

    for (source, expected) in [
        (
            "import errno; OSError(1, 2, 3)",
            "TypeError: too many arguments",
        ),
        (
            "import errno; OSError(1, message='x')",
            "TypeError: nativefunc does not accept keyword arguments",
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
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
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
