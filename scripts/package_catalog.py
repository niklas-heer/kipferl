#!/usr/bin/env python3
"""Refresh exact-artifact package evidence, or validate it without network access.

Refresh downloads pinned candidates but does not run their Python source unless
--execute-reviewed is given. Compilation uses the runtime's non-executing file syntax checker.
Reviewed smoke hooks execute package code inside a macOS sandbox that denies
network access, filesystem writes, and home-directory reads. The sandbox still
permits system/temp-file reads: use a disposable machine for new unaudited hooks.
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import subprocess
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "compatibility/packages"
CATALOG = DIRECTORY / "catalog.json"
MAX_DOWNLOAD = 40 * 1024 * 1024
MAX_EXPANDED = 120 * 1024 * 1024
HASH = re.compile(r"[0-9a-f]{64}\Z")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate(catalog: dict) -> None:
    if catalog.get("schema_version") != 1 or not isinstance(catalog.get("records"), list):
        raise ValueError("unsupported catalog schema")
    seen = set()
    for record in catalog["records"]:
        for key in ("name", "version", "wheel_filename", "wheel_sha256", "runtime_sha256", "target", "status", "evidence"):
            if not isinstance(record.get(key), str) or not record[key]:
                raise ValueError(f"missing or invalid {key}")
        for key in ("wheel_sha256", "runtime_sha256"):
            if not HASH.fullmatch(record[key]):
                raise ValueError(f"invalid {key}")
        if record["status"] not in {"tested", "incompatible", "unverified"}:
            raise ValueError("invalid status")
        if record["status"] == "tested":
            smoke = record.get("smoke")
            if not isinstance(smoke, dict) or not smoke.get("scope") or not HASH.fullmatch(smoke.get("sha256", "")):
                raise ValueError("tested evidence requires a hashed smoke hook and explicit scope")
            hook = DIRECTORY / "smoke" / smoke.get("file", "")
            if hook.parent != DIRECTORY / "smoke" or not hook.is_file() or digest(hook.read_bytes()) != smoke["sha256"]:
                raise ValueError("tested smoke hook is missing or changed; refresh evidence")
        key = tuple(record[field] for field in ("name", "version", "wheel_sha256", "runtime_sha256", "target"))
        if key in seen:
            raise ValueError("duplicate catalog evidence key")
        seen.add(key)


def download(url: str, limit: int = MAX_DOWNLOAD) -> bytes:
    if not url.startswith(("https://pypi.org/", "https://files.pythonhosted.org/")):
        raise ValueError("catalog downloads must use official PyPI HTTPS endpoints")
    with urllib.request.urlopen(url, timeout=30) as response:
        if not response.url.startswith(("https://pypi.org/", "https://files.pythonhosted.org/")):
            raise ValueError("unexpected PyPI redirect")
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError("download exceeds catalog size limit")
    return data


def unpack(wheel: bytes, destination: Path) -> list[Path]:
    with zipfile.ZipFile(io.BytesIO(wheel)) as archive:
        infos = archive.infolist()
        if sum(info.file_size for info in infos) > MAX_EXPANDED or len(infos) > 10000:
            raise ValueError("wheel exceeds catalog extraction limits")
        paths = set()
        for info in infos:
            path = PurePosixPath(info.filename)
            if path.is_absolute() or ".." in path.parts or "\\" in info.filename or stat.S_ISLNK(info.external_attr >> 16):
                raise ValueError("unsafe wheel member")
            if path in paths:
                raise ValueError("duplicate wheel member")
            paths.add(path)
        archive.extractall(destination)
    return sorted(destination.rglob("*.py"))


def run(runtime: Path, code: str, directory: Path, *, sandbox: bool = False) -> subprocess.CompletedProcess:
    command = [str(runtime), "-c", code]
    if sandbox:
        if platform.system() != "Darwin" or not shutil.which("sandbox-exec"):
            raise RuntimeError("reviewed smoke execution currently requires macOS sandbox-exec; compilation remains available on all targets")
        # The executable and wheel live in a new temporary directory, outside
        # user homes. This profile is intentionally documented as a restricted
        # developer runner, not a universal untrusted-code security boundary.
        profile = '(version 1)(allow default)(deny network*)(deny file-write*)(deny file-read* (subpath "/Users") (subpath "/home") (subpath "/root"))'
        command = ["sandbox-exec", "-p", profile, *command]
    return subprocess.run(command, cwd=directory, env={"PATH": "/usr/bin:/bin", "HOME": str(directory), "TMPDIR": str(directory)}, capture_output=True, text=True, errors="replace", timeout=15)



def check_syntax(runtime: Path, source: Path, directory: Path, timeout: int = 15) -> subprocess.CompletedProcess:
    """Compile a source file in module mode without imports or execution."""
    return subprocess.run(
        [str(runtime), "--check-syntax", "--", str(source)],
        cwd=directory,
        env={"PATH": "/usr/bin:/bin", "HOME": str(directory), "TMPDIR": str(directory)},
        capture_output=True, text=True, errors="replace", timeout=timeout,
    )


def verify_syntax_checker(runtime: Path) -> None:
    """Refuse a whole audit if the supplied binary lacks the safe checker."""
    with tempfile.TemporaryDirectory(prefix="kipferl-checker-probe-") as temporary:
        directory = Path(temporary)
        source = directory / "probe.py"
        source.write_text('value = 0\ndef update():\n    global value\n    value = 1\nimport __kipferl_checker_must_not_import_this__\nraise RuntimeError("syntax checker executed source")\n')
        result = check_syntax(runtime.resolve(), source, directory)
        if result.returncode:
            detail = (result.stdout + result.stderr).strip()
            raise RuntimeError(f"Runtime does not provide non-executing --check-syntax mode: {detail[:1000]}")

def diagnostic(result: subprocess.CompletedProcess, temporary: Path) -> str:
    output = (result.stdout + result.stderr).replace(str(temporary), "<staging>").strip()
    # Keep a useful source location and actual parser message, without dumping
    # a megabyte-long source line from a package's generated lookup tables.
    return "\n".join(line[:400] for line in output.splitlines()[-8:]) or f"runtime exited with status {result.returncode}"


def refresh(runtimes: list[Path], execute_reviewed: bool) -> dict:
    for runtime in runtimes:
        verify_syntax_checker(runtime)
    candidates = json.loads((DIRECTORY / "candidates.json").read_text())["packages"]
    target = {"Darwin": "macos", "Linux": "linux"}.get(platform.system(), platform.system().lower()) + "-" + {"arm64": "aarch64", "AMD64": "x86_64"}.get(platform.machine(), platform.machine())
    records = []
    for candidate in candidates:
        name, version = candidate["name"], candidate["version"]
        metadata = json.loads(download(f"https://pypi.org/pypi/{name}/{version}/json", 4 * 1024 * 1024))
        wheels = sorted((item for item in metadata["urls"] if item["filename"].endswith(".whl") and not item.get("yanked")), key=lambda item: item["filename"])
        wheels = [item for item in wheels if item["filename"].endswith("-none-any.whl") != bool(candidate.get("native_example"))]
        if not wheels:
            raise ValueError(f"{name}=={version}: no candidate wheel of the requested kind")
        artifact = wheels[0]
        wheel = download(artifact["url"])
        wheel_hash = digest(wheel)
        if wheel_hash != artifact["digests"]["sha256"]:
            raise ValueError(f"{name}: PyPI wheel hash mismatch")
        for source_runtime in runtimes:
            runtime_hash = digest(source_runtime.read_bytes())
            record = {"name": name, "version": version, "wheel_filename": artifact["filename"], "wheel_sha256": wheel_hash, "source_url": artifact["url"], "runtime_sha256": runtime_hash, "target": target, "syntax_checker": "--check-syntax (module mode, no execution)", "status": "unverified", "evidence": "No reviewed behavior hook executed."}
            if candidate.get("native_example"):
                record.update(status="incompatible", evidence=f"The exact wheel {artifact['filename']} requires a CPython ABI/native platform; Kipferl accepts only pure Python wheels. This is an artifact constraint, not a claim about every release or implementation of this package.")
            else:
                with tempfile.TemporaryDirectory(prefix="kipferl-catalog-") as temporary:
                    temp = Path(temporary).resolve()
                    root = temp / "wheel"
                    root.mkdir()
                    runtime = temp / "runtime"
                    shutil.copyfile(source_runtime, runtime)
                    runtime.chmod(0o700)
                    sources = unpack(wheel, root)
                    failures = []
                    for source in sources:
                        relative = source.relative_to(root).as_posix()
                        try:
                            result = check_syntax(runtime, source, temp)
                            if result.returncode:
                                failures.append({"file": relative, "diagnostic": diagnostic(result, temp)})
                        except subprocess.TimeoutExpired:
                            failures.append({"file": relative, "diagnostic": "Compiler exceeded the 15-second limit; compatibility is unverified."})
                    record["source_files_checked"] = len(sources)
                    if failures:
                        record.update(status="incompatible" if any("SyntaxError:" in item["diagnostic"] for item in failures) else "unverified", evidence=f"{len(failures)} of {len(sources)} Python sources failed runtime compilation. First finding: {failures[0]['file']}\n{failures[0]['diagnostic']}", compile_failures=failures)
                    elif candidate.get("smoke") and execute_reviewed:
                        hook = DIRECTORY / "smoke" / candidate["smoke"]
                        if hook.parent != DIRECTORY / "smoke":
                            raise ValueError("invalid smoke hook path")
                        result = run(runtime, hook.read_text(), root, sandbox=True)
                        record["smoke"] = {"file": hook.name, "sha256": digest(hook.read_bytes()), "scope": candidate["scope"], "runner": "macOS sandbox-exec; no network/writes/home reads"}
                        record.update(status="tested" if result.returncode == 0 else "incompatible", evidence=("All Python sources compile; reviewed sandboxed smoke passed. " + candidate["scope"]) if result.returncode == 0 else diagnostic(result, temp))
            records.append(record)
            print(f"{name}=={version} [{runtime_hash[:12]} {target}]: {record['status']}", flush=True)
    # A caller may pass a fresh runtime that equals the embedded one.
    records = list({tuple(item[field] for field in ("name", "version", "wheel_sha256", "runtime_sha256", "target")): item for item in records}.values())
    result = {"schema_version": 1, "records": records}
    validate(result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate checked-in evidence without network or package execution")
    parser.add_argument("--runtime", action="append", type=Path, help="runtime binary to test; repeat for embedded and source runtimes")
    parser.add_argument("--execute-reviewed", action="store_true", help="execute checked-in reviewed smoke hooks in the documented macOS sandbox")
    args = parser.parse_args()
    if args.check:
        if args.runtime or args.execute_reviewed:
            parser.error("--check cannot be combined with refresh options")
        value = json.loads(CATALOG.read_text())
        validate(value)
        print(f"Validated {len(value['records'])} exact package compatibility records")
        return 0
    if not args.runtime:
        parser.error("refresh requires at least one --runtime; use --check for offline validation")
    value = refresh([runtime.resolve(strict=True) for runtime in args.runtime], args.execute_reviewed)
    CATALOG.write_text(json.dumps(value, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
