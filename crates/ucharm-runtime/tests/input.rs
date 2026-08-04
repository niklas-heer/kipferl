use std::fs::File;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::process::{Command, Output, Stdio};
use std::ptr;
use std::thread;
use std::time::Duration;

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
    let (master, slave) = open_pty();
    let original = terminal_settings(slave.0);
    // SAFETY: duplicating a valid descriptor creates an independently owned
    // descriptor, which `File` closes after the child has inherited it.
    let child_stdin = unsafe { libc::dup(slave.0) };
    assert!(child_stdin >= 0, "duplicate PTY slave");
    // SAFETY: `child_stdin` is a newly duplicated, owned descriptor.
    let child_stdin = unsafe { File::from_raw_fd(child_stdin) };

    let child = Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm"))
        .args([
            "-c",
            "import input; print(repr(input.confirm('Continue?')))",
        ])
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

fn run(source: &str, keys: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-ucharm"))
        .args(["-c", source])
        .env("MCHARM_TEST_KEYS", keys)
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
