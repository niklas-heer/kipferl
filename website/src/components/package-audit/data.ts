import { readFileSync } from "node:fs";
import path from "node:path";
import type { AuditData, AuditRecord } from "./types";

function object(value: unknown, label: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Invalid package audit ${label}`);
  }
  return value as Record<string, unknown>;
}

function text(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Missing package audit ${label}`);
  }
  return value;
}

export function parseAudit(value: unknown): AuditData {
  const data = object(value, "document");
  if (data.schema_version !== 1 || !Array.isArray(data.records)) {
    throw new Error("Unsupported package audit schema");
  }
  const source = object(data.ranking_source, "ranking source");
  const ranks = new Set<number>();
  const names = new Set<string>();
  const records = data.records.map((value): AuditRecord => {
    const row = object(value, "record");
    const rank = row.rank;
    const name = text(row.name, "package name");
    if (
      typeof rank !== "number" ||
      !Number.isInteger(rank) ||
      rank < 1 ||
      ranks.has(rank) ||
      names.has(name)
    ) {
      throw new Error("Package audit ranks and names must be unique");
    }
    ranks.add(rank);
    names.add(name);
    const status = row.status;
    if (!["tested", "incompatible", "unverified"].includes(String(status))) {
      throw new Error(`Invalid package audit status for ${name}`);
    }
    const blocker = row.first_blocker
      ? object(row.first_blocker, "first blocker")
      : null;
    return {
      rank,
      name,
      version: typeof row.version === "string" ? row.version : "Not resolved",
      status: status as AuditRecord["status"],
      category: text(row.category, "category"),
      evidence: text(row.evidence, "evidence"),
      blocker: blocker
        ? `${typeof blocker.file === "string" ? `${blocker.file}\n` : ""}${text(blocker.diagnostic, "blocker diagnostic")}`
        : null,
      wheel: typeof row.wheel_filename === "string" ? row.wheel_filename : null,
      artifactVerified: row.artifact_verified === true,
    };
  });
  const requestedCount = data.requested_count;
  if (
    typeof requestedCount !== "number" ||
    !Number.isInteger(requestedCount) ||
    requestedCount < records.length ||
    requestedCount < 1 ||
    data.completed_count !== records.length
  ) {
    throw new Error("Package audit coverage counts disagree with its records");
  }
  return {
    records: records.sort((a, b) => a.rank - b.rank),
    rankingUrl: text(source.url, "ranking URL"),
    retrievedAt: text(source.retrieved_at, "ranking retrieval date"),
    runtimeHash: text(data.runtime_sha256, "runtime hash"),
    target: text(data.target, "target"),
    complete: data.complete === true && requestedCount === records.length,
    requestedCount,
    windowStart: text(source.window_start, "ranking window start"),
    windowEnd: text(source.window_end_exclusive, "ranking window end"),
  };
}

export function loadAudit(): AuditData {
  // Read the canonical evidence at build time; no copied website dataset can drift.
  const source = path.resolve(
    process.cwd(),
    "../compatibility/packages/popularity-audit.json",
  );
  return parseAudit(JSON.parse(readFileSync(source, "utf8")));
}
