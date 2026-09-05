"""Offline tests for exact, reproducible package evidence and safe extraction."""
import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest.mock import patch

import package_catalog


class PackageCatalogTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(package_catalog.CATALOG.read_text())

    def test_checked_in_catalog_and_smoke_hashes_are_valid(self):
        package_catalog.validate(self.catalog)
        self.assertTrue(self.catalog["records"])

    def test_duplicate_evidence_is_rejected(self):
        self.catalog["records"].append(copy.deepcopy(self.catalog["records"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            package_catalog.validate(self.catalog)

    def test_unknown_status_is_rejected(self):
        self.catalog["records"][0]["status"] = "probably fine"
        with self.assertRaisesRegex(ValueError, "status"):
            package_catalog.validate(self.catalog)

    def test_tested_requires_explicit_hashed_behavior_scope(self):
        self.catalog["records"][0]["status"] = "tested"
        self.catalog["records"][0].pop("smoke", None)
        with self.assertRaisesRegex(ValueError, "hashed smoke"):
            package_catalog.validate(self.catalog)

    def test_inexact_hash_cannot_enter_catalog(self):
        self.catalog["records"][0]["runtime_sha256"] = "latest"
        with self.assertRaisesRegex(ValueError, "runtime_sha256"):
            package_catalog.validate(self.catalog)

    def test_traversal_and_symlink_wheels_are_rejected(self):
        for filename, mode in [("../outside.py", 0), ("/outside.py", 0), ("link", 0o120777 << 16)]:
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as temporary:
                payload = io.BytesIO()
                with zipfile.ZipFile(payload, "w") as archive:
                    info = zipfile.ZipInfo(filename)
                    info.external_attr = mode
                    archive.writestr(info, b"outside")
                with self.assertRaisesRegex(ValueError, "unsafe"):
                    package_catalog.unpack(payload.getvalue(), Path(temporary))

    def test_syntax_checker_passes_file_in_module_mode_without_driver_code(self):
        result = type("Result", (), {"returncode": 0, "stdout": "", "stderr": ""})()
        with patch.object(package_catalog.subprocess, "run", return_value=result) as run:
            package_catalog.check_syntax(Path("/runtime"), Path("/source.py"), Path("/work"), timeout=3)
        self.assertEqual(run.call_args.args[0], ["/runtime", "--check-syntax", "--", "/source.py"])
        self.assertEqual(run.call_args.kwargs["timeout"], 3)

    def test_checker_preflight_refuses_missing_or_executing_checker(self):
        result = type("Result", (), {"returncode": 1, "stdout": "", "stderr": "checker executed source"})()
        with patch.object(package_catalog, "check_syntax", return_value=result):
            with self.assertRaisesRegex(RuntimeError, "non-executing --check-syntax"):
                package_catalog.verify_syntax_checker(Path("/runtime"))

    def test_refresh_rejects_untrusted_download_host(self):
        with self.assertRaisesRegex(ValueError, "official PyPI"):
            package_catalog.download("https://example.com/wheel.whl")


if __name__ == "__main__":
    unittest.main()
