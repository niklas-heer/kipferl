export interface AuditRecord {
  rank: number;
  name: string;
  version: string;
  status: "tested" | "incompatible" | "unverified";
  category: string;
  evidence: string;
  blocker: string | null;
  wheel: string | null;
  artifactVerified: boolean;
}

export interface AuditData {
  records: AuditRecord[];
  rankingUrl: string;
  retrievedAt: string;
  runtimeHash: string;
  target: string;
  complete: boolean;
  requestedCount: number;
  windowStart: string;
  windowEnd: string;
}

export const categoryLabels: Record<string, string> = {
  native_only: "Unsupported wheel / native files",
  source_only: "Source distribution only",
  python_requirement: "Python version constraint",
  unsupported_requirement: "Unsupported dependency requirement",
  syntax: "Source compilation failed",
  limits: "Size or execution limit",
  network: "Download or index failure",
  unverified: "No blocker found in completed checks",
};

export function categoryLabel(category: string): string {
  return categoryLabels[category] ?? category.replaceAll("_", " ");
}
