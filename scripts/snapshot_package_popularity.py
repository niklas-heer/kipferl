#!/usr/bin/env python3
"""Pin a download-ranked PyPI package sample with its original source provenance."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
SOURCE_URL = "https://hugovk.dev/top-pypi-packages/top-pypi-packages.min.json"
QUERY_URL = "https://github.com/hugovk/top-pypi-packages/blob/main/clickhouse.py"
NAME = re.compile(r"[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?\Z")


def snapshot(raw: bytes, limit: int, retrieved_at: str) -> dict:
    if not 1 <= limit <= 15000:
        raise ValueError("sample size must be between 1 and 15000")
    original = json.loads(raw)
    rows = original.get("rows")
    if not isinstance(rows, list) or len(rows) < limit:
        raise ValueError("ranking source has fewer rows than requested")
    updated = dt.datetime.strptime(original["last_update"], "%Y-%m-%d %H:%M:%S").replace(tzinfo=dt.timezone.utc)
    month_end = updated.date().replace(day=1)
    month_start = (month_end - dt.timedelta(days=1)).replace(day=1)
    if original.get("source") != "ClickHouse":
        raise ValueError("ranking source changed; verify its collection window before refreshing")
    projects = []
    seen = set()
    previous = None
    for rank, row in enumerate(rows[:limit], 1):
        name, downloads = row.get("project"), row.get("download_count")
        if not isinstance(name, str) or not NAME.fullmatch(name):
            raise ValueError("invalid package name in ranking")
        name = re.sub(r"[-_.]+", "-", name)
        if name in seen:
            raise ValueError("duplicate normalized package name in ranking")
        if type(downloads) is not int or downloads < 0:
            raise ValueError("download counts must be nonnegative integers")
        if previous is not None and downloads > previous:
            raise ValueError("ranking is not in descending download order")
        projects.append({"rank": rank, "name": name, "downloads": downloads})
        seen.add(name)
        previous = downloads
    return {
        "schema_version": 1,
        "source": {
            "name": "hugovk/top-pypi-packages",
            "url": SOURCE_URL,
            "query_url": QUERY_URL,
            "backend": original["source"],
            "last_update": updated.isoformat(),
            "retrieved_at": retrieved_at,
            "sha256": hashlib.sha256(raw).hexdigest(),
            "window_start": month_start.isoformat(),
            "window_end_exclusive": month_end.isoformat(),
            "metric": "Monthly PyPI download counts; not unique users or direct application adoption.",
            "window_basis": "The source ClickHouse query selects the previous calendar month relative to its last_update date.",
            "source_rows": len(rows),
        },
        "projects": projects,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--limit", type=int, default=1000)
    parser.add_argument("--input", type=Path, help="use a previously downloaded original source JSON")
    parser.add_argument("--output", type=Path, default=ROOT / "compatibility/packages/popularity.json")
    args = parser.parse_args()
    if args.input:
        raw = args.input.read_bytes()
    else:
        with urllib.request.urlopen(SOURCE_URL, timeout=30) as response:
            if response.url != SOURCE_URL:
                raise ValueError("ranking source redirected; verify the new location")
            raw = response.read(2 * 1024 * 1024 + 1)
    if len(raw) > 2 * 1024 * 1024:
        raise ValueError("ranking source exceeds 2 MiB")
    result = snapshot(raw, args.limit, dt.datetime.now(dt.timezone.utc).isoformat())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Pinned {len(result['projects'])} packages ranked by downloads for {result['source']['window_start']} to {result['source']['window_end_exclusive']} (exclusive)")


if __name__ == "__main__":
    main()
