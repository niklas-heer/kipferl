"""Homebrew publishing must fail closed using offline release-download fixtures."""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ('kipferl-macos-aarch64', 'kipferl-macos-x86_64',
          'kipferl-linux-x86_64', 'kipferl-linux-aarch64')


class UpdateHomebrewTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='kipferl-homebrew-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.downloads = self.root / 'downloads'
        self.downloads.mkdir()
        self.tools = self.root / 'tools'
        self.tools.mkdir()
        self.tmp = self.root / 'private temp'
        self.tmp.mkdir()
        self.tap = self.root / 'tap with spaces'
        self.formula = self.tap / 'Formula/kipferl.rb'
        self.formula.parent.mkdir(parents=True)
        self.original = b'previous verified formula\n'
        self.formula.write_bytes(self.original)
        self.hashes = {}
        for name in ASSETS:
            data = ('release artifact: ' + name).encode()
            self.hashes[name] = hashlib.sha256(data).hexdigest()
            (self.downloads / name).write_bytes(data)
            (self.downloads / (name + '.sha256')).write_text(f'{self.hashes[name]}  {name}\n')
        self.tool('curl', r'''
import os
from pathlib import Path
import sys
arguments = sys.argv[1:]
url = arguments[-1]
if not url.startswith('https://github.com/niklas-heer/kipferl/releases/download/v0.7.0/'):
    raise SystemExit(91)
name = url.rsplit('/', 1)[-1]
output = Path(arguments[arguments.index('--output') + 1])
if os.environ.get('FAIL_DOWNLOAD') == name:
    output.write_bytes(b'partial failed response')
    raise SystemExit(22)
output.write_bytes((Path(os.environ['TEST_DOWNLOADS']) / name).read_bytes())
''')
        self.environment = {**os.environ,
                            'PATH': str(self.tools) + os.pathsep + os.environ['PATH'],
                            'TAP_REPO': str(self.tap), 'TMPDIR': str(self.tmp),
                            'TEST_DOWNLOADS': str(self.downloads)}

    def tool(self, name, source):
        path = self.tools / name
        path.write_text(f'#!{sys.executable}\n' + source)
        path.chmod(0o755)

    def run_script(self, arguments=('0.7.0',), **environment):
        return subprocess.run(['/bin/bash', str(ROOT / 'scripts/update-homebrew.sh'), *arguments],
                              env={**self.environment, **environment}, capture_output=True,
                              text=True, timeout=20)

    def assert_unchanged(self, result):
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.formula.read_bytes(), self.original)
        self.assertEqual(list(self.tmp.iterdir()), [])
        self.assertEqual(list(self.formula.parent.iterdir()), [self.formula])

    def test_complete_downloads_verify_hashes_and_atomically_replace_formula(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        formula = self.formula.read_text()
        self.assertIn('version "0.7.0"', formula)
        self.assertIn('bin.install binary_name => "kipferl"', formula)
        self.assertNotIn('install_symlink', formula)
        for digest in self.hashes.values():
            self.assertIn(f'sha256 "{digest}"', formula)
        self.assertNotIn(self.original.decode(), formula)
        self.assertEqual(self.formula.stat().st_mode & 0o777, 0o644)
        self.assertEqual(list(self.tmp.iterdir()), [])
        self.assertEqual(list(self.formula.parent.iterdir()), [self.formula])

    def test_failed_final_binary_or_checksum_download_preserves_formula(self):
        for name in (ASSETS[-1], ASSETS[-1] + '.sha256'):
            with self.subTest(download=name):
                self.assert_unchanged(self.run_script(FAIL_DOWNLOAD=name))

    def test_corrupt_binary_or_wrong_sidecar_identity_preserves_formula(self):
        name = ASSETS[-1]
        original_binary = (self.downloads / name).read_bytes()
        (self.downloads / name).write_bytes(original_binary + b'tampered')
        self.assert_unchanged(self.run_script())
        (self.downloads / name).write_bytes(original_binary)
        for sidecar in ('0' * 64 + f'  {name}\n',
                        self.hashes[name] + '  different-artifact\n',
                        'malformed checksum\n'):
            with self.subTest(sidecar=sidecar):
                (self.downloads / (name + '.sha256')).write_text(sidecar)
                self.assert_unchanged(self.run_script())

    def test_empty_download_cannot_be_published_even_with_matching_empty_hash(self):
        name = ASSETS[-1]
        (self.downloads / name).write_bytes(b'')
        digest = hashlib.sha256(b'').hexdigest()
        (self.downloads / (name + '.sha256')).write_text(f'{digest}  {name}\n')
        self.assert_unchanged(self.run_script())

    def test_formula_generation_failure_preserves_previous_formula(self):
        self.tool('cat', r'''
from pathlib import Path
import sys
if len(sys.argv) == 1:
    print('partial failed formula')
    raise SystemExit(55)
sys.stdout.buffer.write(Path(sys.argv[1]).read_bytes())
''')
        self.assert_unchanged(self.run_script())

    def test_requires_exactly_one_stable_version_before_any_download(self):
        for arguments in ((), ('0.7.0-rc.1',), ('v0.7.0',), ('0.7.0', 'extra'),
                          ('0.7.0"\ninjected',), ('00.7.0',)):
            with self.subTest(arguments=arguments):
                self.assert_unchanged(self.run_script(arguments))


if __name__ == '__main__':
    unittest.main()
