//! Which stream each kind of output reaches, as seen by a user redirecting it.
//!
//! Every test spawns the real CLI, runtime, or built application, so it checks
//! the bytes a shell would capture from stdout and stderr separately.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Environment variables that change colour decisions; tests set them explicitly.
const COLOR_VARIABLES: [&str; 4] = ["NO_COLOR", "CLICOLOR_FORCE", "FORCE_COLOR", "TERM"];

struct Fixture(PathBuf);
impl Fixture {
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture must fail the test immediately"
    )]
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "kipferl-streams-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root.canonicalize().unwrap())
    }

    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture must fail the test immediately"
    )]
    fn write(&self, name: &str, source: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("cache"));
        for variable in COLOR_VARIABLES {
            command.env_remove(variable);
        }
        command
    }

    #[expect(
        clippy::unwrap_used,
        reason = "Failure to execute the child process must fail the test immediately"
    )]
    fn cli(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_kipferl"))
            .args(args)
            .output()
            .unwrap()
    }

    #[expect(
        clippy::unwrap_used,
        reason = "Failure to execute the child process must fail the test immediately"
    )]
    fn cli_with(&self, args: &[&str], variables: &[(&str, &str)]) -> Output {
        let mut command = self.command(env!("CARGO_BIN_EXE_kipferl"));
        for (name, value) in variables {
            command.env(name, value);
        }
        command.args(args).output().unwrap()
    }

    #[expect(
        clippy::unwrap_used,
        reason = "Failure to execute the child process must fail the test immediately"
    )]
    fn binary(&self, name: &str) -> Output {
        self.command(self.0.join(name)).output().unwrap()
    }

    /// Replace this fixture's absolute directory so snapshots stay portable.
    fn anonymize(&self, text: &[u8]) -> String {
        String::from_utf8_lossy(text).replace(&self.0.to_string_lossy().into_owned(), "<dir>")
    }

    fn build(&self, script: &str, output: &str) -> String {
        let built = self.cli(&["build", script, "-o", output]);
        assert!(
            built.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        );
        String::from_utf8_lossy(&built.stdout).into_owned()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const HELPER: &str = "def boom(value):\n    raise ValueError('bad ' + str(value))\n";
const FAILING_APP: &str =
    "import helper\n\nprint('before the failure')\n\n\ndef main():\n    helper.boom(3)\n\nmain()\n";

fn assert_failure_streams(output: &Output, fixture: &Fixture, expected_traceback: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "before the failure\n",
        "stdout must hold only what the script printed"
    );
    assert_eq!(fixture.anonymize(&output.stderr), expected_traceback);
}

#[test]
fn run_keeps_stdout_for_program_output_and_reports_tracebacks_on_stderr() {
    let fixture = Fixture::new();
    fixture.write("helper.py", HELPER);
    fixture.write("app.py", FAILING_APP);
    let output = fixture.cli(&["run", "app.py"]);
    assert_failure_streams(
        &output,
        &fixture,
        "Traceback (most recent call last):\n  File \"<dir>/app.py\", line 9\n    main()\n  File \"<dir>/app.py\", line 7, in main\n    helper.boom(3)\n  File \"<dir>/helper.py\", line 2, in boom\n    raise ValueError('bad ' + str(value))\nValueError: bad 3\nerror: Python execution failed\n",
    );
}

#[test]
fn built_core_and_full_applications_report_tracebacks_on_stderr() {
    let fixture = Fixture::new();
    fixture.write("helper.py", HELPER);
    fixture.write("app.py", FAILING_APP);
    // `re` is outside the core runtime, so this variant selects the full profile.
    fixture.write("full.py", &format!("import re\n{FAILING_APP}"));
    assert!(fixture.build("app.py", "core-app").contains("core"));
    assert!(fixture.build("full.py", "full-app").contains("full"));
    assert_failure_streams(
        &fixture.binary("core-app"),
        &fixture,
        "Traceback (most recent call last):\n  File \"app.py\", line 9\n    main()\n  File \"app.py\", line 7, in main\n    helper.boom(3)\n  File \"helper.py\", line 2, in boom\n    raise ValueError('bad ' + str(value))\nValueError: bad 3\nerror: Python execution failed\n",
    );
    assert_failure_streams(
        &fixture.binary("full-app"),
        &fixture,
        "Traceback (most recent call last):\n  File \"full.py\", line 10\n    main()\n  File \"full.py\", line 8, in main\n    helper.boom(3)\n  File \"helper.py\", line 2, in boom\n    raise ValueError('bad ' + str(value))\nValueError: bad 3\nerror: Python execution failed\n",
    );
}

fn frame_headers(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("  File "))
        .collect()
}

#[test]
fn import_time_and_chained_failures_show_only_user_frames() {
    let fixture = Fixture::new();
    fixture.write("broken.py", "value = 1\nraise RuntimeError('at import')\n");
    fixture.write("importer.py", "print('start')\nimport broken\n");
    fixture.write(
        "chained.py",
        "try:\n    {}['missing']\nexcept KeyError:\n    raise ValueError('second')\n",
    );

    let run = fixture.cli(&["run", "importer.py"]);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "start\n");
    let stderr = fixture.anonymize(&run.stderr);
    assert_eq!(
        frame_headers(&stderr),
        [
            "  File \"<dir>/importer.py\", line 2",
            "  File \"<dir>/broken.py\", line 2",
        ],
        "{stderr}"
    );
    assert!(stderr.ends_with("RuntimeError: at import\nerror: Python execution failed\n"));

    fixture.build("importer.py", "importer");
    let built = fixture.binary("importer");
    assert_eq!(String::from_utf8_lossy(&built.stdout), "start\n");
    let stderr = fixture.anonymize(&built.stderr);
    assert_eq!(
        frame_headers(&stderr),
        [
            "  File \"importer.py\", line 2",
            "  File \"broken.py\", line 2",
        ],
        "{stderr}"
    );

    let chained = fixture.cli(&["run", "chained.py"]);
    assert!(chained.stdout.is_empty());
    assert_eq!(
        fixture.anonymize(&chained.stderr),
        "Traceback (most recent call last):\n  File \"<dir>/chained.py\", line 2\n    {}['missing']\nKeyError: 'missing'\n\nDuring handling of the above exception, another exception occurred:\n\nTraceback (most recent call last):\n  File \"<dir>/chained.py\", line 4\n    raise ValueError('second')\nValueError: second\nerror: Python execution failed\n"
    );
}

#[test]
fn starter_argument_errors_go_to_stderr_with_a_failure_status() {
    let fixture = Fixture::new();
    for (template, arguments) in [
        ("cli", &["--unknown"][..]),
        ("interactive", &["unexpected"][..]),
        ("api", &["https://example.com"][..]),
        ("api", &["example.com", "no-slash"][..]),
    ] {
        let name = format!("{template}_starter");
        let created = fixture.cli(&["new", &name, "--template", template]);
        assert!(created.status.success(), "{template}");
        let project = fixture.0.join(&name);
        let mut command = fixture.command(env!("CARGO_BIN_EXE_kipferl"));
        let output = command
            .current_dir(&project)
            .args(["run", "--"])
            .args(arguments)
            .output();
        let Ok(output) = output else {
            panic!("cannot run the {template} starter");
        };
        assert_eq!(output.status.code(), Some(2), "{template} {arguments:?}");
        assert!(output.stdout.is_empty(), "{template} {arguments:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).starts_with("Error: "),
            "{template} {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(project);
    }
}

fn has_escape(bytes: &[u8]) -> bool {
    bytes.contains(&0x1b)
}

#[test]
fn cli_output_is_plain_when_piped_or_when_no_color_is_set() {
    let fixture = Fixture::new();
    let piped = fixture.cli(&["--version"]);
    assert_eq!(
        String::from_utf8_lossy(&piped.stdout),
        format!("Kipferl v{}\n", env!("CARGO_PKG_VERSION"))
    );

    let missing = fixture.cli(&["run", "missing.py"]);
    assert_eq!(
        String::from_utf8_lossy(&missing.stderr),
        "Error: Script not found: missing.py\n"
    );

    for variables in [
        &[("CLICOLOR_FORCE", "1")][..],
        &[("FORCE_COLOR", "1")][..],
        &[("FORCE_COLOR", "true"), ("NO_COLOR", "")][..],
    ] {
        let forced = fixture.cli_with(&["--version"], variables);
        assert!(has_escape(&forced.stdout), "{variables:?}");
        let forced = fixture.cli_with(&["run", "missing.py"], variables);
        assert!(has_escape(&forced.stderr), "{variables:?}");
    }
    for variables in [
        &[("NO_COLOR", "1"), ("CLICOLOR_FORCE", "1")][..],
        &[("NO_COLOR", "1"), ("FORCE_COLOR", "1")][..],
        &[("CLICOLOR_FORCE", "0")][..],
        &[("FORCE_COLOR", "0")][..],
        &[("FORCE_COLOR", "")][..],
    ] {
        let plain = fixture.cli_with(&["--help"], variables);
        assert!(!has_escape(&plain.stdout), "{variables:?}");
    }
}

#[cfg(unix)]
mod terminal {
    use std::ffi::CStr;
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::process::Stdio;

    use super::{Fixture, has_escape};

    /// Run the CLI with stdout attached to a fresh pseudo-terminal and return its bytes.
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to allocate the pseudo-terminal or run the child must fail the test"
    )]
    fn run_on_terminal(fixture: &Fixture, args: &[&str], variables: &[(&str, &str)]) -> Vec<u8> {
        // SAFETY: posix_openpt returns a new descriptor or -1; ownership moves
        // into OwnedFd immediately after the check.
        let master = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
        assert!(master >= 0, "posix_openpt failed");
        // SAFETY: `master` is a valid descriptor owned only by this function.
        let master = unsafe { OwnedFd::from_raw_fd(master) };
        let raw = std::os::fd::AsRawFd::as_raw_fd(&master);
        // SAFETY: `raw` is an open pseudo-terminal master descriptor.
        assert_eq!(unsafe { libc::grantpt(raw) }, 0);
        // SAFETY: as above.
        assert_eq!(unsafe { libc::unlockpt(raw) }, 0);
        // SAFETY: nextest runs each test in its own process, so no other thread
        // calls ptsname concurrently; the name is copied before the next call.
        let name = unsafe { CStr::from_ptr(libc::ptsname(raw)) }
            .to_string_lossy()
            .into_owned();
        let terminal = File::options().read(true).write(true).open(name).unwrap();

        let mut command = fixture.command(env!("CARGO_BIN_EXE_kipferl"));
        for (name, value) in variables {
            command.env(name, value);
        }
        let status = command
            .args(args)
            .stdin(Stdio::null())
            .stdout(terminal.try_clone().unwrap())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
        // The child has exited, so its output is already buffered. Drain it
        // without blocking: an idle pseudo-terminal never reports end of file
        // on macOS, and reports EIO on Linux once the last handle closes.
        // SAFETY: `raw` stays open while `master` is alive; F_SETFL only
        // changes this descriptor's status flags.
        let nonblocking = unsafe { libc::fcntl(raw, libc::F_SETFL, libc::O_NONBLOCK) };
        assert_eq!(nonblocking, 0);
        let mut output = Vec::new();
        let mut master = File::from(master);
        let mut buffer = [0_u8; 4096];
        while let Ok(read) = master.read(&mut buffer) {
            if read == 0 {
                break;
            }
            output.extend_from_slice(buffer.get(..read).unwrap());
        }
        drop(terminal);
        output
    }

    #[test]
    fn cli_output_is_styled_on_a_terminal_unless_no_color_or_dumb() {
        let fixture = Fixture::new();
        assert!(has_escape(&run_on_terminal(&fixture, &["--version"], &[])));
        assert!(has_escape(&run_on_terminal(
            &fixture,
            &["--version"],
            &[("NO_COLOR", "")]
        )));
        assert!(!has_escape(&run_on_terminal(
            &fixture,
            &["--version"],
            &[("NO_COLOR", "1")]
        )));
        assert!(!has_escape(&run_on_terminal(
            &fixture,
            &["--version"],
            &[("TERM", "dumb")]
        )));
    }
}

#[test]
fn script_styling_follows_no_color_while_raw_ansi_requests_still_work() {
    let fixture = Fixture::new();
    fixture.write(
        "styled.py",
        "import ansi, tui\ntui.success('done')\nprint(repr(tui.style('x', fg='red', bold=True)))\nprint(repr(ansi.fg('red')))\n",
    );
    let default = fixture.cli(&["run", "styled.py"]);
    assert_eq!(
        String::from_utf8_lossy(&default.stdout),
        "\u{1b}[1;32m✓ \u{1b}[0mdone\n'\\x1b[1;31mx\\x1b[0m'\n'\\x1b[31m'\n",
        "piped script output keeps its styling: scripts decide for themselves"
    );
    let plain = fixture.cli_with(&["run", "styled.py"], &[("NO_COLOR", "1")]);
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "✓ done\n'x'\n'\\x1b[31m'\n"
    );
    let empty = fixture.cli_with(&["run", "styled.py"], &[("NO_COLOR", "")]);
    assert_eq!(empty.stdout, default.stdout, "an empty NO_COLOR is unset");
}
