use std::fs::File;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::process::{Command, Output, Stdio};
use std::ptr;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static PTY_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn selection_byte_streams_match_the_zig_runtime() {
    let output = run(
        "import input; print(repr(input.select('Pick:', ['A', 'B'], default=1)))",
        "enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        "\x1b[36m\x1b[1m? \x1b[0mPick:\n\x1b[?25l    A\n\x1b[36m  ❯ B\x1b[0m\n\x1b[?25h'B'\n"
            .as_bytes()
    );
    assert!(output.stderr.is_empty());

    let output = run(
        "import input; print(repr(input.select('Pick:', ['A', 'B'])))",
        "down,enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        "\x1b[36m\x1b[1m? \x1b[0mPick:\n\x1b[?25l\x1b[36m  ❯ A\x1b[0m\n    B\n\x1b[2A\x1b[2K\r    A\n\x1b[2K\r\x1b[36m  ❯ B\x1b[0m\n\x1b[?25h'B'\n".as_bytes()
    );

    let output = run(
        "import input; print(repr(input.select('Pick:', ['A', 'B'])))",
        "escape",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(output.stdout.ends_with(b"\x1b[?25hNone\n"));
}

#[test]
fn accepts_the_legacy_input_test_key_variable() {
    let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args([
            "-c",
            "import input; print(repr(input.confirm('Continue?')))",
        ])
        .env_remove("KIPFERL_TEST_KEYS")
        .env("MCHARM_TEST_KEYS", "y")
        .output()
        .expect("run with legacy test-key variable");

    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(output.stdout.ends_with(b"True\n"));
}

#[test]
fn multiselect_byte_stream_and_defaults_match_the_zig_runtime() {
    let output = run(
        "import input; print(repr(input.multiselect('Pick:', ['A', 'B'], defaults=['B'])))",
        "space,down,space,enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        "\x1b[36m\x1b[1m? \x1b[0mPick:\x1b[2m (space to toggle, enter to confirm)\x1b[0m\n\x1b[?25l\x1b[36m  ○ A\x1b[0m\n  ◉ B\n\x1b[2A\x1b[2K\r\x1b[36m  ◉ A\x1b[0m\n\x1b[2K\r  ◉ B\n\x1b[2A\x1b[2K\r  ◉ A\n\x1b[2K\r\x1b[36m  ◉ B\x1b[0m\n\x1b[2A\x1b[2K\r  ◉ A\n\x1b[2K\r\x1b[36m  ○ B\x1b[0m\n\x1b[?25h['A']\n".as_bytes()
    );
    assert!(output.stderr.is_empty());

    let output = run(
        "import input; print(repr(input.multiselect('Pick:', ['A', 'B'])))",
        "space,escape",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(output.stdout.ends_with(b"\x1b[?25h[]\n"));
}

#[test]
fn confirm_prompt_and_password_match_the_zig_runtime() {
    let output = run(
        "import input; print(repr(input.confirm('Sure?', default=False)))",
        "enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        b"\x1b[36m\x1b[1m? \x1b[0mSure? \x1b[2m(y/N)\x1b[0m \x1b[36mNo\x1b[0m\nFalse\n"
    );

    let output = run(
        "import input; print(repr(input.prompt('Name:', default='X')))",
        "h,e,l,l,o,backspace,!,enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        b"\x1b[36m\x1b[1m? \x1b[0mName:\x1b[2m (X)\x1b[0m hello\x08 \x08!\n'hell!'\n"
    );

    let output = run(
        "import input; print(repr(input.prompt('Name:', default='X')))",
        "escape",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(
        output.stdout,
        b"\x1b[36m\x1b[1m? \x1b[0mName:\x1b[2m (X)\x1b[0m \n'X'\n"
    );

    let output = run(
        "import input; print(repr(input.password('Pass:')))",
        "s,e,c,r,e,t,backspace,!,enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(output.stdout, b"\x1b[36m\x1b[1m? \x1b[0mPass: \n'secre!'\n");
}

#[test]
fn supports_empty_sequences_tuples_and_keyword_defaults() {
    let output = run(
        "import input\n\
assert input.select('Pick:', []) is None\n\
assert input.multiselect('Pick:', ()) == []\n\
print(repr(input.select('Pick:', ('A', 'B'), default=1)))",
        "enter",
    );
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(output.stdout.ends_with(b"'B'\n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn preserves_binding_and_argument_errors() {
    for (source, expected) in [
        (
            "import input; input.select('x')",
            "TypeError: select() takes 2 positional arguments but 1 were given",
        ),
        (
            "import input; input.select(1, [])",
            "TypeError: prompt must be a string",
        ),
        (
            "import input; input.select('x', [], 0, 1)",
            "TypeError: too many arguments (select)",
        ),
        (
            "import input; input.select('x', [], nope=1)",
            "TypeError: 'nope' is an invalid keyword argument for select()",
        ),
        (
            "import input; input.multiselect(1, [])",
            "TypeError: prompt must be a string",
        ),
        (
            "import input; input.confirm(1)",
            "TypeError: prompt must be a string",
        ),
        (
            "import input; input.prompt(1)",
            "TypeError: message must be a string",
        ),
        (
            "import input; input.password(1)",
            "TypeError: message must be a string",
        ),
        (
            "import input; input.password()",
            "TypeError: too few arguments",
        ),
        (
            "import input; input.password('x', 1)",
            "TypeError: too many arguments",
        ),
    ] {
        let output = run(source, "enter");
        assert_eq!(output.status.code(), Some(1), "{}", diagnostic(&output));
        assert!(
            text(&output.stdout).contains(expected),
            "{}",
            diagnostic(&output)
        );
        assert!(text(&output.stderr).contains("Python execution failed"));
    }
}

#[test]
fn restores_real_terminal_settings_after_interaction() {
    let _pty_guard = PTY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (master, slave) = open_pty();
    let original = terminal_settings(slave.0);
    // SAFETY: duplicating a valid descriptor creates an independently owned
    // descriptor, which `File` closes after the child has inherited it.
    let child_stdin = unsafe { libc::dup(slave.0) };
    assert!(child_stdin >= 0, "duplicate PTY slave");
    // SAFETY: `child_stdin` is a newly duplicated, owned descriptor.
    let child_stdin = unsafe { File::from_raw_fd(child_stdin) };

    let child = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args([
            "-c",
            "import input; print(repr(input.confirm('Continue?')))",
        ])
        .env_remove("KIPFERL_TEST_KEYS")
        .env_remove("MCHARM_TEST_KEYS")
        .stdin(Stdio::from(child_stdin))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start input runtime on a PTY");

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

    // SAFETY: `master` is a valid PTY descriptor and the byte remains alive
    // for the duration of the write.
    assert_eq!(unsafe { libc::write(master.0, b"y".as_ptr().cast(), 1) }, 1);
    let output = child.wait_with_output().expect("wait for input runtime");
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(output.stdout.ends_with(b"True\n"));

    let restored = terminal_settings(slave.0);
    assert_eq!(restored.c_iflag, original.c_iflag);
    assert_eq!(restored.c_oflag, original.c_oflag);
    assert_eq!(restored.c_cflag, original.c_cflag);
    assert_eq!(restored.c_lflag, original.c_lflag);
    assert_eq!(restored.c_cc, original.c_cc);
}

#[test]
fn ratatui_select_renders_in_a_real_terminal_and_restores_it() {
    let output = run_ratatui_in_pty(
        "import input; print(repr(input.select('Choose:', ['Build', 'Test', 'Deploy'])))",
        b"j\r",
    );
    assert!(output.contains('╭') && output.contains('╯'), "{output:?}");
    assert!(output.contains("[Enter] select"), "{output:?}");
    assert!(output.contains("'Test'"), "{output:?}");
}

#[test]
fn ratatui_multiselect_handles_batched_keys_in_a_real_terminal() {
    let output = run_ratatui_in_pty(
        "import input; print(repr(input.multiselect('Features:', ['Logging', 'HTTP', 'Config'])))",
        b" j \r",
    );
    assert!(output.contains("[Space] toggle"), "{output:?}");
    assert!(output.contains("['Logging', 'HTTP']"), "{output:?}");
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn run_ratatui_in_pty(source: &str, keys: &[u8]) -> String {
    let _pty_guard = PTY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (master, slave) = open_pty();
    let original = terminal_settings(slave.0);
    set_terminal_size(master.0, 80, 24);
    set_nonblocking(master.0);

    let child_stdin = duplicate_file(slave.0);
    let child_stdout = duplicate_file(slave.0);
    let mut child = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .env_remove("KIPFERL_TEST_KEYS")
        .env_remove("MCHARM_TEST_KEYS")
        .stdin(Stdio::from(child_stdin))
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Ratatui input runtime on a PTY");

    let mut output = Vec::new();
    let mut answered_cursor_query = false;
    let mut sent_selection = false;
    let mut completed = false;
    // Allow enough time for the cursor-position handshake on loaded CI hosts;
    // the loop still exits as soon as the child completes.
    for _ in 0..1_000 {
        read_available(master.0, &mut output);
        if output.windows(4).any(|window| window == b"\x1b[6n") && !answered_cursor_query {
            write_descriptor(master.0, b"\x1b[1;1R");
            answered_cursor_query = true;
        }
        if output.contains(&b'?') && answered_cursor_query && !sent_selection {
            write_descriptor(master.0, keys);
            sent_selection = true;
        }
        if child.try_wait().expect("poll Ratatui child").is_some() {
            completed = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(completed, "Ratatui interaction timed out");
    read_available(master.0, &mut output);
    let child_output = child.wait_with_output().expect("collect Ratatui child");
    assert!(
        child_output.status.success(),
        "{}",
        text(&child_output.stderr)
    );

    let output = text(&output);
    assert!(output.contains("\x1b[?25h"), "cursor was not restored");

    let restored = terminal_settings(slave.0);
    assert_eq!(restored.c_iflag, original.c_iflag);
    assert_eq!(restored.c_oflag, original.c_oflag);
    assert_eq!(restored.c_cflag, original.c_cflag);
    assert_eq!(restored.c_lflag, original.c_lflag);
    assert_eq!(restored.c_cc, original.c_cc);
    output
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn run(source: &str, keys: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .env("KIPFERL_TEST_KEYS", keys)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run Rust PocketPy input module")
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
            &raw mut master,
            &raw mut slave,
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

fn duplicate_file(descriptor: libc::c_int) -> File {
    // SAFETY: duplicating a valid descriptor creates a new owned descriptor.
    let duplicate = unsafe { libc::dup(descriptor) };
    assert!(duplicate >= 0, "duplicate PTY descriptor");
    // SAFETY: `duplicate` is a newly created, independently owned descriptor.
    unsafe { File::from_raw_fd(duplicate) }
}

fn set_terminal_size(descriptor: libc::c_int, columns: u16, rows: u16) {
    let size = libc::winsize {
        ws_row: rows,
        ws_col: columns,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: `descriptor` is a PTY and `size` points to initialized storage.
    let status = unsafe { libc::ioctl(descriptor, libc::TIOCSWINSZ, &size) };
    assert_eq!(status, 0, "set PTY size");
}

fn set_nonblocking(descriptor: libc::c_int) {
    // SAFETY: fcntl receives a valid PTY descriptor.
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    assert!(flags >= 0, "read PTY flags");
    // SAFETY: preserve the existing flags and add nonblocking reads.
    let status = unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    assert_eq!(status, 0, "set PTY nonblocking");
}

fn read_available(descriptor: libc::c_int, output: &mut Vec<u8>) {
    let mut buffer = [0_u8; 4096];
    loop {
        // SAFETY: `buffer` is writable and the descriptor is a nonblocking PTY.
        let count = unsafe { libc::read(descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
        let Ok(count) = usize::try_from(count) else {
            break;
        };
        if count == 0 {
            break;
        }
        output.extend(buffer.iter().copied().take(count));
    }
}
#[expect(
    clippy::expect_used,
    reason = "This test-only helper fails the test immediately when its explicitly described process or fixture setup fails."
)]
fn write_descriptor(descriptor: libc::c_int, bytes: &[u8]) {
    // SAFETY: `bytes` remains live for the synchronous write and the descriptor
    // is the PTY master.
    let count = unsafe { libc::write(descriptor, bytes.as_ptr().cast(), bytes.len()) };
    assert_eq!(
        count,
        isize::try_from(bytes.len()).expect("PTY input length fits ssize_t"),
        "write PTY input"
    );
}
