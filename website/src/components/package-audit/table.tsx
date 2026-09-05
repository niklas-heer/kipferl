"use client";

import { useId, useMemo, useState } from "react";
import { type AuditRecord, categoryLabel } from "./types";

const PAGE_SIZE = 25;
const control =
  "w-full rounded-lg border border-fd-border bg-fd-background px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fd-primary";
const badgeStyles = {
  tested: "bg-emerald-500/10 text-emerald-800 dark:text-emerald-300",
  incompatible: "bg-rose-500/10 text-rose-800 dark:text-rose-300",
  unverified: "bg-amber-500/10 text-amber-800 dark:text-amber-300",
};

export function AuditTable({ records }: { records: AuditRecord[] }) {
  const id = useId();
  const [query, setQuery] = useState("");
  const [scope, setScope] = useState("1000");
  const [status, setStatus] = useState("all");
  const [category, setCategory] = useState("all");
  const [page, setPage] = useState(1);
  const categories = useMemo(
    () => Array.from(new Set(records.map((row) => row.category))).sort(),
    [records],
  );
  const filtered = useMemo(() => {
    const search = query.trim().toLowerCase();
    return records.filter(
      (row) =>
        row.rank <= Number(scope) &&
        (status === "all" || row.status === status) &&
        (category === "all" || row.category === category) &&
        (!search ||
          `${row.name} ${row.version} ${row.evidence} ${row.blocker ?? ""}`
            .toLowerCase()
            .includes(search)),
    );
  }, [records, query, scope, status, category]);
  const pages = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const currentPage = Math.min(page, pages);
  const start = (currentPage - 1) * PAGE_SIZE;
  const visible = filtered.slice(start, start + PAGE_SIZE);
  const setFilter = (setValue: (value: string) => void, value: string) => {
    setValue(value);
    setPage(1);
  };

  return (
    <section aria-label="Search package audit" className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2">
        <label
          htmlFor={`${id}-search`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Find a package or error</span>
          <input
            id={`${id}-search`}
            type="search"
            value={query}
            onChange={(event) => setFilter(setQuery, event.target.value)}
            className={control}
            placeholder="Package name, version, or diagnostic…"
          />
        </label>
        <label
          htmlFor={`${id}-scope`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Popularity group</span>
          <select
            id={`${id}-scope`}
            value={scope}
            onChange={(event) => setFilter(setScope, event.target.value)}
            className={control}
          >
            <option value="1000">Top 1,000 packages</option>
            <option value="100">Top 100 packages</option>
          </select>
        </label>
        <label
          htmlFor={`${id}-status`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Result</span>
          <select
            id={`${id}-status`}
            value={status}
            onChange={(event) => setFilter(setStatus, event.target.value)}
            className={control}
          >
            <option value="all">All results</option>
            <option value="tested">Tested</option>
            <option value="incompatible">Incompatible</option>
            <option value="unverified">Unverified</option>
          </select>
        </label>
        <label
          htmlFor={`${id}-category`}
          className="space-y-1 text-sm font-medium"
        >
          <span>Reason</span>
          <select
            id={`${id}-category`}
            value={category}
            onChange={(event) => setFilter(setCategory, event.target.value)}
            className={control}
          >
            <option value="all">All reasons</option>
            {categories.map((value) => (
              <option key={value} value={value}>
                {categoryLabel(value)}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3 text-sm">
        <output aria-live="polite" aria-atomic="true">
          {filtered.length === 0
            ? "No matching packages"
            : `Showing ${start + 1}–${start + visible.length} of ${filtered.length.toLocaleString("en-US")} ${filtered.length === 1 ? "package" : "packages"}`}
        </output>
        <button
          type="button"
          className="rounded px-2 py-1 underline focus-visible:outline-2 focus-visible:outline-fd-primary"
          onClick={() => {
            setQuery("");
            setScope("1000");
            setStatus("all");
            setCategory("all");
            setPage(1);
          }}
        >
          Reset filters
        </button>
      </div>
      <div className="overflow-x-auto rounded-xl border border-fd-border">
        <table className="w-full min-w-[620px] text-left text-sm">
          <caption className="sr-only">
            Ranked package versions and audit evidence, 25 results per page
          </caption>
          <thead className="border-b border-fd-border bg-fd-muted">
            <tr>
              {[
                "Rank",
                "Package / version",
                "Result",
                "Reason and evidence",
              ].map((title) => (
                <th key={title} scope="col" className="p-3 font-semibold">
                  {title}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {visible.map((row) => (
              <tr
                key={row.rank}
                className="border-b border-fd-border align-top last:border-0"
              >
                <td className="p-3 tabular-nums text-fd-muted-foreground">
                  {row.rank}
                </td>
                <th scope="row" className="max-w-48 p-3 font-normal">
                  <a
                    href={`https://pypi.org/project/${encodeURIComponent(row.name)}/`}
                    className="break-words font-semibold underline decoration-fd-border underline-offset-4"
                  >
                    {row.name}
                  </a>
                  <div className="mt-1 break-all font-mono text-xs text-fd-muted-foreground">
                    {row.version || "Not resolved"}
                  </div>
                </th>
                <td className="p-3">
                  <span
                    className={`inline-block rounded-full px-2 py-1 text-xs font-medium capitalize ${badgeStyles[row.status]}`}
                  >
                    {row.status}
                  </span>
                </td>
                <td className="min-w-64 p-3">
                  <p className="font-medium">{categoryLabel(row.category)}</p>
                  <details className="mt-2">
                    <summary className="cursor-pointer text-fd-muted-foreground underline underline-offset-4">
                      Inspect evidence
                    </summary>
                    <p className="mt-2 whitespace-pre-wrap break-words leading-relaxed">
                      {row.evidence}
                    </p>
                    {row.blocker && (
                      <pre className="mt-3 max-h-60 overflow-auto rounded bg-fd-muted p-3 text-xs">
                        <code>{row.blocker}</code>
                      </pre>
                    )}
                    <p className="mt-3 text-xs text-fd-muted-foreground">
                      {row.artifactVerified
                        ? "Downloaded artifact hash verified."
                        : "No downloaded artifact hash verified; see evidence scope."}
                    </p>
                    {row.wheel && (
                      <p className="mt-1 break-all font-mono text-xs text-fd-muted-foreground">
                        {row.wheel}
                      </p>
                    )}
                  </details>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {visible.length === 0 && (
          <p className="p-8 text-center text-fd-muted-foreground">
            Try a different name, result, or reason.
          </p>
        )}
      </div>
      <nav
        aria-label="Package audit pages"
        className="flex items-center justify-between gap-3 text-sm"
      >
        <button
          type="button"
          disabled={currentPage === 1}
          onClick={() => setPage(currentPage - 1)}
          className="rounded-lg border border-fd-border px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-fd-primary"
        >
          Previous
        </button>
        <span>
          Page {currentPage} of {pages}
        </span>
        <button
          type="button"
          disabled={currentPage === pages}
          onClick={() => setPage(currentPage + 1)}
          className="rounded-lg border border-fd-border px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-fd-primary"
        >
          Next
        </button>
      </nav>
      <noscript>
        <p>
          The first page is shown here. Enable JavaScript to search and filter,
          or read the complete linked JSON, CSV, or Markdown report.
        </p>
      </noscript>
    </section>
  );
}
