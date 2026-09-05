use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reports_version_help_and_unknown_commands() {
    let temporary = TestDirectory::new("dispatch");

    let version = run(&temporary, &["--version"]);
    assert!(version.status.success());
    assert!(text(&version.stdout).contains(kipferl_cli::version()));

    let help = run(&temporary, &["--help"]);
    assert!(help.status.success());
    assert!(text(&help.stdout).contains("COMMANDS"));

    let unknown = run(&temporary, &["unknown"]);
    assert!(!unknown.status.success());
    assert!(text(&unknown.stderr).contains("Unknown command"));

    let build_help = run(&temporary, &["build", "--help"]);
    assert!(build_help.status.success());
    assert!(text(&build_help.stdout).contains("Build standalone binaries"));

    let targets = run(&temporary, &["build", "--targets"]);
    assert!(targets.status.success());
    for target in [
        "macos-aarch64",
        "macos-x86_64",
        "linux-aarch64",
        "linux-x86_64",
    ] {
        assert!(text(&targets.stdout).contains(target), "missing {target}");
    }

    let test_help = run(&temporary, &["test", "--help"]);
    assert!(test_help.status.success());
    assert!(text(&test_help.stdout).contains("CPython Compatibility Testing"));

    let run_help = run(&temporary, &["run", "--help"]);
    assert!(run_help.status.success());
    assert!(text(&run_help.stdout).contains("kipferl run <script.py> [args...]"));

    let dev_help = run(&temporary, &["dev", "--help"]);
    assert!(dev_help.status.success());
    assert!(text(&dev_help.stdout).contains("kipferl dev [OPTIONS] <script.py>"));

    let no_script = run(&temporary, &["run"]);
    assert!(!no_script.status.success());
    assert_eq!(
        text(&no_script.stderr),
        "\x1b[31mError:\x1b[0m No script specified\nUsage: kipferl run <script.py> [args...]\n"
    );

    let missing_script = run(&temporary, &["run", "missing.py"]);
    assert!(!missing_script.status.success());
    assert_eq!(
        text(&missing_script.stderr),
        "\x1b[31mError:\x1b[0m Script not found: missing.py\n"
    );
}

#[test]
fn dev_runs_immediately_and_restarts_after_an_edit() {
    let temporary = TestDirectory::new("dev restart");
    let project = temporary.path.join("project");
    fs::create_dir(&project).expect("create project directory");
    let script = project.join("app.py");
    let config = temporary.path.join("settings.txt");
    fs::write(&script, "print('first run')\n").expect("write initial script");
    fs::write(&config, "initial settings\n").expect("write watched config");

    let process = Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .args([
            "dev",
            "--debounce",
            "25",
            "--watch",
            "settings.txt",
            "project/app.py",
        ])
        .current_dir(&temporary.path)
        .env("KIPFERL_CACHE_DIR", temporary.path.join(".cache"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start dev command");
    let mut process = ChildGuard(process);
    let stdout = process.0.stdout.take().expect("capture dev stdout");
    let (lines, received) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if lines.send(line).is_err() {
                break;
            }
        }
    });

    wait_for_output(&received, "first run");
    fs::write(&config, "changed settings\n").expect("edit extra watched file");
    wait_for_output(&received, "Change detected, restarting");
    wait_for_output(&received, "first run");
    fs::write(&script, "print('script edit')\n").expect("edit watched script");
    wait_for_output(&received, "Change detected, restarting");
    wait_for_output(&received, "script edit");

    process.stop();
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
        "Running passing test.py with pocketpy-kipferl...\n\nsingle test passed\n"
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
        "from kipferl import success\nsuccess('built with Rust')\n",
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
        if mode == "universal" {
            assert!(text(&built.stdout).contains("Runtime profile \x1b[1mcore"));
        }
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
    assert!(single.starts_with("#!/usr/bin/env pocketpy-kipferl\n"));
    assert!(single.contains("from tui import"));
    assert!(!single.contains("from kipferl import"));

    let wrapper = fs::read_to_string(temporary.path.join("app.wrapper")).expect("read wrapper");
    assert!(wrapper.starts_with("#!/bin/bash\n"));
    assert!(wrapper.contains("base64 -d"));

    let universal = fs::read(temporary.path.join("app.universal")).expect("read universal");
    let trailer = kipferl_format::Trailer::decode_from_end(&universal).expect("decode trailer");
    trailer
        .validate_layout(u64::try_from(universal.len()).expect("artifact size fits u64"))
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
    type TargetAssets = (&'static str, &'static [u8], &'static [u8], &'static [u8]);
    const TARGETS: &[TargetAssets] = &[
        (
            "macos-aarch64",
            include_bytes!("../assets/kipferl-loader-macos-aarch64"),
            include_bytes!("../assets/pocketpy-kipferl-macos-aarch64"),
            include_bytes!("../assets/pocketpy-kipferl-core-macos-aarch64"),
        ),
        (
            "macos-x86_64",
            include_bytes!("../assets/kipferl-loader-macos-x86_64"),
            include_bytes!("../assets/pocketpy-kipferl-macos-x86_64"),
            include_bytes!("../assets/pocketpy-kipferl-core-macos-x86_64"),
        ),
        (
            "linux-x86_64",
            include_bytes!("../assets/kipferl-loader-linux-x86_64"),
            include_bytes!("../assets/pocketpy-kipferl-linux-x86_64"),
            include_bytes!("../assets/pocketpy-kipferl-core-linux-x86_64"),
        ),
        (
            "linux-aarch64",
            include_bytes!("../assets/kipferl-loader-linux-aarch64"),
            include_bytes!("../assets/pocketpy-kipferl-linux-aarch64"),
            include_bytes!("../assets/pocketpy-kipferl-core-linux-aarch64"),
        ),
    ];

    let temporary = TestDirectory::new("cross targets");
    fs::write(temporary.path.join("app.py"), "print('cross target')\n")
        .expect("write build fixture");

    for &(target, loader, runtime, core_runtime) in TARGETS {
        let output = format!("app-{target}");
        let built = run(
            &temporary,
            &[
                "build",
                "app.py",
                "-o",
                &output,
                "--target",
                target,
                "--full-runtime",
            ],
        );
        assert!(built.status.success(), "{target}: {}", text(&built.stderr));

        let artifact = fs::read(temporary.path.join(output)).expect("read artifact");
        let trailer = kipferl_format::Trailer::decode_from_end(&artifact).expect("decode trailer");
        assert_eq!(
            trailer.runtime_offset,
            u64::try_from(loader.len()).expect("artifact size fits u64")
        );
        assert_eq!(
            trailer.runtime_size,
            u64::try_from(runtime.len()).expect("artifact size fits u64")
        );
        assert_eq!(
            trailer.python_offset,
            u64::try_from(loader.len() + runtime.len()).expect("artifact size fits u64")
        );
        assert_eq!(&artifact[..loader.len()], loader);
        assert_eq!(
            &artifact[loader.len()..loader.len() + runtime.len()],
            runtime
        );

        let core_output = format!("app-{target}-core");
        let core_built = run(
            &temporary,
            &["build", "app.py", "-o", &core_output, "--target", target],
        );
        assert!(
            core_built.status.success(),
            "{target}: {}",
            text(&core_built.stderr)
        );
        assert!(text(&core_built.stdout).contains("Runtime profile \x1b[1mcore"));

        let core_artifact = fs::read(temporary.path.join(core_output)).expect("read core artifact");
        let core_trailer =
            kipferl_format::Trailer::decode_from_end(&core_artifact).expect("decode core trailer");
        assert_eq!(
            core_trailer.runtime_offset,
            u64::try_from(loader.len()).expect("artifact size fits u64")
        );
        assert_eq!(
            core_trailer.runtime_size,
            u64::try_from(core_runtime.len()).expect("artifact size fits u64")
        );
        assert_eq!(
            &core_artifact[loader.len()..loader.len() + core_runtime.len()],
            core_runtime
        );
    }
}

#[test]
fn build_selects_full_runtime_for_optional_or_dynamic_imports() {
    let temporary = TestDirectory::new("runtime profiles");
    for (name, source) in [
        (
            "sqlite",
            "import sqlite3\ndb = sqlite3.connect(':memory:')\nprint('sqlite ok')\n",
        ),
        (
            "dynamic",
            "name = 'json'\nmodule = __import__(name)\nprint(module.dumps(1))\n",
        ),
    ] {
        let script = format!("{name}.py");
        let output = format!("{name}.app");
        fs::write(temporary.path.join(&script), source).expect("write profile fixture");
        let built = run(&temporary, &["build", &script, "-o", &output]);
        assert!(built.status.success(), "{}", text(&built.stderr));
        assert!(text(&built.stdout).contains("Runtime profile \x1b[1mfull"));

        let artifact = fs::read(temporary.path.join(output)).expect("read full artifact");
        let trailer = kipferl_format::Trailer::decode_from_end(&artifact).expect("decode trailer");
        assert_eq!(
            trailer.runtime_size,
            u64::try_from(crate_runtime().len()).expect("artifact size fits u64"),
            "{name}"
        );
    }
}

const fn crate_runtime() -> &'static [u8] {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return include_bytes!("../assets/pocketpy-kipferl-macos-aarch64");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return include_bytes!("../assets/pocketpy-kipferl-macos-x86_64");
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return include_bytes!("../assets/pocketpy-kipferl-linux-aarch64");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return include_bytes!("../assets/pocketpy-kipferl-linux-x86_64");
    #[allow(
        unreachable_code,
        reason = "Each supported target returns its cfg-selected constant; this fallback only compiles for other targets"
    )]
    &[]
}

#[test]
fn run_transforms_a_script_and_forwards_arguments() {
    let temporary = TestDirectory::new("run command");
    fs::write(
        temporary.path.join("argument test.py"),
        "from kipferl import info\nimport sys\nprint(sys.argv[1])\nprint(sys.argv[2])\n",
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
    let runtime = cache_directories[0].path().join("pocketpy-kipferl");
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
fn run_repairs_same_size_cache_corruption() {
    let temporary = TestDirectory::new("cache repair");
    fs::write(temporary.path.join("app.py"), "print('original')\n").expect("write script");
    let first = run(&temporary, &["run", "app.py"]);
    assert!(first.status.success(), "{}", text(&first.stderr));
    let cache = fs::read_dir(temporary.path.join(".cache"))
        .expect("read cache")
        .next()
        .expect("cache directory")
        .expect("cache entry")
        .path();
    for entry in fs::read_dir(&cache).expect("read cached files") {
        let path = entry.expect("cached file").path();
        let mut bytes = fs::read(&path).expect("read cached file");
        if path.extension().is_some_and(|extension| extension == "py") {
            let script = text(&bytes).replace("original", "tampered");
            assert_eq!(script.len(), bytes.len());
            bytes = script.into_bytes();
        } else {
            bytes[0] ^= 0xff;
        }
        fs::write(&path, bytes).expect("corrupt cached file without changing length");
    }

    let repaired = run(&temporary, &["run", "app.py"]);
    assert!(repaired.status.success(), "{}", text(&repaired.stderr));
    assert_eq!(text(&repaired.stdout), "original\n");

    for entry in fs::read_dir(&cache).expect("read repaired cache") {
        let path = entry.expect("cached file").path();
        let mode = if path.extension().is_some_and(|extension| extension == "py") {
            0o000
        } else {
            0o111
        };
        fs::set_permissions(&path, fs::Permissions::from_mode(mode))
            .expect("remove cache read permissions");
    }
    let repaired = run(&temporary, &["run", "app.py"]);
    assert!(repaired.status.success(), "{}", text(&repaired.stderr));
    assert_eq!(text(&repaired.stdout), "original\n");
    for entry in fs::read_dir(&cache).expect("read accessible cache") {
        let path = entry.expect("cached file").path();
        let expected_mode = if path.extension().is_some_and(|extension| extension == "py") {
            0o600
        } else {
            0o755
        };
        assert_eq!(
            fs::metadata(path)
                .expect("cache metadata")
                .permissions()
                .mode()
                & 0o777,
            expected_mode
        );
    }
}

#[test]
fn run_and_build_enforce_the_same_script_size_boundary() {
    let temporary = TestDirectory::new("script size");
    let source = temporary.path.join("app.py");
    let mut bytes = vec![b' '; 1024 * 1024];
    bytes[0] = b'#';
    fs::write(&source, &bytes).expect("write maximum-size script");
    for arguments in [
        vec!["run", "app.py"],
        vec!["build", "app.py", "-o", "app.single", "--mode", "single"],
    ] {
        let output = run(&temporary, &arguments);
        assert!(output.status.success(), "{}", text(&output.stderr));
    }
    let original_artifact = fs::read(temporary.path.join("app.single")).expect("read artifact");
    bytes.push(b' ');
    fs::write(&source, &bytes).expect("write oversized script");
    for arguments in [
        vec!["run", "app.py"],
        vec!["build", "app.py", "-o", "app.single", "--mode", "single"],
    ] {
        let output = run(&temporary, &arguments);
        assert!(!output.status.success());
    }
    assert_eq!(
        fs::read(temporary.path.join("app.single")).expect("read preserved artifact"),
        original_artifact
    );
}

#[test]
fn build_handles_parenthesized_legacy_imports_in_all_output_modes() {
    let temporary = TestDirectory::new("multiline imports");
    fs::write(
        temporary.path.join("app.py"),
        "from kipferl import (\n    success, # a closing ) in a comment\n    confirm,\n)\nprint(callable(confirm))\nsuccess('multiline works')\n",
    )
    .expect("write multiline fixture");
    let executed = run(&temporary, &["run", "app.py"]);
    assert!(executed.status.success(), "{}", text(&executed.stderr));
    assert!(text(&executed.stdout).contains("multiline works"));

    for mode in ["single", "executable", "universal"] {
        let output = format!("app.{mode}");
        let built = run(
            &temporary,
            &["build", "app.py", "-o", &output, "--mode", mode],
        );
        assert!(built.status.success(), "{}", text(&built.stderr));
        if mode == "single" {
            let executed = run(&temporary, &["run", &output]);
            assert!(executed.status.success(), "{}", text(&executed.stderr));
            assert!(text(&executed.stdout).contains("multiline works"));
        } else if mode == "universal" {
            let executed = Command::new(temporary.path.join(&output))
                .current_dir(&temporary.path)
                .env("KIPFERL_CACHE_DIR", temporary.path.join(".cache"))
                .output()
                .expect("run multiline universal app");
            assert!(executed.status.success(), "{}", text(&executed.stderr));
            assert!(text(&executed.stdout).contains("True\n"));
            assert!(text(&executed.stdout).contains("multiline works"));
        }
    }
}

#[test]
fn run_and_build_preserve_import_examples_inside_multiline_strings() {
    let temporary = TestDirectory::new("literal import examples");
    let literal =
        "\nfrom kipferl import (\nexample text\nfrom kipferl import confirm\nimport kipferl\n";
    let expected = format!("{literal}hello\n");
    for quote in ["\"\"\"", "'''"] {
        let source = format!("doc = {quote}{literal}{quote}\nprint(doc, end='')\nprint('hello')\n");
        fs::write(temporary.path.join("app.py"), &source).expect("write literal fixture");
        let executed = run(&temporary, &["run", "app.py"]);
        assert!(executed.status.success(), "{}", text(&executed.stderr));
        assert_eq!(text(&executed.stdout), expected);

        for mode in ["single", "universal"] {
            let output = format!("app.{mode}");
            let built = run(
                &temporary,
                &["build", "app.py", "-o", &output, "--mode", mode],
            );
            assert!(built.status.success(), "{}", text(&built.stderr));
            let executed = if mode == "single" {
                run(&temporary, &["run", &output])
            } else {
                assert!(text(&built.stdout).contains("Runtime profile \x1b[1mcore"));
                Command::new(temporary.path.join(&output))
                    .current_dir(&temporary.path)
                    .env("KIPFERL_CACHE_DIR", temporary.path.join(".cache"))
                    .output()
                    .expect("run literal universal app")
            };
            assert!(executed.status.success(), "{}", text(&executed.stderr));
            assert_eq!(text(&executed.stdout), expected);
        }
    }
}

#[test]
fn build_replaces_an_output_symlink_without_modifying_its_target() {
    let temporary = TestDirectory::new("atomic build");
    fs::write(temporary.path.join("app.py"), "print('replacement')\n").expect("write source");
    let original = temporary.path.join("original");
    fs::write(&original, "keep the original\n").expect("write original artifact");
    let output = temporary.path.join("app");
    std::os::unix::fs::symlink(&original, &output).expect("link output");
    let built = run(
        &temporary,
        &["build", "app.py", "-o", "app", "--mode", "single"],
    );
    assert!(built.status.success(), "{}", text(&built.stderr));
    assert_eq!(
        fs::read_to_string(&original).expect("read original"),
        "keep the original\n"
    );
    assert!(
        !fs::symlink_metadata(&output)
            .expect("output metadata")
            .file_type()
            .is_symlink()
    );
    assert!(
        fs::read_to_string(&output)
            .expect("read output")
            .contains("replacement")
    );

    let long_name = "a".repeat(250);
    let built = run(
        &temporary,
        &["build", "app.py", "-o", &long_name, "--mode", "single"],
    );
    assert!(built.status.success(), "{}", text(&built.stderr));
    assert!(temporary.path.join(long_name).is_file());
}

#[test]
fn new_preserves_quotes_in_names_as_python_string_data() {
    let temporary = TestDirectory::new("quoted project");
    let name = "A \"\"\"quoted\"\"\" app";
    let created = run(&temporary, &["new", name, "--minimal"]);
    assert!(created.status.success(), "{}", text(&created.stderr));
    let app = temporary.path.join("a_\"\"\"quoted\"\"\"_app.py");
    let checked = Command::new("python3")
        .args([
            "-c",
            "import ast, pathlib, sys; tree = ast.parse(pathlib.Path(sys.argv[1]).read_text()); assert ast.get_docstring(tree) == sys.argv[2] + ' - Built with kipferl'",
        ])
        .arg(&app)
        .arg(name)
        .output()
        .expect("validate generated Python");
    assert!(checked.status.success(), "{}", text(&checked.stderr));
}

#[test]
fn new_creates_an_executable_project_and_refuses_duplicates() {
    let temporary = TestDirectory::new("new project");
    let created = run(&temporary, &["new", "Test App"]);
    assert!(created.status.success(), "{}", text(&created.stderr));

    let help = run(&temporary, &["--help"]);
    let banner = text(&help.stdout)
        .lines()
        .take(6)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(banner.contains("Kipferl"));
    assert!(banner.contains(kipferl_cli::version()));
    assert!(text(&created.stdout).starts_with(&banner));

    let app = temporary.path.join("test_app/test_app.py");
    let content = fs::read_to_string(&app).expect("read generated app");
    assert!(content.contains("Test App - Built with kipferl"));
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

    let stubs = temporary.path.join(".kipferl/stubs");
    let canonical = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stubs");
    let mut installed_names = fs::read_dir(&stubs)
        .expect("read installed stubs")
        .map(|entry| entry.expect("read installed stub entry").file_name())
        .collect::<Vec<_>>();
    let mut canonical_names = fs::read_dir(&canonical)
        .expect("read canonical stubs")
        .filter_map(|entry| {
            let entry = entry.expect("read canonical stub entry");
            (entry.path().extension().and_then(|value| value.to_str()) == Some("pyi"))
                .then(|| entry.file_name())
        })
        .collect::<Vec<_>>();
    installed_names.sort();
    canonical_names.sort();
    assert_eq!(installed_names, canonical_names);
    for name in canonical_names {
        assert_eq!(
            fs::read(stubs.join(&name)).expect("read installed stub"),
            fs::read(canonical.join(name)).expect("read canonical stub")
        );
    }
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

#[expect(
    clippy::expect_used,
    reason = "A failed fixture directory creation or CLI spawn must fail the test with context"
)]
fn run(temporary: &TestDirectory, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kipferl"))
        .args(arguments)
        .current_dir(&temporary.path)
        .env("KIPFERL_CACHE_DIR", temporary.path.join(".cache"))
        .output()
        .expect("run Rust CLI")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[expect(
    clippy::panic,
    reason = "The bounded test wait must fail with all observed output when the child never produces its expected event"
)]
fn wait_for_output(lines: &mpsc::Receiver<String>, expected: &str) {
    let mut observed = Vec::new();
    loop {
        let line = lines
            .recv_timeout(Duration::from_secs(10))
            .unwrap_or_else(|error| {
                panic!("timed out waiting for {expected:?}: {error}; output: {observed:?}")
            });
        let matches = line.contains(expected);
        observed.push(line);
        if matches {
            return;
        }
    }
}

struct TestDirectory {
    path: PathBuf,
}

struct ChildGuard(std::process::Child);

impl ChildGuard {
    fn stop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

impl TestDirectory {
    #[expect(
        clippy::expect_used,
        reason = "A failed fixture directory creation or CLI spawn must fail the test with context"
    )]
    fn new(name: &str) -> Self {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "kipferl-cli-test-{}-{counter}-{name}",
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
