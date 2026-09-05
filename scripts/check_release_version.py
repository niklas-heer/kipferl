#!/usr/bin/env python3
"""Refuse release builds whose tag, workspace, lock, and public versions differ."""
import os
from pathlib import Path
import re
import tomllib


def check(root, tag=None):
    version = (root / 'VERSION').read_text().strip()
    if not re.fullmatch(r'\d+\.\d+\.\d+(?:-rc\.[1-9]\d*)?', version):
        raise ValueError('invalid stable or release-candidate VERSION')
    manifest = tomllib.loads((root / 'Cargo.toml').read_text())
    lock = tomllib.loads((root / 'Cargo.lock').read_text())
    if manifest['workspace']['package']['version'] != version:
        raise ValueError('workspace version differs from VERSION')
    for package in lock['package']:
        if package['name'].startswith('kipferl-') and 'source' not in package and package['version'] != version:
            raise ValueError(f"lock version differs for {package['name']}")
    if tag is not None and tag != 'v' + version:
        raise ValueError('release tag differs from VERSION')
    return version


if __name__ == '__main__':
    tag = os.environ.get('GITHUB_REF_NAME') if os.environ.get('GITHUB_REF_TYPE') == 'tag' else None
    print('Verified release version:', check(Path(__file__).resolve().parents[1], tag))
