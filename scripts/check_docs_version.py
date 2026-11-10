#!/usr/bin/env python3
"""Reject documentation that offers an older stable release than VERSION.

The README and website name the stable version by hand in prose, install
commands, and download links. Release preparation changes VERSION, so this
check makes a stable VERSION fail until the documentation offers it too:

- the website's shared `stableVersion` constant must equal VERSION;
- installation entry points must link to the VERSION release, and no current
  document may link to a newer one (older releases stay linkable as history);
- "`kipferl --version` reports" claims and printed version lines must equal VERSION.

A release-candidate VERSION leaves the stable documentation alone, so it is
skipped. Release articles under website/src/app/blog are history and are not
checked.
"""
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
CONSTANT = "website/src/lib/release.ts"
ENTRY_POINTS = ("README.md", "website/content/docs/getting-started/installation.mdx")
CURRENT_DOCS = ("README.md", "compatibility/packages/README.md", "website/content/docs/**/*.mdx")
STABLE = r"(\d+\.\d+\.\d+)(?![-.\w])"
CONSTANT_PATTERN = re.compile(r'export const stableVersion = "([^"]+)";')
RELEASE_LINK = re.compile(r"/releases/(?:tag|download)/v" + STABLE)
VERSION_CLAIMS = (
    re.compile(r"reports\s+`v" + STABLE + "`"),
    re.compile(r"^Kipferl v" + STABLE + r"\s*$", re.MULTILINE),
)


def parse(version):
    return tuple(int(part) for part in version.split("."))


def current_docs(root):
    paths = []
    for pattern in CURRENT_DOCS:
        paths.extend(sorted(root.glob(pattern)))
    return paths


def check(root):
    version = (root / "VERSION").read_text().strip()
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        return []
    findings = []
    match = CONSTANT_PATTERN.search((root / CONSTANT).read_text())
    if match is None:
        findings.append(f"{CONSTANT}: missing stableVersion constant")
    elif match.group(1) != version:
        findings.append(f"{CONSTANT}: stableVersion is {match.group(1)}, VERSION is {version}")
    for path in current_docs(root):
        name = path.relative_to(root).as_posix()
        text = path.read_text()
        linked = [found.group(1) for found in RELEASE_LINK.finditer(text)]
        for release in linked:
            if parse(release) > parse(version):
                findings.append(f"{name}: links unreleased v{release}, VERSION is {version}")
        if name in ENTRY_POINTS and version not in linked:
            findings.append(f"{name}: does not link the v{version} release")
        for pattern in VERSION_CLAIMS:
            for found in pattern.finditer(text):
                if found.group(1) != version:
                    line = text.count("\n", 0, found.start()) + 1
                    findings.append(f"{name}:{line}: claims v{found.group(1)}, VERSION is {version}")
    return findings


def main():
    findings = check(ROOT)
    for finding in findings:
        print(finding, file=sys.stderr)
    if findings:
        print("Update the stable version in the documentation to match VERSION.", file=sys.stderr)
        return 1
    print("Documentation offers the VERSION release.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
