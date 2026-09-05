import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_release_sizes
import check_release_version
import prepare_release_assets as assets


class ReleaseAssetsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'inputs'
        self.source.mkdir()
        self.dest = self.root / 'assets'
        self.dest.mkdir()
        (self.dest / 'sentinel').write_text('unchanged')
        for target in assets.TARGETS:
            hashes = {}
            for kind in assets.COMPONENTS:
                name = f'{kind}-{target}'
                hashes[kind] = self.write(name, name.encode())
            record = {'name': 'tzdata', 'version': '2025.2', 'target': target,
                      'status': 'tested', 'runtime_sha256': hashes['pocketpy-kipferl']}
            self.write(f'package-catalog-{target}.json', json.dumps({'records': [record]}).encode())

    def write(self, name, data):
        digest = hashlib.sha256(data).hexdigest()
        (self.source / name).write_bytes(data)
        (self.source / (name + '.sha256')).write_text(f'{digest}  {name}\n')
        return digest

    def prepare(self):
        # Catalog schema/hook validation has its own tests; isolate artifact wiring here.
        with patch.object(assets, 'validate'):
            assets.prepare(self.source, self.dest, 'macos-aarch64', self.root / 'catalog.json')

    def test_all_inputs_verified_before_any_fallback_asset_is_replaced(self):
        (self.source / 'kipferl-loader-linux-x86_64').unlink()
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual([p.name for p in self.dest.iterdir()], ['sentinel'])

    def test_checksum_tamper_and_wrong_runtime_evidence_fail_closed(self):
        (self.source / 'pocketpy-kipferl-macos-aarch64').write_bytes(b'tampered')
        with self.assertRaisesRegex(ValueError, 'checksum'):
            self.prepare()
        self.write('pocketpy-kipferl-macos-aarch64', b'new runtime')
        with self.assertRaisesRegex(ValueError, 'tested evidence'):
            self.prepare()

    def test_complete_set_restores_executable_permissions(self):
        self.prepare()
        self.assertEqual(len(list(self.dest.iterdir())), 13)
        self.assertTrue((self.dest / 'pocketpy-kipferl-macos-aarch64').stat().st_mode & 0o111)
        self.assertTrue((self.root / 'catalog.json').is_file())

    def test_size_boundary_and_empty_artifact_are_rejected(self):
        path = self.root / 'binary'
        for size in (0, 10):
            path.write_bytes(b'x' * size)
            with patch.dict(check_release_sizes.LIMITS, {'cli': 10}):
                with self.assertRaises(ValueError):
                    check_release_sizes.check({'cli': path})
        path.write_bytes(b'valid')
        with patch.dict(check_release_sizes.LIMITS, {'cli': 10}):
            check_release_sizes.check({'cli': path})

    def test_release_versions_require_matching_tag_manifest_and_lock(self):
        (self.root / 'VERSION').write_text('0.7.0-rc.1\n')
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.7.0-rc.1"\n')
        (self.root / 'Cargo.lock').write_text('[[package]]\nname="kipferl-cli"\nversion="0.7.0-rc.1"\n')
        self.assertEqual(check_release_version.check(self.root, 'v0.7.0-rc.1'), '0.7.0-rc.1')
        with self.assertRaisesRegex(ValueError, 'tag'):
            check_release_version.check(self.root, 'v0.6.0')
        (self.root / 'Cargo.lock').write_text('[[package]]\nname="kipferl-cli"\nversion="0.6.0"\n')
        with self.assertRaisesRegex(ValueError, 'lock'):
            check_release_version.check(self.root)


if __name__ == '__main__':
    unittest.main()
