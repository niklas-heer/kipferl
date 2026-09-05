"""Offline regressions for package popularity screening and evidence provenance."""
import io
from contextlib import redirect_stderr
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import package_popularity_audit as audit


def artifact(filename, kind="bdist_wheel"):
    return {"filename": filename, "packagetype": kind, "url": "https://files.pythonhosted.org/example.whl", "digests": {"sha256": "a" * 64}, "requires_python": ">=3.8", "size": 1}


def wheel_bytes(name="example", version="1.0", members=None, requires_python=">=3.8"):
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as wheel:
        wheel.writestr(f"{name}-{version}.dist-info/METADATA", f"Metadata-Version: 2.1\nName: {name}\nVersion: {version}\nRequires-Python: {requires_python}\nRequires-Dist: dependency>=1\n\n")
        for filename, content in (members or {f"{name}.py": "value = 1\n"}).items():
            wheel.writestr(filename, content)
    return buffer.getvalue()


def pinned_wheel(payload):
    candidate = artifact("example-1.0-py3-none-any.whl")
    candidate["digests"]["sha256"] = audit.sha256(payload)
    candidate["size"] = len(payload)
    candidate["requires_python"] = None
    return {
        "name": "example", "version": "1.0", "metadata_url": "https://pypi.org/pypi/example/json",
        "metadata_sha256": "e" * 64, "metadata_fetched_at": "2026-09-05T00:00:00+00:00",
        "requires_python": None, "requires_dist": [], "artifact_kind": "pure", "artifact": candidate,
    }


def unreadable_wheel(kind):
    payload = bytearray(wheel_bytes())
    local = payload.index(b"PK\x03\x04")
    central = payload.index(b"PK\x01\x02")
    if kind == "encrypted":
        payload[local + 6:local + 8] = (1).to_bytes(2, "little")
        payload[central + 8:central + 10] = (1).to_bytes(2, "little")
    else:
        payload[local + 8:local + 10] = (99).to_bytes(2, "little")
        payload[central + 10:central + 12] = (99).to_bytes(2, "little")
    return bytes(payload)


class PopularityAuditTests(unittest.TestCase):
    def test_checked_in_audit_and_all_derived_artifacts_match(self):
        report = json.loads(audit.DEFAULT_REPORT.read_text())
        snapshot = (audit.DIRECTORY / "popularity.json").read_bytes()
        audit.validate_outputs(report, snapshot, audit.DEFAULT_REPORT)
        self.assertTrue(report["complete"])
        self.assertEqual(report["requested_count"], 1000)

    def test_embedded_runtime_selects_exact_host_asset(self):
        for operating_system, machine, suffix in [("Darwin", "arm64", "macos-aarch64"), ("Linux", "x86_64", "linux-x86_64")]:
            with self.subTest(target=suffix), patch.object(audit.platform, "system", return_value=operating_system), patch.object(audit.platform, "machine", return_value=machine):
                self.assertEqual(audit.embedded_runtime_path(), audit.ROOT / "crates/kipferl-cli/assets" / f"pocketpy-kipferl-{suffix}")
        with patch.object(audit.platform, "system", return_value="Windows"), self.assertRaisesRegex(ValueError, "supports macOS/Linux"):
            audit.embedded_runtime_path()

    def test_explicit_and_embedded_runtimes_are_mutually_exclusive(self):
        with patch("sys.argv", ["audit", "--runtime", "/some/runtime", "--embedded-runtime"]), redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as result:
            audit.main()
        self.assertEqual(result.exception.code, 2)

    def test_artifact_selection_prefers_python3_generic_wheel(self):
        mixed = artifact("example-1.0-py2.py3-none-any.whl")
        pure = artifact("example-1.0-py3-none-any.whl")
        native = artifact("example-1.0-cp311-cp311-macosx_11_0_arm64.whl")
        kind, result = audit.choose_artifact([native, mixed, pure])
        self.assertEqual(kind, "pure")
        self.assertEqual(result, pure)

    def test_native_and_source_metadata_do_not_claim_verified_bytes(self):
        for filename, kind in [("example-1.0-cp311-cp311-manylinux_x86_64.whl", "native_only"), ("example-1.0.tar.gz", "source_only")]:
            with self.subTest(kind=kind):
                record = audit.initial_record({"rank": 1, "name": "example"})
                pin = {"artifact": artifact(filename), "artifact_kind": kind}
                with patch.object(audit, "verified_wheel", side_effect=AssertionError("must not download")):
                    result = audit.inspect_artifact(record, pin, Path("runtime"), Path("cache"))
                self.assertEqual(result["status"], "incompatible")
                self.assertEqual(result["evidence_scope"], "metadata")
                self.assertFalse(result["artifact_verified"])

    def test_metadata_pinning_survives_a_registry_latest_change(self):
        project = {"rank": 1, "name": "example"}
        raw = json.dumps({"info": {"name": "example", "version": "1.0", "requires_python": ">=3.8", "requires_dist": []}, "urls": [artifact("example-1.0-py3-none-any.whl")]}).encode()
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint = Path(temporary) / "project"
            with patch.object(audit, "download", return_value=raw) as download:
                first = audit.pin_metadata(project, checkpoint)
            download.assert_called_once()
            with patch.object(audit, "download", side_effect=AssertionError("resume must reuse pinned metadata")):
                second = audit.pin_metadata(project, checkpoint)
            self.assertEqual(first, second)
            self.assertEqual(second["version"], "1.0")

    def test_unknown_requirements_fail_as_unverified_and_keep_metadata(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        result = audit.check_requirements(record, ">=3.8", ["invalid ???"])
        self.assertEqual(result["category"], "unsupported_requirement")
        self.assertEqual(result["status"], "unverified")
        self.assertEqual(result["requires_dist"], ["invalid ???"])

    def test_python_requirement_uses_advertised_runtime_target(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        result = audit.check_requirements(record, ">=3.12", [])
        self.assertEqual(result["category"], "python_requirement")
        self.assertIn("Older releases", result["evidence"])
        self.assertFalse(result["artifact_verified"])

    def test_hash_mismatch_cannot_become_verified(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(audit, "download", return_value=b"tampered"):
            with self.assertRaisesRegex(ValueError, "SHA-256"):
                audit.verified_wheel(artifact("example-1.0-py3-none-any.whl"), Path(temporary))

    def test_wheel_metadata_is_read_from_verified_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            sources, metadata = audit.extract_sources(wheel_bytes(), Path(temporary))
            self.assertEqual(len(sources), 1)
            self.assertEqual(metadata.get_all("Requires-Dist"), ["dependency>=1"])

    def test_unsafe_archive_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            for filename in ("../outside.py", "/outside.py", "folder\\outside.py"):
                with self.subTest(filename=filename), self.assertRaisesRegex(ValueError, "unsafe"):
                    audit.extract_sources(wheel_bytes(members={filename: "value=1"}), Path(temporary))

    def test_compilation_success_stays_unverified(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        record["version"] = "1.0"
        pin = {"artifact_kind": "pure", "artifact": artifact("example-1.0-py3-none-any.whl"), "requires_python": ">=3.8", "requires_dist": []}
        result = type("Result", (), {"returncode": 0, "stdout": "", "stderr": ""})()
        with tempfile.TemporaryDirectory() as temporary, patch.object(audit, "verified_wheel", return_value=wheel_bytes()), patch.object(audit.subprocess, "run", return_value=result) as run:
            report = audit.inspect_artifact(record, pin, Path("/runtime"), Path(temporary))
        self.assertEqual(report["status"], "unverified")
        self.assertEqual(report["source_files_total"], 1)
        self.assertEqual(report["sources_checked"], 1)
        self.assertEqual(report["remaining"], 0)
        self.assertTrue(report["compilation_completed"])
        self.assertEqual(report["requirement_metadata_source"], "verified_wheel_metadata")
        arguments = run.call_args.args[0]
        self.assertEqual(arguments[:3], ["/runtime", "--check-syntax", "--"])
        self.assertTrue(arguments[3].endswith("/example.py"))
        self.assertNotIn("-c", arguments)

    def test_error_on_final_source_is_not_compilation_completion(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        record["version"] = "1.0"
        pin = pinned_wheel(wheel_bytes())
        result = type("Result", (), {"returncode": 1, "stdout": "ValueError: parser could not complete\n", "stderr": ""})()
        with tempfile.TemporaryDirectory() as temporary, patch.object(audit, "verified_wheel", return_value=wheel_bytes()), patch.object(audit.subprocess, "run", return_value=result):
            report = audit.inspect_artifact(record, pin, Path("/runtime"), Path(temporary))
        self.assertEqual(report["category"], "unverified")
        self.assertEqual(report["sources_checked"], 1)
        self.assertEqual(report["remaining"], 0)
        self.assertFalse(report["compilation_completed"])
        self.assertIsNotNone(report["first_blocker"])

    def test_no_source_wheel_blocked_by_requirements_is_not_complete(self):
        payload = wheel_bytes(members={"package.pyi": "value: int"}, requires_python=">=3.12")
        record = audit.initial_record({"rank": 1, "name": "example"})
        record["version"] = "1.0"
        with tempfile.TemporaryDirectory() as temporary, patch.object(audit, "verified_wheel", return_value=payload):
            result = audit.inspect_artifact(record, pinned_wheel(payload), Path("/runtime"), Path(temporary))
        self.assertEqual(result["category"], "python_requirement")
        self.assertEqual(result["source_files_total"], 0)
        self.assertFalse(result["compilation_completed"])

    def test_completion_flag_cannot_hide_blocker_or_missing_artifact(self):
        record = audit.finish(audit.initial_record({"rank": 1, "name": "example"}), "unverified", "No package code executed.")
        report = audit.make_report([record], {"source": {}}, "a" * 64, "b" * 64, "macos-aarch64", 1)
        record["compilation_completed"] = True
        with self.assertRaisesRegex(ValueError, "compilation completion contradicts"):
            audit.validate_report(report)
        record.pop("compilation_completed")
        with self.assertRaisesRegex(ValueError, "explicit compilation completion"):
            audit.validate_report(report)

    def test_first_syntax_failure_stops_and_reports_remaining_sources(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        record["version"] = "1.0"
        pin = {"artifact_kind": "pure", "artifact": artifact("example-1.0-py3-none-any.whl"), "requires_python": None, "requires_dist": []}
        result = type("Result", (), {"returncode": 1, "stdout": 'File "a.py", line 2\nSyntaxError: unsupported syntax\n', "stderr": "error: Python execution failed"})()
        with tempfile.TemporaryDirectory() as temporary, patch.object(audit, "verified_wheel", return_value=wheel_bytes(members={"a.py": "invalid", "b.py": "untouched"})), patch.object(audit.subprocess, "run", return_value=result) as run:
            report = audit.inspect_artifact(record, pin, Path("/runtime"), Path(temporary))
        self.assertEqual(report["category"], "syntax")
        self.assertTrue(report["artifact_verified"])
        self.assertEqual(report["first_blocker"]["file"], "a.py")
        self.assertEqual(report["remaining"], 1)
        self.assertEqual(run.call_count, 1)

    def test_report_rejects_tested_status_and_unverified_syntax_proof(self):
        record = audit.initial_record({"rank": 1, "name": "example"})
        record = audit.finish(record, "unverified", "Source was not executed.")
        report = audit.make_report([record], {"source": {}}, "a" * 64, "b" * 64, "macos-aarch64", 1)
        record["status"] = "tested"
        with self.assertRaisesRegex(ValueError, "promote"):
            audit.validate_report(report)
        record.update(status="incompatible", category="syntax")
        report["counts"] = {"syntax": 1}
        with self.assertRaisesRegex(ValueError, "verified bytes"):
            audit.validate_report(report)

    def test_encrypted_and_unsupported_zip_are_checkpointed_unverified(self):
        for kind in ("encrypted", "unsupported_compression"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temporary:
                payload = unreadable_wheel(kind)
                root = Path(temporary)
                with patch.object(audit, "pin_metadata", return_value=pinned_wheel(payload)), patch.object(audit, "download", return_value=payload), patch.object(audit.subprocess, "run", side_effect=AssertionError("must not execute")):
                    result = audit.audit_project({"rank": 1, "name": "example"}, root / "runtime", root, root / "state")
                self.assertEqual(result["category"], "unverified")
                self.assertEqual(result["status"], "unverified")
                self.assertTrue(result["artifact_verified"])
                self.assertEqual(result["sources_checked"], 0)
                self.assertEqual(json.loads((root / "state/example/result.json").read_text()), result)
                audit.make_report([result], {"source": {}}, "a" * 64, "b" * 64, "macos-aarch64", 1)

    def test_recursive_archive_walk_failure_does_not_abort_audit(self):
        payload = wheel_bytes()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(audit, "pin_metadata", return_value=pinned_wheel(payload)), patch.object(audit, "download", return_value=payload), patch.object(audit.Path, "rglob", side_effect=RecursionError("directory traversal depth exceeded")):
                result = audit.audit_project({"rank": 1, "name": "example"}, root / "runtime", root, root / "state")
            self.assertEqual(result["status"], "unverified")
            self.assertIn("directory traversal depth exceeded", result["evidence"])
            self.assertTrue((root / "state/example/result.json").is_file())

    def test_verified_wheel_requirement_conflict_keeps_verified_scope(self):
        payload = wheel_bytes(requires_python=">=3.12")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(audit, "pin_metadata", return_value=pinned_wheel(payload)), patch.object(audit, "download", return_value=payload), patch.object(audit.subprocess, "run", side_effect=AssertionError("requirement conflict must stop before compile")):
                result = audit.audit_project({"rank": 1, "name": "example"}, root / "runtime", root, root / "state")
            self.assertEqual(result["category"], "python_requirement")
            self.assertEqual(result["evidence_scope"], "verified_artifact")
            self.assertEqual(result["requirement_metadata_source"], "verified_wheel_metadata")
            self.assertTrue(result["artifact_verified"])
            report = audit.make_report([result], {"source": {}}, "a" * 64, "b" * 64, "macos-aarch64", 1)
            result["evidence_scope"] = "metadata"
            with self.assertRaisesRegex(ValueError, "verification and evidence scope"):
                audit.validate_report(report)

    def test_policy_and_parser_changes_invalidate_checkpoint_identity(self):
        original = audit.current_policy()
        original_key = audit.cache_key("a" * 64, "b" * 64, original)
        for field, value in [("max_source_files", 1), ("python_metadata_target", "3.12.0"), ("requirement_parser", "other parser"), ("version", original["version"] + 1)]:
            with self.subTest(field=field):
                changed = dict(original)
                changed[field] = value
                self.assertNotEqual(audit.cache_key("a" * 64, "b" * 64, changed), original_key)
                with tempfile.TemporaryDirectory() as temporary, self.assertRaisesRegex(ValueError, "exact original"):
                    audit.migrate_legacy(Path(temporary), Path(temporary) / "destination", changed)

    def test_metadata_seed_preserves_versions_but_never_reuses_results(self):
        payload = wheel_bytes()
        pin = pinned_wheel(payload)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            old, new = root / "old", root / "new"
            audit.atomic_json(old / "policy.json", {"snapshot_sha256": "a" * 64, "runtime_sha256": "b" * 64, "cache_key": "old-key"})
            audit.atomic_json(old / "example/metadata.json", pin)
            audit.atomic_json(old / "example/result.json", {"status": "incompatible", "evidence": "old runtime"})
            with patch.object(audit, "download", side_effect=AssertionError("seeding must not fetch latest metadata")):
                summary = audit.seed_metadata(old, new, "a" * 64, [{"name": "example"}, {"name": "missing"}])
                copied = audit.pin_metadata({"name": "example"}, new / "example")
            self.assertEqual(copied, pin)
            self.assertEqual(summary["seeded_count"], 1)
            self.assertEqual(summary["missing_projects"], ["missing"])
            self.assertFalse((new / "example/result.json").exists())
            self.assertEqual((old / "example/metadata.json").read_bytes(), (new / "example/metadata.json").read_bytes())

    def test_metadata_seed_rejects_other_snapshot_or_conflicting_pin(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            old, new = root / "old", root / "new"
            audit.atomic_json(old / "policy.json", {"snapshot_sha256": "a" * 64})
            audit.atomic_json(old / "example/metadata.json", pinned_wheel(wheel_bytes()))
            with self.assertRaisesRegex(ValueError, "different popularity snapshot"):
                audit.seed_metadata(old, new, "b" * 64, [{"name": "example"}])
            conflict = pinned_wheel(wheel_bytes())
            conflict["version"] = "2.0"
            audit.atomic_json(new / "example/metadata.json", conflict)
            with self.assertRaisesRegex(ValueError, "immutable pin"):
                audit.seed_metadata(old, new, "a" * 64, [{"name": "example"}])

    def test_module_checker_cannot_reuse_legacy_dynamic_compile_results(self):
        policy = audit.current_policy()
        self.assertEqual(policy["version"], 2)
        self.assertIn("dynamic=false", policy["syntax_checker"])
        self.assertNotEqual(audit.cache_key("a" * 64, "b" * 64, policy), audit.cache_key("a" * 64, "b" * 64, audit.LEGACY_POLICY_V1))
        with tempfile.TemporaryDirectory() as temporary, self.assertRaisesRegex(ValueError, "exact original"):
            audit.migrate_legacy(Path(temporary), Path(temporary) / "new", policy)

    def test_offline_check_links_snapshot_csv_and_catalog_to_report(self):
        project = {"rank": 1, "name": "example", "downloads": 42}
        snapshot_bytes = json.dumps({"source": {}, "projects": [project]}).encode()
        record = audit.finish(audit.initial_record(project), "unverified", "No package code was executed.")
        report = audit.make_report([record], json.loads(snapshot_bytes), audit.sha256(snapshot_bytes), "b" * 64, "macos-aarch64", 1)
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "popularity-audit.json"
            audit.export(report, destination)
            audit.validate_outputs(report, snapshot_bytes, destination)
            record["downloads"] = 43
            with self.assertRaisesRegex(ValueError, "download count"):
                audit.validate_outputs(report, snapshot_bytes, destination)
            record["downloads"] = 42
            destination.with_suffix(".csv").write_text("drift")
            with self.assertRaisesRegex(ValueError, "CSV"):
                audit.validate_outputs(report, snapshot_bytes, destination)
            audit.export(report, destination)
            destination.with_name("popularity-catalog.json").write_text('{"schema_version": 1, "records": [{"invented": true}]}')
            with self.assertRaisesRegex(ValueError, "syntax catalog"):
                audit.validate_outputs(report, snapshot_bytes, destination)

    def test_policy_tampering_is_detected_even_with_valid_counts(self):
        record = audit.finish(audit.initial_record({"rank": 1, "name": "example"}), "unverified", "No code executed.")
        report = audit.make_report([record], {"source": {}}, "a" * 64, "b" * 64, "macos-aarch64", 1)
        report["audit_policy"]["max_source_files"] = 1
        with self.assertRaisesRegex(ValueError, "policy digest"):
            audit.validate_report(report)

    def test_download_denies_untrusted_host_before_network(self):
        with patch.object(audit.urllib.request, "build_opener", side_effect=AssertionError("must not connect")):
            with self.assertRaisesRegex(ValueError, "official PyPI"):
                audit.download("https://pypi.org.evil.example/wheel.whl", 1024)


if __name__ == "__main__":
    unittest.main()
