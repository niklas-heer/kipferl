use std::fs::File;
use std::io::Write;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::process::{Command, Output, Stdio};
use std::ptr;
use std::thread;
use std::time::Duration;

#[test]
fn exposes_terminal_size_tty_and_output_controls() {
    let output = run(
        "import term\n\
assert term.size() == (80, 24)\n\
assert term.is_tty() is False\n\
assert term.cursor_pos(2, 3) is None\n\
assert term.cursor_up() is None\n\
assert term.cursor_down(2) is None\n\
assert term.cursor_left('defaults-to-one') is None\n\
assert term.cursor_right(-4) is None\n\
assert term.clear() is None\n\
assert term.clear_line() is None\n\
assert term.hide_cursor() is None\n\
assert term.show_cursor() is None\n\
assert term.write('done') is None",
        b"",
    );

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        b"\x1b[4;3H\x1b[1A\x1b[2B\x1b[1D\x1b[-4C\x1b[2J\x1b[H\x1b[2K\r\x1b[?25l\x1b[?25hdone"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn decodes_terminal_keys() {
    for (input, expected) in [
        (&b"\x1b[A"[..], "'up'\n"),
        (&b"\x1b[B"[..], "'down'\n"),
        (&b"\x1b[C"[..], "'right'\n"),
        (&b"\x1b[D"[..], "'left'\n"),
        (&b"\x1b[H"[..], "'home'\n"),
        (&b"\x1b[F"[..], "'end'\n"),
        (&b"\x1b[3~"[..], "'delete'\n"),
        (&b"\x1b[5~"[..], "'pageup'\n"),
        (&b"\x1b[6~"[..], "'pagedown'\n"),
        (&b"\r"[..], "'enter'\n"),
        (&b"\x1b"[..], "'escape'\n"),
        (&b"\x7f"[..], "'backspace'\n"),
        (&b"\t"[..], "'tab'\n"),
        (&b"\x03"[..], "'ctrl-c'\n"),
        (&b"abc"[..], "'abc'\n"),
        ("é".as_bytes(), "'é'\n"),
        (&b"\x1b[Z"[..], "'\\x1b[Z'\n"),
    ] {
        let output = run("import term; print(repr(term.read_key()))", input);
        assert!(output.status.success(), "{}", diagnostic(&output));
        assert_eq!(text(&output.stdout), expected, "input: {input:?}");
        assert!(output.stderr.is_empty());
    }

    let output = run("import term; print(repr(term.read_key()))", b"");
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout), "None\n");
}

#[test]
fn preserves_terminal_argument_and_formatting_errors() {
    for (source, expected) in [
        ("import term; term.size(1)", "TypeError: too many arguments"),
        (
            "import term; term.raw_mode()",
            "TypeError: too few arguments",
        ),
        ("import term; term.raw_mode(1)", "TypeError: expected bool"),
        (
            "import term; term.cursor_pos('x', 0)",
            "TypeError: x must be int",
        ),
        (
            "import term; term.cursor_pos(0, 'y')",
            "TypeError: y must be int",
        ),
        (
            "import term; term.write(1)",
            "TypeError: text must be a string",
        ),
        (
            "import term; term.cursor_pos(9223372036854775807, 9223372036854775807)",
            "RuntimeError: failed to format cursor position",
        ),
        (
            "import term; term.cursor_up(-9223372036854775807 - 1)",
            "RuntimeError: failed to format cursor move",
        ),
        (
            "import term; term.raw_mode(True)",
            "RuntimeError: failed to read terminal settings",
        ),
    ] {
        let output = run(source, b"");
        assert_eq!(output.status.code(), Some(1), "{}", diagnostic(&output));
        assert!(
            text(&output.stdout).contains(expected),
            "{}",
            diagnostic(&output)
        );
        assert!(text(&output.stderr).contains("Python execution failed"));
    }

    let output = run("import term; assert term.raw_mode(False) is None", b"");
    assert!(output.status.success(), "{}", diagnostic(&output));
}

#[test]
fn restores_terminal_settings_when_the_vm_exits() {
    let (master, slave) = open_pty();
    let original = terminal_settings(slave.0);
    // SAFETY: duplicating a valid descriptor creates an independently owned
    // descriptor, which `File` closes after the child has inherited it.
    let child_stdin = unsafe { libc::dup(slave.0) };
    assert!(child_stdin >= 0, "duplicate PTY slave");
    // SAFETY: `child_stdin` is a newly duplicated, owned descriptor.
    let child_stdin = unsafe { File::from_raw_fd(child_stdin) };

    let child = Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm-rs"))
        .args([
            "-c",
            "import term; term.raw_mode(True)\nwhile term.read_key() is None:\n    pass",
        ])
        .stdin(Stdio::from(child_stdin))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start runtime on a PTY");

    let mut raw_was_observed = false;
    for _ in 0..100 {
        let current = terminal_settings(slave.0);
        if current.c_lflag & (libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN) == 0 {
            raw_was_observed = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(raw_was_observed, "child never enabled raw mode");

    // SAFETY: `master` is a valid PTY descriptor and the single-byte buffer is
    // alive for the duration of the write.
    assert_eq!(unsafe { libc::write(master.0, b"x".as_ptr().cast(), 1) }, 1);
    let output = child.wait_with_output().expect("wait for PTY runtime");
    assert!(output.status.success(), "{}", diagnostic(&output));

    let restored = terminal_settings(slave.0);
    assert_eq!(restored.c_iflag, original.c_iflag);
    assert_eq!(restored.c_oflag, original.c_oflag);
    assert_eq!(restored.c_cflag, original.c_cflag);
    assert_eq!(restored.c_lflag, original.c_lflag);
    assert_eq!(restored.c_cc, original.c_cc);
}

fn run(source: &str, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm-rs"))
        .args(["-c", source])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Rust PocketPy runtime");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(input)
        .expect("write test input");
    child.wait_with_output().expect("run Rust PocketPy runtime")
}

fn text(output: &impl AsRef<[u8]>) -> String {
    String::from_utf8_lossy(output.as_ref()).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}

struct FileDescriptor(libc::c_int);

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the descriptor.
        unsafe { libc::close(self.0) };
    }
}

fn open_pty() -> (FileDescriptor, FileDescriptor) {
    let mut master = -1;
    let mut slave = -1;
    // SAFETY: both descriptor pointers are writable and the optional name,
    // settings, and window-size pointers are intentionally null.
    let status = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    assert_eq!(status, 0, "open PTY");
    (FileDescriptor(master), FileDescriptor(slave))
}

fn terminal_settings(descriptor: libc::c_int) -> libc::termios {
    let mut settings = MaybeUninit::<libc::termios>::uninit();
    // SAFETY: `settings` is writable and `descriptor` is a valid PTY.
    let status = unsafe { libc::tcgetattr(descriptor, settings.as_mut_ptr()) };
    assert_eq!(status, 0, "read PTY terminal settings");
    // SAFETY: successful `tcgetattr` initialized the structure.
    unsafe { settings.assume_init() }
}
