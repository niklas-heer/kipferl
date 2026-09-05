#!/usr/bin/env python3
"""Build and verify this host's runtime assets before embedding them in the CLI."""
from pathlib import Path
import os
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(*arguments: str) -> None:
    subprocess.run(arguments, cwd=ROOT, check=True)


def publish(source: Path, destination: Path) -> None:
    if destination.is_file() and destination.read_bytes() == source.read_bytes():
        return
    with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
        staged = Path(temporary.name)
    try:
        shutil.copyfile(source, staged)
        staged.chmod(source.stat().st_mode & 0o777)
        staged.replace(destination)
    finally:
        staged.unlink(missing_ok=True)


def main() -> None:
    operating_system = {"Darwin": "macos", "Linux": "linux"}.get(platform.system())
    architecture = {"arm64": "aarch64", "aarch64": "aarch64", "x86_64": "x86_64", "AMD64": "x86_64"}.get(platform.machine())
    if not operating_system or not architecture:
        raise SystemExit("Host asset refresh supports macOS/Linux on ARM64 or x86-64")
    suffix = f"{operating_system}-{architecture}"
    rustc = subprocess.run(["rustc", "-vV"], cwd=ROOT, check=True, capture_output=True, text=True).stdout
    host = next((line.removeprefix("host: ") for line in rustc.splitlines() if line.startswith("host: ")), None)
    if not host or "/" in host or "\\" in host:
        raise SystemExit("rustc did not report a valid host target")
    # Explicit targets prevent Cargo configuration from redirecting the build
    # while stale host output is accidentally published. Separate output
    # directories preserve the full runtime during core builds.
    full = ROOT / "target"
    core = ROOT / "target/runtime-core"
    run("cargo", "build", "--locked", "--release", "--target", host, "--target-dir", str(full), "-p", "kipferl-runtime", "-p", "kipferl-loader")
    run("cargo", "build", "--locked", "--release", "--target", host, "--target-dir", str(core), "-p", "kipferl-runtime", "--no-default-features")
    full_binary = full / host / "release"
    core_binary = core / host / "release"
    with tempfile.TemporaryDirectory(prefix="kipferl-asset-probe-") as temporary:
        probe = Path(temporary) / "compile_only.py"
        probe.write_text("value = 0\ndef update():\n    global value\n    value = 1\nraise RuntimeError('syntax checking must not execute this file')\n")
        for directory in (full_binary, core_binary):
            subprocess.run([str(directory / "pocketpy-kipferl"), "--check-syntax", "--", str(probe)], cwd=temporary, env={"PATH": os.defpath}, check=True, timeout=10)
    assets = ROOT / "crates/kipferl-cli/assets"
    publish(full_binary / "pocketpy-kipferl", assets / f"pocketpy-kipferl-{suffix}")
    publish(core_binary / "pocketpy-kipferl", assets / f"pocketpy-kipferl-core-{suffix}")
    publish(full_binary / "kipferl-loader", assets / f"kipferl-loader-{suffix}")
    print(f"Verified and refreshed {suffix} runtime and loader assets.")


if __name__ == "__main__":
    main()
