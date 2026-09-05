#!/usr/bin/env python3
"""Compare complete package screens while rejecting accidental release drift."""
import argparse
from collections import Counter
import hashlib
import json
import re
from pathlib import Path

from package_popularity_audit import validate_report


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def identity(report: dict, digest: str) -> dict:
    return {"report_sha256": digest, **{key: report[key] for key in (
        "runtime_sha256", "target", "audit_policy_sha256", "audit_policy",
        "snapshot_sha256", "completed_count", "counts",
    )}}


def finding(record: dict) -> dict:
    return {key: record.get(key) for key in (
        "status", "category", "artifact_verified", "source_files_total",
        "sources_checked", "remaining", "first_blocker", "evidence", "compilation_completed",
    )}


def blocker_identity(finding: dict) -> tuple | None:
    blocker = finding.get("first_blocker")
    if not blocker:
        return None
    diagnostic = blocker["diagnostic"]
    lines = re.findall(r"line (\d+)", diagnostic)
    error = next((line for line in diagnostic.splitlines() if line.startswith("SyntaxError:")), None)
    return blocker["file"], lines[-1] if lines else None, error


def compare(before: dict, after: dict, before_hash: str, after_hash: str) -> dict:
    for report in (before, after):
        validate_report(report)
        if not report.get("complete") or report.get("completed_count") != len(report["records"]):
            raise ValueError("comparison requires complete audits")
    for field in ("snapshot_sha256", "target"):
        if before[field] != after[field]:
            raise ValueError(f"comparison requires the same {field}")
    old = {record["name"]: record for record in before["records"]}
    new = {record["name"]: record for record in after["records"]}
    if len(old) != len(before["records"]) or len(new) != len(after["records"]) or old.keys() != new.keys():
        raise ValueError("comparison requires the same unique projects")
    records = []
    transitions = Counter()
    same_metadata = 0
    for name, previous in sorted(old.items(), key=lambda item: item[1]["rank"]):
        current = new[name]
        for field in ("rank", "downloads"):
            if previous[field] != current[field]:
                raise ValueError(f"popularity identity changed for {name}: {field}")
        pinned = previous.get("metadata_sha256") is not None
        if pinned:
            for field in ("metadata_sha256", "version", "selected_artifact_filename", "artifact_declared_sha256", "wheel_sha256", "source_url"):
                if previous.get(field) != current.get(field):
                    raise ValueError(f"pinned release or artifact changed for {name}: {field}")
            same_metadata += 1
        transitions[f"{previous['category']} -> {current['category']}"] += 1
        records.append({
            "rank": previous["rank"], "name": name, "version": current.get("version"),
            "same_pinned_metadata": pinned,
            "metadata_sha256": current.get("metadata_sha256"),
            "wheel_sha256": current.get("wheel_sha256"),
            "before": finding(previous), "after": finding(current),
        })
    dynamic_global_stops = sum(
        before["audit_policy"].get("version") == 1
        and "cannot use global keyword here" in (row["before"].get("first_blocker") or {}).get("diagnostic", "")
        for row in records
    )
    return {
        "schema_version": 1,
        "scope": "Same popularity snapshot and target. Every previously pinned release and selected artifact must remain identical. Missing baseline metadata is disclosed separately. Compilation completion is not behavioral compatibility.",
        "baseline_dynamic_global_stops": dynamic_global_stops,
        "baseline_caveat": f"The baseline contains {dynamic_global_stops} global-statement stops under policy v1's dynamic builtin compile(). Those require checker correction rather than new global language support." if dynamic_global_stops else None,
        "changed_first_blocker_count": sum(row["before"]["category"] == "syntax" and blocker_identity(row["before"]) != blocker_identity(row["after"]) for row in records),
        "still_blocked_with_changed_first_blocker_count": sum(row["before"]["category"] == "syntax" and row["after"]["category"] == "syntax" and blocker_identity(row["before"]) != blocker_identity(row["after"]) for row in records),
        "before": identity(before, before_hash), "after": identity(after, after_hash),
        "same_pinned_metadata_count": same_metadata,
        "missing_baseline_metadata": [record["name"] for record in records if not record["same_pinned_metadata"]],
        "transitions": dict(sorted(transitions.items())), "records": records,
    }


def markdown(report: dict, comparison_filename: str = "language-patch-comparison.json") -> str:
    rows = report["records"]
    categories = sorted({row[side]["category"] for row in rows for side in ("before", "after")})
    title = report.get("title", "Package audit after the first language patches")
    description = report.get("change_description", "The runtime adds trailing commas in parenthesized imports/function signatures and adjacent plain string/bytes literals. The audit now compiles in normal module mode without executing package source.")
    lines = [f"# {title}", "", description, "",
        f"Compared {len(rows)} ranked projects on `{report['after']['target']}`. {report['same_pinned_metadata_count']} reused identical pinned metadata, releases, and selected artifacts.", "",
        "| Result | Before: top 100 | After: top 100 | Before: top 1,000 | After: top 1,000 |",
        "| --- | ---: | ---: | ---: | ---: |"]
    for category in categories:
        counts = [sum(row[side]["category"] == category and row["rank"] <= limit for row in rows) for limit in (100, 1000) for side in ("before", "after")]
        lines.append(f"| {category} | " + " | ".join(map(str, counts)) + " |")
    completed = [row for row in rows if row["after"].get("compilation_completed") is True]
    source_bearing = [row for row in completed if (row["after"]["source_files_total"] or 0) > 0]
    newly_completed = [row for row in completed if row["before"]["category"] == "syntax"]
    lines += ["", f"{len(newly_completed)} previously syntax-blocked releases now complete source compilation. The new report contains {len(completed)} compilation-complete distributions, of which {len(source_bearing)} contain Python source. These remain **unverified** until imports, dependencies, and behavior are tested.", ""]
    if report.get("baseline_dynamic_global_stops"):
        lines += ["The original global-statement diagnostics included a checker limitation: dynamic `compile()` rejected constructs accepted during normal module compilation. Improvements from correcting that checker must not all be attributed to parser patches.", ""]
    lines += [f"The first blocker changed in {report['changed_first_blocker_count']} packages, including {report['still_blocked_with_changed_first_blocker_count']} that remain syntax-blocked at another source location or diagnostic. This comparison uses the source file, final source line number, and SyntaxError message; it ignores checker traceback wrapper differences.", ""]
    if newly_completed:
        lines += ["Newly compilation-complete releases: " + ", ".join(f"{row['name']}=={row['version']}" for row in newly_completed) + ".", ""]
    if report["missing_baseline_metadata"]:
        lines += ["Baseline metadata was unavailable for: " + ", ".join(report["missing_baseline_metadata"]) + ". These rows cannot establish an identical-release comparison.", ""]
    lines += [f"Before runtime: `{report['before']['runtime_sha256']}`. After runtime: `{report['after']['runtime_sha256']}`.", "",
        f"See [the comparison JSON]({comparison_filename}) for every transition and exact report/policy hashes, and [the current audit](popularity-audit.json) for complete current evidence.", ""]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--output", type=Path, default=Path("compatibility/packages/language-patch-comparison.json"))
    parser.add_argument("--title", help="Human-readable title for this comparison")
    parser.add_argument("--description", help="Describe the specific runtime changes being compared")
    arguments = parser.parse_args()
    before = arguments.before.read_bytes()
    after = arguments.after.read_bytes()
    result = compare(json.loads(before), json.loads(after), sha256(before), sha256(after))
    if arguments.title:
        result["title"] = arguments.title
    if arguments.description:
        result["change_description"] = arguments.description
    arguments.output.write_text(json.dumps(result, indent=2) + "\n")
    arguments.output.with_suffix(".md").write_text(markdown(result, arguments.output.name))
    print(f"Compared {len(result['records'])} projects; {result['same_pinned_metadata_count']} identical metadata pins.")


if __name__ == "__main__":
    main()
