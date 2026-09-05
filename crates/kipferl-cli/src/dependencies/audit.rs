//! Offline presentation of the pinned package popularity audit.
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;

use super::{invalid, registry::valid_hash};

const REPORT_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/popularity-audit.json.gz"));
static REPORT: OnceLock<io::Result<(Audit, Value)>> = OnceLock::new();
const DEFAULT_LIMIT: usize = 20;
const MAX_PACKAGES: usize = 1000;

#[derive(Deserialize)]
struct Audit {
    schema_version: u32,
    complete: bool,
    requested_count: usize,
    completed_count: usize,
    snapshot_sha256: String,
    runtime_sha256: String,
    target: String,
    ranking_source: Value,
    counts: BTreeMap<String, usize>,
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    rank: usize,
    name: String,
    version: Option<String>,
    artifact_verified: bool,
    evidence_scope: Scope,
    category: Category,
    status: Status,
    evidence: String,
    source_files_total: Option<usize>,
    sources_checked: usize,
    remaining: Option<usize>,
    first_blocker: Option<Blocker>,
    wheel_filename: Option<String>,
    wheel_sha256: Option<String>,
    source_url: Option<String>,
}

#[derive(Deserialize)]
struct Blocker {
    file: String,
    diagnostic: String,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Scope {
    Metadata,
    VerifiedArtifact,
    None,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Incompatible,
    Unverified,
}

#[derive(Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum Category {
    NativeOnly,
    SourceOnly,
    PythonRequirement,
    UnsupportedRequirement,
    Syntax,
    Limits,
    Network,
    Unverified,
}

impl Category {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NativeOnly => "native_only",
            Self::SourceOnly => "source_only",
            Self::PythonRequirement => "python_requirement",
            Self::UnsupportedRequirement => "unsupported_requirement",
            Self::Syntax => "syntax",
            Self::Limits => "limits",
            Self::Network => "network",
            Self::Unverified => "unverified",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::NativeOnly => "unsupported wheel/native blocker",
            Self::SourceOnly => "source distribution only",
            Self::PythonRequirement => "Python version requirement",
            Self::UnsupportedRequirement => "unsupported dependency requirement",
            Self::Syntax => "verified syntax blocker",
            Self::Limits => "audit limit reached",
            Self::Network => "metadata/download failure",
            Self::Unverified => "no demonstrated blocker",
        }
    }

    const fn status(self) -> Status {
        match self {
            Self::NativeOnly | Self::SourceOnly | Self::PythonRequirement | Self::Syntax => {
                Status::Incompatible
            }
            Self::UnsupportedRequirement | Self::Limits | Self::Network | Self::Unverified => {
                Status::Unverified
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    limit: usize,
    json: bool,
}

fn options(arguments: &[String]) -> io::Result<Options> {
    let mut limit = None;
    let mut json = false;
    let mut arguments = arguments.iter();
    if arguments.next().map(String::as_str) != Some("audit") {
        return Err(invalid(
            "usage: kipferl deps audit [--limit 1..1000 | --json]",
        ));
    }
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--json" if !json => json = true,
            "--limit" if limit.is_none() => {
                let number = arguments
                    .next()
                    .ok_or_else(|| invalid("--limit needs a number from 1 to 1000"))?;
                let parsed = number
                    .parse::<usize>()
                    .map_err(|_| invalid("--limit must be a number from 1 to 1000"))?;
                if !(1..=MAX_PACKAGES).contains(&parsed) {
                    return Err(invalid("--limit must be between 1 and 1000"));
                }
                limit = Some(parsed);
            }
            _ => {
                return Err(invalid(format!(
                    "unexpected audit argument {argument:?}; use --limit 1..1000 or --json"
                )));
            }
        }
    }
    if json && limit.is_some() {
        return Err(invalid(
            "--json exports the complete audit; use --limit for the human-readable report",
        ));
    }
    Ok(Options {
        limit: limit.unwrap_or(DEFAULT_LIMIT),
        json,
    })
}

pub(super) fn show(arguments: &[String], stdout: &mut dyn Write) -> io::Result<()> {
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        return writeln!(
            stdout,
            "Usage: kipferl deps audit [--limit 1..1000 | --json]\n\nShow the bundled popularity audit without a project or network access.\nThe default includes summary counts and the 20 highest-ranked packages.\n--limit N  Show up to N packages in the human-readable report.\n--json     Export the complete canonical audit, including exact artifact evidence.\n\nCompilation passes remain unverified; this audit does not run package behavior tests."
        );
    }
    let options = options(arguments)?;
    let (audit, value) = REPORT
        .get_or_init(|| parse(&crate::embedded_json::decode(REPORT_GZ)?))
        .as_ref()
        .map_err(|error| io::Error::new(error.kind(), error.to_string()))?;
    if options.json {
        serde_json::to_writer_pretty(&mut *stdout, &value).map_err(invalid)?;
        writeln!(stdout)
    } else {
        render(
            audit,
            options.limit,
            &super::runtime_hash()?,
            crate::run_command::embedded_runtime_target(),
            stdout,
        )
    }
}

fn parse(text: &str) -> io::Result<(Audit, Value)> {
    let value: Value = serde_json::from_str(text).map_err(invalid)?;
    let audit: Audit = serde_json::from_value(value.clone()).map_err(invalid)?;
    if audit.schema_version != 1
        || !(1..=MAX_PACKAGES).contains(&audit.requested_count)
        || audit.completed_count != audit.records.len()
        || audit.completed_count > audit.requested_count
        || (audit.complete && audit.completed_count != audit.requested_count)
        || audit.records.len() > MAX_PACKAGES
        || !valid_hash(&audit.snapshot_sha256)
        || !valid_hash(&audit.runtime_sha256)
        || audit.target.is_empty()
    {
        return Err(invalid("unsupported or invalid popularity audit schema"));
    }
    let mut ranks = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut counts = BTreeMap::new();
    for record in &audit.records {
        if !(1..=MAX_PACKAGES).contains(&record.rank)
            || !ranks.insert(record.rank)
            || record.name.is_empty()
            || !record
                .name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
            || !names.insert(record.name.to_ascii_lowercase())
        {
            return Err(invalid(
                "audit ranks and distribution names must be valid and unique",
            ));
        }
        validate_record(record)?;
        let count = counts
            .entry(record.category.as_str().to_owned())
            .or_insert(0_usize);
        *count = count.saturating_add(1);
    }
    let reported_counts: BTreeMap<_, _> = audit
        .counts
        .iter()
        .filter(|(_, count)| **count != 0)
        .map(|(category, count)| (category.clone(), *count))
        .collect();
    if reported_counts != counts {
        return Err(invalid("audit category counts do not match its records"));
    }
    Ok((audit, value))
}

fn validate_record(record: &Record) -> io::Result<()> {
    if record.status != record.category.status()
        || record.evidence.trim().is_empty()
        || record.artifact_verified != (record.evidence_scope == Scope::VerifiedArtifact)
    {
        return Err(invalid(format!(
            "{}: inconsistent audit status or evidence scope",
            record.name
        )));
    }
    if record.artifact_verified
        && (!record.wheel_sha256.as_deref().is_some_and(valid_hash)
            || record.wheel_filename.as_deref().is_none_or(str::is_empty)
            || record.source_url.as_deref().is_none_or(str::is_empty))
    {
        return Err(invalid(format!(
            "{}: verified artifact is missing its exact wheel identity",
            record.name
        )));
    }
    match (record.source_files_total, record.remaining) {
        (Some(total), Some(remaining))
            if total.checked_sub(record.sources_checked) == Some(remaining) => {}
        (None, None) if record.sources_checked == 0 => {}
        _ => {
            return Err(invalid(format!(
                "{}: source coverage counts disagree",
                record.name
            )));
        }
    }
    if record.sources_checked > 0 && !record.artifact_verified {
        return Err(invalid("source compilation requires verified wheel bytes"));
    }
    if record.category == Category::Syntax
        && (!record.artifact_verified
            || record.sources_checked == 0
            || record
                .first_blocker
                .as_ref()
                .is_none_or(|blocker| blocker.file.is_empty() || blocker.diagnostic.is_empty()))
    {
        return Err(invalid(
            "syntax blockers require verified artifacts and a concrete compiler diagnostic",
        ));
    }
    Ok(())
}

fn render(
    audit: &Audit,
    limit: usize,
    runtime: &str,
    target: &str,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    if audit.records.is_empty() {
        return writeln!(
            stdout,
            "Popularity audit is not available in this build yet."
        );
    }
    let verified = audit
        .records
        .iter()
        .filter(|record| record.artifact_verified)
        .count();
    let metadata = audit
        .records
        .iter()
        .filter(|record| record.evidence_scope == Scope::Metadata)
        .count();
    writeln!(
        stdout,
        "Popularity audit: {} of {} ranked packages audited ({}).",
        audit.completed_count,
        audit.requested_count,
        if audit.complete {
            "complete"
        } else {
            "partial snapshot"
        }
    )?;
    writeln!(
        stdout,
        "One selected release per package; older releases are not evaluated."
    )?;
    if let Some(url) = audit.ranking_source.get("url").and_then(Value::as_str) {
        writeln!(stdout, "Ranking source: {url}")?;
    }
    writeln!(
        stdout,
        "Ranking snapshot SHA-256: {}",
        audit.snapshot_sha256
    )?;
    writeln!(
        stdout,
        "Evidence runtime: {} / {}",
        audit.target, audit.runtime_sha256
    )?;
    if audit.runtime_sha256 != runtime || audit.target != target {
        writeln!(
            stdout,
            "The recorded runtime differs from this CLI; results apply to the recorded runtime."
        )?;
    }
    let unavailable = audit
        .records
        .iter()
        .filter(|record| record.evidence_scope == Scope::None)
        .count();
    writeln!(
        stdout,
        "Evidence: {verified} verified wheel artifacts; {metadata} metadata-only observations; {unavailable} without artifact evidence."
    )?;
    writeln!(
        stdout,
        "This audit runs no package behavior tests. Compilation passes remain unverified.\n"
    )?;
    let mut counts = BTreeMap::new();
    for record in &audit.records {
        let count = counts.entry(record.category).or_insert(0_usize);
        *count = count.saturating_add(1);
    }
    for (category, count) in counts {
        writeln!(stdout, "{count:>4}  {}", category.label())?;
    }
    render_rows(audit, limit, stdout)
}

fn render_rows(audit: &Audit, limit: usize, stdout: &mut dyn Write) -> io::Result<()> {
    let mut records: Vec<_> = audit.records.iter().collect();
    records.sort_by_key(|record| record.rank);
    let shown = records.len().min(limit);
    writeln!(
        stdout,
        "\nShowing {shown} of {} packages (popularity order):",
        records.len()
    )?;
    for record in records.into_iter().take(limit) {
        let scope = match record.evidence_scope {
            Scope::VerifiedArtifact => "verified wheel",
            Scope::Metadata => "metadata only",
            Scope::None => "no artifact evidence",
        };
        writeln!(
            stdout,
            "{:>4}  {}=={}  {} [{scope}]",
            record.rank,
            record.name,
            record.version.as_deref().unwrap_or("unknown"),
            record.category.label()
        )?;
        let first_line = record
            .evidence
            .lines()
            .next()
            .unwrap_or("No further diagnostic");
        let brief: String = first_line
            .chars()
            .take(160)
            .map(|character| {
                if character.is_control() {
                    ' '
                } else {
                    character
                }
            })
            .collect();
        writeln!(stdout, "      {brief}")?;
    }
    writeln!(
        stdout,
        "\nUse --limit 1000 for all rows or --json for complete source coverage and diagnostics."
    )
}

#[cfg(test)]
mod tests {
    use std::io;

    use serde_json::{Value, json};

    use super::{invalid, options, parse, render};
    const CANONICAL_REPORT: &str =
        include_str!("../../../../compatibility/packages/popularity-audit.json");

    fn check(condition: bool, message: &str) -> io::Result<()> {
        if condition {
            Ok(())
        } else {
            Err(invalid(message))
        }
    }

    fn fixture() -> Value {
        json!({
            "schema_version": 1, "complete": true, "requested_count": 2, "completed_count": 2,
            "snapshot_sha256": "a".repeat(64), "runtime_sha256": "b".repeat(64), "target": "macos-aarch64",
            "ranking_source": {"url": "https://example.com/ranking.json"}, "counts": {"syntax": 1, "unverified": 1},
            "records": [
                {"rank": 2, "name": "blocked", "version": "1.0", "artifact_verified": true,
                 "evidence_scope": "verified_artifact", "category": "syntax", "status": "incompatible",
                 "evidence": "module.py fails compilation", "source_files_total": 10, "sources_checked": 1, "remaining": 9,
                 "first_blocker": {"file": "module.py", "diagnostic": "SyntaxError"},
                 "wheel_filename": "blocked-1.0-py3-none-any.whl", "wheel_sha256": "c".repeat(64),
                 "source_url": "https://files.pythonhosted.org/packages/blocked.whl"},
                {"rank": 1, "name": "unknown", "version": "2.0", "artifact_verified": true,
                 "evidence_scope": "verified_artifact", "category": "unverified", "status": "unverified",
                 "evidence": "All three sources compile; behavior was not tested", "source_files_total": 3, "sources_checked": 3, "remaining": 0,
                 "first_blocker": null, "wheel_filename": "unknown-2.0-py3-none-any.whl", "wheel_sha256": "d".repeat(64),
                 "source_url": "https://files.pythonhosted.org/packages/unknown.whl"}
            ]
        })
    }

    #[test]
    fn limits_and_json_have_explicit_nontruncating_semantics() -> io::Result<()> {
        let options = options(&["audit".to_owned()])?;
        check(
            options.limit == 20 && !options.json,
            "audit default must show 20 rows",
        )?;
        for args in [
            vec!["audit", "--limit", "0"],
            vec!["audit", "--limit", "1001"],
            vec!["audit", "--limit"],
            vec!["audit", "--json", "--limit", "10"],
            vec!["audit", "--json", "--json"],
        ] {
            check(
                super::options(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err(),
                "ambiguous audit options accepted",
            )?;
        }
        let options = super::options(&["audit".to_owned(), "--json".to_owned()])?;
        check(options.json, "JSON export option missing")
    }

    #[test]
    fn summary_sorts_ranks_limits_rows_and_preserves_unverified_status() -> io::Result<()> {
        let (audit, _) = parse(&fixture().to_string())?;
        let mut bytes = Vec::new();
        render(&audit, 1, &"e".repeat(64), "linux-x86_64", &mut bytes)?;
        let output = String::from_utf8(bytes).map_err(invalid)?;
        check(
            output.contains("Showing 1 of 2")
                && output.contains("unknown==2.0")
                && !output.contains("blocked==1.0"),
            "rank sort or row limit failed",
        )?;
        check(
            output.contains("Compilation passes remain unverified"),
            "compile-only report claimed behavior compatibility",
        )?;
        check(
            output.contains("recorded runtime differs"),
            "different runtime evidence was presented as current",
        )
    }

    #[test]
    fn audit_cannot_promote_compile_pass_to_tested() -> io::Result<()> {
        let mut value = fixture();
        let record = value
            .get_mut("records")
            .and_then(Value::as_array_mut)
            .and_then(|records| records.get_mut(1))
            .ok_or_else(|| invalid("fixture record"))?;
        record
            .as_object_mut()
            .ok_or_else(|| invalid("record object"))?
            .insert("status".to_owned(), json!("tested"));
        check(
            parse(&value.to_string()).is_err(),
            "compile-only audit accepted tested status",
        )
    }

    #[test]
    fn syntax_blockers_require_verified_artifacts_and_consistent_coverage() -> io::Result<()> {
        for (field, bad) in [
            ("artifact_verified", json!(false)),
            ("remaining", json!(10)),
            ("wheel_sha256", json!(null)),
            ("first_blocker", json!(null)),
        ] {
            let mut value = fixture();
            let record = value
                .get_mut("records")
                .and_then(Value::as_array_mut)
                .and_then(|records| records.first_mut())
                .and_then(Value::as_object_mut)
                .ok_or_else(|| invalid("fixture object"))?;
            record.insert(field.to_owned(), bad);
            check(
                parse(&value.to_string()).is_err(),
                "unsupported exact-artifact evidence accepted",
            )?;
        }
        Ok(())
    }

    #[test]
    fn summary_counts_ranks_and_completion_cannot_disagree() -> io::Result<()> {
        for (field, bad) in [
            ("counts", json!({"syntax": 2})),
            ("completed_count", json!(1000)),
            ("requested_count", json!(3)),
            ("schema_version", json!(2)),
        ] {
            let mut value = fixture();
            value
                .as_object_mut()
                .ok_or_else(|| invalid("fixture"))?
                .insert(field.to_owned(), bad);
            check(
                parse(&value.to_string()).is_err(),
                "inconsistent audit summary accepted",
            )?;
        }
        let mut value = fixture();
        let record = value
            .get_mut("records")
            .and_then(Value::as_array_mut)
            .and_then(|records| records.first_mut())
            .and_then(Value::as_object_mut)
            .ok_or_else(|| invalid("fixture record"))?;
        record.insert("rank".to_owned(), json!(1));
        check(
            parse(&value.to_string()).is_err(),
            "duplicate popularity rank accepted",
        )
    }

    #[test]
    fn metadata_observations_remain_distinct_from_verified_wheel_findings() -> io::Result<()> {
        let mut value = fixture();
        let records = value
            .get_mut("records")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| invalid("records"))?;
        let record = records.first_mut().ok_or_else(|| invalid("record"))?;
        *record = json!({"rank": 2, "name": "native", "version": "1.0", "artifact_verified": false,
            "evidence_scope": "metadata", "category": "native_only", "status": "incompatible", "evidence": "Only native wheels listed by PyPI",
            "source_files_total": null, "sources_checked": 0, "remaining": null, "first_blocker": null,
            "wheel_filename": null, "wheel_sha256": null, "source_url": null});
        value
            .as_object_mut()
            .ok_or_else(|| invalid("fixture"))?
            .insert(
                "counts".to_owned(),
                json!({"native_only": 1, "unverified": 1}),
            );
        let (audit, _) = parse(&value.to_string())?;
        let mut bytes = Vec::new();
        render(&audit, 20, &"b".repeat(64), "macos-aarch64", &mut bytes)?;
        let output = String::from_utf8(bytes).map_err(invalid)?;
        check(
            output.contains("1 verified wheel artifacts; 1 metadata-only observations")
                && output.contains("unsupported wheel/native blocker [metadata only]"),
            "metadata-only findings misrepresented as verified bytes",
        )
    }
    #[test]
    fn public_command_exports_canonical_report_without_a_project() -> io::Result<()> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = crate::run(
            &["deps".to_owned(), "audit".to_owned(), "--json".to_owned()],
            std::path::Path::new("unused-project-directory"),
            &mut stdout,
            &mut stderr,
        )?;
        let expected: Value = serde_json::from_str(CANONICAL_REPORT).map_err(invalid)?;
        let actual: Value = serde_json::from_slice(&stdout).map_err(invalid)?;
        check(
            code == 0 && stderr.is_empty() && actual == expected,
            "public audit JSON command requires a project or modifies canonical evidence",
        )
    }

    #[test]
    fn public_command_explains_audit_options_and_rejects_bad_limits() -> io::Result<()> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let arguments = ["deps".to_owned(), "audit".to_owned(), "--help".to_owned()];
        let code = crate::run(
            &arguments,
            std::path::Path::new("unused-project-directory"),
            &mut stdout,
            &mut stderr,
        )?;
        let help = String::from_utf8(stdout).map_err(invalid)?;
        check(
            code == 0
                && help.contains("--limit 1..1000")
                && help.contains("without a project or network"),
            "audit help does not explain available options",
        )?;
        let code = crate::run(
            &[
                "deps".to_owned(),
                "audit".to_owned(),
                "--limit".to_owned(),
                "0".to_owned(),
            ],
            std::path::Path::new("unused-project-directory"),
            &mut Vec::new(),
            &mut stderr,
        )?;
        check(
            code == 1 && !stderr.is_empty(),
            "bad audit limits did not return an actionable CLI error",
        )
    }
    #[test]
    fn bundled_audit_contains_the_completed_thousand_package_snapshot() -> io::Result<()> {
        let (audit, _) = parse(CANONICAL_REPORT)?;
        check(
            audit.complete && audit.requested_count == 1000 && audit.completed_count == 1000,
            "the bundled popularity audit must contain all 1000 ranked results before release",
        )
    }
}
