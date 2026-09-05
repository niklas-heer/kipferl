//! The release version must be inspectable without loading a VM or a script.
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(condition: bool, message: impl std::fmt::Display) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message.to_string()))
    }
}

#[test]
fn version_is_exact_and_never_executes_a_flag_named_script() -> io::Result<()> {
    let fixture = Fixture(
        std::env::temp_dir().join(format!("kipferl-runtime-version-{}", std::process::id())),
    );
    fs::create_dir(&fixture.0)?;
    fs::write(
        fixture.0.join("--version"),
        "open('EXECUTED', 'w').write('unexpected script execution')\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .arg("--version")
        .current_dir(&fixture.0)
        .output()?;
    check(
        output.status.success(),
        format!("version failed: {output:?}"),
    )?;
    check(
        output.stdout == format!("Kipferl runtime {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
            && output.stderr.is_empty(),
        format!("unexpected version output: {output:?}"),
    )?;
    let extra = Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["--version", "extra"])
        .current_dir(&fixture.0)
        .output()?;
    check(
        extra.status.code() == Some(1)
            && String::from_utf8_lossy(&extra.stderr).contains("does not accept arguments"),
        format!("version arguments were not rejected: {extra:?}"),
    )?;
    check(
        !fixture.0.join("EXECUTED").exists(),
        "the file named --version was executed",
    )
}
