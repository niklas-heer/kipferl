use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reports_version_help_and_unknown_commands() {
    let temporary = TestDirectory::new("dispatch");

    let version = run(&temporary, &["--version"]);
    assert!(version.status.success());
    assert!(text(&version.stdout).contains(ucharm_cli::version()));

    let help = run(&temporary, &["--help"]);
    assert!(help.status.success());
    assert!(text(&help.stdout).contains("COMMANDS"));

    let unknown = run(&temporary, &["unknown"]);
    assert!(!unknown.status.success());
    assert!(text(&unknown.stderr).contains("Unknown command"));

    let build_help = run(&temporary, &["build", "--help"]);
    assert!(build_help.status.success());
    assert!(text(&build_help.stdout).contains("Build standalone binaries"));

    let test_help = run(&temporary, &["test", "--help"]);
    assert!(test_help.status.success());
    assert!(text(&test_help.stdout).contains("CPython Compatibility Testing"));

    let run_help = run(&temporary, &["run", "--help"]);
    assert!(run_help.status.success());
    assert!(text(&run_help.stdout).contains("ucharm run <script.py> [args...]"));

    let no_script = run(&temporary, &["run"]);
    assert!(!no_script.status.success());
    assert_eq!(
        text(&no_script.stderr),
        "\x1b[31mError:\x1b[0m No script specified\nUsage: ucharm run <script.py> [args...]\n"
    );

    let missing_script = run(&temporary, &["run", "missing.py"]);
    assert!(!missing_script.status.success());
    assert_eq!(
        text(&missing_script.stderr),
        "\x1b[31mError:\x1b[0m Script not found: missing.py\n"
    );
}

#[test]
fn test_runs_single_files_and_propagates_failures() {
    let temporary = TestDirectory::new("test files");
    fs::write(
        temporary.path.join("passing test.py"),
        "print('single test passed')\n",
    )
    .expect("write passing test");

    let passed = run(&temporary, &["test", "passing test.py"]);
    assert!(passed.status.success(), "{}", text(&passed.stderr));
    assert_eq!(
        text(&passed.stdout),
        "Running passing test.py with pocketpy-ucharm...\n\nsingle test passed\n"
    );
    assert_eq!(text(&passed.stderr), "");

    fs::write(
        temporary.path.join("failing.py"),
        "raise RuntimeError('expected test failure')\n",
    )
    .expect("write failing test");
    let failed = run(&temporary, &["test", "failing.py"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(text(&failed.stdout).contains("RuntimeError: expected test failure"));
    assert!(
        text(&failed.stderr).contains("error: Python execution failed"),
        "{}",
        text(&failed.stderr)
    );
}

#[test]
fn test_runs_the_compatibility_runner_with_the_resolved_runtime() {
    let temporary = TestDirectory::new("compatibility");
    let tested = run(&temporary, &["test", "--compat", "--module", "errno"]);

    assert!(tested.status.success(), "{}", text(&tested.stderr));
    assert!(text(&tested.stdout).contains("errno"));
    assert!(text(&tested.stdout).contains("38/38"));
    assert_eq!(text(&tested.stderr), "");
}

#[test]
fn build_creates_all_three_modes_and_runs_a_universal_binary() {
    let temporary = TestDirectory::new("build modes");
    fs::write(
        temporary.path.join("app.py"),
        "from ucharm import success\nsuccess('built with Rust')\n",
    )
    .expect("write build fixture");

    for (mode, output) in [
        ("single", "app.single"),
        ("executable", "app.wrapper"),
        ("universal", "app.universal"),
    ] {
        let built = run(
            &temporary,
            &["build", "app.py", "-o", output, "--mode", mode],
        );
        assert!(built.status.success(), "{}", text(&built.stderr));
        let output_path = temporary.path.join(output);
        assert!(output_path.is_file());
        assert_ne!(
            fs::metadata(&output_path)
                .expect("build metadata")
                .permissions()
                .mode()
                & 0o111,
            0
        );
    }

    let single = fs::read_to_string(temporary.path.join("app.single")).expect("read single");
    assert!(single.starts_with("#!/usr/bin/env pocketpy-ucharm\n"));
    assert!(single.contains("from charm import"));
    assert!(!single.contains("from ucharm import"));

    let wrapper = fs::read_to_string(temporary.path.join("app.wrapper")).expect("read wrapper");
    assert!(wrapper.starts_with("#!/bin/bash\n"));
    assert!(wrapper.contains("base64 -d"));

    let universal = fs::read(temporary.path.join("app.universal")).expect("read universal");
    let trailer = ucharm_format::Trailer::decode_from_end(&universal).expect("decode trailer");
    trailer
        .validate_layout(universal.len() as u64)
        .expect("validate universal layout");

    let executed = Command::new(temporary.path.join("app.universal"))
        .current_dir(&temporary.path)
        .output()
        .expect("run universal application");
    assert!(executed.status.success(), "{}", text(&executed.stderr));
    assert!(text(&executed.stdout).contains("built with Rust"));
}

#[test]
fn build_packages_every_release_target_with_the_matching_assets() {
    const TARGETS: &[(&str, &[u8], &[u8])] = &[
        (
            "macos-aarch64",
            include_bytes!("../../../cli/src/stubs/loader-macos-aarch64"),
            include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-macos-aarch64"),
        ),
        (
            "macos-x86_64",
            include_bytes!("../../../cli/src/stubs/loader-macos-x86_64"),
            include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-macos-x86_64"),
        ),
        (
            "linux-x86_64",
            include_bytes!("../../../cli/src/stubs/loader-linux-x86_64"),
            include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-linux-x86_64"),
        ),
        (
            "linux-aarch64",
            include_bytes!("../../../cli/src/stubs/loader-linux-aarch64"),
            include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-linux-aarch64"),
        ),
    ];

    let temporary = TestDirectory::new("cross targets");
    fs::write(temporary.path.join("app.py"), "print('cross target')\n")
        .expect("write build fixture");

    for &(target, loader, runtime) in TARGETS {
        let output = format!("app-{target}");
        let built = run(
            &temporary,
            &["build", "app.py", "-o", &output, "--target", target],
        );
        assert!(built.status.success(), "{target}: {}", text(&built.stderr));

        let artifact = fs::read(temporary.path.join(output)).expect("read artifact");
        let trailer = ucharm_format::Trailer::decode_from_end(&artifact).expect("decode trailer");
        assert_eq!(trailer.runtime_offset, loader.len() as u64);
        assert_eq!(trailer.runtime_size, runtime.len() as u64);
        assert_eq!(trailer.python_offset, (loader.len() + runtime.len()) as u64);
        assert_eq!(&artifact[..loader.len()], loader);
        assert_eq!(
            &artifact[loader.len()..loader.len() + runtime.len()],
            runtime
        );
    }
}

#[test]
fn run_transforms_a_script_and_forwards_arguments() {
    let temporary = TestDirectory::new("run command");
    fs::write(
        temporary.path.join("argument test.py"),
        "from ucharm import info\nimport sys\nprint(sys.argv[1])\nprint(sys.argv[2])\n",
    )
    .expect("write script");

    let executed = run(
        &temporary,
        &["run", "argument test.py", "hello world", "--flag"],
    );
    assert!(executed.status.success(), "{}", text(&executed.stderr));
    assert_eq!(text(&executed.stdout), "hello world\n--flag\n");
    assert_eq!(text(&executed.stderr), "");

    let cache_directories = fs::read_dir(temporary.path.join(".cache"))
        .expect("read run cache")
        .collect::<Result<Vec<_>, _>>()
        .expect("read cache entries");
    assert_eq!(cache_directories.len(), 1);
    let runtime = cache_directories[0].path().join("pocketpy-ucharm");
    assert_ne!(
        fs::metadata(runtime)
            .expect("runtime metadata")
            .permissions()
            .mode()
            & 0o111,
        0
    );

    fs::write(
        temporary.path.join("failure.py"),
        "raise RuntimeError('expected failure')\n",
    )
    .expect("write failing script");
    let failed = run(&temporary, &["run", "failure.py"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(text(&failed.stdout).contains("RuntimeError: expected failure"));
    assert!(text(&failed.stderr).contains("error: Python execution failed"));
}

#[test]
fn new_creates_an_executable_project_and_refuses_duplicates() {
    let temporary = TestDirectory::new("new project");
    let created = run(&temporary, &["new", "Test App"]);
    assert!(created.status.success(), "{}", text(&created.stderr));

    let app = temporary.path.join("test_app/test_app.py");
    let content = fs::read_to_string(&app).expect("read generated app");
    assert!(content.contains("Test App - Built with ucharm"));
    assert_ne!(
        fs::metadata(&app)
            .expect("app metadata")
            .permissions()
            .mode()
            & 0o111,
        0
    );

    let duplicate = run(&temporary, &["new", "Test App"]);
    assert!(!duplicate.status.success());
    assert!(text(&duplicate.stderr).contains("already exists"));
}

#[test]
fn minimal_project_stays_in_the_current_directory() {
    let temporary = TestDirectory::new("minimal");
    let created = run(&temporary, &["new", "Minimal App", "--minimal"]);
    assert!(created.status.success(), "{}", text(&created.stderr));
    assert!(temporary.path.join("minimal_app.py").is_file());
    assert!(!temporary.path.join("minimal_app").exists());
}

#[test]
fn init_installs_stubs_and_ai_files_without_overwriting_instructions() {
    let temporary = TestDirectory::new("init");
    let initialized = run(&temporary, &["init", "--all"]);
    assert!(
        initialized.status.success(),
        "{}",
        text(&initialized.stderr)
    );

    let stubs = temporary.path.join(".ucharm/stubs");
    assert_eq!(fs::read_dir(stubs).expect("read stubs").count(), 24);
    assert!(temporary.path.join("pyrightconfig.json").is_file());
    assert!(temporary.path.join("AGENTS.md").is_file());
    assert!(temporary.path.join("CLAUDE.md").is_file());
    assert!(
        temporary
            .path
            .join(".github/copilot-instructions.md")
            .is_file()
    );

    fs::write(temporary.path.join("AGENTS.md"), "keep me\n").expect("replace fixture");
    let repeated = run(&temporary, &["init", "--ai", "agents"]);
    assert!(repeated.status.success());
    assert_eq!(
        fs::read_to_string(temporary.path.join("AGENTS.md")).expect("read fixture"),
        "keep me\n"
    );
    assert!(text(&repeated.stdout).contains("already exists (skipped)"));
}

#[test]
fn rejects_invalid_ai_types_and_path_traversal_names() {
    let temporary = TestDirectory::new("invalid");

    let ai = run(&temporary, &["init", "--ai", "robot"]);
    assert!(!ai.status.success());
    assert!(text(&ai.stderr).contains("invalid --ai type"));

    let traversal = run(&temporary, &["new", "../escape"]);
    assert!(!traversal.status.success());
    assert!(text(&traversal.stderr).contains("path separators"));
}

fn run(temporary: &TestDirectory, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ucharm"))
        .args(arguments)
        .current_dir(&temporary.path)
        .env("UCHARM_CACHE_DIR", temporary.path.join(".cache"))
        .output()
        .expect("run Rust CLI")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(name: &str) -> Self {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ucharm-cli-test-{}-{counter}-{name}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
