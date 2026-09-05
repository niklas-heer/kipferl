import assert from "node:assert/strict";
import { test } from "node:test";
import { parseAudit } from "./data";

function fixture() {
  return {
    schema_version: 1,
    complete: true,
    requested_count: 1000,
    completed_count: 2,
    ranking_source: {
      url: "https://example.com/ranking",
      retrieved_at: "2026-09-05T12:00:00Z",
      window_start: "2026-08-01",
      window_end_exclusive: "2026-09-01",
    },
    runtime_sha256: "a".repeat(64),
    target: "macos-aarch64",
    records: [
      {
        rank: 2,
        name: "compile-only",
        version: "1.0",
        status: "unverified",
        category: "unverified",
        evidence: "Sources compiled; behavior was not executed.",
        artifact_verified: true,
        wheel_filename: "compile_only-1.0-py3-none-any.whl",
        first_blocker: null,
      },
      {
        rank: 1,
        name: "unavailable",
        version: null,
        status: "unverified",
        category: "network",
        evidence: "Index download failed.",
        artifact_verified: false,
        wheel_filename: null,
        first_blocker: null,
      },
    ],
  };
}

test("preserves incomplete and compilation-only evidence without promoting status", () => {
  const data = parseAudit(fixture());
  assert.deepEqual(
    data.records.map((record) => record.rank),
    [1, 2],
  );
  assert.equal(data.records[0]?.version, "Not resolved");
  assert.equal(data.records[0]?.artifactVerified, false);
  assert.equal(data.records[1]?.artifactVerified, true);
  assert.equal(data.records[1]?.status, "unverified");
});

test("rejects duplicate ranks rather than publishing misleading group counts", () => {
  const data = fixture();
  data.records.push({ ...data.records[0], name: "another-package" });
  assert.throws(() => parseAudit(data), /unique/);
});

test("rejects unsupported statuses rather than counting them as passing", () => {
  const data = fixture();
  data.records[0].status = "compatible";
  assert.throws(() => parseAudit(data), /Invalid package audit status/);
});

test("retains a partial checkpoint marker even when some records are available", () => {
  const parsed = parseAudit({ ...fixture(), complete: false });
  assert.equal(parsed.complete, false);
  assert.equal(parsed.requestedCount, 1000);
  assert.equal(parsed.records.length, 2);
});

test("does not display a complete result when requested coverage is missing", () => {
  assert.equal(parseAudit(fixture()).complete, false);
});

test("rejects coverage metadata that disagrees with the actual rows", () => {
  assert.throws(
    () => parseAudit({ ...fixture(), completed_count: 999 }),
    /coverage counts/,
  );
});
