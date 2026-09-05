//! User-facing approvals must match this executable and preserve their reviewed scope.
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use sha2::{Digest, Sha256};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "kipferl-verified-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn cli(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_kipferl"))
            .args(arguments)
            .current_dir(&self.0)
            .env_clear()
            .env("HOME", &self.0)
            .env("KIPFERL_CACHE_DIR", self.0.join("cache"))
            .output()
    }

    fn json(&self, subcommand: &str) -> io::Result<Value> {
        let output = self.cli(&["deps", subcommand, "--json"])?;
        success(&output);
        serde_json::from_slice(&output.stdout).map_err(io::Error::other)
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
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn host_runtime_identity() -> io::Result<(String, String)> {
    let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let runtime = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/pocketpy-kipferl-{target}")),
    )?;
    let mut hash = String::with_capacity(64);
    for byte in Sha256::digest(runtime) {
        write!(hash, "{byte:02x}").map_err(io::Error::other)?;
    }
    Ok((target, hash))
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "I/O setup errors propagate while public-command assertions intentionally fail this integration test"
)]
fn json_approvals_match_host_assets_and_preserve_complete_catalog_evidence() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let catalog = fixture.json("catalog")?;
    let verified = fixture.json("verified")?;
    let (target, hash) = host_runtime_identity()?;
    let records = catalog["records"]
        .as_array()
        .ok_or_else(|| io::Error::other("catalog records missing"))?;
    let expected: Vec<_> = records
        .iter()
        .filter(|record| {
            record["status"] == "tested"
                && record["runtime_sha256"] == hash
                && record["target"] == target
        })
        .collect();
    // The full catalog deliberately retains old releases and other platforms.
    assert!(records.len() > expected.len());
    assert_eq!(verified["schema_version"], catalog["schema_version"]);
    assert_eq!(verified["records"], serde_json::to_value(expected)?);
    assert_eq!(fs::read_dir(&fixture.0)?.count(), 0);
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "I/O setup errors propagate while user-visible scope assertions intentionally fail this integration test"
)]
fn human_output_shows_reviewed_scope_and_install_commands_without_a_project() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let verified = fixture.json("verified")?;
    let output = fixture.cli(&["deps", "verified"])?;
    success(&output);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("only the stated workflow, not every package API"));
    assert!(!text.contains("--allow-unverified"));
    let records = verified["records"]
        .as_array()
        .ok_or_else(|| io::Error::other("verified records missing"))?;
    if records.is_empty() {
        assert!(text.contains("No verified workflows for this runtime."));
    }
    for record in records {
        let name = record["name"].as_str().unwrap_or_default();
        let version = record["version"].as_str().unwrap_or_default();
        let scope = record["smoke"]["scope"].as_str().unwrap_or_default();
        assert!(!name.is_empty() && !version.is_empty() && !scope.is_empty());
        assert!(text.contains(&format!("\n{name}=={version}\n")));
        assert!(text.contains(scope));
        assert!(text.contains(&format!("Install: kipferl add {name}=={version}")));
    }
    assert_eq!(fs::read_dir(&fixture.0)?.count(), 0);
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "I/O setup errors propagate while argument-contract assertions intentionally fail this integration test"
)]
fn verified_help_is_specific_and_unknown_or_duplicate_flags_fail() -> io::Result<()> {
    let fixture = Fixture::new()?;
    for flag in ["--help", "-h"] {
        let output = fixture.cli(&["deps", "verified", flag])?;
        success(&output);
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("Usage: kipferl deps verified [--json]"));
        assert!(text.contains("No project or network access is required."));
    }
    for arguments in [
        vec!["deps", "verified", "--bogus"],
        vec!["deps", "verified", "--json", "--json"],
        vec!["deps", "verified", "--json", "unexpected"],
        vec!["deps", "verified", "unexpected"],
        vec!["deps", "verified", "--help", "--bogus"],
    ] {
        let output = fixture.cli(&arguments)?;
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("usage: kipferl deps verified [--json]")
        );
    }
    let output = fixture.cli(&["deps", "--help"])?;
    success(&output);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("check|list|catalog|audit|verified"));
    assert!(text.contains("Optional extra-only transitive dependencies are ignored"));
    Ok(())
}
