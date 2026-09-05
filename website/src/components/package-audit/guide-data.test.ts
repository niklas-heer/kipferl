import assert from "node:assert/strict";
import { test } from "node:test";
import { parseSupportPriorities } from "./support-data";
import { parseVerifiedPackages } from "./verified-data";

function packageFixture() {
  return {
    schema_version: 1,
    release: "0.7.2",
    generated_at: "2026-09-06",
    records: [
      {
        name: "example",
        version: "1.0",
        status: "verified",
        kind: "library",
        summary: "A narrow workflow passed.",
        scope: ["Install, run the reviewed example, and package it."],
        limitations: ["Other APIs were not tested."],
        platforms: [
          {
            target: "macos-aarch64",
            runtime_sha256: "a".repeat(64),
            status: "verified",
            evidence: "Reviewed workflow passed.",
          },
        ],
        evidence: { standalone: true },
        pypi_rank: 1,
      },
    ],
  };
}

test("guide retains the exact scope and only the platforms actually recorded", () => {
  const result = parseVerifiedPackages(packageFixture());
  assert.equal(result.records[0].platforms.length, 1);
  assert.equal(result.records[0].platforms[0].target, "macos-aarch64");
  assert.deepEqual(result.records[0].limitations, [
    "Other APIs were not tested.",
  ]);
});

test("a verified badge needs matching platform evidence and a nonempty scope", () => {
  const missingPlatform = packageFixture();
  missingPlatform.records[0].platforms = [];
  assert.throws(() => parseVerifiedPackages(missingPlatform), /matching scope/);
  const missingScope = packageFixture();
  missingScope.records[0].scope = [];
  assert.throws(() => parseVerifiedPackages(missingScope), /matching scope/);
});

test("typing distributions cannot become verified working libraries", () => {
  const fixture = packageFixture();
  fixture.records[0].kind = "typing";
  assert.throws(() => parseVerifiedPackages(fixture), /Metadata-only/);
});

test("untested results do not inherit verification from compilation", () => {
  const fixture = packageFixture();
  fixture.records[0].status = "untested";
  fixture.records[0].scope = [];
  fixture.records[0].platforms = [];
  fixture.records[0].evidence = { standalone: false };
  assert.equal(parseVerifiedPackages(fixture).records[0].status, "untested");
});

function prioritiesFixture() {
  return {
    schema_version: 1,
    kind: "implementation_priorities_not_compatibility",
    release: "0.7.2",
    evaluated_at: "2026-09-06",
    methodology: { formula: "4*demand+6*usefulness+3*reach-5*effort" },
    records: Array.from({ length: 100 }, (_, index) => ({
      priority: index + 1,
      name: `package-${index}`,
      version: "1.0",
      pypi_rank: index + 1,
      category: "CLI",
      use_case: "Read configuration",
      acceptance_test: "Load a configuration fixture",
      why: "A useful workflow",
      blocker_summary: "Missing language feature",
      audit: { category: "syntax" },
      score: { demand: 1, usefulness: 2, reach: 1, effort: 2, total: 9 },
    })),
  };
}

test("priority rankings remain a complete development shortlist", () => {
  const result = parseSupportPriorities(prioritiesFixture());
  assert.equal(result.records.length, 100);
  assert.equal(result.records[99].priority, 100);
});

test("priority rankings reject missing entries, duplicate ranks, and score drift", () => {
  const missing = prioritiesFixture();
  missing.records.pop();
  assert.throws(() => parseSupportPriorities(missing), /exactly 100/);
  const duplicate = prioritiesFixture();
  duplicate.records[1].priority = 1;
  assert.throws(() => parseSupportPriorities(duplicate), /Duplicate/);
  const drift = prioritiesFixture();
  drift.records[0].score.total = 10;
  assert.throws(() => parseSupportPriorities(drift), /disagrees/);
});
