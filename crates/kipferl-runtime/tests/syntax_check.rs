use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "kipferl-syntax-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn check(&self, files: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
            .args(["--check-syntax", "--"])
            .args(files)
            .current_dir(&self.0)
            .output()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn compiles_modules_with_globals_without_imports_or_side_effects() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.0.join("globals.py"),
        "global value\nvalue = 1\ndef update():\n    global value\n    value = 2\n",
    )?;
    fs::write(
        fixture.0.join("effects.py"),
        "import module_that_does_not_exist\nopen('EXECUTED', 'w').write('unsafe')\nraise RuntimeError('must never execute')\n",
    )?;
    let output = fixture.check(&["globals.py", "effects.py"])?;
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(!fixture.0.join("EXECUTED").exists());
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn reports_actual_syntax_errors_with_the_original_filename() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.0.join("bad name 'quoted'.py"),
        "def broken(:\n    pass\n",
    )?;
    let output = fixture.check(&["bad name 'quoted'.py"])?;
    assert_eq!(output.status.code(), Some(1));
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostic.contains("SyntaxError:"), "{diagnostic}");
    assert!(diagnostic.contains("bad name 'quoted'.py"), "{diagnostic}");
    assert!(
        diagnostic.contains("no source was executed"),
        "{diagnostic}"
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn treats_flag_like_names_as_literal_paths_after_separator() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.0.join("-c"), "global value\nvalue = 1\n")?;
    fs::write(
        fixture.0.join("--check-syntax"),
        "raise RuntimeError('not executed')\n",
    )?;
    let output = fixture.check(&["-c", "--check-syntax"])?;
    assert!(output.status.success(), "{output:?}");
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn rejects_missing_non_regular_and_non_text_sources() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.0.join("binary.py"), [0xff, 0xfe])?;
    fs::write(fixture.0.join("nul.py"), b"pass\0")?;
    for file in ["missing.py", ".", "binary.py", "nul.py"] {
        let output = fixture.check(&[file])?;
        assert_eq!(output.status.code(), Some(1), "{file}: {output:?}");
        assert!(!output.stderr.is_empty(), "{file}: {output:?}");
    }
    assert_eq!(fixture.check(&[])?.status.code(), Some(1));
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn rejects_oversized_regular_files_before_reading_them() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::File::create(fixture.0.join("large.py"))?.set_len(134_217_729)?;
    let output = fixture.check(&["large.py"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("at most 128 MiB"));
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn wrapper_compiler_preserves_main_and_imported_module_namespaces() -> io::Result<()> {
    let fixture = Fixture::new()?;
    fs::write(
        fixture.0.join("wrapped.py"),
        "import sys\ncode = sys._kipferl_compile_module('global counter\\ncounter = 3\\ndef bump():\\n    global counter\\n    counter += 1\\n', 'original-module.py')\nassert 'counter' not in globals()\nexec(code)\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", "import sys\ncode = sys._kipferl_compile_module('global counter\\ncounter = 10\\n', 'original-entry.py')\nassert 'counter' not in globals()\nexec(code)\nimport wrapped\nwrapped.bump()\nassert wrapped.counter == 4\nassert counter == 10\nassert 'counter' not in sys.__dict__\nprint('module namespaces preserved')"])
        .current_dir(&fixture.0).output()?;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "module namespaces preserved\n"
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Fixture I/O errors propagate; behavior assertions intentionally fail this specific test"
)]
fn wrapper_code_tracebacks_keep_original_filenames() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", "import sys\ncode = sys._kipferl_compile_module('value = 1\\nraise ValueError(\\\"expected failure\\\")', 'original module.py')\nexec(code)"])
        .current_dir(&fixture.0).output()?;
    assert_eq!(output.status.code(), Some(1));
    let diagnostic = String::from_utf8_lossy(&output.stdout);
    assert!(diagnostic.contains("original module.py"), "{diagnostic}");
    assert!(diagnostic.contains("line 2"), "{diagnostic}");
    assert!(
        diagnostic.contains("ValueError: expected failure"),
        "{diagnostic}"
    );
    Ok(())
}
