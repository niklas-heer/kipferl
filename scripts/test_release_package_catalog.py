"""Offline provenance and execution-guard tests; package code never runs."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import package_catalog
import release_package_catalog as release


def elf(architecture=62):
    header = bytearray(64)
    header[:7] = b"\x7fELF\x02\x01\x01"
    header[18:20] = architecture.to_bytes(2, "little")
    return bytes(header)


class ReleaseCatalogTests(unittest.TestCase):
    def setUp(self):
        self.existing = json.loads(package_catalog.CATALOG.read_text())
        self.candidates = json.loads((package_catalog.DIRECTORY / "candidates.json").read_text())["packages"]
        self.tzdata = next(c for c in self.candidates if c["name"] == "tzdata")

    def test_real_reviewed_pins_are_unambiguous(self):
        self.assertEqual(len(release.reviewed_pins(self.existing, self.candidates)), len(self.candidates))

    def test_missing_expected_tested_candidate_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "absent"):
            release.reviewed_pins(self.existing, [c for c in self.candidates if c["name"] != "tzdata"])

    def test_changed_wheel_pin_is_rejected(self):
        for r in self.existing["records"]:
            if r["name"] == "tzdata":
                r["wheel_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "wheel pin changed"):
            release.reviewed_pins(self.existing, self.candidates)

    def test_additional_hook_cannot_execute(self):
        self.candidates[0]["smoke"] = "new.py"
        with self.assertRaisesRegex(ValueError, "only permits"):
            release.reviewed_pins(self.existing, self.candidates)

    def test_changed_behavior_scope_is_rejected(self):
        self.tzdata["scope"] = "Entire timezone API is compatible"
        with self.assertRaisesRegex(ValueError, "scope"):
            release.reviewed_pins(self.existing, self.candidates)

    def test_conflicting_historical_artifacts_are_rejected(self):
        record = copy.deepcopy(self.existing["records"][0])
        record["wheel_sha256"] = "0" * 64
        self.existing["records"].append(record)
        with self.assertRaisesRegex(ValueError, "ambiguous"):
            release.reviewed_pins(self.existing, self.candidates)

    def test_binary_header_not_filename_determines_architecture(self):
        self.assertEqual(release.binary_target(elf()), "linux-x86_64")
        self.assertEqual(release.binary_target(elf(183)), "linux-aarch64")
        macho = b"\xcf\xfa\xed\xfe" + (0x100000C).to_bytes(4, "little") + bytes(24)
        self.assertEqual(release.binary_target(macho), "macos-aarch64")
        with self.assertRaisesRegex(ValueError, "native ELF or Mach-O"):
            release.binary_target(b"#!/bin/sh\nexit 0\n")

    def test_linux_requires_both_explicit_flag_and_ci_environment(self):
        for flag, environment in ((False, {}), (True, {}), (False, {"GITHUB_ACTIONS": "true"}), (True, {"GITHUB_ACTIONS": "false"})):
            with self.subTest(flag=flag, environment=environment), patch.dict(os.environ, environment, clear=True):
                with self.assertRaisesRegex(ValueError, "--disposable-ci"):
                    release.execution_mode("linux-x86_64", flag)
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}, clear=True):
            sandbox, description = release.execution_mode("linux-x86_64", True)
            self.assertFalse(sandbox)
            self.assertIn("no OS sandbox", description)

    def test_macos_never_falls_back_from_missing_sandbox(self):
        with patch.object(release.shutil, "which", return_value=None), patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}):
            with self.assertRaisesRegex(ValueError, "requires sandbox-exec"):
                release.execution_mode("macos-aarch64", True)

    def test_target_mismatch_fails_before_network_or_behavior(self):
        with tempfile.TemporaryDirectory() as temporary:
            runtime = Path(temporary) / "runtime"
            runtime.write_bytes(elf())
            with patch.object(release, "host_target", return_value="linux-x86_64"), patch.object(package_catalog, "download") as download, patch.object(package_catalog, "run") as run:
                with self.assertRaisesRegex(ValueError, "mismatch"):
                    release.generate(runtime, "linux-aarch64", True)
                download.assert_not_called()
                run.assert_not_called()

    def generate_fixture(self, *, compile_fails=False, smoke_fails=False, corrupt_wheel=False, changed_hook=False):
        """Mock artifact IO and compiler/behavior processes, retaining merge logic."""
        pin = next(r for r in self.existing["records"] if r["name"] == "tzdata")
        real_digest = package_catalog.digest
        def fixture_digest(data):
            if data == b"wheel-fixture":
                return "0" * 64 if corrupt_wheel else release.TZDATA_WHEEL
            if changed_hook and data.startswith(b'"""Test tzdata'):
                return "0" * 64
            return real_digest(data)
        compile_result = subprocess.CompletedProcess([], int(compile_fails), "", "SyntaxError: unsupported" if compile_fails else "")
        smoke_result = subprocess.CompletedProcess([], int(smoke_fails), "", "failed" if smoke_fails else "")
        with tempfile.TemporaryDirectory() as temporary:
            runtime = Path(temporary) / "runtime"
            runtime.write_bytes(elf())
            def sources(wheel, root):
                return [root / "__init__.py"]
            with patch.object(release, "host_target", return_value="linux-x86_64"), patch.object(release, "execution_mode", return_value=(False, "disposable CI fixture")), patch.object(release, "reviewed_pins", return_value=[(self.tzdata, pin)]), patch.object(package_catalog, "verify_syntax_checker"), patch.object(package_catalog, "download", return_value=b"wheel-fixture"), patch.object(package_catalog, "digest", side_effect=fixture_digest), patch.object(package_catalog, "unpack", side_effect=sources), patch.object(package_catalog, "check_syntax", return_value=compile_result), patch.object(package_catalog, "run", return_value=smoke_result) as run:
                result = release.generate(runtime, "linux-x86_64", True)
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.kwargs, {"sandbox": False})
                return result

    def test_merge_preserves_history_and_records_exact_new_identity(self):
        result = self.generate_fixture()
        self.assertEqual(result["records"][:-1], self.existing["records"])
        fresh = result["records"][-1]
        self.assertEqual(fresh["runtime_sha256"], package_catalog.digest(elf()))
        self.assertEqual(fresh["target"], "linux-x86_64")
        self.assertEqual(fresh["status"], "tested")
        self.assertEqual(fresh["wheel_sha256"], release.TZDATA_WHEEL)
        self.assertEqual(fresh["smoke"]["sha256"], release.TZDATA_SMOKE)

    def test_failed_required_compilation_cannot_emit_catalog(self):
        with self.assertRaisesRegex(ValueError, "tested tzdata evidence is absent"):
            self.generate_fixture(compile_fails=True)

    def test_failed_required_smoke_cannot_emit_catalog(self):
        with self.assertRaisesRegex(ValueError, "smoke failed"):
            self.generate_fixture(smoke_fails=True)

    def test_download_hash_mismatch_cannot_emit_catalog(self):
        with self.assertRaisesRegex(ValueError, "wheel hash mismatch"):
            self.generate_fixture(corrupt_wheel=True)

    def test_changed_reviewed_hook_cannot_emit_catalog(self):
        with self.assertRaisesRegex(ValueError, "smoke.*changed"):
            self.generate_fixture(changed_hook=True)


if __name__ == "__main__":
    unittest.main()
