#!/usr/bin/env python3
"""Audit a pinned popularity snapshot without executing any package source.

Latest-release metadata is pinned before artifact work and reused on resume.
Only exact, hash-verified syntax failures are exported as reusable catalog
records. Compilation success is always unverified, never behaviorally tested.
"""
from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed
import csv
from datetime import datetime, timezone
from email.parser import BytesParser
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import zipfile

from pip._vendor import packaging
from pip._vendor.packaging.requirements import InvalidRequirement, Requirement
from pip._vendor.packaging.specifiers import InvalidSpecifier, SpecifierSet

from package_catalog import check_syntax, verify_syntax_checker

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "compatibility/packages"
DEFAULT_REPORT = DIRECTORY / "popularity-audit.json"
PYTHON_TARGET = "3.11.0"
MAX_WHEEL = 40 * 1024 * 1024
MAX_EXPANDED = 120 * 1024 * 1024
MAX_METADATA = 32 * 1024 * 1024
MAX_FILES = 10000
MAX_SOURCES = 2000
COMPILER_TIMEOUT = 10
PACKAGE_TIMEOUT = 90
NAME = re.compile(r"[A-Za-z0-9](?:[A-Za-z0-9._-]*[A-Za-z0-9])?\Z")
HASH = re.compile(r"[a-f0-9]{64}\Z")
CATEGORIES = {"native_only", "source_only", "python_requirement", "unsupported_requirement", "syntax", "limits", "network", "unverified"}


class AuditLimit(ValueError):
    """A bounded audit deliberately stopped before reaching a verdict."""


# Frozen description of the initial run, used only by the explicit migration
# switch. A future parser or policy change must never inherit these results.
LEGACY_POLICY_V1 = {
    "version": 1,
    "python_metadata_target": "3.11.0",
    "requirement_parser": "pip-vendored packaging 26.2",
    "max_wheel_bytes": 40 * 1024 * 1024,
    "max_expanded_bytes": 120 * 1024 * 1024,
    "max_metadata_bytes": 32 * 1024 * 1024,
    "max_archive_members": 10000,
    "max_source_files": 2000,
    "compiler_timeout_seconds": 10,
    "package_timeout_seconds": 90,
    "selection": "latest-release; prefer py3-none-any, then py2.py3-none-any",
    "execution": "compile only; no imports or package behavior",
    "coverage": "stop at first compiler failure; unvisited files remain unchecked",
}


def current_policy() -> dict:
    return {
        "version": 2,
        "python_metadata_target": PYTHON_TARGET,
        "requirement_parser": f"pip-vendored packaging {packaging.__version__}",
        "max_wheel_bytes": MAX_WHEEL,
        "max_expanded_bytes": MAX_EXPANDED,
        "max_metadata_bytes": MAX_METADATA,
        "max_archive_members": MAX_FILES,
        "max_source_files": MAX_SOURCES,
        "compiler_timeout_seconds": COMPILER_TIMEOUT,
        "package_timeout_seconds": PACKAGE_TIMEOUT,
        "selection": "latest-release; prefer py3-none-any, then py2.py3-none-any",
        "execution": "runtime --check-syntax in module mode; no imports or package behavior",
        "syntax_checker": "--check-syntax -- <source>; EXEC_MODE with dynamic=false",
        "coverage": "stop at first compiler failure; unvisited files remain unchecked",
    }


def policy_digest(policy: dict) -> str:
    return sha256(json.dumps(policy, sort_keys=True, separators=(",", ":")).encode())


def cache_key(snapshot_hash: str, runtime_hash: str, policy: dict) -> str:
    return policy_digest({"snapshot_sha256": snapshot_hash, "runtime_sha256": runtime_hash, "audit_policy_sha256": policy_digest(policy)})


def migrate_legacy(legacy: Path, state: Path, policy: dict) -> None:
    if policy != LEGACY_POLICY_V1:
        raise ValueError("legacy checkpoints can only migrate under their exact original parser, limits, and policy")
    if not legacy.is_dir():
        raise ValueError("legacy checkpoint directory does not exist")
    for project in legacy.iterdir():
        if not project.is_dir() or not NAME.fullmatch(project.name):
            continue
        destination = state / project.name
        destination.mkdir(parents=True, exist_ok=True)
        for filename in ("metadata.json", "result.json"):
            source = project / filename
            target = destination / filename
            if source.is_file() and not target.exists():
                shutil.copyfile(source, target)
    atomic_json(state / "migration.json", {"source": legacy.name, "policy": policy, "note": "Explicit migration of the known initial run; policy and parser settings must match exactly."})



def seed_metadata(source: Path, destination: Path, snapshot_hash: str, projects: list[dict]) -> dict:
    """Reuse exact release/artifact pins across runtimes; never copy results."""
    policy_file = source / "policy.json"
    if not policy_file.is_file():
        raise ValueError("metadata seed requires a checkpoint policy.json")
    origin = json.loads(policy_file.read_text())
    if origin.get("snapshot_sha256") != snapshot_hash:
        raise ValueError("metadata seed belongs to a different popularity snapshot")
    copied, missing = [], []
    for project in projects:
        name = normalized(project["name"])
        original = source / name / "metadata.json"
        if not original.is_file():
            missing.append(name)
            continue
        payload = original.read_bytes()
        pin = json.loads(payload)
        if pin.get("name") != name or not pin.get("version") or not HASH.fullmatch(pin.get("metadata_sha256", "")):
            raise ValueError(f"invalid metadata pin for {name}")
        target = destination / name / "metadata.json"
        if target.exists() and target.read_bytes() != payload:
            raise ValueError(f"metadata seed conflicts with an existing immutable pin for {name}")
        if not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = target.with_suffix(".json.tmp")
            temporary.write_bytes(payload)
            temporary.replace(target)
        copied.append(name)
    summary = {
        "source_cache_key": origin.get("cache_key"),
        "source_runtime_sha256": origin.get("runtime_sha256"),
        "snapshot_sha256": snapshot_hash,
        "seeded_count": len(copied), "missing_projects": missing,
        "policy": "Metadata pins only; no prior compatibility results are reused. Projects without metadata pins use a fresh bounded metadata request.",
    }
    atomic_json(destination / "metadata-seed.json", summary)
    return summary


def embedded_runtime_path() -> Path:
    operating_system = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    architecture = {"arm64": "aarch64", "aarch64": "aarch64", "x86_64": "x86_64", "AMD64": "x86_64"}.get(platform.machine())
    if not operating_system or not architecture:
        raise ValueError("embedded runtime auditing supports macOS/Linux on ARM64 or x86-64")
    return ROOT / "crates/kipferl-cli/assets" / f"pocketpy-kipferl-{operating_system}-{architecture}"

def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def normalized(name: str) -> str:
    if not NAME.fullmatch(name):
        raise ValueError(f"invalid distribution name: {name!r}")
    return re.sub(r"[-_.]+", "-", name).lower()


def atomic_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, ensure_ascii=True) + "\n")
    temporary.replace(path)


def official_url(url: str) -> bool:
    return url.startswith(("https://pypi.org/", "https://files.pythonhosted.org/"))


class OfficialRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, file, code, message, headers, newurl):
        if not official_url(newurl):
            raise ValueError("refusing redirect outside official PyPI HTTPS endpoints")
        return super().redirect_request(request, file, code, message, headers, newurl)


def download(url: str, maximum: int) -> bytes:
    if not official_url(url):
        raise ValueError("refusing download outside official PyPI HTTPS endpoints")
    opener = urllib.request.build_opener(OfficialRedirect())
    request = urllib.request.Request(url, headers={"User-Agent": "Kipferl-compatibility-audit/1 (static source compilation; bounded concurrency)"})
    for attempt in range(3):
        try:
            with opener.open(request, timeout=25) as response:
                if not official_url(response.url):
                    raise ValueError("unexpected PyPI response endpoint")
                length = response.headers.get("Content-Length")
                if length and int(length) > maximum:
                    raise AuditLimit(f"download exceeds {maximum // (1024 * 1024)} MiB limit")
                data = response.read(maximum + 1)
            if len(data) > maximum:
                raise AuditLimit(f"download exceeds {maximum // (1024 * 1024)} MiB limit")
            return data
        except (urllib.error.URLError, TimeoutError, OSError) as error:
            if attempt == 2 or isinstance(error, urllib.error.HTTPError) and error.code in {400, 401, 403, 404, 410}:
                raise
            time.sleep(attempt + 1)
    raise RuntimeError("unreachable retry state")


def choose_artifact(files: list[dict]) -> tuple[str, dict | None]:
    usable = [item for item in files if not item.get("yanked")]
    wheels = [item for item in usable if item.get("packagetype") == "bdist_wheel" and item.get("filename", "").endswith(".whl")]
    pure = []
    for item in wheels:
        parts = item["filename"].removesuffix(".whl").rsplit("-", 3)
        if len(parts) == 4 and parts[2:] == ["none", "any"] and "py3" in parts[1].split("."):
            pure.append(item)
    if pure:
        # Exact Python3-only universal wheels take precedence over mixed2/3.
        pure.sort(key=lambda item: (not item["filename"].endswith("-py3-none-any.whl"), item["filename"]))
        return "pure", pure[0]
    if wheels:
        return "native_only", sorted(wheels, key=lambda item: item["filename"])[0]
    sources = [item for item in usable if item.get("packagetype") == "sdist"]
    return "source_only", sorted(sources, key=lambda item: item["filename"])[0] if sources else None


def pin_metadata(project: dict, checkpoint: Path, metadata_prefetch: Path | None = None) -> dict:
    path = checkpoint / "metadata.json"
    if path.exists():
        return json.loads(path.read_text())
    name = normalized(project["name"])
    metadata_url = f"https://pypi.org/pypi/{name}/json"
    prefetched = None if metadata_prefetch is None else metadata_prefetch / f"{name}.json"
    if prefetched is not None and prefetched.is_file() and prefetched.stat().st_size <= MAX_METADATA:
        raw = prefetched.read_bytes()
        fetched_at = datetime.fromtimestamp(prefetched.stat().st_mtime, timezone.utc).isoformat(timespec="seconds")
    else:
        raw = download(metadata_url, MAX_METADATA)
        fetched_at = now()
    metadata = json.loads(raw)
    info = metadata["info"]
    if normalized(info["name"]) != name or not isinstance(info.get("version"), str):
        raise ValueError("PyPI returned inconsistent project identity")
    kind, artifact = choose_artifact(metadata.get("urls", []))
    pin = {
        "name": name, "version": info["version"], "metadata_url": metadata_url,
        "metadata_sha256": sha256(raw), "metadata_fetched_at": fetched_at,
        "requires_python": info.get("requires_python"), "requires_dist": info.get("requires_dist") or [],
        "artifact_kind": kind,
        "artifact": None if artifact is None else {key: artifact.get(key) for key in ("filename", "url", "digests", "size", "requires_python", "upload_time_iso_8601", "packagetype")},
    }
    atomic_json(path, pin)
    return pin


def initial_record(project: dict) -> dict:
    return {
        "rank": project["rank"], "name": normalized(project["name"]), "downloads": project.get("downloads"),
        "version": None, "status": "unverified", "category": "unverified", "evidence_scope": "none", "evidence": "Audit pending.",
        "artifact_verified": False, "compilation_completed": False, "requires_python": None, "requires_dist": [],
        "source_files_total": None, "sources_checked": 0, "remaining": None, "first_blocker": None,
        "wheel_filename": None, "wheel_sha256": None, "source_url": None,
    }


def finish(record: dict, category: str, evidence: str, scope: str | None = None) -> dict:
    record["category"] = category
    record["status"] = "incompatible" if category in {"syntax", "native_only", "source_only", "python_requirement"} else "unverified"
    record["evidence"] = evidence
    if scope is not None:
        record["evidence_scope"] = scope
    total = record["source_files_total"]
    record["remaining"] = None if total is None else total - record["sources_checked"]
    return record


def check_requirements(record: dict, requires_python: str | None, requires_dist: list[str]) -> dict | None:
    record["requires_python"] = requires_python
    record["requires_dist"] = requires_dist
    scope = "verified_artifact" if record["artifact_verified"] else "metadata"
    if requires_python:
        try:
            if not SpecifierSet(requires_python).contains(PYTHON_TARGET, prereleases=True):
                return finish(record, "python_requirement", f"The selected release declares Requires-Python {requires_python}; Kipferl advertises {PYTHON_TARGET}. Older releases were not evaluated.", scope)
        except InvalidSpecifier:
            return finish(record, "unsupported_requirement", f"Cannot interpret Requires-Python {requires_python!r}; no compatibility claim was made.", scope)
    for text in requires_dist:
        try:
            requirement = Requirement(text)
            if requirement.url:
                return finish(record, "unsupported_requirement", f"Dependency {text!r} uses a direct URL, which Kipferl's initial package manager does not support.", scope)
        except (InvalidRequirement, TypeError):
            return finish(record, "unsupported_requirement", f"Cannot interpret dependency requirement {text!r}; dependency metadata is preserved for investigation.", scope)
    return None


def verified_wheel(artifact: dict, cache: Path) -> bytes:
    expected = artifact.get("digests", {}).get("sha256", "")
    if not HASH.fullmatch(expected):
        raise ValueError("PyPI artifact does not declare a valid SHA-256")
    path = cache / "wheels" / (expected + ".whl")
    if path.exists():
        if path.stat().st_size > MAX_WHEEL:
            raise AuditLimit("cached wheel exceeds size limit")
        data = path.read_bytes()
    else:
        if artifact.get("size", 0) > MAX_WHEEL:
            raise AuditLimit("wheel exceeds 40 MiB download limit")
        data = download(artifact["url"], MAX_WHEEL)
    if sha256(data) != expected:
        raise ValueError("wheel SHA-256 does not match pinned PyPI metadata")
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(".tmp")
        temporary.write_bytes(data)
        temporary.replace(path)
    return data


def extract_sources(wheel: bytes, root: Path) -> tuple[list[Path], object]:
    with zipfile.ZipFile(io.BytesIO(wheel)) as archive:
        infos = archive.infolist()
        if len(infos) > MAX_FILES or sum(item.file_size for item in infos) > MAX_EXPANDED:
            raise AuditLimit("wheel exceeds 10,000 members or 120 MiB expanded-size limit")
        seen = set()
        for info in infos:
            path = PurePosixPath(info.filename)
            if path.is_absolute() or ".." in path.parts or "\\" in info.filename or any(ord(char) < 32 or ord(char) == 127 for char in info.filename) or stat.S_ISLNK(info.external_attr >> 16):
                raise ValueError("wheel contains an unsafe path or symlink")
            if path in seen:
                raise ValueError("wheel contains duplicate normalized paths")
            seen.add(path)
        metadata_files = [info for info in infos if info.filename.endswith(".dist-info/METADATA") and len(PurePosixPath(info.filename).parts) == 2]
        if len(metadata_files) != 1:
            raise ValueError("wheel must contain exactly one distribution METADATA file")
        wheel_metadata = BytesParser().parsebytes(archive.read(metadata_files[0]))
        archive.extractall(root)
    return sorted(root.rglob("*.py")), wheel_metadata


def concise_output(result: subprocess.CompletedProcess, temporary: Path) -> str:
    output = (result.stdout + result.stderr).replace(str(temporary), "<staging>").strip()
    return "\n".join(line[:350] for line in output.splitlines()[-8:]) or f"compiler exited with code {result.returncode}"


def inspect_artifact(record: dict, pin: dict, runtime: Path, cache: Path) -> dict:
    artifact = pin["artifact"]
    kind = pin["artifact_kind"]
    if kind == "native_only":
        return finish(record, "native_only", "The selected release has no generic Python3 pure wheel; its published wheels require a platform/ABI or a different Python implementation tag. Artifact bytes were not downloaded. Older releases and source builds were not evaluated.", "metadata")
    if kind == "source_only":
        return finish(record, "source_only", "The selected release publishes no usable wheel. Kipferl does not run source-build backends; artifact bytes were not downloaded and older releases were not evaluated.", "metadata")
    requirement_result = check_requirements(record, artifact.get("requires_python") or pin["requires_python"], pin["requires_dist"])
    if requirement_result:
        return requirement_result
    data = verified_wheel(artifact, cache)
    record["artifact_verified"] = True
    record["evidence_scope"] = "verified_artifact"
    with tempfile.TemporaryDirectory(prefix="compile-", dir=cache) as temporary:
        work = Path(temporary)
        root = work / "wheel"
        root.mkdir()
        sources, metadata = extract_sources(data, root)
        record["source_files_total"] = len(sources)
        if normalized(metadata.get("Name", "")) != record["name"] or metadata.get("Version") != record["version"]:
            raise ValueError("wheel METADATA name/version does not match the pinned PyPI release")
        record["wheel_requires_python"] = metadata.get("Requires-Python")
        record["wheel_requires_dist"] = metadata.get_all("Requires-Dist", [])
        record["requirement_metadata_source"] = "verified_wheel_metadata"
        requirement_result = check_requirements(record, metadata.get("Requires-Python"), metadata.get_all("Requires-Dist", []))
        if requirement_result:
            return requirement_result
        native = next((path for path in root.rglob("*") if path.suffix.lower() in {".so", ".pyd", ".dll", ".dylib"}), None)
        if native:
            return finish(record, "native_only", f"The downloaded wheel is tagged pure but contains native library {native.relative_to(root).as_posix()}; Kipferl cannot load it.", "verified_artifact")
        deadline = time.monotonic() + PACKAGE_TIMEOUT
        for source in sources:
            if record["sources_checked"] >= MAX_SOURCES or time.monotonic() >= deadline:
                return finish(record, "limits", "Source compilation reached the 2,000-file or 90-second per-package bound. Remaining source files were not checked.")
            relative = source.relative_to(root).as_posix()
            try:
                # A dedicated module-mode compiler handles the file. Builtin
                # compile() uses different semantics for relative imports and
                # future statements, so it is not a faithful package checker.
                result = check_syntax(runtime, source, work, timeout=COMPILER_TIMEOUT)
            except subprocess.TimeoutExpired:
                return finish(record, "limits", f"Compiler exceeded the {COMPILER_TIMEOUT}-second limit on {relative}; no behavioral compatibility claim was made.")
            record["sources_checked"] += 1
            if result.returncode:
                diagnostic = concise_output(result, work)
                record["first_blocker"] = {"file": relative, "diagnostic": diagnostic}
                if any(line.startswith("SyntaxError:") for line in diagnostic.splitlines()):
                    return finish(record, "syntax", f"Verified wheel source fails compilation at {relative}. This is the first concrete blocker; later files were not checked.\n{diagnostic}")
                return finish(record, "unverified", f"Compiler could not finish {relative}; this is not treated as proof of a package-language incompatibility.\n{diagnostic}")
        record["compilation_completed"] = True
        if not sources:
            return finish(record, "unverified", "The wheel contains no .py source files to compile; it may be a stub, data, or dependency-only distribution. Dependencies, runtime APIs, assets, and behavior were not exercised.")
        return finish(record, "unverified", f"All {len(sources)} Python source files compile. Imports, runtime APIs, dependency closure, assets, and behavior were not exercised; this is not a tested compatibility result.")


def audit_project(project: dict, runtime: Path, cache: Path, state: Path, retry_network: bool = False, metadata_prefetch: Path | None = None) -> dict:
    checkpoint = state / normalized(project["name"])
    result_path = checkpoint / "result.json"
    if result_path.exists():
        previous = json.loads(result_path.read_text())
        if not retry_network or previous["category"] != "network":
            previous.setdefault("requirement_metadata_source", "verified_wheel_metadata" if "wheel_requires_dist" in previous else "pypi_release_json" if "metadata_url" in previous else "none")
            if previous["category"] == "unverified" and previous.get("source_files_total") == 0:
                previous["evidence"] = "The wheel contains no .py source files to compile; it may be a stub, data, or dependency-only distribution. Dependencies, runtime APIs, assets, and behavior were not exercised."
            atomic_json(result_path, previous)
            return previous
    record = initial_record(project)
    try:
        pin = pin_metadata(project, checkpoint, metadata_prefetch)
        record.update(version=pin["version"], metadata_url=pin["metadata_url"], metadata_sha256=pin["metadata_sha256"], metadata_fetched_at=pin["metadata_fetched_at"], requires_python=pin["requires_python"], requires_dist=pin["requires_dist"], requirement_metadata_source="pypi_release_json")
        artifact = pin["artifact"]
        if artifact:
            record.update(source_url=artifact["url"], selected_artifact_filename=artifact["filename"], artifact_declared_sha256=artifact["digests"].get("sha256"))
            if artifact["filename"].endswith(".whl"):
                record.update(wheel_filename=artifact["filename"], wheel_sha256=artifact["digests"].get("sha256"))
        record = inspect_artifact(record, pin, runtime, cache)
    except AuditLimit as error:
        record = finish(record, "limits", str(error))
    except (urllib.error.URLError, TimeoutError, ConnectionError) as error:
        record = finish(record, "network", f"Registry/download request did not complete: {error}. Resume with --retry-network; already pinned metadata is retained.")
    # ZIP readers raise RuntimeError for encryption and unsupported codecs;
    # RecursionError/NotImplementedError inherit it. One unreadable artifact
    # must become a resumable unverified row instead of aborting the batch.
    except (ValueError, OSError, zipfile.BadZipFile, KeyError, TypeError, RuntimeError, EOFError) as error:
        record = finish(record, "unverified", f"Artifact or metadata validation could not establish compatibility: {error}")
    atomic_json(result_path, record)
    return record


def validate_report(report: dict) -> None:
    if report.get("schema_version") != 1 or not isinstance(report.get("records"), list):
        raise ValueError("unsupported audit report schema")
    for key in ("runtime_sha256", "snapshot_sha256"):
        if not HASH.fullmatch(report.get(key, "")):
            raise ValueError(f"invalid report {key}")
    policy = report.get("audit_policy")
    if not isinstance(policy, dict) or report.get("audit_policy_sha256") != policy_digest(policy):
        raise ValueError("audit policy digest does not match its settings")
    if report.get("cache_key") != cache_key(report["snapshot_sha256"], report["runtime_sha256"], policy):
        raise ValueError("cache identity does not include snapshot, runtime, and policy")
    ranks, names = set(), set()
    for record in report["records"]:
        if type(record.get("rank")) is not int or record["rank"] <= 0 or record["rank"] in ranks:
            raise ValueError("invalid/duplicate popularity rank")
        name = normalized(record["name"])
        if name in names:
            raise ValueError("duplicate distribution name")
        ranks.add(record["rank"])
        names.add(name)
        category = record.get("category")
        if category not in CATEGORIES:
            raise ValueError("invalid audit category")
        expected = "incompatible" if category in {"syntax", "native_only", "source_only", "python_requirement"} else "unverified"
        if record.get("status") != expected:
            raise ValueError("audit status must not promote compilation to tested")
        if type(record.get("artifact_verified")) is not bool or record.get("evidence_scope") not in {"none", "metadata", "verified_artifact"}:
            raise ValueError("invalid artifact evidence provenance")
        if record["artifact_verified"] != (record["evidence_scope"] == "verified_artifact"):
            raise ValueError("artifact verification and evidence scope disagree")
        if record.get("artifact_verified"):
            if not HASH.fullmatch(record.get("wheel_sha256") or "") or not record.get("wheel_filename") or not official_url(record.get("source_url") or ""):
                raise ValueError("verified artifact requires exact wheel identity")
        if category == "syntax" and (not record.get("artifact_verified") or record.get("evidence_scope") != "verified_artifact" or not record.get("first_blocker") or not any(line.startswith("SyntaxError:") for line in record["first_blocker"].get("diagnostic", "").splitlines())):
            raise ValueError("syntax evidence requires verified bytes and a source blocker")
        if isinstance(policy.get("version"), int) and policy["version"] >= 2:
            if type(record.get("compilation_completed")) is not bool:
                raise ValueError("module-mode audit rows require explicit compilation completion")
            if record["compilation_completed"] and (
                not record["artifact_verified"] or record["category"] != "unverified"
                or record.get("remaining") != 0 or record.get("first_blocker") is not None
                or record.get("source_files_total") is None
            ):
                raise ValueError("compilation completion contradicts artifact or compiler evidence")
        total, checked = record.get("source_files_total"), record.get("sources_checked")
        if type(checked) is not int or checked < 0 or total is not None and (type(total) is not int or total < checked or record.get("remaining") != total - checked):
            raise ValueError("inconsistent source coverage")
    if report.get("counts") != dict(Counter(record["category"] for record in report["records"])):
        raise ValueError("audit counts do not match records")
    if report.get("completed_count") != len(report["records"]):
        raise ValueError("completed count does not match records")
    if report.get("complete") and report.get("requested_count") != len(report["records"]):
        raise ValueError("complete audit does not cover requested projects")


def make_report(records: list[dict], snapshot: dict, snapshot_hash: str, runtime_hash: str, target: str, requested: int, metadata_seed: dict | None = None) -> dict:
    ordered = sorted(records, key=lambda record: record["rank"])
    policy = current_policy()
    report = {
        "schema_version": 1, "complete": len(ordered) == requested, "requested_count": requested, "completed_count": len(ordered),
        "snapshot_sha256": snapshot_hash, "ranking_source": snapshot.get("source", {}),
        "audit_policy": policy, "audit_policy_sha256": policy_digest(policy), "cache_key": cache_key(snapshot_hash, runtime_hash, policy),
        "metadata_seed": metadata_seed,
        "runtime_sha256": runtime_hash, "target": target, "python_metadata_target": PYTHON_TARGET,
        "requirement_parser": f"pip-vendored packaging {packaging.__version__}",
        "selection": "Latest PyPI release at each pinned metadata fetch. Older releases and alternative wheels are not evaluated.",
        "execution_policy": "No package imports, setup.py, build backends, or package behavior are executed. Only the runtime compiler processes source.",
        "counts": dict(Counter(record["category"] for record in ordered)), "records": ordered,
    }
    validate_report(report)
    return report


def catalog_export(report: dict) -> dict:
    catalog = {"schema_version": 1, "records": []}
    for record in report["records"]:
        if record["category"] == "syntax" and record["artifact_verified"]:
            catalog["records"].append({
                "name": record["name"], "version": record["version"], "wheel_filename": record["wheel_filename"], "wheel_sha256": record["wheel_sha256"],
                "runtime_sha256": report["runtime_sha256"], "target": report["target"], "status": "incompatible", "evidence": record["evidence"],
                "source_url": record["source_url"], "source_files_checked": record["sources_checked"], "source_files_total": record["source_files_total"], "compile_failures": [record["first_blocker"]],
            })
    return catalog


def csv_export(report: dict) -> str:
    fields = ["rank", "name", "version", "status", "category", "artifact_verified", "compilation_completed", "evidence_scope", "sources_checked", "source_files_total", "remaining", "wheel_filename", "wheel_sha256", "requires_python", "evidence"]
    if report["audit_policy"].get("version", 1) < 2:
        fields.remove("compilation_completed")
    stream = io.StringIO(newline="")
    writer = csv.DictWriter(stream, fieldnames=fields, extrasaction="ignore", lineterminator="\n")
    writer.writeheader()
    writer.writerows(report["records"])
    return stream.getvalue()


def validate_outputs(report: dict, snapshot_bytes: bytes, destination: Path) -> None:
    validate_report(report)
    if sha256(snapshot_bytes) != report["snapshot_sha256"]:
        raise ValueError("audit snapshot hash differs from the supplied popularity snapshot")
    snapshot = json.loads(snapshot_bytes)
    if snapshot.get("source", {}) != report.get("ranking_source"):
        raise ValueError("ranking source provenance differs from the popularity snapshot")
    projects = snapshot["projects"]
    expected = {project["rank"]: project for project in projects[:report["requested_count"]]}
    if len(expected) != report["requested_count"]:
        raise ValueError("audit request exceeds the snapshot or includes duplicate ranks")
    for record in report["records"]:
        project = expected.get(record["rank"])
        if not project or normalized(project["name"]) != record["name"] or project.get("downloads") != record.get("downloads"):
            raise ValueError("audit rank, project name, or download count differs from the popularity snapshot")
    if json.loads(destination.with_name("popularity-catalog.json").read_text()) != catalog_export(report):
        raise ValueError("generated syntax catalog differs from the canonical audit report")
    if destination.with_suffix(".csv").read_bytes() != csv_export(report).encode():
        raise ValueError("generated CSV differs from the canonical audit report")


def export(report: dict, destination: Path) -> None:
    atomic_json(destination, report)
    atomic_json(destination.with_name("popularity-catalog.json"), catalog_export(report))
    destination.with_suffix(".csv").write_bytes(csv_export(report).encode())
    lines = ["# Popular-package compatibility audit", "", f"{report['completed_count']} of {report['requested_count']} ranked projects audited on `{report['target']}`; runtime SHA-256 `{report['runtime_sha256']}`.", "", "This is a latest-release screening report. Compilation success remains **unverified**. No package code, imports, or behavior tests were executed. Metadata-only blockers are distinguished from downloaded, hash-verified artifact evidence. Older releases may differ.", "", "| Category | Top 100 | Top 1,000 |", "| --- | ---: | ---: |"]
    for category in sorted(CATEGORIES):
        lines.append(f"| {category} | {sum(r['category'] == category and r['rank'] <= 100 for r in report['records'])} | {sum(r['category'] == category and r['rank'] <= 1000 for r in report['records'])} |")
    lines.extend(["", "Full per-project details and dependency metadata are in [the JSON report](popularity-audit.json); a [CSV export](popularity-audit.csv) is available for analysis. Exact verified syntax blockers are exported separately in [popularity-catalog.json](popularity-catalog.json); the original reviewed behavior catalog is preserved.", "", "The download ranking is a popularity signal, not a quality or compatibility assessment. Its source URL, query, reporting window, snapshot hash, and retrieval time are recorded in the report.", ""])
    destination.with_suffix(".md").write_text("\n".join(lines))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot", type=Path, default=DIRECTORY / "popularity.json")
    runtime_options = parser.add_mutually_exclusive_group()
    runtime_options.add_argument("--runtime", type=Path)
    runtime_options.add_argument("--embedded-runtime", action="store_true", help="audit the exact checked-in full runtime asset embedded by the CLI on this host")
    parser.add_argument("--output", type=Path, default=DEFAULT_REPORT)
    parser.add_argument("--cache", type=Path, default=ROOT / "target/package-audit")
    parser.add_argument("--seed-metadata-from", type=Path, help="copy only release/artifact metadata pins from a prior checkpoint for this ranking snapshot; never reuse its results")
    parser.add_argument("--metadata-prefetch", type=Path, help="explicitly seed new pins from previously fetched raw PyPI JSON files; existing pins are never replaced")
    parser.add_argument("--workers", type=int, default=4, choices=range(1, 7))
    parser.add_argument("--limit", type=int, default=1000)
    parser.add_argument("--migrate-legacy-checkpoints", action="store_true", help="explicitly reuse the known initial policy-v1 run only when its parser and all policy settings match")
    parser.add_argument("--retry-network", action="store_true", help="retry network results while retaining already pinned release metadata")
    parser.add_argument("--check", action="store_true", help="validate an existing report offline")
    args = parser.parse_args()
    if args.check:
        report = json.loads(args.output.read_text())
        validate_outputs(report, args.snapshot.read_bytes(), args.output)
        print(f"Validated {len(report['records'])} audit records; complete={report['complete']}")
        return 0
    if not (args.runtime or args.embedded_runtime) or args.limit < 1:
        parser.error("audit requires --runtime or --embedded-runtime and a positive --limit")
    snapshot_bytes = args.snapshot.read_bytes()
    snapshot = json.loads(snapshot_bytes)
    projects = snapshot["projects"][:args.limit]
    names = [normalized(project["name"]) for project in projects]
    if len(set(names)) != len(names) or any(type(project.get("rank")) is not int or project["rank"] <= 0 for project in projects):
        parser.error("snapshot contains duplicate names or invalid ranks")
    try:
        selected_runtime = embedded_runtime_path() if args.embedded_runtime else args.runtime
    except ValueError as error:
        parser.error(str(error))
    original_runtime = selected_runtime.resolve(strict=True)
    verify_syntax_checker(original_runtime)
    runtime_bytes = original_runtime.read_bytes()
    runtime_hash, snapshot_hash = sha256(runtime_bytes), sha256(snapshot_bytes)
    args.cache = args.cache.resolve()
    policy = current_policy()
    state = args.cache / "checkpoints" / f"v{policy['version']}-{cache_key(snapshot_hash, runtime_hash, policy)}"
    state.mkdir(parents=True, exist_ok=True)
    if args.migrate_legacy_checkpoints:
        legacy = args.cache / "checkpoints" / f"v1-{snapshot_hash[:16]}-{runtime_hash[:16]}"
        migrate_legacy(legacy, state, policy)
    atomic_json(state / "policy.json", {"audit_policy": policy, "snapshot_sha256": snapshot_hash, "runtime_sha256": runtime_hash, "cache_key": cache_key(snapshot_hash, runtime_hash, policy)})
    seed_path = state / "metadata-seed.json"
    seed = json.loads(seed_path.read_text()) if seed_path.is_file() else None
    if args.seed_metadata_from:
        seed = seed_metadata(args.seed_metadata_from.resolve(strict=True), state, snapshot_hash, projects)
        print(f"Seeded {seed['seeded_count']} exact metadata pins; {len(seed['missing_projects'])} projects have no prior metadata. No previous results copied.", flush=True)
    runtime = args.cache / f"runtime-{runtime_hash}"
    if not runtime.exists() or sha256(runtime.read_bytes()) != runtime_hash:
        runtime.write_bytes(runtime_bytes)
        runtime.chmod(0o700)
    target = {"Darwin": "macos", "Linux": "linux"}.get(platform.system(), platform.system().lower()) + "-" + {"arm64": "aarch64", "AMD64": "x86_64"}.get(platform.machine(), platform.machine())
    records = []
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(audit_project, project, runtime, args.cache, state, args.retry_network, args.metadata_prefetch): project for project in projects}
        for future in as_completed(futures):
            record = future.result()
            records.append(record)
            if len(records) % 25 == 0 or len(records) == len(projects):
                report = make_report(records, snapshot, snapshot_hash, runtime_hash, target, len(projects), seed)
                export(report, args.output)
                print(f"{len(records)}/{len(projects)}: {dict(sorted(Counter(r['category'] for r in records).items()))}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
