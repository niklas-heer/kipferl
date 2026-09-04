#!/usr/bin/env python3
"""Check the shared development tool pins and required native compiler."""
import json
from pathlib import Path
import shlex
import subprocess
import sys
import os
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def main():
    config = tomllib.loads((ROOT / "mise.toml").read_text())
    tools = config["tools"]
    rust = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]
    if any(tools["rust"][key] != rust[key if key != "version" else "channel"]
           for key in ("version", "profile", "components")):
        raise RuntimeError("Keep mise.toml Rust settings aligned with rust-toolchain.toml")
    package = json.loads((ROOT / "website/package.json").read_text())
    if package.get("packageManager") != "bun@" + tools["bun"]:
        raise RuntimeError("Keep website/package.json packageManager aligned with mise.toml")
    for command, expected in (("rustc", tools["rust"]["version"]),
                              ("python3", tools["python"]),
                              ("node", tools["node"]), ("bun", tools["bun"]),
                              ("bacon", tools["cargo:bacon"]["version"]),
                              ("cargo-nextest", tools["aqua:nextest-rs/nextest/cargo-nextest"]),
                              ("watchexec", tools["watchexec"])):
        version = subprocess.check_output([command, "--version"], text=True).strip()
        actual = version.split()[1] if command not in ("node", "bun") else version.lstrip("v")
        if actual != expected:
            raise RuntimeError(f"{command}: expected {expected}, got {version}; run mise install")
        print(version.splitlines()[0])
    # Check actual compilation, not merely the presence of a compiler shim.
    with tempfile.TemporaryDirectory(prefix="kipferl-setup-") as directory:
        source = Path(directory) / "check.c"
        source.write_text("int main(void) { return 0; }\n")
        compiler = shlex.split(os.environ.get("CC", "cc"))
        subprocess.run([*compiler, str(source), "-o", str(Path(directory) / "check")], check=True)
    print("Tool pins match; native C compiler works.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        print(f"Setup error: {error}", file=sys.stderr)
        print("Use mise run setup. Install Xcode Command Line Tools on macOS or build-essential on Debian/Ubuntu.", file=sys.stderr)
        sys.exit(1)
