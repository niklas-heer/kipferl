"""Offline regressions for trustworthy popularity ordering and provenance."""
import json
import unittest
from snapshot_package_popularity import snapshot


def source(rows, **extra):
    return json.dumps({"last_update": "2026-09-01 06:34:08", "source": "ClickHouse", "rows": rows, **extra}).encode()


class PopularitySnapshotTests(unittest.TestCase):
    def test_snapshot_preserves_order_counts_and_window(self):
        value = snapshot(source([{"project": "typing_extensions", "download_count": 50}, {"project": "example", "download_count": 20}]), 2, "2026-09-05T12:00:00+00:00")
        self.assertEqual(value["projects"], [{"rank": 1, "name": "typing-extensions", "downloads": 50}, {"rank": 2, "name": "example", "downloads": 20}])
        self.assertEqual(value["source"]["window_start"], "2026-08-01")
        self.assertEqual(value["source"]["window_end_exclusive"], "2026-09-01")
        self.assertRegex(value["source"]["sha256"], r"^[0-9a-f]{64}$")

    def test_duplicate_normalized_names_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            snapshot(source([{"project": "some_pkg", "download_count": 2}, {"project": "some-pkg", "download_count": 1}]), 2, "now")

    def test_invalid_count_order_or_name_rejected(self):
        for rows in [
            [{"project": "first", "download_count": 1}, {"project": "second", "download_count": 2}],
            [{"project": "first", "download_count": True}],
            [{"project": "first", "download_count": -1}],
            [{"project": "../first", "download_count": 3}],
        ]:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                snapshot(source(rows), len(rows), "now")

    def test_missing_rows_or_changed_backend_rejected(self):
        with self.assertRaisesRegex(ValueError, "fewer rows"):
            snapshot(source([]), 1, "now")
        with self.assertRaisesRegex(ValueError, "source changed"):
            snapshot(source([{"project": "first", "download_count": 3}], source="changed"), 1, "now")


if __name__ == "__main__":
    unittest.main()
