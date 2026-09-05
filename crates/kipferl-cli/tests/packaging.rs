use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "kipferl-packaging-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root.canonicalize().unwrap())
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn write(&self, name: &str, source: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kipferl"))
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("cache"))
            .args(args)
            .output()
            .unwrap()
    }
    #[expect(
        clippy::unwrap_used,
        reason = "Failure to create this isolated fixture or execute its child process must fail the test immediately"
    )]
    fn run(&self, binary: &str, args: &[&str]) -> Output {
        Command::new(self.0.join(binary))
            .current_dir(&self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("loader-cache"))
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn universal_carries_packages_lazy_modules_and_binary_assets_after_source_removal() {
    let fixture = Fixture::new();
    fixture.write("source/app.py", "import os\nfrom package import greet\nprint(greet())\nwith open(os.path.join(os.path.dirname(__file__), 'assets/payload.bin'), 'rb') as data:\n    assert data.read() == b'\\x00\\xffhello'\nwith open('caller.txt', 'r') as caller:\n    print(caller.read())\n");
    fixture.write("source/package/__init__.py", "from .helpers import greet\n");
    fixture.write(
        "source/package/helpers.py",
        "def greet():\n    import message\n    return message.VALUE\n",
    );
    fixture.write("source/message.py", "VALUE = 'portable package'\n");
    fixture.write("source/assets/payload.bin", b"\x00\xffhello");
    fixture.write("caller.txt", "caller cwd preserved");
    let build = fixture.cli(&[
        "build",
        "source/app.py",
        "--asset",
        "assets",
        "-o",
        "dist/program",
    ]);
    success(&build);
    assert!(String::from_utf8_lossy(&build.stdout).contains("Runtime profile \u{1b}[1mcore"));
    fs::remove_dir_all(fixture.0.join("source")).unwrap();
    let run = fixture.run("dist/program", &[]);
    success(&run);
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "portable package\ncaller cwd preserved\n"
    );
}

#[test]
fn transitive_imports_select_full_profile_and_keep_local_traceback_lines() {
    let fixture = Fixture::new();
    fixture.write("src/app.py", "import helper\nhelper.fail()\n");
    fixture.write(
        "src/helper.py",
        "import re\ndef fail():\n    raise RuntimeError('precise local failure')\n",
    );
    let build = fixture.cli(&["build", "src/app.py", "-o", "program"]);
    success(&build);
    assert!(String::from_utf8_lossy(&build.stdout).contains("Runtime profile \u{1b}[1mfull"));
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    let run = fixture.run("program", &[]);
    assert!(!run.status.success());
    let trace = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(trace.contains("File \"helper.py\", line 3"), "{trace}");
    assert!(trace.contains("File \"app.py\", line 2"), "{trace}");
    assert!(
        trace.len() < 3500,
        "traceback included encoded program: {trace}"
    );
}

#[test]
fn configuration_assets_resolve_at_project_root_with_subdirectory_entry() {
    let fixture = Fixture::new();
    fixture.write(
        "project/kipferl.json",
        r#"{"entry":"src/app.py","output":"dist/program","assets":["data"]}"#,
    );
    fixture.write("project/src/app.py", "import os\nimport helper\nwith open(os.path.join(os.path.dirname(__file__), '../data/message.txt'), 'r') as data:\n    print(data.read(), helper.value)\n");
    fixture.write("project/src/helper.py", "value = 7\n");
    fixture.write("project/data/message.txt", "configured asset");
    let output = Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .current_dir(fixture.0.join("project/src"))
        .args(["build"])
        .output()
        .unwrap();
    success(&output);
    fs::rename(
        fixture.0.join("project/dist/program"),
        fixture.0.join("program"),
    )
    .unwrap();
    fs::remove_dir_all(fixture.0.join("project")).unwrap();
    let run = fixture.run("program", &[]);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "configured asset 7\n");
}

#[test]
fn unsupported_imports_and_syntax_fail_before_overwriting_output_or_running_app() {
    let fixture = Fixture::new();
    fixture.write("program", "old output");
    for (source, expected) in [
        ("import requests\n", "unsupported import 'requests'"),
        (
            "from urllib.request import urlopen\n",
            "unsupported import 'urllib.request'",
        ),
        (
            "print('must not execute')\nif :\n",
            "Python syntax check failed",
        ),
    ] {
        fixture.write("app.py", source);
        let build = fixture.cli(&["build", "app.py", "-o", "program", "--full-runtime"]);
        assert!(!build.status.success());
        assert!(
            String::from_utf8_lossy(&build.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("program")).unwrap(),
            "old output"
        );
    }
    fixture.write(
        "app.py",
        "raise RuntimeError('build must not execute me')\n",
    );
    success(&fixture.cli(&["build", "app.py", "-o", "program"]));
}

#[test]
fn unsupported_imports_in_dependencies_are_reported_with_local_source_location() {
    let fixture = Fixture::new();
    fixture.write("app.py", "import helper\n");
    fixture.write("helper.py", "# transitive\nimport pandas\n");
    let build = fixture.cli(&["build", "app.py", "-o", "program"]);
    assert!(!build.status.success());
    assert!(
        String::from_utf8_lossy(&build.stderr).contains("helper.py:2: unsupported import 'pandas'")
    );
    assert!(!fixture.0.join("program").exists());
}

#[test]
fn rejects_asset_escapes_symlinks_and_oversized_assets() {
    let fixture = Fixture::new();
    fixture.write("src/app.py", "print('app')\n");
    fixture.write("outside.txt", "secret");
    std::os::unix::fs::symlink(
        fixture.0.join("outside.txt"),
        fixture.0.join("src/link.txt"),
    )
    .unwrap();
    for asset in ["../outside.txt", "link.txt"] {
        let build = fixture.cli(&["build", "src/app.py", "-o", "program", "--asset", asset]);
        assert!(!build.status.success(), "accepted {asset}");
        assert!(!fixture.0.join("program").exists());
    }
    fixture.write("src/large.bin", []);
    fs::OpenOptions::new()
        .write(true)
        .open(fixture.0.join("src/large.bin"))
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();
    let build = fixture.cli(&[
        "build",
        "src/app.py",
        "-o",
        "program",
        "--asset",
        "large.bin",
    ]);
    assert!(!build.status.success());
    assert!(String::from_utf8_lossy(&build.stderr).contains("8 MiB file limit"));
}

#[test]
fn development_local_imports_preserve_cwd_original_files_and_refresh_changes() {
    let fixture = Fixture::new();
    fixture.write("source/app.py", "import os\nimport helper\nprint(helper.value)\nprint(__file__)\nprint(helper.__file__)\nprint(os.getcwd())\n");
    for value in [12, 42] {
        fixture.write("source/helper.py", format!("value = {value}\n"));
        let run = fixture.cli(&["run", "source/app.py"]);
        success(&run);
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            format!(
                "{value}\n{}\n{}\n{}\n",
                fixture.0.join("source/app.py").display(),
                fixture.0.join("source/helper.py").display(),
                fixture.0.display()
            )
        );
    }
}

#[test]
fn supports_dotted_local_imports_with_and_without_aliases() {
    let fixture = Fixture::new();
    fixture.write("src/app.py", "import package.helper\nimport package.helper as helper\nprint(package.helper.VALUE, helper.VALUE)\n");
    fixture.write("src/package/__init__.py", "NAME = 'package'\n");
    fixture.write("src/package/helper.py", "VALUE = 42\n");
    success(&fixture.cli(&["build", "src/app.py", "-o", "program"]));
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    let run = fixture.run("program", &[]);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42 42\n");
}

#[test]
fn application_exit_codes_and_resource_cleanup_work() {
    let fixture = Fixture::new();
    fixture.write(
        "src/app.py",
        "import os, sys\nprint(os.path.dirname(__file__))\nsys.exit(int(sys.argv[1]))\n",
    );
    fixture.write("src/data.txt", "resource");
    success(&fixture.cli(&[
        "build",
        "src/app.py",
        "-o",
        "program",
        "--asset",
        "data.txt",
    ]));
    for code in [0, 7] {
        let run = fixture.run("program", &[&code.to_string()]);
        assert_eq!(
            run.status.code(),
            Some(code),
            "{}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        let path = stdout.lines().next().unwrap();
        assert!(
            !Path::new(path).exists(),
            "temporary resources leaked at {path}"
        );
    }
}

#[test]
fn retains_empty_asset_directories() {
    let fixture = Fixture::new();
    fixture.write("src/app.py", "import os\nassert os.path.isdir(os.path.join(os.path.dirname(__file__), 'empty/nested'))\nprint('empty directory bundled')\n");
    fs::create_dir_all(fixture.0.join("src/empty/nested")).unwrap();
    success(&fixture.cli(&["build", "src/app.py", "-o", "program", "--asset", "empty"]));
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    let run = fixture.run("program", &[]);
    success(&run);
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "empty directory bundled\n"
    );
}

#[test]
fn rejects_fifo_assets_before_opening_them() {
    let fixture = Fixture::new();
    fixture.write("app.py", "print('app')\n");
    let fifo = std::ffi::CString::new(fixture.0.join("pipe").to_str().unwrap()).unwrap();
    // SAFETY: the CString remains live and points to a valid, new test pathname.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let build = fixture.cli(&["build", "app.py", "-o", "program", "--asset", "pipe"]);
    assert!(!build.status.success());
    assert!(String::from_utf8_lossy(&build.stderr).contains("expected a regular file"));
    assert!(!fixture.0.join("program").exists());
}

#[test]
fn importing_package_child_executes_parent_initialization_once() {
    let fixture = Fixture::new();
    fixture.write(
        "src/app.py",
        "from package.child import VALUE\nfrom package import child\nprint(VALUE, child.VALUE)\n",
    );
    fixture.write("src/package/__init__.py", "print('package initialized')\n");
    fixture.write("src/package/child.py", "VALUE = 42\n");
    success(&fixture.cli(&["build", "src/app.py", "-o", "program"]));
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    let run = fixture.run("program", &[]);
    success(&run);
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "package initialized\n42 42\n"
    );
}

#[test]
fn module_names_cannot_exceed_the_runtime_limit() {
    let fixture = Fixture::new();
    let name = "a".repeat(64);
    fixture.write("app.py", format!("import {name}\n"));
    fixture.write(&format!("{name}.py"), "VALUE = 42\n");
    let build = fixture.cli(&["build", "app.py", "-o", "program"]);
    assert!(!build.status.success());
    assert!(String::from_utf8_lossy(&build.stderr).contains("63-byte limit"));
}

#[test]
fn dotted_imports_keep_semicolon_statements_and_support_native_aliases() {
    let fixture = Fixture::new();
    fixture.write("src/app.py", "import package.helper; print(package.helper.VALUE)\nimport http.client as client; print(callable(client.HTTPSConnection))\nimport package.\\\nhelper as helper; print(helper.VALUE)\n");
    fixture.write("src/package/__init__.py", "");
    fixture.write("src/package/helper.py", "VALUE = 42\n");
    success(&fixture.cli(&["build", "src/app.py", "-o", "program"]));
    fs::remove_dir_all(fixture.0.join("src")).unwrap();
    let run = fixture.run("program", &[]);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\nTrue\n42\n");
}

#[test]
fn standalone_entry_globals_execute_with_module_semantics() {
    let fixture = Fixture::new();
    fixture.write("app.py", "global counter\ncounter = 10\ndef bump():\n    global counter\n    counter += 1\nbump()\nprint(counter)\n");
    let run = fixture.cli(&["run", "app.py"]);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "11\n");
    success(&fixture.cli(&["build", "app.py", "-o", "global-app"]));
    let packaged = fixture.run("global-app", &[]);
    success(&packaged);
    assert_eq!(String::from_utf8_lossy(&packaged.stdout), "11\n");
}
