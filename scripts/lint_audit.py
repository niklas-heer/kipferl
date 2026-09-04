#!/usr/bin/env python3
"""Run every review lint and retain navigable, deduplicated diagnostics.

Reviewed lint exceptions are listed alongside diagnostics, never hidden.
Use --deny-findings to enforce zero outstanding findings.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
LINTS = (
    "pedantic", "nursery", "unwrap_used", "expect_used", "indexing_slicing",
    "arithmetic_side_effects", "unreachable", "unimplemented",
    "unchecked_time_subtraction", "todo", "string_slice", "panic_in_result_fn",
    "panic", "exit", "as_conversions",
)


def diagnostics(lines, profile):
    """Extract source diagnostics; Cargo emits duplicates for lib and lib-test."""
    records = {}
    for line in lines:
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if event.get("reason") != "compiler-message":
            continue
        message = event["message"]
        if message["level"] not in ("warning", "error"):
            continue
        span = next((s for s in message["spans"] if s["is_primary"]), None)
        code = (message.get("code") or {}).get("code", "compiler")
        record = {
            "lint": code,
            "level": message["level"],
            "file": span["file_name"] if span else "<compiler>",
            "line": span["line_start"] if span else 0,
            "column": span["column_start"] if span else 0,
            "message": message["message"],
            "profiles": [profile],
            "rendered": message.get("rendered", ""),
        }
        key = tuple(record[k] for k in ("lint", "file", "line", "column", "message"))
        records[key] = record
    return records


def _skip_comment(source, index):
    if source.startswith("//", index):
        newline = source.find("\n", index)
        return len(source) if newline < 0 else newline
    if not source.startswith("/*", index):
        return None
    cursor = index + 2
    depth = 1
    while cursor < len(source) and depth:
        if source.startswith("/*", cursor):
            depth += 1
            cursor += 2
        elif source.startswith("*/", cursor):
            depth -= 1
            cursor += 2
        else:
            cursor += 1
    return cursor


def _literal_end(source, index):
    """Recognize Rust strings/chars without treating lifetimes as char literals."""
    raw_start = index + 1 if source.startswith(("br", "cr"), index) else index
    if source[raw_start:raw_start + 1] == "r":
        cursor = raw_start + 1
        while cursor < len(source) and source[cursor] == "#":
            cursor += 1
        if cursor < len(source) and source[cursor] == '"':
            terminator = '"' + "#" * (cursor - raw_start - 1)
            end = source.find(terminator, cursor + 1)
            return len(source) if end < 0 else end + len(terminator)
    if source[index] == '"':
        cursor = index + 1
        while cursor < len(source):
            if source[cursor] == "\\":
                cursor += 2
            elif source[cursor] == '"':
                return cursor + 1
            else:
                cursor += 1
        return len(source)
    if source[index] == "'":
        cursor = index + 1
        if cursor >= len(source):
            return None
        if source[cursor] == "\\":
            cursor += 1
            if source.startswith("u{", cursor):
                end = source.find("}", cursor + 2)
                cursor = len(source) if end < 0 else end + 1
            elif source.startswith("x", cursor):
                cursor += 3
            else:
                cursor += 1
        else:
            cursor += 1
        if cursor < len(source) and source[cursor] == "'":
            return cursor + 1
    return None


def _tokens(source, start=0, end=None):
    """Yield lexical tokens and source positions, omitting whitespace/comments."""
    end = len(source) if end is None else end
    cursor = start
    while cursor < end:
        if source[cursor].isspace():
            cursor += 1
            continue
        comment_end = _skip_comment(source, cursor)
        if comment_end is not None:
            cursor = comment_end
            continue
        literal_end = _literal_end(source, cursor)
        if literal_end is not None:
            yield ("literal", source[cursor:literal_end], cursor)
            cursor = literal_end
            continue
        if source[cursor].isalpha() or source[cursor] == "_":
            token_end = cursor + 1
            while token_end < end and (source[token_end].isalnum() or source[token_end] == "_"):
                token_end += 1
            yield ("name", source[cursor:token_end], cursor)
            cursor = token_end
            continue
        yield ("punctuation", source[cursor], cursor)
        cursor += 1


def _attributes(source):
    """Find genuine attributes, including inline and nested conditional forms."""
    tokens = list(_tokens(source))
    cursor = 0
    while cursor < len(tokens):
        if tokens[cursor][1] != "#":
            cursor += 1
            continue
        opening = cursor + 1
        if opening < len(tokens) and tokens[opening][1] == "!":
            opening += 1
        if opening >= len(tokens) or tokens[opening][1] != "[":
            cursor += 1
            continue
        depth = 1
        closing = opening + 1
        while closing < len(tokens) and depth:
            if tokens[closing][1] == "[":
                depth += 1
            elif tokens[closing][1] == "]":
                depth -= 1
            closing += 1
        if depth:
            return  # rustc diagnoses malformed attributes during the lint pass.
        yield tokens[opening + 1:closing - 1], source[tokens[cursor][2]:tokens[closing - 1][2] + 1]
        cursor = closing


def _decode_reason(literal):
    """Decode Rust reason strings, whose escapes are not identical to JSON."""
    if literal.startswith("r"):
        quote = literal.find('"')
        hashes = quote - 1
        return literal[quote + 1:len(literal) - hashes - 1]
    if not literal.startswith('"'):
        return None
    content = literal[1:-1]
    output = []
    cursor = 0
    escapes = {"n": "\n", "r": "\r", "t": "\t", "0": "\0", "\\": "\\", '"': '"', "'": "'"}
    while cursor < len(content):
        character = content[cursor]
        if character != "\\":
            output.append(character)
            cursor += 1
            continue
        cursor += 1
        if cursor >= len(content):
            return None
        escaped = content[cursor]
        if escaped in escapes:
            output.append(escapes[escaped])
            cursor += 1
        elif escaped == "x":
            output.append(chr(int(content[cursor + 1:cursor + 3], 16)))
            cursor += 3
        elif escaped == "u" and content[cursor + 1:cursor + 2] == "{":
            closing = content.find("}", cursor + 2)
            if closing < 0:
                return None
            output.append(chr(int(content[cursor + 2:closing].replace("_", ""), 16)))
            cursor = closing + 1
        elif escaped in "\r\n":
            while cursor < len(content) and content[cursor].isspace():
                cursor += 1
        else:
            return None
    return "".join(output)


def _split_items(tokens):
    parts = []
    part = []
    depth = 0
    for token in tokens:
        if token[0] == "punctuation" and token[1] in ("(", "[", "{"):
            depth += 1
        elif token[0] == "punctuation" and token[1] in (")", "]", "}"):
            depth -= 1
        if token[1] == "," and depth == 0:
            parts.append(part)
            part = []
        else:
            part.append(token)
    parts.append(part)
    return parts


def _declarations(tokens):
    """Walk lint attributes and cfg_attr branches, never arbitrary macro arguments."""
    if len(tokens) < 3 or tokens[0][0] != "name" or tokens[1][1] != "(" or tokens[-1][1] != ")":
        return
    kind = tokens[0][1]
    parts = _split_items(tokens[2:-1])
    if kind == "cfg_attr":
        for branch in parts[1:]:  # The first argument is a condition, not an attribute.
            yield from _declarations(branch)
        return
    if kind not in ("allow", "expect"):
        return
    names = []
    reason = None
    for part in parts:
        if not part:
            continue
        if len(part) == 3 and part[0][1] == "reason" and part[1][1] == "=":
            try:
                reason = _decode_reason(part[2][1])
            except (ValueError, OverflowError):
                reason = None  # Invalid Rust strings are also rejected by rustc.
        else:
            names.append("".join(item[1] for item in part))
    yield tokens[0], names, reason


def exception_inventory(root):
    """List each genuine allow/expect with its own reason and exact location."""
    records = []
    for path in sorted((root / "crates").rglob("*.rs")):
        source = path.read_text()
        for tokens, attribute in _attributes(source):
            for token, names, reason in _declarations(tokens):
                records.append({
                    "file": str(path.relative_to(root)),
                    "line": source.count("\n", 0, token[2]) + 1,
                    "kinds": [token[1]],
                    "lints": [name for name in names if name.startswith("clippy::")],
                    "compiler_lints": [name for name in names if not name.startswith("clippy::")],
                    "reason": reason,
                    "attribute": attribute,
                })
    return records


def exception_issues(records):
    """Require individual Clippy exceptions with reasons; reject broad suppressions."""
    issues = []
    groups = {f"clippy::{name}" for name in (
        "all", "pedantic", "nursery", "restriction", "cargo", "correctness",
        "suspicious", "style", "complexity", "perf", "internal", "internal_warn",
    )}
    for record in records:
        location = f'{record["file"]}:{record["line"]}'
        if record["lints"] and not (record["reason"] or "").strip():
            issues.append(f"{location}: Clippy exception needs a concrete reason")
        if groups.intersection(record["lints"]) or "warnings" in record.get("compiler_lints", []):
            issues.append(f"{location}: blanket lint group exceptions are prohibited")
    return issues

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/lint-audit")
    parser.add_argument("--deny-findings", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    findings = {}
    failed = False
    for profile, selection in (
        ("full", ["--workspace", "--all-features"]),
        ("core", ["-p", "kipferl-runtime", "--no-default-features"]),
    ):
        command = ["cargo", "clippy", "--locked", *selection, "--all-targets", "--message-format=json", "--"]
        # Ordinary warning levels honor reviewed expectations. Their exact
        # declarations and reasons are retained in the separate inventory.
        command.extend(f"-Wclippy::{lint}" for lint in LINTS)
        log_path = args.output / f"{profile}.jsonl"
        with log_path.open("w") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, check=False)
        failed |= result.returncode != 0
        with log_path.open() as log:
            for key, record in diagnostics(log, profile).items():
                if key in findings:
                    findings[key]["profiles"].append(profile)
                else:
                    findings[key] = record
    records = sorted(findings.values(), key=lambda r: (r["file"], r["line"], r["lint"]))
    counts = Counter(record["lint"] for record in records)
    (args.output / "diagnostics.json").write_text(json.dumps(records, indent=2) + "\n")
    exceptions = exception_inventory(ROOT)
    issues = exception_issues(exceptions)
    (args.output / "exceptions.json").write_text(json.dumps(exceptions, indent=2) + "\n")
    lines = ["# Strict Rust lint audit", "", f"{len(records)} unique source diagnostics across full/core profiles.",
             "", f"{len(exceptions)} explicit exception declarations are inventoried below (including Rust naming/platform allowances).",
             "Tests and benchmark fixtures are included. Reviewed expectations are honored; no allowances are generated.",
             "", "| Lint | Findings |", "| --- | ---: |"]
    lines.extend(f"| `{lint}` | {count} |" for lint, count in counts.most_common())
    lines.extend(["", "## Locations", ""])
    for record in records:
        location = f'{record["file"]}:{record["line"]}:{record["column"]}'
        message = record["message"].replace("\n", " ")
        lines.append(f'- `{location}` — `{record["lint"]}`: {message}')
    lines.extend(["", "## Explicit exceptions", ""])
    for exception in exceptions:
        location = f'{exception["file"]}:{exception["line"]}'
        names = ", ".join(exception["lints"] + exception["compiler_lints"])
        reason = exception["reason"] or "See declaration (non-Clippy allowance)"
        kind = exception["kinds"][0]
        lines.append(f"- `{location}` — `{kind}({names})`: {reason}")
    if issues:
        lines.extend(["", "## Exception policy violations", "", *issues])
    (args.output / "report.md").write_text("\n".join(lines) + "\n")
    print(f"Strict audit: {len(records)} diagnostics, {len(exceptions)} exception declarations, {len(issues)} exception policy violations. Report: {args.output / 'report.md'}")
    return 1 if failed or issues or (args.deny_findings and records) else 0


if __name__ == "__main__":
    sys.exit(main())
