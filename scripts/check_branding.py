#!/usr/bin/env python3
"""Reject retired product names in tracked paths and UTF-8 source text.

The universal trailer's frozen wire signature is a compatibility contract, not
product branding. Historical release prose and instructions for uninstalling an
old installation also need the original spelling. Those exceptions are exact
lines in exact files below, so adding a new mention to those files still fails.
Binary assets and recordings require build verification and visual review; this
text guard does not claim to recognize lettering drawn in image pixels.
"""
import argparse
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RETIRED_NAME = re.compile(r"m\s*charm|u\s*charm|µ\s*charm|μ\s*charm|kipval", re.IGNORECASE)
# The retired new-project banner spells its name with box-drawing characters.
RETIRED_BANNER = "┌┬┐┌─┐┬ ┬┌─┐┬─┐┌┬┐"
# These two files must spell the forbidden names to define and test the guard.
POLICY_FIXTURES = {
    "scripts/check_branding.py": "Defines forbidden names and reviewed exceptions.",
    "scripts/test_check_branding.py": "Contains deliberately invalid branding fixtures.",
}
ALLOWED_LINES = {
    "crates/kipferl-format/src/lib.rs": {
        'pub const TRAILER_MAGIC: [u8; 8] = *b"MCHARM01";',
        'assert_eq!(TRAILER_MAGIC, *b"MCHARM01");',
    },
    # Published release history: the migration aliases existed in 0.6.
    '.github/release-notes/v0.6.0.md': {
        '# Existing μcharm 0.5 installation:',
        '- Keep legacy `from ucharm ...`, environment variables, download aliases, and',
        '- Rename **μcharm/ucharm** to **Kipferl**, with `kipferl.dev` as the public home',
        'brew uninstall --force ucharm',
        'the `ucharm` command as a deprecated 0.6 transition path',
    },
    # Historical changelog entries describe the original rename and aliases.
    'CHANGELOG.md': {
        '- Accepts legacy `from ucharm ...` source and environment variables, publishes',
        'temporary `ucharm-*` assets, and installs a deprecated `ucharm` command alias',
        'μcharm/ucharm to Kipferl.',
    },
    # Completed 0.6 publication checklist, retained as historical evidence.
    'RELEASE_CHECKLIST.md': {
        'deprecated `ucharm` alias.',
    },
    # Historical project name explains the Rust migration.
    'RUST_MIGRATION.md': {
        'The project formerly known as μcharm migrated its native implementation from',
    },
    # Project history and actionable upgrade/removal instructions.
    'README.md': {
        '### From μcharm to Kipferl',
        'Kipferl is the new name for μcharm beginning with the Rust-based 0.6 release.',
        'The deprecated `ucharm` command alias ended in 0.7.1. Update scripts to invoke',
        'The temporary `ucharm` migration aliases introduced in 0.6 end in 0.7.1.',
        'binary. Before 0.6, the project was called μcharm.',
        'brew uninstall --force ucharm',
        'for dependency-lock changes. Users with the old μcharm 0.5 formula should replace',
    },
    # Explicit old-to-current spelling table for upgrading to 0.7.1.
    'website/content/docs/guides/packages.mdx': {
        '| `UCHARM_*` environment variables | The matching `KIPFERL_*` variables |',
        '| `MCHARM_TEST_KEYS` | `KIPFERL_TEST_KEYS` |',
        '| `from ucharm import ...` | `from kipferl import ...` |',
        '| `ucharm-*` release download filenames | `kipferl-*` filenames from the installation guide |',
        '| `ucharm` command or Homebrew formula | `kipferl` |',
    },
    # Uninstalling the old formula requires its original name.
    'website/content/docs/getting-started/installation.mdx': {
        'If the old μcharm 0.5 Homebrew formula is installed, replace it once.',
        'The deprecated `ucharm` command, imports, environment variables, and release',
        'brew uninstall --force ucharm',
    },
    # Published 0.6 history with current removal notice and upgrade commands.
    'website/src/app/blog/kipferl-0-6/page.tsx': {
        '<code>brew upgrade kipferl</code>. If the old μcharm 0.5 formula is',
        '<code>{`brew uninstall --force ucharm\\nbrew install niklas-heer/tap/kipferl`}</code>',
        'Kipferl 0.6 installed <code>ucharm</code> as a deprecated migration',
        'Kipferl is the new name for μcharm/ucharm. The pastry name gives a',
        'temporary <code>ucharm</code> command, imports, environment',
    },
    # Historical project identity in the migration article.
    'website/src/app/blog/rust-migration/page.tsx': {
        'The project formerly known as μcharm became Kipferl while changing',
        'Zig helped μcharm prove that a small, embedded Python runtime could',
    },
}


def check_paths(root, paths):
    """Return actionable findings; exceptions never exempt a filename."""
    root = Path(root)
    findings = []
    for relative in sorted(set(map(str, paths))):
        if RETIRED_NAME.search(relative):
            findings.append(f"{relative}: retired product name in filename; use kipferl")
        path = root / relative
        if not path.is_file() or relative in POLICY_FIXTURES:
            continue
        data = path.read_bytes()
        if b"\0" in data:
            continue
        try:
            content = data.decode("utf-8")
        except UnicodeDecodeError:
            continue
        for number, line in enumerate(content.splitlines(), 1):
            if not (RETIRED_NAME.search(line) or RETIRED_BANNER in line):
                continue
            if line.strip() in ALLOWED_LINES.get(relative, set()):
                continue
            findings.append(
                f"{relative}:{number}: retired product branding; use Kipferl: {line.strip()}"
            )
    return findings


def tracked_paths(root):
    result = subprocess.run(
        ["git", "ls-files", "-z", "--cached"],
        cwd=root, check=True, stdout=subprocess.PIPE,
    )
    return [name for name in result.stdout.decode("utf-8").split("\0") if name]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    try:
        paths = tracked_paths(args.root)
        findings = check_paths(args.root, paths)
    except (OSError, subprocess.CalledProcessError, UnicodeDecodeError) as error:
        parser.exit(2, f"Branding audit could not complete: {error}\n")
    if findings:
        print("\n".join(findings))
        return 1
    print(f"Branding audit passed for {len(paths)} tracked paths (UTF-8 text checked; media reviewed separately).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
