"""A runtime comparison must not silently compare different package releases."""
import copy
import json
from pathlib import Path
import unittest

import compare_package_audits as comparison
from package_popularity_audit import cache_key


class AuditComparisonTests(unittest.TestCase):
    def setUp(self):
        self.before = json.loads((Path(__file__).resolve().parents[1] / "compatibility/packages/popularity-audit.json").read_text())
        record = next(record for record in self.before["records"] if record["category"] == "syntax")
        self.before.update(records=[record], completed_count=1, requested_count=1, counts={"syntax": 1})
        self.after = copy.deepcopy(self.before)
        self.after["runtime_sha256"] = "a" * 64
        self.after["cache_key"] = cache_key(self.after["snapshot_sha256"], self.after["runtime_sha256"], self.after["audit_policy"])

    def test_same_artifact_can_change_verdict(self):
        self.after["records"][0].update(category="unverified", status="unverified")
        self.after["counts"] = {"unverified": 1}
        result = comparison.compare(self.before, self.after, "before", "after")
        self.assertEqual(result["transitions"], {"syntax -> unverified": 1})
        self.assertEqual(result["same_pinned_metadata_count"], 1)

    def test_version_artifact_metadata_and_ranking_drift_are_rejected(self):
        for field in ("version", "metadata_sha256", "selected_artifact_filename", "artifact_declared_sha256", "wheel_sha256", "source_url", "rank", "downloads"):
            with self.subTest(field=field):
                changed = copy.deepcopy(self.after)
                changed["records"][0][field] = "changed"
                with self.assertRaises(ValueError):
                    comparison.compare(self.before, changed, "before", "after")

    def test_partial_audit_is_rejected(self):
        self.after["complete"] = False
        with self.assertRaisesRegex(ValueError, "complete"):
            comparison.compare(self.before, self.after, "before", "after")

    def test_missing_original_metadata_is_explicit(self):
        self.before["records"][0].pop("metadata_sha256")
        result = comparison.compare(self.before, self.after, "before", "after")
        self.assertEqual(result["same_pinned_metadata_count"], 0)
        self.assertEqual(result["missing_baseline_metadata"], [self.before["records"][0]["name"]])

    def test_checker_wrapper_changes_do_not_count_as_new_blockers(self):
        original = {"first_blocker": {"file": "package.py", "diagnostic": 'File "<string>", line 1\nFile "package.py", line 10\nSyntaxError: expected expression\nerror: execution failed'}}
        changed = {"first_blocker": {"file": "package.py", "diagnostic": 'File "<staging>/package.py", line 10\nSyntaxError: expected expression\nerror: compilation failed'}}
        self.assertEqual(comparison.blocker_identity(original), comparison.blocker_identity(changed))


if __name__ == "__main__":
    unittest.main()
