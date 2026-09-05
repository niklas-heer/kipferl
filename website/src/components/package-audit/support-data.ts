import { readFileSync } from "node:fs";
import path from "node:path";
import {
  type CompatibilityStatus,
  loadVerifiedPackages,
} from "./verified-data";

export interface SupportPriority {
  priority: number;
  name: string;
  version: string;
  pypiRank: number;
  category: string;
  useCase: string;
  acceptanceTest: string;
  why: string;
  blockerSummary: string;
  effort: number;
  score: number;
  blockerCategory: string;
  currentStatus?: CompatibilityStatus;
  currentSummary?: string;
}
export interface SupportPriorities {
  release: string;
  evaluatedAt: string;
  records: SupportPriority[];
  methodology: string;
}
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Invalid support priorities object");
  return value as Record<string, unknown>;
}
function text(value: unknown): string {
  if (typeof value !== "string" || !value.trim())
    throw new Error("Missing support priorities text");
  return value;
}
function integer(value: unknown, min = 1, max = 1000): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < min ||
    value > max
  )
    throw new Error("Invalid support priorities number");
  return value;
}
export function parseSupportPriorities(value: unknown): SupportPriorities {
  const data = object(value);
  if (
    data.schema_version !== 1 ||
    data.kind !== "implementation_priorities_not_compatibility" ||
    !Array.isArray(data.records) ||
    data.records.length !== 100
  )
    throw new Error("Expected exactly 100 implementation priorities");
  const names = new Set<string>();
  const ranks = new Set<number>();
  const records = data.records
    .map((value): SupportPriority => {
      const row = object(value);
      const priority = integer(row.priority, 1, 100);
      const name = text(row.name);
      if (names.has(name) || ranks.has(priority))
        throw new Error("Duplicate support priority");
      names.add(name);
      ranks.add(priority);
      const score = object(row.score);
      const demand = integer(score.demand, 1, 5);
      const usefulness = integer(score.usefulness, 1, 5);
      const reach = integer(score.reach, 1, 5);
      const effort = integer(score.effort, 1, 5);
      const total = integer(score.total, -25, 100);
      if (total !== 4 * demand + 6 * usefulness + 3 * reach - 5 * effort)
        throw new Error("Support priority score disagrees with its factors");
      return {
        priority,
        name,
        version: text(row.version),
        pypiRank: integer(row.pypi_rank),
        category: text(row.category),
        useCase: text(row.use_case),
        acceptanceTest: text(row.acceptance_test),
        why: text(row.why),
        blockerSummary: text(row.blocker_summary),
        effort,
        score: total,
        blockerCategory: text(object(row.audit).category),
      };
    })
    .sort((a, b) => a.priority - b.priority);
  return {
    release: text(data.release),
    evaluatedAt: text(data.evaluated_at),
    records,
    methodology: JSON.stringify(data.methodology, null, 2),
  };
}
export function loadSupportPriorities(): SupportPriorities {
  const priorities = parseSupportPriorities(
    JSON.parse(
      readFileSync(
        path.resolve(
          process.cwd(),
          "../compatibility/packages/support-priorities.json",
        ),
        "utf8",
      ),
    ),
  );
  const reviewed = loadVerifiedPackages();
  return {
    ...priorities,
    records: priorities.records.map((row) => {
      const current = reviewed.records.find(
        (item) => item.name === row.name && item.version === row.version,
      );
      return {
        ...row,
        currentStatus: current?.status,
        currentSummary: current?.summary,
      };
    }),
  };
}
