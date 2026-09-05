"use client";

import { useId, useMemo, useState } from "react";
import type { SupportPriorities, SupportPriority } from "./support-data";

function plainBlocker(row: SupportPriority): string {
  if (row.currentStatus === "verified")
    return `A narrow workflow is already verified: ${row.currentSummary}. Broader package support remains a separate goal.`;
  if (row.currentStatus === "limited")
    return `A limited workflow has evidence: ${row.currentSummary}. Read its package card for boundaries.`;
  if (row.currentStatus === "unsupported")
    return "The reviewed application workflow has a concrete blocker. Its package card explains the current result.";
  const labels: Record<string, string> = {
    syntax:
      "Needs language constructs that the current runtime cannot compile.",
    native_only:
      "Needs a native extension or a wheel format outside current support.",
    source_only: "Needs source-build support or a usable prebuilt wheel.",
    python_requirement:
      "Declares a Python version outside the current runtime target.",
    unsupported_requirement:
      "Needs dependency requirement features the installer cannot resolve yet.",
    limits:
      "Needs a bounded audit that can complete within the resource limits.",
    network:
      "Needs complete download evidence before compatibility can be assessed.",
    unverified:
      "Source checks completed; installation and the useful workflow still need verification.",
  };
  return (
    labels[row.blockerCategory] ??
    "Needs focused implementation and workflow verification."
  );
}

const control =
  "w-full rounded-xl border border-fd-border bg-fd-background px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fd-primary";

export function PriorityGuide({ data }: { data: SupportPriorities }) {
  const id = useId();
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("all");
  const [limit, setLimit] = useState(10);
  const categories = useMemo(
    () => [...new Set(data.records.map((row) => row.category))].sort(),
    [data.records],
  );
  const filtered = data.records.filter(
    (row) =>
      (category === "all" || row.category === category) &&
      `${row.name} ${row.useCase} ${plainBlocker(row)}`
        .toLowerCase()
        .includes(query.trim().toLowerCase()),
  );
  return (
    <section
      className="not-prose my-8 space-y-5"
      aria-label="Package support priorities"
    >
      <div className="rounded-2xl border border-fd-border bg-fd-muted/40 p-5">
        <p className="text-xs font-semibold uppercase tracking-widest text-fd-muted-foreground">
          Where we should go next
        </p>
        <h3 className="mt-2 text-2xl font-semibold tracking-tight">
          100 packages worth working toward
        </h3>
        <p className="mt-3 max-w-3xl text-sm leading-relaxed text-fd-muted-foreground">
          A ranked development shortlist, based on useful CLI workflows,
          popularity, shared dependencies, and estimated work. This is an
          investment shortlist, not a list of fully supported packages or
          promised release dates. Existing verified scopes are identified below.
        </p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <label
          htmlFor={`${id}-query`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Find a package or use case</span>
          <input
            id={`${id}-query`}
            className={control}
            type="search"
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setLimit(10);
            }}
            placeholder="HTTP, configuration, validation…"
          />
        </label>
        <label
          htmlFor={`${id}-category`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Workflow</span>
          <select
            id={`${id}-category`}
            className={control}
            value={category}
            onChange={(event) => {
              setCategory(event.target.value);
              setLimit(10);
            }}
          >
            <option value="all">All workflows</option>
            {categories.map((value) => (
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
        {filtered.length} priorities · showing{" "}
        {Math.min(limit, filtered.length)}
      </output>
      <ol className="divide-y divide-fd-border overflow-hidden rounded-2xl border border-fd-border">
        {filtered.slice(0, limit).map((row) => (
          <li key={row.name} className="p-5">
            <div className="flex items-start gap-4">
              <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-fd-muted font-mono text-sm">
                {row.priority}
              </span>
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-baseline justify-between gap-2">
                  <h4 className="break-words text-lg font-semibold">
                    {row.name}
                  </h4>
                  <span className="text-xs text-fd-muted-foreground">
                    {row.category}
                  </span>
                </div>
                <p className="mt-1 text-sm leading-relaxed">{row.useCase}</p>
                <p className="mt-2 text-sm text-fd-muted-foreground">
                  <span className="font-medium">Current support:</span>{" "}
                  {plainBlocker(row)}
                </p>
                <details className="mt-3 text-sm">
                  <summary className="cursor-pointer font-medium underline underline-offset-4">
                    Why this rank & what success means
                  </summary>
                  <p className="mt-3 leading-relaxed">{row.why}</p>
                  <p className="mt-2 text-fd-muted-foreground">
                    <strong>Source-audit detail:</strong> {row.blockerSummary}
                  </p>
                  <p className="mt-2 leading-relaxed">
                    <strong>Acceptance test:</strong> {row.acceptanceTest}
                  </p>
                  <p className="mt-3 text-xs text-fd-muted-foreground">
                    Selected version {row.version} · PyPI popularity #
                    {row.pypiRank} · Priority score {row.score} · Estimated
                    effort {row.effort}/5
                  </p>
                </details>
              </div>
            </div>
          </li>
        ))}
      </ol>
      {filtered.length === 0 && (
        <p className="py-6 text-center text-fd-muted-foreground">
          No matching priorities. Try another workflow or search.
        </p>
      )}
      {limit < filtered.length && (
        <button
          type="button"
          className="rounded-xl border border-fd-border px-4 py-2 text-sm font-medium focus-visible:outline-2 focus-visible:outline-fd-primary"
          onClick={() => setLimit(limit + 20)}
        >
          Show 20 more priorities
        </button>
      )}
      <details className="rounded-xl border border-fd-border p-4 text-sm">
        <summary className="cursor-pointer font-medium">
          Ranking method & source evidence
        </summary>
        <p className="mt-3 text-fd-muted-foreground">
          Assessment for {data.release}, recorded {data.evaluatedAt}. Scores are
          prioritization judgments, not measured implementation costs.
          Dependency reach uses the pinned package snapshot.
        </p>
        <pre className="mt-3 overflow-auto rounded-lg bg-fd-muted p-3 text-xs">
          {data.methodology}
        </pre>
        <a
          className="mt-3 inline-block underline"
          href="https://github.com/niklas-heer/kipferl/blob/main/compatibility/packages/support-priorities.md"
        >
          Read all 100 priorities, package sources, and the scoring rationale
        </a>
      </details>
      <noscript>
        <p className="text-sm">
          Enable JavaScript to search and load more priorities, or use the
          complete Markdown list linked above.
        </p>
      </noscript>
    </section>
  );
}
