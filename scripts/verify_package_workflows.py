#!/usr/bin/env python3
"""Verify reviewed package workflows through the real installer and standalone builder.

Package installation downloads wheels but never executes package build code.
Reviewed hooks run with network and home reads denied, and writes limited to a
fresh temporary test directory. Results apply only to the recorded artifacts,
CLI, runtime, platform, dependency lock and hook. No import-only approvals.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import sys
import tempfile

from check_release_packages import command, environment, host_target, SmokeFailure, ANSI
from package_catalog import validate as validate_catalog
from release_package_catalog import KEY_FIELDS

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'compatibility/packages'
STATUSES = {'verified', 'limited', 'unsupported', 'untested'}
KINDS = {'source': 'library', 'stub_only': 'typing', 'dependency_only': 'dependency-only',
         'resource_only': 'data', 'native_only': 'dependency-only'}
SUCCESS = 'KIPFERL_VERIFICATION_PASSED'


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def json_digest(value: dict) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def read_cases(path: Path, audit: dict) -> list[dict]:
    value = json.loads(path.read_text())
    if value.get('schema_version') != 1:
        raise ValueError('unsupported verification case schema')
    expected = {r['name']: r for r in audit['records'] if r['category'] == 'unverified' and r['compilation_completed']}
    cases = value.get('cases', [])
    if len(cases) != len(expected) or {c['name'] for c in cases} != set(expected):
        raise ValueError('verification cases must cover each compilation-complete candidate exactly once')
    for case in cases:
        record = expected[case['name']]
        for key in ('version', 'wheel_sha256', 'wheel_filename'):
            if case.get(key) != record.get(key):
                raise ValueError(f'{case["name"]}: {key} differs from the audited artifact')
        if case.get('kind') not in KINDS:
            raise ValueError('unknown verification case kind')
        hook = case.get('hook')
        if hook:
            hook_path = DIRECTORY / hook
            if hook_path.parent != DIRECTORY / 'smoke' or not hook_path.is_file():
                raise ValueError('verification hooks must be checked-in files in smoke/')
        if record.get('source_files_total', 0) and not hook:
            raise ValueError(f'{case["name"]}: source-bearing candidate needs a reviewed behavior hook')
    return cases


def sandbox_prefix(root: Path, *, python_control=False) -> list[str]:
    if platform.system() != 'Darwin' or not Path('/usr/bin/sandbox-exec').is_file():
        raise ValueError('workflow verification currently requires native macOS sandbox-exec')
    # JSON quoting is also valid for sandbox string literals. The interpreter
    # exception permits only the pinned Python installation for the control run.
    profile = ('(version 1)(allow default)(deny network*)(deny file-write*)'
               '(deny file-read* (subpath "/Users") (subpath "/home") (subpath "/root"))'
               f'(allow file-write* (subpath {json.dumps(str(root))}))')
    if python_control:
        profile += f'(allow file-read* (subpath {json.dumps(str(Path(sys.base_prefix).resolve()))}))'
    return ['/usr/bin/sandbox-exec', '-p', profile]


def short_error(text: str, root: Path) -> str:
    clean = ANSI.sub('', text).replace(str(root), '<test>')
    return '\n'.join(line[:400] for line in clean.strip().splitlines()[-10:])[:3000]


def installation_status(detail: str) -> str:
    # Operational failures must never become incompatibility evidence.
    if any(token in detail.lower() for token in ('timed out', 'timeout', 'http status', 'connection', 'dns', 'download failed', 'network', '429', '502', '503')):
        return 'untested'
    if any(token in detail.lower() for token in ('not supported', 'unsupported', 'incompatible', 'syntaxerror', 'no non-yanked', 'no compatible', 'metadata exceeds', 'exceeds the', 'exceeded')):
        return 'unsupported'
    return 'untested'


def friendly_reason(detail: str) -> str:
    """Keep the user-facing decision readable; preserve raw diagnostics in evidence."""
    missing = re.search(r"No module named '([^']+)'", detail)
    if missing:
        return f'Needs the {missing[1]} module, which this runtime does not provide yet.'
    if 'environment markers are not supported' in detail:
        return 'Its dependency declarations need conditional-package handling that the released installer does not support yet.'
    if '.data installation schemes' in detail:
        return 'Needs an external Python installation layout, such as notebook extension folders, which Kipferl does not install.'
    if '.pth' in detail:
        return 'Needs Python startup or namespace-package hooks that Kipferl does not load.'
    if 'SyntaxError:' in detail:
        return 'The package or a required dependency uses Python syntax this runtime cannot run yet.'
    if 'keyword arguments' in detail:
        return 'Calls a Python API with keyword arguments that this runtime does not support yet.'
    if 'unsupported import' in detail:
        return 'Uses an import path that the application bundler cannot package yet.'
    if 'expected' in detail and 'arguments, got' in detail:
        return 'Uses a Python API whose argument handling is not yet compatible.'
    if 'no non-yanked' in detail or 'native extension' in detail:
        return 'A required package has no usable pure-Python wheel for this installation.'
    if 'extras are not supported' in detail:
        return 'Requires an optional dependency group that the installer cannot select yet.'
    return 'Could not complete this package workflow. Open the test evidence for the exact failure.'


def validate_lock(lock: dict, case: dict, runtime_hash: str, target: str) -> None:
    if lock.get('schema') != 1 or lock.get('runtime_sha256') != runtime_hash or lock.get('target') != target:
        raise ValueError('installed lock does not match the runtime and platform')
    records = [p for p in lock.get('packages', []) if p.get('name') == case['name']]
    if len(records) != 1:
        raise ValueError('installed lock omits the exact root package')
    for key, expected in (('version', case['version']), ('sha256', case['wheel_sha256']), ('filename', case['wheel_filename'])):
        if records[0].get(key) != expected:
            raise ValueError('installer selected a different audited artifact')


def verify_case(case: dict, audit_row: dict, cli: Path, runtime_hash: str, target: str,
                cli_hash: str, output: Path) -> dict:
    scope = case.get('scope', [])
    scope = [scope] if isinstance(scope, str) else scope
    record = {'name': case['name'], 'version': case['version'], 'status': 'untested',
              'kind': case.get('display_kind', KINDS[case['kind']]), 'summary': case.get('summary', case.get('classification', case['name'])),
              'scope': scope, 'limitations': case.get('limitations', []), 'pypi_rank': audit_row['rank'],
              'workflow': case.get('workflow', ''), 'platforms': [],
              'evidence': {'wheel_sha256': case['wheel_sha256'], 'wheel_filename': case['wheel_filename'],
                           'runtime_sha256': runtime_hash, 'cli_sha256': cli_hash, 'target': target,
                           'source_files': audit_row.get('source_files_total', 0), 'steps': [],
                           'cpython_control_version': platform.python_version(),
                           'cpython_control_sha256': digest(Path(sys.executable)),
                           'classification': case.get('classification_evidence', {}),
                           'isolation': 'macOS sandbox; no network or home reads; writes confined to temporary test directory'}}
    proof = record['evidence']
    hook_path = DIRECTORY / case['hook'] if case.get('hook') else None
    if hook_path:
        proof['hook'] = case['hook']
        proof['hook_sha256'] = digest(hook_path)
    with tempfile.TemporaryDirectory(prefix='kipferl-verify-', dir='/tmp') as temporary:
        root = Path(temporary).resolve()
        binary = root / 'kipferl'
        shutil.copy2(cli, binary)
        binary.chmod(0o755)
        work = root / 'work'
        env = environment(work)
        project = work / 'project'
        project.mkdir()
        (project / 'kipferl.json').write_text(json.dumps({'entry': 'app.py'}) + '\n')
        app = (hook_path.read_text() if hook_path else '') + f'\nprint({SUCCESS!r})\n'
        (project / 'app.py').write_text(app)
        prefix = sandbox_prefix(root)

        def step(label: str, arguments: list[str], *, offline=True, cwd=project, control=False) -> dict:
            try:
                result = command([*((sandbox_prefix(root, python_control=True) if control else prefix) if offline else []), *arguments],
                                 cwd=cwd, env=env, timeout=90, limit=32768, success=False)
                result['stdout'] = short_error(result['stdout'], root)
                result['stderr'] = short_error(result['stderr'], root)
            except SmokeFailure as error:
                result = {'returncode': None, 'stdout': '', 'stderr': short_error(str(error), root), 'output_truncated': False}
            proof['steps'].append({'name': label, **result})
            return result

        def passed(result: dict, *, marker=False) -> bool:
            return result['returncode'] == 0 and not result['output_truncated'] and (not marker or SUCCESS in result['stdout'].splitlines())

        result = step('install-with-dependencies', [str(binary), 'add', f'{case["name"]}=={case["version"]}', '--allow-unverified'], offline=False)
        if not passed(result):
            record['reason'] = (result['stdout'] + '\n' + result['stderr']).strip()
            record['status'] = installation_status(record['reason'])
        else:
            lock_path = project / 'kipferl.lock'
            lock = json.loads(lock_path.read_text())
            validate_lock(lock, case, runtime_hash, target)
            for package in lock['packages']:
                wheel = project / '.kipferl/cache' / (package['sha256'] + '.whl')
                if not wheel.is_file() or digest(wheel) != package['sha256']:
                    raise ValueError('installed dependency wheel does not match its lock')
            proof['lock'] = lock
            proof['lock_sha256'] = digest(lock_path)
            proof['lock_json_sha256'] = json_digest(lock)
            if not hook_path:
                record['reason'] = 'Installs as support files or a dependency bundle; it does not provide a runtime workflow to verify.'
            else:
                installed = project / '.kipferl/packages'
                control_code = 'import sys\nsys.path.insert(0, ' + repr(str(installed)) + ')\n' + app
                result = step('cpython-control', [sys.executable, '-B', '-E', '-S', '-c', control_code], control=True)
                if not passed(result, marker=True):
                    record['reason'] = 'The verification scenario did not pass its CPython control: ' + ((result['stdout'] + '\n' + result['stderr']).strip())
                else:
                    result = step('installed-workflow', [str(binary), 'run'])
                    if not passed(result, marker=True):
                        record['status'] = 'unsupported' if result['returncode'] is not None else 'untested'
                        record['reason'] = (result['stdout'] + '\n' + result['stderr']).strip()
                    else:
                        record['status'] = 'limited'
                        record['reason'] = 'Installed workflow passed; standalone and offline restoration still need to pass.'
                        shutil.rmtree(installed)
                        result = step('offline-locked-restore', [str(binary), 'sync', '--locked', '--offline'])
                        if passed(result) and digest(lock_path) == proof['lock_sha256']:
                            result = step('restored-workflow', [str(binary), 'run'])
                            if passed(result, marker=True):
                                result = step('standalone-build', [str(binary), 'build', '--mode', 'universal', '-o', 'program'])
                                if passed(result):
                                    detached = root / 'detached'
                                    detached.mkdir()
                                    program = detached / 'program'
                                    shutil.copy2(project / 'program', program)
                                    proof['standalone_sha256'] = digest(program)
                                    shutil.rmtree(work)
                                    env = environment(root / 'fresh-environment')
                                    result = step('detached-standalone-workflow', [str(program)], cwd=detached)
                                    if passed(result, marker=True):
                                        record['status'] = 'verified'
                                        record['reason'] = 'Installed, restored offline, and ran the stated workflow as a standalone app after removing the project and caches.'
                        if record['status'] == 'limited':
                            record['reason'] = 'Installed workflow passed, but portability verification stopped: ' + ((result['stdout'] + '\n' + result['stderr']).strip())
                if record['status'] in {'verified', 'limited'}:
                    record['install_command'] = f'kipferl add {case["name"]}=={case["version"]} --allow-unverified'
    if record['status'] == 'verified' and case.get('approval_ceiling') == 'limited':
        record['status'] = 'limited'
        record['reason'] += ' Only the listed supporting resource/configuration workflow is covered.'
    if record['status'] == 'unsupported':
        proof['diagnostic'] = record['reason']
        record['reason'] = friendly_reason(record['reason'])
    record['platforms'] = [{'target': target, 'runtime_sha256': runtime_hash, 'status': record['status'], 'evidence': record['reason']}]
    proof['completed_at'] = datetime.now(timezone.utc).isoformat()
    output.mkdir(parents=True, exist_ok=True)
    (output / f'{case["name"]}.json').write_text(json.dumps(record, indent=2) + '\n')
    print(f'{case["name"]}: {record["status"]}', flush=True)
    return record


def validate_report(report: dict, audit: dict, *, audit_path: Path = DIRECTORY / 'popularity-audit.json',
                    cases_path: Path = DIRECTORY / 'verification-cases.json') -> None:
    if report.get('schema_version') != 1:
        raise ValueError('unsupported verification report schema')
    if report.get('release') != 'development' and (
            report.get('cli_version') != f'Kipferl v{report.get("release")}'
            or report.get('runtime_version') != f'Kipferl runtime {report.get("release")}'):
        raise ValueError('displayed release differs from observed binary versions')
    if report.get('source_report_sha256') != digest(audit_path) or audit != json.loads(audit_path.read_text()):
        raise ValueError('verification source audit changed; rerun verification')
    if report.get('cases_sha256') != digest(cases_path):
        raise ValueError('reviewed verification cases changed; rerun verification')
    cases = {case['name']: case for case in read_cases(cases_path, audit)}
    expected = {r['name']: r for r in audit['records'] if r['category'] == 'unverified' and r['compilation_completed']}
    records = report.get('records', [])
    if len(records) != len(expected) or {r['name'] for r in records} != set(expected):
        raise ValueError('verification report does not cover all compilation-complete candidates')
    for record in records:
        original = expected[record['name']]
        case = cases[record['name']]
        proof = record['evidence']
        for field in ('runtime_sha256', 'cli_sha256', 'wheel_sha256', 'cpython_control_sha256'):
            if not re.fullmatch(r'[0-9a-f]{64}', proof.get(field, '')):
                raise ValueError('verification is missing an executable or artifact identity')
        if record['status'] not in STATUSES or record['version'] != original['version'] or proof['wheel_sha256'] != original['wheel_sha256']:
            raise ValueError('verification identity or status mismatch')
        if proof['runtime_sha256'] != audit['runtime_sha256'] or proof['target'] != audit['target']:
            raise ValueError('verification evidence belongs to a different runtime or platform')
        scope = case['scope'] if isinstance(case['scope'], list) else [case['scope']]
        if (record['scope'] != scope or record['limitations'] != case.get('limitations', [])
                or record['kind'] != case.get('display_kind', KINDS[case['kind']])
                or record['summary'] != case.get('summary', case.get('classification', case['name']))
                or record['workflow'] != case.get('workflow', '')
                or proof.get('hook') != case.get('hook')
                or proof['source_files'] != original.get('source_files_total', 0)):
            raise ValueError('displayed verification claims differ from the reviewed case')
        if record['platforms'] != [{'target': proof['target'], 'runtime_sha256': proof['runtime_sha256'], 'status': record['status'], 'evidence': record['reason']}]:
            raise ValueError('badge platform coverage differs from execution evidence')
        if proof.get('hook'):
            hook = DIRECTORY / proof['hook']
            if hook.parent != DIRECTORY / 'smoke' or not hook.is_file() or digest(hook) != proof['hook_sha256']:
                raise ValueError('reviewed verification hook changed; rerun verification')
        if record['status'] in {'verified', 'limited'}:
            if record['kind'] not in {'library', 'data'} or not record['scope']:
                raise ValueError('verified workflows require a meaningful source/resource scope')
            by_name = {s['name']: s for s in proof['steps']}
            required = ('install-with-dependencies', 'cpython-control', 'installed-workflow')
            if record['status'] == 'verified':
                if case.get('approval_ceiling') == 'limited':
                    raise ValueError('reviewed case permits only a limited badge')
                required += ('offline-locked-restore', 'restored-workflow', 'standalone-build', 'detached-standalone-workflow')
            if any(name not in by_name or by_name[name]['returncode'] != 0 or by_name[name].get('output_truncated') for name in required):
                raise ValueError('verified workflow is missing successful execution stages')
            for name in ('cpython-control', 'installed-workflow', 'restored-workflow', 'detached-standalone-workflow'):
                if name in required and SUCCESS not in by_name[name]['stdout'].splitlines():
                    raise ValueError('verified workflow lacks its completion assertion')
            if not proof.get('lock') or not proof.get('hook_sha256') or (record['status'] == 'verified' and not proof.get('standalone_sha256')):
                raise ValueError('verified workflow lacks artifact, dependency or hook evidence')
            if proof.get('lock_json_sha256') != json_digest(proof['lock']):
                raise ValueError('verification dependency lock changed')
            validate_lock(proof['lock'], original, proof['runtime_sha256'], proof['target'])


def promote(report: dict, audit: dict, destination: Path) -> None:
    validate_report(report, audit)
    catalog = json.loads(destination.read_text())
    rows = {tuple(row[k] for k in KEY_FIELDS): row for row in catalog['records']}
    original = {row['name']: row for row in audit['records']}
    for record in report['records']:
        if record['status'] != 'verified':
            continue
        proof = record['evidence']
        if len(proof['lock']['packages']) != 1:
            raise ValueError('dependency-bearing approvals need lock-bound catalog support before promotion')
        row = {key: proof[key] for key in ('wheel_sha256', 'wheel_filename', 'runtime_sha256', 'target')}
        row.update(name=record['name'], version=record['version'], status='tested',
                   source_url=original[record['name']]['source_url'],
                   source_files_checked=proof['source_files'],
                   evidence='Reviewed real-package workflow passed installation with locked dependencies, CPython control, runtime execution, offline restoration and detached standalone execution. ' + ' '.join(record['scope']),
                   smoke={'file': Path(proof['hook']).name, 'sha256': proof['hook_sha256'], 'scope': ' '.join(record['scope']), 'runner': proof['isolation']},
                   workflow_verification={'cli_sha256': proof['cli_sha256'], 'lock_sha256': proof['lock_sha256'], 'standalone_sha256': proof['standalone_sha256']})
        rows[tuple(row[k] for k in KEY_FIELDS)] = row
    catalog['records'] = list(rows.values())
    validate_catalog(catalog)
    destination.write_text(json.dumps(catalog, indent=2) + '\n')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--cases', type=Path, default=DIRECTORY / 'verification-cases.json')
    parser.add_argument('--audit', type=Path, default=DIRECTORY / 'popularity-audit.json')
    parser.add_argument('--output', type=Path, default=DIRECTORY / 'verified-packages.json')
    parser.add_argument('--artifacts', type=Path, default=ROOT / 'target/package-verification')
    parser.add_argument('--workers', type=int, choices=range(1, 5), default=4)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--promote', action='store_true')
    parser.add_argument('--development', action='store_true', help='label an unreleased CLI build explicitly')
    args = parser.parse_args()
    audit = json.loads(args.audit.read_text())
    if args.check or args.promote:
        report = json.loads(args.output.read_text())
        validate_report(report, audit, audit_path=args.audit, cases_path=args.cases)
        if args.promote:
            promote(report, audit, DIRECTORY / 'catalog.json')
        print(f'Validated {len(report["records"])} workflow verification records')
        return 0
    if not args.cli or not args.runtime:
        parser.error('--cli and --runtime are required for execution')
    cli, runtime = args.cli.resolve(), args.runtime.resolve()
    runtime_hash = digest(runtime)
    if host_target() != audit['target'] or runtime_hash != audit['runtime_sha256']:
        raise ValueError('use the exact native runtime from the source audit; rerun the source audit for a new runtime')
    sandbox_prefix(Path('/tmp'))  # Fail before downloads on unsupported execution hosts.
    cases = read_cases(args.cases, audit)
    originals = {r['name']: r for r in audit['records']}
    with tempfile.TemporaryDirectory(prefix='kipferl-verify-identity-', dir='/tmp') as temporary:
        path = Path(temporary)
        identity_env = environment(path / 'env')
        expected_version = (ROOT / 'VERSION').read_text().strip()
        cli_version = ANSI.sub('', command([str(cli), '--version'], cwd=path, env=identity_env)['stdout']).strip()
        runtime_version = command([str(runtime), '--version'], cwd=path, env=identity_env)['stdout'].strip()
        if cli_version != f'Kipferl v{expected_version}' or runtime_version != f'Kipferl runtime {expected_version}':
            raise ValueError('observed binary versions differ from VERSION; do not relabel old evidence')
        result = command([str(cli), 'deps', 'audit', '--json'], cwd=path, env=identity_env, limit=8*1024*1024)
        embedded = json.loads(result['stdout'])
        if embedded['runtime_sha256'] != runtime_hash or embedded['target'] != host_target():
            raise ValueError('CLI audit does not match the verification runtime')
    cli_hash = digest(cli)
    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = [pool.submit(verify_case, case, originals[case['name']], cli, runtime_hash, host_target(), cli_hash, args.artifacts) for case in cases]
        records = [future.result() for future in as_completed(futures)]
    report = {'schema_version': 1, 'release': 'development' if args.development else expected_version,
              'cli_version': cli_version, 'runtime_version': runtime_version,
              'generated_at': datetime.now(timezone.utc).isoformat(),
              'source_report_sha256': digest(args.audit), 'cases_sha256': digest(args.cases),
              'records': sorted(records, key=lambda r: r['pypi_rank'])}
    validate_report(report, audit, audit_path=args.audit, cases_path=args.cases)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
