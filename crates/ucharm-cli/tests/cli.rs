use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reports_version_help_unknown_and_unmigrated_commands() {
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

    let build = run(&temporary, &["build"]);
    assert!(!build.status.success());
    assert!(text(&build.stderr).contains("has not migrated to Rust yet"));
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
    Command::new(env!("CARGO_BIN_EXE_ucharm-rs"))
        .args(arguments)
        .current_dir(&temporary.path)
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
