use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kipferl-projects-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[expect(
    clippy::unwrap_used,
    reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
)]
fn run(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .args(args)
        .current_dir(directory)
        .env("KIPFERL_CACHE_DIR", directory.join(".cache"))
        .output()
        .unwrap()
}
#[expect(
    clippy::expect_used,
    reason = "These CLI fixtures produce UTF-8 stdout; invalid encoding is a test failure"
)]
fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 fixture output")
}

#[test]
fn all_starters_have_working_help_tests_and_project_defaults() {
    for template in ["cli", "api", "interactive"] {
        let temporary = Directory::new();
        success(run(
            &temporary.0,
            &["new", "My App", "--template", template],
        ));
        let project = temporary.0.join("my_app");
        for file in [
            "my_app.py",
            "README.md",
            "kipferl.json",
            "tests/test_app.py",
            "pyrightconfig.json",
            ".kipferl/stubs/tui.pyi",
        ] {
            assert!(project.join(file).is_file(), "missing {template}/{file}");
        }
        assert!(success(run(&project, &["run", "--", "--help"])).contains("Usage: My App"));
        assert!(success(run(&project, &["test"])).contains("1 passed, 0 failed"));
        let child = project.join("nested");
        fs::create_dir(&child).unwrap();
        assert!(success(run(&child, &["test"])).contains("1 passed, 0 failed"));
        success(run(&child, &["build", "--mode", "single"]));
        assert!(project.join("dist/my_app").is_file());
    }
}

#[test]
fn cli_starter_greets_and_explicit_script_overrides_project_entry() {
    let temporary = Directory::new();
    success(run(&temporary.0, &["new", "greet"]));
    let project = temporary.0.join("greet");
    assert_eq!(success(run(&project, &["run"])), "Hello, World!\n");
    assert_eq!(
        success(run(&project, &["run", "--", "--name", "Ada"])),
        "Hello, Ada!\n"
    );
    fs::write(project.join("other.py"), "print('override')\n").unwrap();
    assert_eq!(success(run(&project, &["run", "other.py"])), "override\n");
    assert!(!run(&project, &["run", "--", "--unknown"]).status.success());
}

#[test]
fn project_tests_fail_on_assertion_and_still_run_other_files() {
    let temporary = Directory::new();
    fs::create_dir_all(temporary.0.join("tests/nested")).unwrap();
    fs::write(
        temporary.0.join("tests/test_fail.py"),
        "assert False, 'intentional failure'\n",
    )
    .unwrap();
    fs::write(
        temporary.0.join("tests/nested/test_pass.py"),
        "print('second test ran')\n",
    )
    .unwrap();
    fs::write(temporary.0.join("tests/ignored.py"), "assert False\n").unwrap();
    let output = run(&temporary.0, &["test"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("second test ran"));
    assert!(stdout.contains("1 passed, 1 failed (2 test files)"));
}

#[test]
fn config_errors_are_actionable_and_help_does_not_require_valid_config() {
    let temporary = Directory::new();
    for bad in [
        r#"{"entyr":"app.py"}"#,
        r#"{"entry":"../app.py"}"#,
        r#"{"assets":"data"}"#,
        r"[]",
        r#"{"entry":""}"#,
    ] {
        fs::write(temporary.0.join("kipferl.json"), bad).unwrap();
        let output = run(&temporary.0, &["build"]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("kipferl.json"));
        success(run(&temporary.0, &["build", "--help"]));
    }
    fs::write(temporary.0.join("kipferl.json"), " ".repeat(65537)).unwrap();
    let output = run(&temporary.0, &["run"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("64 KiB"));
}

#[test]
fn init_and_minimal_have_predictable_editor_behavior() {
    let temporary = Directory::new();
    success(run(&temporary.0, &["new", "tiny", "--minimal"]));
    assert!(temporary.0.join("tiny.py").is_file());
    assert!(!temporary.0.join(".kipferl").exists());
    assert!(!temporary.0.join("kipferl.json").exists());
    success(run(&temporary.0, &["init"]));
    assert!(temporary.0.join(".kipferl/stubs/tui.pyi").is_file());
}

#[test]
fn invalid_arguments_and_empty_test_suites_fail() {
    let temporary = Directory::new();
    for args in [
        vec!["new", "app", "--template", "unknown"],
        vec!["test", "--typo"],
        vec!["test", "--module"],
        vec!["test", "a.py", "b.py"],
        vec!["completions", "nope"],
        vec!["test"],
    ] {
        assert!(
            !run(&temporary.0, &args).status.success(),
            "accepted {args:?}"
        );
    }
    fs::create_dir(temporary.0.join("tests")).unwrap();
    assert!(!run(&temporary.0, &["test"]).status.success());
}

#[test]
fn completions_generate_sourceable_shell_scripts() {
    let temporary = Directory::new();
    for shell in ["bash", "zsh", "fish"] {
        let output = success(run(&temporary.0, &["completions", shell]));
        assert!(output.contains("cli api interactive"));
        let script = temporary.0.join(format!("completion.{shell}"));
        fs::write(&script, output).unwrap();
        if let Ok(output) = Command::new(shell).arg("-n").arg(script).output() {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

#[test]
fn nearest_project_and_custom_test_paths_take_precedence() {
    let temporary = Directory::new();
    fs::write(
        temporary.0.join("kipferl.json"),
        r#"{"entry":"outer.py","tests":["checks/smoke.py"]}"#,
    )
    .unwrap();
    fs::write(temporary.0.join("outer.py"), "print('outer')\n").unwrap();
    fs::create_dir_all(temporary.0.join("checks")).unwrap();
    fs::write(temporary.0.join("checks/smoke.py"), "assert True\n").unwrap();
    assert!(success(run(&temporary.0, &["test"])).contains("1 passed"));
    let inner = temporary.0.join("inner");
    fs::create_dir(&inner).unwrap();
    fs::write(
        inner.join("kipferl.json"),
        r#"{"entry":"inner.py","tests":[]}"#,
    )
    .unwrap();
    fs::write(inner.join("inner.py"), "print('inner')\n").unwrap();
    assert_eq!(success(run(&inner, &["run"])), "inner\n");
    assert!(!run(&inner, &["test"]).status.success());
}

#[test]
fn project_dev_runs_the_default_entry_with_arguments() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;

    let temporary = Directory::new();
    success(run(&temporary.0, &["new", "watchme"]));
    let project = temporary.0.join("watchme");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .args(["dev", "--debounce", "25", "--", "--name", "Dev"])
        .current_dir(&project)
        .env("KIPFERL_CACHE_DIR", project.join(".cache"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if line.contains("Hello, Dev!") {
                let _ = sender.send(());
                break;
            }
        }
    });
    let result = receiver.recv_timeout(Duration::from_secs(10));
    let _ = child.kill();
    let output = child.wait_with_output().unwrap();
    assert!(
        result.is_ok(),
        "dev did not run the default app: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bash_completion_distinguishes_test_modules_from_build_modes() {
    let temporary = Directory::new();
    let script = temporary.0.join("completion.bash");
    fs::write(
        &script,
        success(run(&temporary.0, &["completions", "bash"])),
    )
    .unwrap();
    for command in ["test", "build"] {
        let output = Command::new("bash")
            .args(["-c", "source \"$1\"; COMP_WORDS=(kipferl \"$2\" -m ''); COMP_CWORD=3; _kipferl; printf '%s\\n' \"${COMPREPLY[@]}\"", "completion-test"])
            .arg(&script).arg(command).current_dir(&temporary.0).output().unwrap();
        let choices = success(output);
        assert_eq!(
            choices.lines().any(|value| value == "universal"),
            command == "build"
        );
        assert_eq!(
            choices.lines().any(|value| value == "single"),
            command == "build"
        );
    }
}

#[test]
fn fifo_project_configuration_fails_without_waiting_for_a_writer() {
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let temporary = Directory::new();
    let config = temporary.0.join("kipferl.json");
    assert!(
        Command::new("mkfifo")
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .arg("run")
        .current_dir(&temporary.0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("reading FIFO configuration blocked waiting for a writer");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("kipferl.json"), "{text}");
    assert!(text.contains("regular file"), "{text}");
}
