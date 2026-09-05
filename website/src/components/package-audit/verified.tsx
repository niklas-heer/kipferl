"use client";

import {
  CheckCircle2,
  CircleHelp,
  CircleMinus,
  TriangleAlert,
} from "lucide-react";
import { useId, useState } from "react";
import type { CompatibilityStatus, VerifiedPackages } from "./verified-data";

const compatibilityLabels: Record<CompatibilityStatus, string> = {
  verified: "Verified",
  limited: "Limited",
  unsupported: "Unsupported",
  untested: "Untested",
};

const badges = {
  verified:
    "border-emerald-500/30 bg-emerald-500/10 text-emerald-800 dark:text-emerald-300",
  limited:
    "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300",
  unsupported:
    "border-rose-500/30 bg-rose-500/10 text-rose-800 dark:text-rose-300",
  untested: "border-fd-border bg-fd-muted text-fd-muted-foreground",
};
const icons = {
  verified: CheckCircle2,
  limited: TriangleAlert,
  unsupported: CircleMinus,
  untested: CircleHelp,
};
const explanations = {
  verified:
    "The stated workflow passed installation, execution, and standalone packaging.",
  limited:
    "A useful, narrower scope works. Read the limits before choosing it.",
  unsupported: "A concrete blocker prevents the selected package or workflow.",
  untested: "There is not enough behavior evidence to recommend it yet.",
};
const order: Record<CompatibilityStatus, number> = {
  verified: 0,
  limited: 1,
  unsupported: 2,
  untested: 3,
};
const platforms: Record<string, string> = {
  "macos-aarch64": "macOS · Apple Silicon",
  "macos-x86_64": "macOS · Intel",
  "linux-aarch64": "Linux · ARM64",
  "linux-x86_64": "Linux · x86_64",
};
const control =
  "w-full rounded-xl border border-fd-border bg-fd-background px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fd-primary";

export function VerifiedGuide({ data }: { data: VerifiedPackages }) {
  const id = useId();
  const [query, setQuery] = useState("");
  const [result, setResult] = useState("usable");
  const [kind, setKind] = useState("runtime");
  const [workflow, setWorkflow] = useState("all");
  const workflows = [
    ...new Set(
      data.records.flatMap((row) => (row.workflow ? [row.workflow] : [])),
    ),
  ].sort();
  const [limit, setLimit] = useState(12);
  const runnable = data.records.filter(
    (row) => row.kind === "library" || row.kind === "data",
  );
  const metadataCount = data.records.length - runnable.length;
  const filtered = data.records
    .filter(
      (row) =>
        (kind === "all" || row.kind === "library" || row.kind === "data") &&
        (result === "all" ||
          row.status === result ||
          (result === "usable" &&
            (row.status === "verified" || row.status === "limited"))) &&
        (workflow === "all" || row.workflow === workflow) &&
        `${row.name} ${row.summary} ${row.workflow ?? ""}`
          .toLowerCase()
          .includes(query.trim().toLowerCase()),
    )
    .sort(
      (a, b) => order[a.status] - order[b.status] || a.pypiRank - b.pypiRank,
    );
  return (
    <section
      className="not-prose my-8 space-y-5"
      aria-label="Package compatibility guide"
    >
      <p className="rounded-xl border border-fd-border bg-fd-muted/30 p-4 text-sm font-medium">
        Verified packages:{" "}
        {
          runnable.filter(
            (row) => row.kind === "library" && row.status === "verified",
          ).length
        }{" "}
        libraries ·{" "}
        {
          runnable.filter(
            (row) => row.kind === "data" && row.status === "verified",
          ).length
        }{" "}
        data packages. The exact useful scope is shown on each card.
      </p>
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {(Object.keys(compatibilityLabels) as CompatibilityStatus[]).map(
          (status) => {
            const Icon = icons[status];
            return (
              <div
                key={status}
                className={`rounded-2xl border p-4 ${badges[status]}`}
              >
                <div className="flex items-center gap-2">
                  <Icon size={18} aria-hidden="true" />
                  <h3 className="font-semibold">
                    {compatibilityLabels[status]}
                  </h3>
                  <span className="ml-auto font-mono text-lg">
                    {runnable.filter((row) => row.status === status).length}
                  </span>
                </div>
                <p className="mt-2 text-xs leading-relaxed">
                  {explanations[status]}
                </p>
              </div>
            );
          },
        )}
      </div>
      <p className="text-sm leading-relaxed text-fd-muted-foreground">
        These ratings cover the reviewed packages below, not all 1,000 screened
        distributions. Counts include libraries and data packages.{" "}
        {metadataCount} typing or dependency-only distributions are kept
        separate. A badge applies only to the version, workflow, and platform
        shown.
      </p>
      <div className="grid gap-3 sm:grid-cols-2">
        <label
          htmlFor={`${id}-search`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Find a package or workflow</span>
          <input
            id={`${id}-search`}
            type="search"
            className={control}
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setLimit(12);
            }}
            placeholder="Certificates, time zones, configuration…"
          />
        </label>
        <label
          htmlFor={`${id}-result`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Compatibility</span>
          <select
            id={`${id}-result`}
            className={control}
            value={result}
            onChange={(event) => {
              setResult(event.target.value);
              setLimit(12);
            }}
          >
            <option value="usable">Verified or limited</option>
            <option value="all">All ratings</option>
            {Object.entries(compatibilityLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <label htmlFor={`${id}-kind`} className="space-y-1 text-sm font-medium">
          <span>Package type</span>
          <select
            id={`${id}-kind`}
            className={control}
            value={kind}
            onChange={(event) => {
              setKind(event.target.value);
              setLimit(12);
            }}
          >
            <option value="runtime">Libraries & data packages</option>
            <option value="all">
              Include typing & dependency-only packages
            </option>
          </select>
        </label>
        <label
          htmlFor={`${id}-workflow`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Workflow</span>
          <select
            id={`${id}-workflow`}
            className={control}
            value={workflow}
            onChange={(event) => {
              setWorkflow(event.target.value);
              setLimit(12);
            }}
          >
            <option value="all">All workflows</option>
            {workflows.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </label>
      </div>
      <output
        className="block text-sm text-fd-muted-foreground"
        aria-live="polite"
      >
        {filtered.length} matching packages · useful verified scopes shown first
      </output>
      {result !== "all" && (
        <button
          type="button"
          className="text-sm font-medium underline underline-offset-4 focus-visible:outline-2 focus-visible:outline-fd-primary"
          onClick={() => {
            setResult("all");
            setQuery("");
            setWorkflow("all");
            setLimit(12);
          }}
        >
          Show all reviewed libraries and data packages
        </button>
      )}
      <div className="grid gap-4 lg:grid-cols-2">
        {filtered.slice(0, limit).map((row) => {
          const Icon = icons[row.status];
          return (
            <article
              key={row.name}
              className="min-w-0 rounded-2xl border border-fd-border p-5"
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0">
                  <h3 className="break-words text-xl font-semibold tracking-tight">
                    {row.name}
                  </h3>
                  <p className="mt-1 font-mono text-xs text-fd-muted-foreground">
                    {row.version} · {row.kind}
                  </p>
                </div>
                <span
                  className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-semibold ${badges[row.status]}`}
                >
                  <Icon size={14} aria-hidden="true" />
                  {compatibilityLabels[row.status]}
                </span>
              </div>
              {row.workflow && row.workflow.trim() !== row.summary.trim() && (
                <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-fd-muted-foreground">
                  {row.workflow}
                </p>
              )}
              <p className="mt-2 text-sm leading-relaxed">{row.summary}</p>
              {row.scope.length > 0 && (
                <div className="mt-4 text-sm">
                  <h4 className="font-semibold">
                    {row.status === "verified" || row.status === "limited"
                      ? "What passed"
                      : "What we tried"}
                  </h4>
                  <ul className="mt-1 list-disc space-y-1 pl-4 text-fd-muted-foreground">
                    {row.scope.map((scope) => (
                      <li key={scope}>{scope}</li>
                    ))}
                  </ul>
                </div>
              )}
              {row.limitations.length > 0 && (
                <div className="mt-3 text-sm">
                  <h4 className="font-semibold">Know before you use it</h4>
                  <ul className="mt-1 list-disc space-y-1 pl-4 text-fd-muted-foreground">
                    {row.limitations.map((value) => (
                      <li key={value}>{value}</li>
                    ))}
                  </ul>
                </div>
              )}
              {row.reason && (
                <p className="mt-3 text-sm text-fd-muted-foreground">
                  {row.reason}
                </p>
              )}
              <p className="mt-4 text-xs font-medium">
                Evidence on:{" "}
                {row.platforms.length
                  ? row.platforms
                      .map(
                        (platform) =>
                          `${platforms[platform.target] ?? platform.target} (${compatibilityLabels[platform.status]})`,
                      )
                      .join("; ")
                  : "No platform behavior verification yet"}
              </p>
              {row.installCommand &&
                (row.status === "verified" || row.status === "limited") && (
                  <pre className="mt-3 overflow-x-auto rounded-lg bg-fd-muted p-3 text-xs">
                    <code>{row.installCommand}</code>
                  </pre>
                )}
              {row.examplePath &&
                (row.status === "verified" || row.status === "limited") && (
                  <a
                    className="mt-3 inline-block text-sm font-medium underline underline-offset-4"
                    href={`https://github.com/niklas-heer/kipferl/blob/main/compatibility/packages/${row.examplePath}`}
                  >
                    Read the tested example
                  </a>
                )}
              <details className="mt-4 border-t border-fd-border pt-3 text-sm">
                <summary className="cursor-pointer text-fd-muted-foreground underline underline-offset-4">
                  Exact evidence & package details
                </summary>
                <p className="mt-3">
                  Release {data.release} · Popularity #{row.pypiRank} · Recorded{" "}
                  {data.generatedAt}
                </p>
                {row.platforms.map((platform) => (
                  <div key={platform.target} className="mt-3">
                    <p>{platform.evidence}</p>
                    <p className="mt-1 break-all font-mono text-xs text-fd-muted-foreground">
                      {platform.target} · SHA-256 {platform.runtime_sha256}
                    </p>
                  </div>
                ))}
                <pre className="mt-3 max-h-64 overflow-auto rounded-lg bg-fd-muted p-3 text-xs">
                  {row.evidence}
                </pre>
                <a
                  className="mt-3 inline-block underline"
                  href={`https://pypi.org/project/${encodeURIComponent(row.name)}/${encodeURIComponent(row.version)}/`}
                >
                  View this version on PyPI
                </a>
              </details>
            </article>
          );
        })}
      </div>
      {filtered.length === 0 && (
        <p className="rounded-2xl border border-dashed border-fd-border p-8 text-center text-fd-muted-foreground">
          No packages match these filters. Untested means evidence is still
          needed, not that a package works.
        </p>
      )}
      {limit < filtered.length && (
        <button
          type="button"
          className="rounded-xl border border-fd-border px-4 py-2 text-sm font-medium focus-visible:outline-2 focus-visible:outline-fd-primary"
          onClick={() => setLimit(limit + 12)}
        >
          Show 12 more packages
        </button>
      )}
      <noscript>
        <p className="text-sm">
          Enable JavaScript to filter and load more packages, or{" "}
          <a
            className="underline"
            href="https://github.com/niklas-heer/kipferl/blob/main/compatibility/packages/verified-packages.json"
          >
            read the complete verification report
          </a>
          .
        </p>
      </noscript>
    </section>
  );
}
