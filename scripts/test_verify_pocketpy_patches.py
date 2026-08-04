import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("verify-pocketpy-patches.py")
SPEC = importlib.util.spec_from_file_location("verify_pocketpy_patches", SCRIPT)
verify = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(verify)


class PocketPyPatchVerificationTests(unittest.TestCase):
    def test_replays_a_patch_and_requires_exact_vendor_output(self):
        with tempfile.TemporaryDirectory() as raw_directory:
            root = Path(raw_directory)
            source = root / "pocketpy/vendor/pocketpy.c"
            source.parent.mkdir(parents=True)
            source.write_text("before\n", encoding="utf-8")
            patch = root / "pocketpy/patches/0001-change.patch"
            patch.parent.mkdir(parents=True)
            patch.write_text(
                "diff --git a/pocketpy/vendor/pocketpy.c b/pocketpy/vendor/pocketpy.c\n"
                "--- a/pocketpy/vendor/pocketpy.c\n"
                "+++ b/pocketpy/vendor/pocketpy.c\n"
                "@@ -1 +1 @@\n"
                "-upstream\n"
                "+before\n",
                encoding="utf-8",
            )
            failures = []
            self.assertTrue(
                verify._replay_patchset(
                    root,
                    "upstream\n",
                    [patch],
                    [{"path": "pocketpy/vendor/pocketpy.c"}],
                    failures,
                )
            )
            self.assertEqual(failures, [])

            source.write_text("different\n", encoding="utf-8")
            failures = []
            self.assertFalse(
                verify._replay_patchset(
                    root,
                    "upstream\n",
                    [patch],
                    [{"path": "pocketpy/vendor/pocketpy.c"}],
                    failures,
                )
            )
            self.assertIn("differs from vendored", failures[0])

    def test_manifest_requires_every_patch_on_disk(self):
        with tempfile.TemporaryDirectory() as raw_directory:
            root = Path(raw_directory)
            patch_directory = root / "pocketpy/patches"
            patch_directory.mkdir(parents=True)
            (patch_directory / "0001-declared.patch").write_text("declared\n")
            (patch_directory / "0002-untracked.patch").write_text("untracked\n")
            manifest = {
                "patch_files": [
                    {
                        "id": "0001-declared",
                        "file": "pocketpy/patches/0001-declared.patch",
                    }
                ],
                "tracked_files": [{"path": "pocketpy/vendor/pocketpy.c"}],
            }
            failures = []
            verify._validate_manifest(root, manifest, failures)
            self.assertTrue(
                any("differs from disk" in failure for failure in failures), failures
            )

    def test_json_report_is_deterministic_and_machine_readable(self):
        with tempfile.TemporaryDirectory() as raw_directory:
            report_path = Path(raw_directory) / "nested/report.json"
            report = {"status": "pass", "schema_version": 1}
            verify._write_report(report_path, report)
            self.assertEqual(json.loads(report_path.read_text()), report)
            self.assertTrue(report_path.read_text().endswith("\n"))


if __name__ == "__main__":
    unittest.main()
