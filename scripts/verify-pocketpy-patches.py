#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _download_pocketpy_c(version: str) -> str:
    url = (
        f"https://github.com/pocketpy/pocketpy/releases/download/v{version}/pocketpy.c"
    )
    req = urllib.request.Request(url, headers={"User-Agent": "ucharm/patch-verify"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = resp.read()
    return data.decode("utf-8", errors="replace")


def _repository_path(repo_root: Path, relative: str, failures: list[str]) -> Path | None:
    path = (repo_root / relative).resolve()
    try:
        path.relative_to(repo_root.resolve())
    except ValueError:
        failures.append(f"manifest path escapes repository: {relative}")
        return None
    return path


def _validate_manifest(
    repo_root: Path, manifest: dict, failures: list[str]
) -> tuple[list[Path], list[dict]]:
    patch_entries = manifest.get("patch_files", [])
    tracked_entries = manifest.get("tracked_files", [])
    if not patch_entries:
        failures.append("manifest has no patch_files")
    if not tracked_entries:
        failures.append("manifest has no tracked_files")

    patch_paths: list[Path] = []
    declared_files: list[str] = []
    declared_ids: list[str] = []
    for entry in patch_entries:
        patch_id = entry.get("id")
        relative = entry.get("file")
        if not patch_id or not relative:
            failures.append(f"invalid patch_files entry: {entry}")
            continue
        declared_ids.append(patch_id)
        declared_files.append(relative)
        if Path(relative).name != f"{patch_id}.patch":
            failures.append(f"patch id and filename disagree: {patch_id}, {relative}")
        path = _repository_path(repo_root, relative, failures)
        if path is None:
            continue
        if not path.is_file():
            failures.append(f"missing patch file: {relative}")
            continue
        patch_paths.append(path)

    if len(declared_ids) != len(set(declared_ids)):
        failures.append("manifest contains duplicate patch ids")
    if len(declared_files) != len(set(declared_files)):
        failures.append("manifest contains duplicate patch files")

    patch_directory = repo_root / "pocketpy" / "patches"
    actual_files = {
        str(path.relative_to(repo_root)) for path in patch_directory.glob("*.patch")
    }
    declared_set = set(declared_files)
    if actual_files != declared_set:
        failures.append(
            "manifest patch file set differs from disk: "
            f"missing={sorted(actual_files - declared_set)}, "
            f"extra={sorted(declared_set - actual_files)}"
        )

    return patch_paths, tracked_entries


def _verify_tracked_files(
    repo_root: Path, tracked: list[dict], failures: list[str]
) -> list[dict]:
    records = []
    for entry in tracked:
        relative = entry.get("path")
        if not relative:
            failures.append(f"invalid tracked_files entry: {entry}")
            continue
        path = _repository_path(repo_root, relative, failures)
        if path is None:
            continue
        if not path.is_file():
            failures.append(f"missing file: {relative}")
            continue

        text = _read_text(path)
        anchors = entry.get("anchors", [])
        if not anchors:
            failures.append(f"tracked file has no anchors: {relative}")
        for anchor in anchors:
            if anchor not in text:
                failures.append(f"missing anchor in {relative}: {anchor}")
        records.append(
            {
                "path": relative,
                "sha256": _sha256(path),
                "anchor_count": len(anchors),
            }
        )
    return records


def _replay_patchset(
    repo_root: Path,
    upstream_text: str,
    patch_paths: list[Path],
    tracked: list[dict],
    failures: list[str],
) -> bool:
    upstream_relative = Path("pocketpy/vendor/pocketpy.c")
    with tempfile.TemporaryDirectory() as raw_directory:
        replay_root = Path(raw_directory)
        upstream_path = replay_root / upstream_relative
        upstream_path.parent.mkdir(parents=True)
        upstream_path.write_text(upstream_text, encoding="utf-8")

        for patch_path in patch_paths:
            try:
                applied = subprocess.run(
                    ["git", "apply", str(patch_path)],
                    cwd=replay_root,
                    capture_output=True,
                    text=True,
                    check=False,
                )
            except FileNotFoundError:
                failures.append("git is required to replay the PocketPy patchset")
                return False
            if applied.returncode != 0:
                detail = applied.stderr.strip() or applied.stdout.strip()
                failures.append(f"cannot replay {patch_path.name}: {detail}")
                return False

        matches = True
        for entry in tracked:
            relative = entry.get("path")
            if not relative:
                continue
            replayed = replay_root / relative
            vendored = repo_root / relative
            if not replayed.is_file():
                failures.append(f"patch replay did not produce tracked file: {relative}")
                matches = False
            elif replayed.read_bytes() != vendored.read_bytes():
                failures.append(f"patch replay differs from vendored file: {relative}")
                matches = False
        return matches


def _write_report(path: Path, report: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Verify μcharm PocketPy vendor patchset.")
    parser.add_argument(
        "--check-upstream",
        action="store_true",
        help="Verify anchors are absent upstream and replay patches onto pristine PocketPy.",
    )
    parser.add_argument(
        "--upstream-path",
        type=Path,
        default=None,
        help="Use a local pristine upstream pocketpy.c instead of downloading.",
    )
    parser.add_argument(
        "--pocketpy-version",
        default=None,
        help="PocketPy version to check upstream against (defaults to pocketpy/POCKETPY_VERSION).",
    )
    parser.add_argument(
        "--report",
        type=Path,
        default=None,
        help="Write a machine-readable JSON verification report.",
    )
    args = parser.parse_args(argv)

    repo_root = Path(__file__).resolve().parent.parent
    manifest_path = repo_root / "pocketpy" / "patches" / "manifest.json"
    if not manifest_path.exists():
        print(f"error: missing manifest: {manifest_path}")
        return 2

    try:
        manifest = json.loads(_read_text(manifest_path))
    except json.JSONDecodeError as error:
        print(f"error: invalid patch manifest: {error}")
        return 2

    failures: list[str] = []
    patch_paths, tracked = _validate_manifest(repo_root, manifest, failures)
    tracked_records = _verify_tracked_files(repo_root, tracked, failures)
    version = args.pocketpy_version
    replay_matches_vendor: bool | None = None

    if args.check_upstream:
        vendor_anchors = [
            anchor for entry in tracked for anchor in entry.get("anchors", [])
        ]
        upstream_text = ""
        if args.upstream_path is not None:
            try:
                upstream_text = _read_text(args.upstream_path)
            except OSError as error:
                failures.append(f"failed to read upstream PocketPy: {error}")
        else:
            if version is None:
                version_file = repo_root / "pocketpy" / "POCKETPY_VERSION"
                if not version_file.exists():
                    failures.append(
                        "missing pocketpy/POCKETPY_VERSION (needed for --check-upstream)"
                    )
                else:
                    version = version_file.read_text(encoding="utf-8").strip()
            if version:
                try:
                    upstream_text = _download_pocketpy_c(version)
                except (urllib.error.URLError, TimeoutError) as error:
                    failures.append(
                        f"failed to download upstream pocketpy.c for v{version}: {error}"
                    )

        if upstream_text:
            for anchor in vendor_anchors:
                if anchor in upstream_text:
                    failures.append(
                        f"upstream contains vendor anchor unexpectedly: {anchor}"
                    )
            if "ucharm patch:" in upstream_text:
                failures.append("upstream contains 'ucharm patch:' markers unexpectedly")
            replay_matches_vendor = _replay_patchset(
                repo_root, upstream_text, patch_paths, tracked, failures
            )

    patch_records = [
        {
            "id": entry.get("id"),
            "path": entry.get("file"),
            "sha256": _sha256(repo_root / entry["file"])
            if entry.get("file") and (repo_root / entry["file"]).is_file()
            else None,
        }
        for entry in manifest.get("patch_files", [])
    ]
    report = {
        "schema_version": 1,
        "status": "fail" if failures else "pass",
        "patchset_id": manifest.get("patchset_id"),
        "patchset_version": manifest.get("patchset_version"),
        "pocketpy_version": version,
        "upstream_checked": args.check_upstream,
        "replay_matches_vendor": replay_matches_vendor,
        "patch_files": patch_records,
        "tracked_files": tracked_records,
        "failures": failures,
    }
    if args.report is not None:
        _write_report(args.report, report)
        print(f"Wrote patch verification report: {args.report}")

    if failures:
        print("PocketPy vendor patch verification failed:")
        for item in failures:
            print(f"  - {item}")
        return 1

    patchset_id = manifest.get("patchset_id", "<unknown>")
    patchset_version = manifest.get("patchset_version", "<unknown>")
    suffix = "; pristine replay matches vendor" if replay_matches_vendor else ""
    print(f"PocketPy vendor patches OK: {patchset_id} v{patchset_version}{suffix}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
