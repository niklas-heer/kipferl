"""Fail closed on mixed runtime, snapshot, metadata, or partial release screens."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import package_popularity_audit as audit
import release_popularity_audit as release


class ReleasePopularityTests(unittest.TestCase):
    def setUp(self):
        self.snapshot = {'source': {'name': 'fixture'}, 'projects': [
            {'rank': rank, 'name': f'project-{rank}', 'downloads': 1000 - rank}
            for rank in range(1, 1001)]}
        self.snapshot_bytes = json.dumps(self.snapshot).encode()
        self.pins = {'schema_version': 1, 'snapshot_sha256': audit.sha256(self.snapshot_bytes),
                     'missing_projects': ['project-1000'], 'records': []}
        rows = []
        for project in self.snapshot['projects']:
            name = project['name']
            pin = {'name': name, 'version': '1.0', 'metadata_url': f'https://pypi.org/pypi/{name}/json',
                   'metadata_sha256': 'b' * 64, 'metadata_fetched_at': '2026-09-05T00:00:00Z',
                   'artifact_kind': 'source_only', 'artifact': None,
                   'requires_python': None, 'requires_dist': []}
            row = audit.initial_record(project)
            row.update({key: pin[key] for key in ('version', 'metadata_url', 'metadata_sha256', 'metadata_fetched_at')})
            row.update(category='source_only', status='incompatible', evidence_scope='metadata')
            rows.append(row)
            if name not in self.pins['missing_projects']:
                self.pins['records'].append(pin)
        self.report = audit.make_report(rows, self.snapshot, audit.sha256(self.snapshot_bytes),
                                        'a' * 64, 'macos-aarch64', 1000)
        self.report['release_metadata_sha256'] = audit.policy_digest(self.pins)

    def validate(self, report=None, pins=None):
        release.validate_release_report(report or self.report, 'a' * 64, 'macos-aarch64',
                                        self.snapshot_bytes, pins or self.pins)

    def test_each_release_run_has_fresh_results_and_only_reuses_wheel_bytes(self):
        import check_release_packages as smoke
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime = root / 'runtime'
            runtime.write_bytes(b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01' + bytes(24))
            snapshot, pins = root / 'snapshot.json', root / 'pins.json'
            snapshot.write_bytes(self.snapshot_bytes)
            pins.write_text(json.dumps(self.pins))
            cache_paths = []

            def fake_auditor(arguments, **options):
                self.assertTrue(options['check'])
                selected = lambda flag: Path(arguments[arguments.index(flag) + 1])
                cache = selected('--cache')
                cache_paths.append(cache)
                self.assertFalse((cache / 'checkpoints').exists())
                self.assertEqual((cache / 'wheels').resolve(), (root / 'wheel-cache').resolve())
                seed = selected('--seed-metadata-from')
                self.assertEqual(len(list(seed.glob('*/metadata.json'))), 999)
                self.assertEqual(list(seed.rglob('result.json')), [])
                report = copy.deepcopy(self.report)
                report['runtime_sha256'] = audit.sha256(runtime.read_bytes())
                report['cache_key'] = audit.cache_key(report['snapshot_sha256'], report['runtime_sha256'], report['audit_policy'])
                audit.export(report, selected('--output'))

            with patch.object(smoke, 'host_target', return_value='macos-aarch64'), \
                    patch.object(smoke, 'command', return_value={'stdout': 'Kipferl runtime ' + (release.ROOT / 'VERSION').read_text().strip()}), \
                    patch.object(release.subprocess, 'run', side_effect=fake_auditor):
                for _ in range(2):
                    release.run(runtime, 'macos-aarch64', snapshot, pins, root / 'out', root / 'wheel-cache',
                                root / 'canonical/popularity-audit.json')
            self.assertEqual(len(set(cache_paths)), 2)
            self.assertTrue(all(not path.exists() for path in cache_paths))
            exported = root / 'out/popularity-audit-macos-aarch64.json'
            report = json.loads(exported.read_text())
            self.assertEqual(report['release_metadata_sha256'], audit.policy_digest(self.pins))
            self.assertEqual(exported.with_suffix('.json.sha256').read_text(),
                             f'{audit.sha256(exported.read_bytes())}  {exported.name}\n')
            self.assertEqual(report, json.loads((root / 'canonical/popularity-audit.json').read_text()))
            audit.validate_outputs(report, self.snapshot_bytes, root / 'canonical/popularity-audit.json')

    def test_executable_header_requires_native_os_and_architecture(self):
        macho = b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01' + bytes(24)
        elf = bytearray(64)
        elf[:6] = b'\x7fELF\x02\x01'
        elf[18:20] = (62).to_bytes(2, 'little')
        release.validate_runtime_header(macho, 'macos-aarch64')
        release.validate_runtime_header(elf, 'linux-x86_64')
        for data, target in [(macho, 'macos-x86_64'), (macho, 'linux-aarch64'),
                             (elf, 'linux-aarch64'), (elf, 'macos-x86_64'),
                             (b'#!/bin/sh', 'macos-aarch64')]:
            with self.subTest(target=target), self.assertRaisesRegex(ValueError, 'header'):
                release.validate_runtime_header(data, target)

    def test_runtime_version_must_be_the_exact_release(self):
        release.validate_runtime_version('Kipferl runtime 0.7.2\n', '0.7.2')
        for value in ('Kipferl runtime 0.7.1', '3.11.0', 'Kipferl runtime 0.7.2 extra'):
            with self.assertRaisesRegex(ValueError, 'version'):
                release.validate_runtime_version(value, '0.7.2')

    def test_complete_exact_report_with_explicit_missing_pin_is_accepted(self):
        self.validate()

    def test_runtime_target_partial_and_policy_changes_are_rejected(self):
        for field, value in [('runtime_sha256', 'c' * 64), ('target', 'linux-x86_64'),
                             ('complete', False), ('requested_count', 1001),
                             ('release_metadata_sha256', 'c' * 64)]:
            changed = copy.deepcopy(self.report)
            changed[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(changed)
        with patch.object(audit, 'current_policy', return_value={'version': 99}), self.assertRaisesRegex(ValueError, 'policy'):
            self.validate()

    def test_changed_pin_and_rank_identity_fail(self):
        for field, value in [('version', '2.0'), ('metadata_sha256', 'c' * 64),
                             ('selected_artifact_filename', 'other.whl'), ('downloads', 1)]:
            report = copy.deepcopy(self.report)
            report['records'][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(report)

    def test_missing_duplicate_or_untrusted_metadata_pins_fail(self):
        for edit in (lambda pins: pins['records'].pop(),
                     lambda pins: pins['records'].reverse(),
                     lambda pins: pins.update(missing_projects=['unknown']),
                     lambda pins: pins['records'][0].update(metadata_url='https://other.example/data')):
            pins = copy.deepcopy(self.pins)
            edit(pins)
            with self.assertRaises(ValueError):
                release.validate_pins(pins, self.snapshot_bytes)

    def test_network_results_cannot_be_published_as_a_finished_release_screen(self):
        report = copy.deepcopy(self.report)
        report['records'][0].update(category='network', status='unverified')
        report['counts']['network'] = 1
        report['counts']['source_only'] -= 1
        with self.assertRaisesRegex(ValueError, 'network'):
            self.validate(report)


if __name__ == '__main__':
    unittest.main()
