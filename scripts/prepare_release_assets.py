#!/usr/bin/env python3
"""Verify a complete release component set before replacing embedded assets."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from package_catalog import validate

TARGETS = ('macos-aarch64', 'macos-x86_64', 'linux-aarch64', 'linux-x86_64')
COMPONENTS = ('pocketpy-kipferl', 'pocketpy-kipferl-core', 'kipferl-loader')


def checked_file(directory, name):
    path = directory / name
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'missing or nonregular release artifact: {name}')
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    expected = f'{digest}  {name}\n'
    if not data or (directory / (name + '.sha256')).read_text() != expected:
        raise ValueError(f'invalid release artifact checksum: {name}')
    return path, digest


def prepare(source, destination, target, catalog_output):
    if target not in TARGETS:
        raise ValueError('unsupported release target')
    binaries = []
    selected = None
    # Validate the entire set before touching the checked-in fallback assets.
    for suffix in TARGETS:
        hashes = {}
        for component in COMPONENTS:
            path, digest = checked_file(source, f'{component}-{suffix}')
            binaries.append(path)
            hashes[component] = digest
        catalog, _ = checked_file(source, f'package-catalog-{suffix}.json')
        value = json.loads(catalog.read_text())
        validate(value)
        if not any(row['name'] == 'tzdata' and row['version'] == '2025.2'
                   and row['target'] == suffix and row['status'] == 'tested'
                   and row['runtime_sha256'] == hashes['pocketpy-kipferl']
                   for row in value['records']):
            raise ValueError(f'catalog lacks tested evidence for release runtime: {suffix}')
        if suffix == target:
            selected = catalog
    destination.mkdir(parents=True, exist_ok=True)
    for path in binaries:
        output = destination / path.name
        shutil.copyfile(path, output)
        output.chmod(0o755)
    catalog_output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(selected, catalog_output)
    print(f'Verified all {len(binaries)} components and four catalogs; embedded {target} evidence')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--destination', type=Path, required=True)
    parser.add_argument('--target', choices=TARGETS, required=True)
    parser.add_argument('--catalog-output', type=Path, required=True)
    args = parser.parse_args()
    prepare(args.input, args.destination, args.target, args.catalog_output)


if __name__ == '__main__':
    main()
