import { readFileSync } from "node:fs";
import path from "node:path";

export type CompatibilityStatus =
  | "verified"
  | "limited"
  | "unsupported"
  | "untested";
export interface VerifiedPackage {
  name: string;
  version: string;
  status: CompatibilityStatus;
  kind: "library" | "data" | "typing" | "dependency-only";
  summary: string;
  scope: string[];
  limitations: string[];
  platforms: {
    target: string;
    runtime_sha256: string;
    status: CompatibilityStatus;
    evidence: string;
  }[];
  installCommand?: string;
  examplePath?: string;
  workflow?: string;
  reason?: string;
  evidence: string;
  pypiRank: number;
}
export interface VerifiedPackages {
  release: string;
  generatedAt: string;
  records: VerifiedPackage[];
}
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Invalid verified package object");
  return value as Record<string, unknown>;
}
function text(value: unknown): string {
  if (typeof value !== "string" || !value.trim())
    throw new Error("Missing verified package text");
  return value;
}
function strings(value: unknown): string[] {
  if (!Array.isArray(value)) throw new Error("Missing verified package scope");
  return value.map(text);
}
function status(value: unknown): CompatibilityStatus {
  if (
    value !== "verified" &&
    value !== "limited" &&
    value !== "unsupported" &&
    value !== "untested"
  )
    throw new Error("Unknown package compatibility status");
  return value;
}
export function parseVerifiedPackages(value: unknown): VerifiedPackages {
  const data = object(value);
  if (data.schema_version !== 1 || !Array.isArray(data.records))
    throw new Error("Unsupported verified packages schema");
  const names = new Set<string>();
  const records = data.records.map((value): VerifiedPackage => {
    const row = object(value);
    const name = text(row.name);
    if (names.has(name)) throw new Error("Duplicate verified package");
    names.add(name);
    const result = status(row.status);
    if (
      row.kind !== "library" &&
      row.kind !== "data" &&
      row.kind !== "typing" &&
      row.kind !== "dependency-only"
    )
      throw new Error("Unknown package kind");
    if (!Array.isArray(row.platforms))
      throw new Error("Missing verified platform coverage");
    const platforms = row.platforms.map((value) => {
      const platform = object(value);
      const hash = text(platform.runtime_sha256);
      if (!/^[a-f0-9]{64}$/.test(hash))
        throw new Error("Invalid verified runtime hash");
      return {
        target: text(platform.target),
        runtime_sha256: hash,
        status: status(platform.status),
        evidence: text(platform.evidence),
      };
    });
    const scope = strings(row.scope);
    if (
      (result === "verified" || result === "limited") &&
      (scope.length === 0 ||
        !platforms.some((platform) => platform.status === result))
    )
      throw new Error(
        "Working package requires matching scope and platform evidence",
      );
    if (
      result === "verified" &&
      (row.kind === "typing" || row.kind === "dependency-only")
    )
      throw new Error(
        "Metadata-only distribution cannot be a verified library",
      );
    if (
      typeof row.pypi_rank !== "number" ||
      !Number.isInteger(row.pypi_rank) ||
      row.pypi_rank < 1
    )
      throw new Error("Missing package popularity rank");
    const evidence = object(row.evidence);
    const examplePath =
      typeof evidence.hook === "string" &&
      /^smoke\/verify_[a-z0-9_]+\.py$/.test(evidence.hook)
        ? evidence.hook
        : undefined;
    return {
      name,
      version: text(row.version),
      status: result,
      kind: row.kind,
      summary: text(row.summary),
      scope,
      limitations: strings(row.limitations),
      platforms,
      examplePath,
      installCommand:
        typeof row.install_command === "string"
          ? row.install_command
          : undefined,
      workflow: typeof row.workflow === "string" ? row.workflow : undefined,
      reason: typeof row.reason === "string" ? row.reason : undefined,
      evidence: JSON.stringify(row.evidence, null, 2),
      pypiRank: row.pypi_rank,
    };
  });
  return {
    release: text(data.release),
    generatedAt: text(data.generated_at),
    records,
  };
}
export function loadVerifiedPackages(): VerifiedPackages {
  return parseVerifiedPackages(
    JSON.parse(
      readFileSync(
        path.resolve(
          process.cwd(),
          "../compatibility/packages/verified-packages.json",
        ),
        "utf8",
      ),
    ),
  );
}
