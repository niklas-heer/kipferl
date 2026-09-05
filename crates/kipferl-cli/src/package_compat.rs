//! Exact-artifact compatibility evidence. An unknown artifact never inherits a
//! tested verdict from another version, runtime binary, or operating system.
use std::io;
use std::path::Path;
use std::sync::OnceLock;

use serde_json::Value;

const CATALOG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/catalog.json.gz"));
const POPULARITY_CATALOG_GZ: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/popularity-catalog.json.gz"));
static COMBINED_CATALOG: OnceLock<io::Result<Value>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Tested,
    Incompatible,
    Unverified,
}

impl Status {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tested => "tested",
            Self::Incompatible => "incompatible",
            Self::Unverified => "unverified",
        }
    }
}

#[derive(Debug)]
pub struct Report {
    pub status: Status,
    pub diagnostics: Vec<String>,
}

pub fn catalog() -> io::Result<Value> {
    cached_catalog().cloned()
}

fn cached_catalog() -> io::Result<&'static Value> {
    COMBINED_CATALOG
        .get_or_init(|| {
            combine_catalogs(
                &crate::embedded_json::decode(CATALOG_GZ)?,
                &crate::embedded_json::decode(POPULARITY_CATALOG_GZ)?,
            )
        })
        .as_ref()
        .map_err(|error| io::Error::new(error.kind(), error.to_string()))
}

fn evidence_key(record: &Value) -> [String; 5] {
    [
        "name",
        "version",
        "wheel_sha256",
        "runtime_sha256",
        "target",
    ]
    .map(|field| record[field].to_string())
}

fn combine_catalogs(reviewed: &str, automated: &str) -> io::Result<Value> {
    let mut combined: Value = serde_json::from_str(reviewed).map_err(io::Error::other)?;
    let automated: Value = serde_json::from_str(automated).map_err(io::Error::other)?;
    validate_catalog(&combined)?;
    validate_catalog(&automated)?;
    let records = combined
        .get_mut("records")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| io::Error::other("reviewed catalog records missing"))?;
    let mut keys: std::collections::BTreeSet<_> = records.iter().map(evidence_key).collect();
    for record in automated
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::other("automated catalog records missing"))?
    {
        // Broad screening can add demonstrated compiler failures, never turn
        // an unexecuted package into an approved dependency.
        if record["status"] != "incompatible"
            || record
                .get("compile_failures")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
        {
            return Err(io::Error::other(
                "automated catalog may contain only exact compiler blockers",
            ));
        }
        if keys.insert(evidence_key(record)) {
            records.push(record.clone());
        }
    }
    Ok(combined)
}

pub fn inspect(
    name: &str,
    version: &str,
    sha256: &str,
    runtime_sha256: &str,
    root: &Path,
) -> io::Result<Report> {
    let mut native = Vec::new();
    find_native(root, root, &mut native)?;
    if !native.is_empty() {
        return Ok(Report {
            status: Status::Incompatible,
            diagnostics: native,
        });
    }
    lookup(name, version, sha256, runtime_sha256)
}

/// Look up exact evidence without reading installed files or executing code.
pub fn lookup(name: &str, version: &str, sha256: &str, runtime_sha256: &str) -> io::Result<Report> {
    assess(
        cached_catalog()?,
        name,
        version,
        sha256,
        runtime_sha256,
        &target(),
    )
}

fn target() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn assess(
    catalog: &Value,
    name: &str,
    version: &str,
    sha256: &str,
    runtime_sha256: &str,
    target: &str,
) -> io::Result<Report> {
    let records = catalog["records"]
        .as_array()
        .ok_or_else(|| io::Error::other("catalog records must be an array"))?;
    for record in records {
        if record["name"] != name
            || record["version"] != version
            || record["wheel_sha256"] != sha256
            || record["runtime_sha256"] != runtime_sha256
            || record["target"] != target
        {
            continue;
        }
        let status = match record["status"].as_str() {
            Some("tested") => Status::Tested,
            Some("incompatible") => Status::Incompatible,
            _ => Status::Unverified,
        };
        let evidence = record["evidence"]
            .as_str()
            .ok_or_else(|| io::Error::other("catalog evidence must be text"))?;
        return Ok(Report {
            status,
            diagnostics: vec![evidence.to_owned()],
        });
    }
    Ok(Report {
        status: Status::Unverified,
        diagnostics: vec![format!(
            "No compatibility evidence matches {name}=={version}, this wheel, runtime binary, and {target}. Source compilation cannot establish behavioral compatibility."
        )],
    })
}

fn find_native(root: &Path, directory: &Path, diagnostics: &mut Vec<String>) -> io::Result<()> {
    // Wheel paths are untrusted. An explicit work list avoids overflowing the
    // stack on a deeply nested archive before the installer reports a result.
    let mut pending = vec![directory.to_owned()];
    while let Some(current) = pending.pop() {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_symlink() {
                return Err(io::Error::other("package inspection refuses symlinks"));
            }
            if kind.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| ["so", "pyd", "dll", "dylib"].contains(&extension))
            {
                let relative = path.strip_prefix(root).map_err(io::Error::other)?;
                diagnostics.push(format!(
                    "{}: native libraries cannot be loaded by Kipferl's Python runtime",
                    relative.display()
                ));
            }
        }
    }
    diagnostics.sort();
    Ok(())
}

fn validate_catalog(value: &Value) -> io::Result<()> {
    if value["schema_version"] != 1 {
        return Err(io::Error::other("unsupported package catalog schema"));
    }
    let records = value["records"]
        .as_array()
        .ok_or_else(|| io::Error::other("catalog records must be an array"))?;
    let mut keys = std::collections::BTreeSet::new();
    for record in records {
        for field in [
            "name",
            "version",
            "wheel_filename",
            "wheel_sha256",
            "runtime_sha256",
            "target",
            "status",
            "evidence",
        ] {
            if record[field].as_str().is_none_or(str::is_empty) {
                return Err(io::Error::other(format!(
                    "catalog record is missing {field}"
                )));
            }
        }
        for field in ["wheel_sha256", "runtime_sha256"] {
            if !record[field].as_str().is_some_and(|hash| {
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            }) {
                return Err(io::Error::other(format!("invalid catalog {field}")));
            }
        }
        if !record["status"]
            .as_str()
            .is_some_and(|status| ["tested", "incompatible", "unverified"].contains(&status))
        {
            return Err(io::Error::other("invalid catalog status"));
        }
        if record["status"] == "tested"
            && (!record
                .get("smoke")
                .and_then(|smoke| smoke.get("file"))
                .and_then(Value::as_str)
                .is_some_and(|file| {
                    !file.is_empty() && !file.contains('/') && !file.contains('\\')
                })
                || record
                    .get("smoke")
                    .and_then(|smoke| smoke.get("scope"))
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                || !record
                    .get("smoke")
                    .and_then(|smoke| smoke.get("sha256"))
                    .and_then(Value::as_str)
                    .is_some_and(|hash| {
                        hash.len() == 64
                            && hash
                                .bytes()
                                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    }))
        {
            return Err(io::Error::other(
                "tested catalog evidence requires a hashed smoke hook and explicit scope",
            ));
        }
        let key = [
            "name",
            "version",
            "wheel_sha256",
            "runtime_sha256",
            "target",
        ]
        .map(|field| record[field].to_string());
        if !keys.insert(key) {
            return Err(io::Error::other("duplicate catalog evidence key"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Status, assess, catalog, combine_catalogs, evidence_key, validate_catalog};
    const CATALOG: &str = include_str!("../../../compatibility/packages/catalog.json");
    const POPULARITY_CATALOG: &str =
        include_str!("../../../compatibility/packages/popularity-catalog.json");

    #[test]
    fn bundled_catalog_is_valid() -> Result<(), Box<dyn std::error::Error>> {
        let value = catalog()?;
        if value["records"].as_array().ok_or("records")?.is_empty() {
            return Err("catalog contains no evidence".into());
        }
        Ok(())
    }

    #[test]
    fn evidence_is_bound_to_every_identity_field() -> Result<(), Box<dyn std::error::Error>> {
        let value = serde_json::json!({"records": [{
            "name": "example", "version": "1.0", "wheel_sha256": "wheel", "runtime_sha256": "runtime", "target": "macos-aarch64", "status": "tested", "evidence": "asserted behavior"
        }]});
        if assess(
            &value,
            "example",
            "1.0",
            "wheel",
            "runtime",
            "macos-aarch64",
        )?
        .status
            != Status::Tested
        {
            return Err("exact evidence did not match".into());
        }
        for (name, version, wheel, runtime, target) in [
            ("other", "1.0", "wheel", "runtime", "macos-aarch64"),
            ("example", "1.1", "wheel", "runtime", "macos-aarch64"),
            ("example", "1.0", "changed", "runtime", "macos-aarch64"),
            ("example", "1.0", "wheel", "changed", "macos-aarch64"),
            ("example", "1.0", "wheel", "runtime", "linux-x86_64"),
        ] {
            if assess(&value, name, version, wheel, runtime, target)?.status != Status::Unverified {
                return Err("mismatched identity inherited tested evidence".into());
            }
        }
        Ok(())
    }

    #[test]
    fn malformed_catalog_fails_closed() {
        assert!(
            validate_catalog(&serde_json::json!({"schema_version": 2, "records": []})).is_err()
        );
        assert!(
            validate_catalog(
                &serde_json::json!({"schema_version": 1, "records": [{"name": "missing fields"}]})
            )
            .is_err()
        );
    }

    #[test]
    fn combined_catalog_keeps_reviewed_evidence_and_adds_unique_blockers()
    -> Result<(), Box<dyn std::error::Error>> {
        let reviewed: serde_json::Value = serde_json::from_str(CATALOG)?;
        let automated: serde_json::Value = serde_json::from_str(POPULARITY_CATALOG)?;
        let combined = combine_catalogs(CATALOG, POPULARITY_CATALOG)?;
        if catalog()? != combined {
            return Err("compressed catalog differs from canonical evidence".into());
        }
        let records = combined
            .get("records")
            .and_then(serde_json::Value::as_array)
            .ok_or("combined records")?;
        let mut expected_keys = std::collections::BTreeSet::new();
        for original in reviewed
            .get("records")
            .and_then(serde_json::Value::as_array)
            .ok_or("reviewed records")?
        {
            expected_keys.insert(evidence_key(original));
            if !records.contains(original) {
                return Err("reviewed evidence was overwritten by automated screening".into());
            }
        }
        for original in automated
            .get("records")
            .and_then(serde_json::Value::as_array)
            .ok_or("automated records")?
        {
            expected_keys.insert(evidence_key(original));
        }
        if records.len() != expected_keys.len() {
            return Err("combined catalog omitted or duplicated an artifact".into());
        }
        Ok(())
    }

    #[test]
    fn automated_screening_cannot_add_behavioral_approval() {
        // The reviewed catalog contains tested entries. It is deliberately
        // invalid as an automated-screening input even though its schema passes.
        assert!(combine_catalogs(CATALOG, CATALOG).is_err());
    }
}
