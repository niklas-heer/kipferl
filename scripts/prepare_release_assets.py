#!/usr/bin/env python3
"""Verify a complete release component set before replacing embedded assets."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from package_catalog import validate
import package_popularity_audit as audit
from release_popularity_audit import validate_release_report, DIRECTORY

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


def prepare(source, destination, target, catalog_output, audit_output, syntax_catalog_output, audit_csv_output,
            snapshot_path=DIRECTORY / "popularity.json", pins_path=DIRECTORY / "popularity-metadata.json"):
    if target not in TARGETS:
        raise ValueError('unsupported release target')
    binaries = []
    selected = None
    selected_audit = selected_syntax = selected_csv = None
    snapshot_bytes = snapshot_path.read_bytes()
    pins = json.loads(pins_path.read_text())
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
        report_path, _ = checked_file(source, f'popularity-audit-{suffix}.json')
        syntax_path, _ = checked_file(source, f'popularity-catalog-{suffix}.json')
        csv_path, _ = checked_file(source, f'popularity-audit-{suffix}.csv')
        report = json.loads(report_path.read_text())
        validate_release_report(report, hashes['pocketpy-kipferl'], suffix, snapshot_bytes, pins)
        if json.loads(syntax_path.read_text()) != audit.catalog_export(report):
            raise ValueError(f'release syntax catalog differs from exact runtime audit: {suffix}')
        if csv_path.read_bytes() != audit.csv_export(report).encode():
            raise ValueError(f'release audit CSV differs from exact runtime audit: {suffix}')
        if suffix == target:
            selected = catalog
            selected_audit, selected_syntax, selected_csv = report_path, syntax_path, csv_path
    destination.mkdir(parents=True, exist_ok=True)
    for path in binaries:
        output = destination / path.name
        shutil.copyfile(path, output)
        output.chmod(0o755)
    catalog_output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(selected, catalog_output)
    for source_path, output_path in ((selected_audit, audit_output), (selected_syntax, syntax_catalog_output), (selected_csv, audit_csv_output)):
        output_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_path, output_path)
    print(f'Verified {len(binaries)} components, four reviewed catalogs and four 1,000-package audits; embedded {target} evidence')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--destination', type=Path, required=True)
    parser.add_argument('--target', choices=TARGETS, required=True)
    parser.add_argument('--catalog-output', type=Path, required=True)
    parser.add_argument('--audit-output', type=Path, required=True)
    parser.add_argument('--syntax-catalog-output', type=Path, required=True)
    parser.add_argument('--audit-csv-output', type=Path, required=True)
    args = parser.parse_args()
    prepare(args.input, args.destination, args.target, args.catalog_output, args.audit_output, args.syntax_catalog_output, args.audit_csv_output)


if __name__ == '__main__':
    main()
