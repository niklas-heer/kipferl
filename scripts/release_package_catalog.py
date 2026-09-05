#!/usr/bin/env python3
"""Generate exact release-runtime evidence from the existing reviewed wheel pins.

Linux behavior execution requires explicit --disposable-ci on a GitHub Actions
runner. This is a disposable-CI trust decision, not an OS sandbox. Local macOS
execution retains sandbox-exec. Only the hardcoded reviewed tzdata hook runs.
"""
from __future__ import annotations

import argparse
import copy
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

import package_catalog as catalog

TZDATA_VERSION = "2025.2"
TZDATA_WHEEL = "1a403fada01ff9221ca8044d701868fa132215d84beb92242d9acd2147f667a8"
TZDATA_SMOKE = "0b7cbdbc4fc2d0b1ebcd24db137cf25655261b8ad61b9b736ee5facdfd9f89c6"
KEY_FIELDS = ("name", "version", "wheel_sha256", "runtime_sha256", "target")


def host_target() -> str:
    system = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    arch = {"arm64": "aarch64", "aarch64": "aarch64", "x86_64": "x86_64", "AMD64": "x86_64"}.get(platform.machine())
    if not system or not arch:
        raise ValueError("unsupported release host")
    return f"{system}-{arch}"


def binary_target(data: bytes) -> str:
    """Inspect native executable headers; filenames are not target evidence."""
    if data[:4] == b"\x7fELF" and len(data) >= 64 and data[4:7] == b"\x02\x01\x01":
        arch = {62: "x86_64", 183: "aarch64"}.get(int.from_bytes(data[18:20], "little"))
        if arch:
            return f"linux-{arch}"
    if data[:4] == b"\xcf\xfa\xed\xfe" and len(data) >= 32:
        arch = {0x1000007: "x86_64", 0x100000C: "aarch64"}.get(int.from_bytes(data[4:8], "little"))
        if arch:
            return f"macos-{arch}"
    raise ValueError("runtime is not a supported native ELF or Mach-O executable")


def execution_mode(target: str, disposable_ci: bool) -> tuple[bool, str]:
    if target.startswith("macos-"):
        if not shutil.which("sandbox-exec"):
            raise ValueError("macOS release evidence requires sandbox-exec")
        return True, "macOS sandbox-exec; no network/writes/home reads"
    if target.startswith("linux-") and disposable_ci and os.environ.get("GITHUB_ACTIONS") == "true":
        return False, "Explicit disposable GitHub Actions runner; sanitized environment, no OS sandbox"
    raise ValueError("Linux reviewed execution requires --disposable-ci and GITHUB_ACTIONS=true")


def reviewed_pins(existing: dict, candidates: list[dict]) -> list[tuple[dict, dict]]:
    """Refuse pin drift or candidate additions without historical exact evidence."""
    result = []
    seen = set()
    for candidate in candidates:
        key = candidate["name"], candidate["version"]
        if key in seen:
            raise ValueError("duplicate reviewed candidate")
        seen.add(key)
        matches = [r for r in existing["records"] if (r["name"], r["version"]) == key]
        pins = {(r["wheel_filename"], r["wheel_sha256"], r.get("source_url")) for r in matches}
        if len(pins) != 1 or not matches[0].get("source_url"):
            raise ValueError(f"{key}: missing or ambiguous historical wheel pin")
        if candidate.get("smoke"):
            if key != ("tzdata", TZDATA_VERSION) or candidate["smoke"] != "tzdata.py":
                raise ValueError("release execution only permits the reviewed tzdata hook")
            if matches[0]["wheel_sha256"] != TZDATA_WHEEL:
                raise ValueError("reviewed tzdata wheel pin changed")
            reviewed = [r for r in matches if r["status"] == "tested" and r.get("smoke", {}).get("sha256") == TZDATA_SMOKE]
            if not reviewed or any(r["smoke"]["scope"] != candidate.get("scope") for r in reviewed):
                raise ValueError("reviewed tzdata behavior scope is missing or changed")
        result.append((candidate, matches[0]))
    if ("tzdata", TZDATA_VERSION) not in seen or not any(c.get("smoke") for c, _ in result):
        raise ValueError("expected tested tzdata candidate is absent")
    return result


def generate(runtime_path: Path, target: str, disposable_ci: bool = False) -> dict:
    runtime_bytes = runtime_path.read_bytes()
    if target != host_target() or target != binary_target(runtime_bytes):
        raise ValueError("runtime/target/host mismatch; run evidence on the native release runner")
    sandbox, runner = execution_mode(target, disposable_ci)
    existing = json.loads(catalog.CATALOG.read_text())
    catalog.validate(existing)
    candidates = json.loads((catalog.DIRECTORY / "candidates.json").read_text())["packages"]
    pins = reviewed_pins(existing, candidates)
    hook = catalog.DIRECTORY / "smoke/tzdata.py"
    hook_bytes = hook.read_bytes()
    if catalog.digest(hook_bytes) != TZDATA_SMOKE:
        raise ValueError("reviewed tzdata smoke hash changed")
    runtime_hash = catalog.digest(runtime_bytes)
    fresh = []
    with tempfile.TemporaryDirectory(prefix="kipferl-release-catalog-") as temporary:
        temp = Path(temporary).resolve()
        runtime = temp / "runtime"
        runtime.write_bytes(runtime_bytes)
        runtime.chmod(0o700)
        catalog.verify_syntax_checker(runtime)
        for candidate, pin in pins:
            wheel = catalog.download(pin["source_url"])
            if catalog.digest(wheel) != pin["wheel_sha256"]:
                raise ValueError(f"{pin['name']}: pinned wheel hash mismatch")
            record = {key: pin[key] for key in ("name", "version", "wheel_filename", "wheel_sha256", "source_url")}
            record.update(runtime_sha256=runtime_hash, target=target, syntax_checker="--check-syntax (module mode, no execution)", status="unverified", evidence="No reviewed behavior hook executed.")
            if candidate.get("native_example"):
                if record["wheel_filename"].endswith("-none-any.whl"):
                    raise ValueError("native candidate unexpectedly has a pure-wheel pin")
                record.update(status="incompatible", evidence=f"The exact wheel {record['wheel_filename']} requires a CPython ABI/native platform; Kipferl accepts only pure Python wheels. Other artifacts and releases were not evaluated.")
            else:
                if not record["wheel_filename"].endswith("-none-any.whl"):
                    raise ValueError("reviewed pure candidate has a native wheel pin")
                root = temp / candidate["name"]
                root.mkdir()
                sources = catalog.unpack(wheel, root)
                failures = []
                for source in sources:
                    try:
                        result = catalog.check_syntax(runtime, source, temp)
                        if result.returncode:
                            failures.append({"file": source.relative_to(root).as_posix(), "diagnostic": catalog.diagnostic(result, temp)})
                    except subprocess.TimeoutExpired:
                        failures.append({"file": source.relative_to(root).as_posix(), "diagnostic": "Compiler exceeded the 15-second limit; compatibility is unverified."})
                record["source_files_checked"] = len(sources)
                if failures:
                    record.update(status="incompatible" if any("SyntaxError:" in f["diagnostic"] for f in failures) else "unverified", compile_failures=failures, evidence=f"{len(failures)} of {len(sources)} Python sources failed compilation. First finding: {failures[0]['file']}\n{failures[0]['diagnostic']}")
                elif candidate.get("smoke"):
                    result = catalog.run(runtime, hook_bytes.decode("utf-8"), root, sandbox=sandbox)
                    if result.returncode:
                        raise ValueError("expected tested tzdata smoke failed: " + catalog.diagnostic(result, temp))
                    record.update(status="tested", evidence="All Python sources compile; reviewed smoke passed. " + candidate["scope"], smoke={"file": "tzdata.py", "sha256": TZDATA_SMOKE, "scope": candidate["scope"], "runner": runner})
            fresh.append(record)
            print(f"{record['name']}=={record['version']} [{runtime_hash[:12]} {target}]: {record['status']}", flush=True)
    if not any(r["name"] == "tzdata" and r["status"] == "tested" and r["wheel_sha256"] == TZDATA_WHEEL for r in fresh):
        raise ValueError("expected tested tzdata evidence is absent; release catalog was not written")
    merged = {tuple(r[k] for k in KEY_FIELDS): r for r in copy.deepcopy(existing["records"])}
    merged.update({tuple(r[k] for k in KEY_FIELDS): r for r in fresh})
    output = {**existing, "records": list(merged.values())}
    catalog.validate(output)
    return output


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--disposable-ci", action="store_true", help="explicitly permit the pinned reviewed hook on a disposable Linux GitHub Actions runner")
    args = parser.parse_args()
    value = generate(args.runtime.resolve(strict=True), args.target, args.disposable_ci)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", dir=args.output.parent, prefix=args.output.name + ".", delete=False) as output:
        temporary = Path(output.name)
        json.dump(value, output, indent=2)
        output.write("\n")
    try:
        temporary.replace(args.output)
    finally:
        temporary.unlink(missing_ok=True)
    print(f"Wrote {len(value['records'])} exact evidence records to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
