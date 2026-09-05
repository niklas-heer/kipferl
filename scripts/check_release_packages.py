#!/usr/bin/env python3
"""Exercise exact release binaries with reviewed tzdata evidence and isolated caches.

Only an actual invocation downloads packages. Unit tests use local subprocesses
and mocks. ``required`` denies network access during offline/standalone checks;
``cli`` explicitly proves only the CLI's locked offline restoration contract.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
DIAGNOSTIC_LIMIT = 64 * 1024
CATALOG_LIMIT = 8 * 1024 * 1024
SUCCESS = "release package smoke passed"
ANSI = re.compile(r"\x1b\[[0-9;]*m")
TARGETS = ("macos-aarch64", "macos-x86_64", "linux-aarch64", "linux-x86_64")
APP = '''import os
import tzdata
assert tzdata.__version__ == "2025.2"
assert tzdata.IANA_VERSION == "2025b"
for name in ["UTC", "Europe/Berlin", "America/New_York", "Asia/Tokyo"]:
    with open(os.path.join(os.path.dirname(tzdata.__file__), "zoneinfo", name), "rb") as resource:
        assert resource.read(4) == b"TZif"
print("release package smoke passed")
'''


class SmokeFailure(RuntimeError):
    """An actionable, bounded release validation failure."""


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def environment(root: Path) -> dict[str, str]:
    """Never inherit user package paths, cache locations, proxies or credentials."""
    paths = {"HOME": "home", "TMPDIR": "tmp", "XDG_CACHE_HOME": "xdg-cache",
             "XDG_CONFIG_HOME": "xdg-config", "KIPFERL_CACHE_DIR": "runtime-cache"}
    result = {"PATH": os.defpath, "LANG": "C", "LC_ALL": "C", "NO_COLOR": "1"}
    for key, directory in paths.items():
        path = root / directory
        path.mkdir(parents=True, exist_ok=True)
        result[key] = str(path)
    return result


def command(arguments: list[str], *, cwd: Path, env: dict[str, str], timeout: float = 120,
            limit: int = DIAGNOSTIC_LIMIT, success: bool = True) -> dict:
    """Drain both pipes while retaining bounded diagnostics; kill the process group on timeout."""
    started = time.monotonic()
    retained = [bytearray(), bytearray()]
    lengths = [0, 0]
    with subprocess.Popen(arguments, cwd=cwd, env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, start_new_session=True) as process:
        def drain(stream, index):
            try:
                while chunk := stream.read(65536):
                    lengths[index] += len(chunk)
                    retained[index].extend(chunk[:max(0, limit - len(retained[index]))])
            finally:
                stream.close()
        readers = [threading.Thread(target=drain, args=(stream, index), daemon=True)
                   for index, stream in enumerate((process.stdout, process.stderr))]
        for reader in readers:
            reader.start()
        timed_out = False
        try:
            process.wait(timeout=timeout)
            # Child processes must not keep inherited pipes open indefinitely.
            for reader in readers:
                reader.join(timeout=max(0, timeout - (time.monotonic() - started)))
            timed_out = any(reader.is_alive() for reader in readers)
        except subprocess.TimeoutExpired:
            timed_out = True
        finally:
            if timed_out:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
                for reader in readers:
                    reader.join(timeout=2)
        stdout, stderr = (bytes(item).decode("utf-8", "replace") for item in retained)
        if timed_out or (success and process.returncode != 0):
            reason = f"timed out after {timeout}s" if timed_out else f"exited {process.returncode}"
            raise SmokeFailure(f"{Path(arguments[0]).name} {reason}\n"
                               f"stdout: {stdout[:DIAGNOSTIC_LIMIT]}\nstderr: {stderr[:DIAGNOSTIC_LIMIT]}")
        if success and any(length > limit for length in lengths):
            raise SmokeFailure(f"{Path(arguments[0]).name} exceeded its {limit}-byte output limit")
        return {"stdout": stdout, "stderr": stderr, "returncode": process.returncode,
                "seconds": round(time.monotonic() - started, 3),
                "output_truncated": any(length > limit for length in lengths)}


def offline_prefix(mode: str, *, cwd: Path, env: dict[str, str]) -> list[str]:
    if mode == "cli":
        return []
    sandbox = Path("/usr/bin/sandbox-exec")
    if platform.system() != "Darwin" or not sandbox.is_file():
        raise SmokeFailure("required offline isolation needs macOS sandbox-exec; "
                           "--offline-isolation cli explicitly tests only the CLI offline flag")
    prefix = [str(sandbox), "-p", "(version 1)(allow default)(deny network*)"]
    command([*prefix, "/usr/bin/true"], cwd=cwd, env=env, timeout=10)
    return prefix


def validate_versions(cli_text: str, runtime_text: str, version: str) -> None:
    if ANSI.sub("", cli_text).strip() != f"Kipferl v{version}":
        raise SmokeFailure(f"CLI version does not match VERSION ({version})")
    if runtime_text.strip() != f"Kipferl runtime {version}":
        raise SmokeFailure(f"runtime version does not match VERSION ({version})")


def tested_record(catalog: dict, runtime_sha256: str, target: str) -> dict:
    records = [record for record in catalog.get("records", [])
               if record.get("name") == "tzdata" and record.get("version") == "2025.2"
               and record.get("status") == "tested" and record.get("target") == target
               and record.get("runtime_sha256") == runtime_sha256]
    if len(records) != 1:
        raise SmokeFailure("embedded catalog must contain one tested tzdata==2025.2 record "
                           "for the supplied runtime hash and target")
    record = records[0]
    smoke = record.get("smoke", {})
    if (smoke.get("file") != "tzdata.py"
            or smoke.get("sha256") != digest(ROOT / "compatibility/packages/smoke/tzdata.py")
            or not re.fullmatch(r"[0-9a-f]{64}", record.get("wheel_sha256", ""))):
        raise SmokeFailure("tested tzdata record does not identify the reviewed smoke and wheel")
    return record


def validate_lock(lock: dict, record: dict, runtime_sha256: str, target: str) -> None:
    packages = lock.get("packages", [])
    if (lock.get("schema") != 1 or lock.get("runtime_sha256") != runtime_sha256
            or lock.get("target") != target or lock.get("allow_unverified") is not False
            or lock.get("requirements") != ["tzdata==2025.2"] or len(packages) != 1):
        raise SmokeFailure("lock does not preserve the exact reviewed runtime/target/requirement")
    package = packages[0]
    for field, expected in {"name": "tzdata", "version": "2025.2",
                            "sha256": record["wheel_sha256"],
                            "filename": record["wheel_filename"]}.items():
        if package.get(field) != expected:
            raise SmokeFailure(f"locked package {field} differs from reviewed artifact")


def host_target() -> str:
    system = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    architecture = {"arm64": "aarch64", "aarch64": "aarch64", "x86_64": "x86_64",
                    "AMD64": "x86_64"}.get(platform.machine())
    return f"{system}-{architecture}"


def run_smoke(cli: Path, runtime: Path, target: str, mode: str) -> dict:
    if target != host_target():
        raise SmokeFailure(f"target {target} must match this execution host ({host_target()})")
    for binary in (cli, runtime):
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise SmokeFailure(f"not an executable release binary: {binary}")
    version = (ROOT / "VERSION").read_text().strip()
    runtime_hash = digest(runtime)
    evidence = {"schema_version": 1, "status": "passed", "version": version, "target": target,
                "cli_sha256": digest(cli), "runtime_sha256": runtime_hash,
                "offline_isolation": "os-network-denied" if mode == "required" else "cli-offline-flag-only",
                "steps": [], "scope": "tzdata 2025.2/2025b version constants and four TZif headers; no timezone calculations"}
    with tempfile.TemporaryDirectory(prefix="kipferl-release-packages-") as temporary:
        root = Path(temporary).resolve()
        work = root / "work"
        env = environment(work)
        project = work / "project"
        project.mkdir()
        prefix = offline_prefix(mode, cwd=project, env=env)

        def run(label: str, args: list[str], *, offline=False, cwd=project, environ=env,
                limit=DIAGNOSTIC_LIMIT, success=True) -> dict:
            result = command([*(prefix if offline else []), *args], cwd=cwd, env=environ,
                             limit=limit, success=success)
            evidence["steps"].append({"name": label, "seconds": result["seconds"],
                                      "returncode": result["returncode"]})
            return result

        cli_version = run("cli-version", [str(cli), "--version"])["stdout"]
        runtime_version = run("runtime-version", [str(runtime), "--version"])["stdout"]
        validate_versions(cli_version, runtime_version, version)
        catalog = json.loads(run("embedded-catalog", [str(cli), "deps", "catalog", "--json"],
                                 limit=CATALOG_LIMIT)["stdout"])
        record = tested_record(catalog, runtime_hash, target)
        evidence["wheel_sha256"] = record["wheel_sha256"]
        evidence["reviewed_smoke_sha256"] = record["smoke"]["sha256"]
        (project / "kipferl.json").write_text(json.dumps({"entry": "app.py"}) + "\n")
        (project / "app.py").write_text(APP)
        run("online-reviewed-add", [str(cli), "add", "tzdata==2025.2"])
        lock_path = project / "kipferl.lock"
        validate_lock(json.loads(lock_path.read_text()), record, runtime_hash, target)
        evidence["lock_sha256"] = digest(lock_path)
        installed = project / ".kipferl/packages"
        wheel = project / ".kipferl/cache" / (record["wheel_sha256"] + ".whl")
        if not wheel.is_file() or digest(wheel) != record["wheel_sha256"]:
            raise SmokeFailure("online add did not create the exact isolated cached wheel")
        run("reviewed-runtime-resources", [str(runtime), str(ROOT / "compatibility/packages/smoke/tzdata.py")],
            offline=True, cwd=installed)
        run("installed-integrity", [str(cli), "deps", "check"], offline=True)
        if run("installed-cli-resources", [str(cli), "run"], offline=True)["stdout"].strip() != SUCCESS:
            raise SmokeFailure("installed CLI resource smoke did not report success")
        shutil.rmtree(installed)
        withheld = work / "withheld.whl"
        wheel.rename(withheld)
        missing = run("offline-missing-cache-rejected", [str(cli), "sync", "--locked", "--offline"],
                      offline=True, success=False)
        if (missing["returncode"] == 0 or installed.exists()
                or "wheel is missing from the offline cache" not in missing["stdout"] + missing["stderr"]):
            raise SmokeFailure("offline sync did not report the expected missing-cache failure")
        withheld.rename(wheel)
        run("offline-locked-cache-restoration", [str(cli), "sync", "--locked", "--offline"], offline=True)
        if digest(lock_path) != evidence["lock_sha256"]:
            raise SmokeFailure("offline restoration changed the lock")
        run("restored-integrity", [str(cli), "deps", "check"], offline=True)
        if run("restored-cli-resources", [str(cli), "run"], offline=True)["stdout"].strip() != SUCCESS:
            raise SmokeFailure("offline restored resource smoke did not report success")
        run("universal-build", [str(cli), "build", "--mode", "universal", "-o", "program"], offline=True)
        detached = root / "detached"
        detached.mkdir()
        executable = detached / "program"
        shutil.copy2(project / "program", executable)
        evidence["standalone_sha256"] = digest(executable)
        shutil.rmtree(work)  # Removes source, installation, wheel cache, HOME and runtime cache.
        detached_env = environment(detached / "environment")
        if run("detached-standalone-resources", [str(executable)], offline=True, cwd=detached,
               environ=detached_env)["stdout"].strip() != SUCCESS:
            raise SmokeFailure("standalone resource smoke failed after deleting source and caches")
    evidence["completed_at"] = datetime.now(timezone.utc).isoformat()
    return evidence


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--offline-isolation", choices=("required", "cli"), default="required")
    args = parser.parse_args()
    try:
        evidence = run_smoke(args.cli.resolve(), args.runtime.resolve(), args.target, args.offline_isolation)
    except (SmokeFailure, OSError, ValueError) as error:
        evidence = {"schema_version": 1, "status": "failed", "target": args.target,
                    "error": str(error)[:2 * DIAGNOSTIC_LIMIT]}
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(evidence, indent=2) + "\n")
    if evidence["status"] != "passed":
        print(evidence["error"], file=sys.stderr)
        return 1
    print(f"Release package smoke passed for {args.target}; offline isolation: {evidence['offline_isolation']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
