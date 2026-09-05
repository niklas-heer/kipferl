import { loadAudit } from "./data";
import { AuditTable } from "./table";

export function PackageAudit() {
  const audit = loadAudit();
  const summaries = [100, 1000].map((limit) => {
    const rows = audit.records.filter((record) => record.rank <= limit);
    return { limit, rows };
  });
  return (
    <div className="not-prose my-8 space-y-6">
      <div className="rounded-xl border border-fd-border bg-fd-muted/50 p-5 text-sm">
        <p>
          Ranking:{" "}
          <a className="underline" href={audit.rankingUrl}>
            monthly PyPI downloads
          </a>
          , snapshot retrieved {audit.retrievedAt}. Downloads include CI and
          automated installations; they do not count unique users.
        </p>
        <p className="mt-3">
          Download window: {audit.windowStart} to {audit.windowEnd} (exclusive).
        </p>
        <p className="mt-3">
          Runtime target: <strong>{audit.target}</strong>
        </p>
        <p className="mt-1 break-all font-mono text-xs">
          SHA-256: {audit.runtimeHash}
        </p>
      </div>
      {!audit.complete && (
        <p className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-4 text-sm">
          Partial checkpoint: {audit.records.length} of {audit.requestedCount}{" "}
          requested packages have results. Counts below describe only completed
          records.
        </p>
      )}
      <div className="overflow-x-auto rounded-xl border border-fd-border">
        <table className="w-full text-left text-sm">
          <caption className="p-4 text-left font-semibold">
            Evidence in each popularity group
          </caption>
          <thead className="border-y border-fd-border bg-fd-muted">
            <tr>
              {["Group", "Audited", "Tested", "Incompatible", "Unverified"].map(
                (title) => (
                  <th key={title} className="p-3" scope="col">
                    {title}
                  </th>
                ),
              )}
            </tr>
          </thead>
          <tbody>
            {summaries.map(({ limit, rows }) => (
              <tr
                key={limit}
                className="border-b border-fd-border last:border-0"
              >
                <th scope="row" className="p-3 font-medium">
                  Top {limit.toLocaleString("en-US")}
                </th>
                <td className="p-3">{rows.length}</td>
                {["tested", "incompatible", "unverified"].map((status) => (
                  <td key={status} className="p-3">
                    {rows.filter((row) => row.status === status).length}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="text-sm text-fd-muted-foreground">
        Unverified includes sources that compile and checks that could not
        finish. Use the reason filter to distinguish them. Source compilation
        does not execute imports, exercise package APIs, or establish full
        compatibility.
      </p>
      <AuditTable records={audit.records} />
    </div>
  );
}
