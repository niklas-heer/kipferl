use std::path::PathBuf;
use std::process::{Command, Output};

#[test]
fn passes_all_filesystem_wave_compatibility_fixtures() {
    for (module, source, summary) in [
        (
            "os",
            include_str!("../../../tests/cpython/test_os.py"),
            "Results: 45 passed, 0 failed, 0 skipped",
        ),
        (
            "glob",
            include_str!("../../../tests/cpython/test_glob.py"),
            "Results: 4 passed, 0 failed, 0 skipped",
        ),
        (
            "tempfile",
            include_str!("../../../tests/cpython/test_tempfile.py"),
            "Results: 12 passed, 0 failed, 0 skipped",
        ),
        (
            "shutil",
            include_str!("../../../tests/cpython/test_shutil.py"),
            "Results: 8 passed, 0 failed, 0 skipped",
        ),
    ] {
        let output = run_runtime(source);
        assert!(
            output.status.success(),
            "{module} fixture failed:\n{}",
            diagnostic(&output)
        );
        assert!(
            text(&output.stdout).contains(summary),
            "{module} summary missing:\n{}",
            diagnostic(&output)
        );
        assert_eq!(text(&output.stderr), "", "{module}");
    }

    let pathlib = fixture_path("test_pathlib.py");
    let output = run_file(&pathlib);
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert!(
        text(&output.stdout).contains("Results: 40 passed, 0 failed, 0 skipped"),
        "{}",
        diagnostic(&output)
    );
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn matches_cpython_for_deterministic_path_operations() {
    let source = concat!(
        "import os\n",
        "from pathlib import Path\n",
        "print(os.path.join('alpha', 'beta', 'file.tar.gz'))\n",
        "print(os.path.split('/alpha/beta.txt'))\n",
        "print(os.path.splitext('/alpha/file.tar.gz'))\n",
        "print(os.path.normpath('/alpha/./beta/../gamma'))\n",
        "path = Path('/alpha', 'beta', 'file.tar.gz')\n",
        "print(str(path))\n",
        "print(path.name, path.stem, path.suffix, str(path.parent))\n",
        "print(str(path.with_name('other.txt')))\n",
        "print(str(path.with_suffix('.zip')))\n",
        "print(path.is_absolute())\n",
    );
    let rust = run_runtime(source);
    assert!(rust.status.success(), "{}", diagnostic(&rust));
    let cpython = Command::new("python3")
        .args(["-c", source])
        .output()
        .expect("run CPython differential oracle");
    assert!(cpython.status.success(), "{}", diagnostic(&cpython));
    assert_eq!(rust.stdout, cpython.stdout);
    assert_eq!(text(&rust.stderr), "");
}

#[test]
fn preserves_recursive_filesystem_lifecycle_and_errors_under_stress() {
    let output = run_runtime(concat!(
        "import glob, os, shutil, tempfile\n",
        "from pathlib import Path\n",
        "root = tempfile.mkdtemp('/tmp/kipferl_fs_wave_')\n",
        "nested = os.path.join(root, 'a', 'b')\n",
        "os.makedirs(nested, exist_ok=True)\n",
        "for i in range(200):\n",
        "    path = os.path.join(nested, 'item' + str(i) + '.txt')\n",
        "    with open(path, 'w') as output:\n",
        "        output.write('value-' + str(i))\n",
        "matches = glob.glob(os.path.join(root, '**', '*.txt'), None, None, True)\n",
        "assert len(matches) == 200\n",
        "assert Path(nested).exists() and Path(nested).is_dir()\n",
        "source = os.path.join(nested, 'item0.txt')\n",
        "copied = os.path.join(root, 'copied.txt')\n",
        "moved = os.path.join(root, 'moved.txt')\n",
        "assert shutil.copy(source, copied) == copied\n",
        "assert shutil.move(copied, moved) == moved\n",
        "assert Path(moved).is_file() and not Path(copied).exists()\n",
        "assert os.path.abspath('.') == str(Path('.').resolve())\n",
        "for operation in (\n",
        "    lambda: os.stat(os.path.join(root, 'missing')),\n",
        "    lambda: os.remove(os.path.join(root, 'missing')),\n",
        "    lambda: os.listdir(os.path.join(root, 'missing')),\n",
        "):\n",
        "    caught = False\n",
        "    try:\n",
        "        operation()\n",
        "    except OSError:\n",
        "        caught = True\n",
        "    assert caught\n",
        "shutil.rmtree(root)\n",
        "assert not os.path.exists(root)\n",
    ));
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn exposes_script_file_to_python() {
    let path = std::env::temp_dir().join(format!("kipferl-file-test-{}.py", std::process::id()));
    std::fs::write(&path, "print(__file__)").expect("write temporary script");
    let output = run_file(&path);
    std::fs::remove_file(&path).expect("remove temporary script");
    assert!(output.status.success(), "{}", diagnostic(&output));
    assert_eq!(text(&output.stdout).trim(), path.to_string_lossy());
    assert_eq!(text(&output.stderr), "");
}

fn run_runtime(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run Rust PocketPy runtime")
}

fn run_file(path: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .arg(path)
        .output()
        .expect("run Rust PocketPy script")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/cpython")
        .join(name)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        text(&output.stdout),
        text(&output.stderr)
    )
}
