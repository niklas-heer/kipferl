import { ArrowUpRight, Database, Globe, Terminal } from "lucide-react";
import Link from "next/link";
import { PriorityGuide } from "./priorities";
import { loadSupportPriorities } from "./support-data";
import { VerifiedGuide } from "./verified";
import { loadVerifiedPackages } from "./verified-data";

export function CompatibilityGuide() {
  const data = loadVerifiedPackages();
  return (
    <>
      <div className="not-prose my-8 grid gap-4 lg:grid-cols-3">
        {[
          {
            title: "Build a terminal tool",
            description:
              "Argument parsing, prompts, tables, colors, and progress. Start with Kipferl's native CLI and terminal modules.",
            href: "/docs/getting-started/quick-start",
            Icon: Terminal,
          },
          {
            title: "Talk to an API",
            description:
              "Make HTTP requests and read JSON using the built-in modules. The API template gives you a working starting point.",
            href: "/docs/commands/projects",
            Icon: Globe,
          },
          {
            title: "Work with local data",
            description:
              "Use the supported SQLite, JSON, CSV, and file APIs for small data tools. Check the module reference for each API's scope.",
            href: "/docs/modules",
            Icon: Database,
          },
        ].map(({ title, description, href, Icon }) => (
          <Link
            href={href}
            key={title}
            className="group rounded-2xl border border-fd-border bg-fd-muted/30 p-5 transition-colors hover:bg-fd-muted focus-visible:outline-2 focus-visible:outline-fd-primary"
          >
            <Icon size={24} className="text-fd-primary" />
            <span className="mt-5 flex items-center justify-between gap-2 text-lg font-semibold tracking-tight">
              {title}
              <ArrowUpRight size={18} aria-hidden="true" />
            </span>
            <p className="mt-2 text-sm leading-relaxed text-fd-muted-foreground">
              {description}
            </p>
            <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-fd-primary">
              Built in · no PyPI dependency
            </p>
          </Link>
        ))}
      </div>
      <h2 id="reviewed-packages">Find a package for your app</h2>
      <p>
        Start with the useful workflows below. “Verified” describes a real
        tested workflow; “Limited” tells you exactly which smaller part works.
        Neither badge promises every API or a different platform.
      </p>
      <VerifiedGuide data={data} />
      <p>
        The guide can contain newer behavior evidence than the catalog embedded
        in the released CLI. Copy the exact install command shown, including{" "}
        <code>--allow-unverified</code> where needed, and run your application
        tests. A website badge does not change an already downloaded binary.
      </p>
    </>
  );
}

export function SupportPrioritiesGuide() {
  return <PriorityGuide data={loadSupportPriorities()} />;
}
