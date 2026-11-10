//! Which stream each kind of output reaches, as seen by a user redirecting it.
//!
//! Every test spawns the real CLI, runtime, or built application, so it checks
//! the bytes a shell would capture from stdout and stderr separately.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Environment variables that change colour decisions; tests start without them.
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
