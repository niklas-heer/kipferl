#!/usr/bin/env python3
"""Screen the same 1,000 pinned releases against each exact release runtime.

Only metadata and hash-verified wheel bytes are reusable. Every invocation uses
fresh compiler-result checkpoints; successful compilation is never a behavior
compatibility claim.
"""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import package_popularity_audit as audit

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'compatibility/packages'
TARGETS = ('macos-aarch64', 'macos-x86_64', 'linux-aarch64', 'linux-x86_64')
COUNT = 1000


def validate_pins(pins, snapshot_bytes):
    snapshot = json.loads(snapshot_bytes)
    if (pins.get('schema_version') != 1
            or pins.get('snapshot_sha256') != audit.sha256(snapshot_bytes)
            or not isinstance(pins.get('records'), list)
            or not isinstance(pins.get('missing_projects'), list)
            or len(pins['records']) + len(pins['missing_projects']) != COUNT):
        raise ValueError('release metadata pins must cover the exact 1,000-project snapshot')
    projects = snapshot['projects'][:COUNT]
    names = [audit.normalized(project['name']) for project in projects]
    records = pins['records']
    missing = pins['missing_projects']
    if (len(names) != COUNT or len(set(names)) != COUNT or len(set(missing)) != len(missing)
            or not set(missing).issubset(names)
            or [row.get('name') for row in records] != [name for name in names if name not in missing]):
        raise ValueError('release metadata pin identities and missing projects must match snapshot order')
    for pin in records:
        if (not isinstance(pin.get('version'), str) or not pin['version']
                or not audit.HASH.fullmatch(pin.get('metadata_sha256', ''))
                or pin.get('metadata_url') != f"https://pypi.org/pypi/{pin['name']}/json"
                or not pin.get('metadata_fetched_at')
                or pin.get('artifact_kind') not in ('pure', 'native_only', 'source_only')
                or not isinstance(pin.get('requires_dist'), list)):
            raise ValueError(f"invalid release metadata pin: {pin['name']}")
        artifact = pin.get('artifact')
        if artifact is not None and (
                not audit.official_url(artifact.get('url', ''))
                or not audit.HASH.fullmatch(artifact.get('digests', {}).get('sha256', ''))
                or not isinstance(artifact.get('filename'), str)
                or type(artifact.get('size')) is not int or artifact['size'] < 0):
            raise ValueError(f"invalid release artifact pin: {pin['name']}")
    return snapshot


def validate_release_report(report, runtime_hash, target, snapshot_bytes, pins):
    snapshot = validate_pins(pins, snapshot_bytes)
    audit.validate_report(report)
    if report.get('release_metadata_sha256') != audit.policy_digest(pins):
        raise ValueError('release popularity audit metadata manifest digest does not match')
    if (report.get('complete') is not True or report.get('requested_count') != COUNT
            or report.get('completed_count') != COUNT):
        raise ValueError('release popularity audit must be complete for all 1,000 projects')
    if report.get('runtime_sha256') != runtime_hash or report.get('target') != target:
        raise ValueError('release popularity audit does not match exact runtime and target')
    if (report.get('snapshot_sha256') != audit.sha256(snapshot_bytes)
            or report.get('ranking_source') != snapshot.get('source', {})
            or report.get('audit_policy') != audit.current_policy()):
        raise ValueError('release popularity audit snapshot or policy differs from release inputs')
    if report['counts'].get('network', 0):
        raise ValueError('release popularity audit still contains incomplete network requests')
    by_name = {pin['name']: pin for pin in pins['records']}
    for project, row in zip(snapshot['projects'][:COUNT], report['records'], strict=True):
        if (row['name'] != audit.normalized(project['name']) or row['rank'] != project['rank']
                or row.get('downloads') != project.get('downloads')):
            raise ValueError('release popularity audit rankings differ from the pinned snapshot')
        pin = by_name.get(row['name'])
        if pin is None:
            # Explicitly unpinned projects are fetched afresh under the same limits.
            continue
        for field in ('version', 'metadata_sha256', 'metadata_url', 'metadata_fetched_at'):
            if row.get(field) != pin.get(field):
                raise ValueError(f"release popularity audit changed metadata pin: {pin['name']} {field}")
        artifact = pin.get('artifact') or {}
        for field, expected in (
                ('selected_artifact_filename', artifact.get('filename')),
                ('source_url', artifact.get('url')),
                ('artifact_declared_sha256', artifact.get('digests', {}).get('sha256'))):
            if row.get(field) != expected:
                raise ValueError(f"release popularity audit changed artifact pin: {pin['name']} {field}")


def validate_runtime_header(data, target):
    """Require the thin native format/CPU emitted by this release matrix."""
    system, architecture = target.split('-', 1)
    if system == 'macos':
        expected_cpu = {'aarch64': 0x0100000C, 'x86_64': 0x01000007}[architecture]
        valid = (len(data) >= 32 and data[:4] == b'\xcf\xfa\xed\xfe'
                 and int.from_bytes(data[4:8], 'little') == expected_cpu)
    else:
        expected_cpu = {'aarch64': 183, 'x86_64': 62}[architecture]
        valid = (len(data) >= 64 and data[:6] == b'\x7fELF\x02\x01'
                 and data[7] in (0, 3) and int.from_bytes(data[18:20], 'little') == expected_cpu)
    if not valid:
        raise ValueError(f'release runtime executable header does not match {target}')


def validate_runtime_version(output, version):
    if output.strip() != f'Kipferl runtime {version}':
        raise ValueError('release runtime version does not match VERSION')


def run(runtime, target, snapshot_path, pins_path, output, cache, canonical_output=None):
    from check_release_packages import host_target, command, environment
    if target != host_target():
        raise ValueError('release popularity audit target must match the execution host')
    snapshot_bytes = snapshot_path.read_bytes()
    pins = json.loads(pins_path.read_text())
    validate_pins(pins, snapshot_bytes)
    runtime_bytes = runtime.read_bytes()
    validate_runtime_header(runtime_bytes, target)
    runtime_hash = audit.sha256(runtime_bytes)
    output.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    # Results never cross invocations or platforms; only verified wheel bytes can.
    with tempfile.TemporaryDirectory(prefix='kipferl-release-popularity-') as temporary:
        work = Path(temporary)
        version = command([str(runtime.resolve()), '--version'], cwd=work,
                          env=environment(work / 'version-environment'), timeout=10, limit=1024)['stdout']
        validate_runtime_version(version, (ROOT / 'VERSION').read_text().strip())
        seed = work / 'seed'
        audit.atomic_json(seed / 'policy.json', {'snapshot_sha256': audit.sha256(snapshot_bytes)})
        for pin in pins['records']:
            audit.atomic_json(seed / pin['name'] / 'metadata.json', pin)
        work_cache = work / 'cache'
        work_cache.mkdir()
        # The auditor's wheel cache is content addressed and verifies every hit.
        (work_cache / 'wheels').symlink_to(cache.resolve(), target_is_directory=True)
        report_path = work / 'popularity-audit.json'
        arguments = [sys.executable, str(ROOT / 'scripts/package_popularity_audit.py'),
                     '--runtime', str(runtime.resolve()), '--snapshot', str(snapshot_path.resolve()),
                     '--seed-metadata-from', str(seed), '--cache', str(work_cache),
                     '--output', str(report_path), '--workers', '4', '--limit', str(COUNT), '--retry-network']
        for attempt in range(3):
            subprocess.run(arguments, check=True)
            report = json.loads(report_path.read_text())
            if not report['counts'].get('network', 0):
                break
            print(f"Retrying {report['counts']['network']} network results ({attempt + 1}/3)", flush=True)
        audit.validate_outputs(report, snapshot_bytes, report_path)
        report['release_metadata_sha256'] = audit.policy_digest(pins)
        audit.atomic_json(report_path, report)
        validate_release_report(report, runtime_hash, target, snapshot_bytes, pins)
        for source, name in (
                (report_path, f'popularity-audit-{target}.json'),
                (report_path.with_suffix('.csv'), f'popularity-audit-{target}.csv'),
                (work / 'popularity-catalog.json', f'popularity-catalog-{target}.json')):
            destination = output / name
            shutil.copyfile(source, destination)
            destination.with_name(name + '.sha256').write_text(f'{audit.sha256(destination.read_bytes())}  {name}\n')
        if canonical_output is not None:
            audit.export(report, canonical_output)
            audit.validate_outputs(report, snapshot_bytes, canonical_output)
    print(f'Fresh 1,000-package release audit verified for {target}: {runtime_hash}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    runtime_options = parser.add_mutually_exclusive_group(required=True)
    runtime_options.add_argument('--runtime', type=Path)
    runtime_options.add_argument('--embedded-runtime', action='store_true')
    parser.add_argument('--target', choices=TARGETS)
    parser.add_argument('--snapshot', type=Path, default=DIRECTORY / 'popularity.json')
    parser.add_argument('--pins', type=Path, default=DIRECTORY / 'popularity-metadata.json')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cache', type=Path, default=ROOT / 'target/release-popularity-wheels')
    parser.add_argument('--canonical-output', type=Path, help='also install the verified report, CSV, Markdown and syntax catalog at this canonical path')
    args = parser.parse_args()
    from check_release_packages import host_target
    runtime = audit.embedded_runtime_path() if args.embedded_runtime else args.runtime
    run(runtime, args.target or host_target(), args.snapshot, args.pins, args.output, args.cache, args.canonical_output)


if __name__ == '__main__':
    main()
