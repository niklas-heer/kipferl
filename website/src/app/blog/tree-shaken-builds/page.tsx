import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Tree-shaken standalone builds",
  description:
    "How Kipferl shrank a minimal standalone app to 1.4 MB without requiring a Rust toolchain.",
};

function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div className="rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900/60 p-5">
      <div className="text-2xl md:text-3xl font-bold text-amber-600 dark:text-amber-400 mb-1">
        {value}
      </div>
      <div className="text-sm text-gray-600 dark:text-gray-400">{label}</div>
    </div>
  );
}

export default function TreeShakenBuildsPage() {
  return (
    <main className="min-h-screen px-6 py-16 md:py-24">
      <article className="max-w-4xl mx-auto">
        <nav className="flex items-center gap-3 text-sm text-gray-500 mb-14">
          <Link href="/" className="hover:text-amber-500">
            Kipferl
          </Link>
          <span>/</span>
          <Link href="/blog" className="hover:text-amber-500">
            Blog
          </Link>
        </nav>

        <header className="mb-16">
          <div className="font-mono text-sm text-amber-600 dark:text-amber-400 mb-5">
            AUGUST 2026 · BUILD ENGINEERING
          </div>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight leading-[1.05] mb-8">
            A 1.4 MB Kipferl without putting Rust on your machine
          </h1>
          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 leading-relaxed">
            Rust gave Kipferl a much better maintenance story, but the complete
            runtime grew with Ratatui, Rustls, SQLite, archives, and data-format
            parsers. Version 0.6 takes most of that cost back when an app does
            not use those capabilities.
          </p>
        </header>
        <p className="mb-10 rounded-xl border border-gray-200 dark:border-gray-800 px-5 py-4 text-sm text-gray-600 dark:text-gray-400">
          This article records the August 2026 release and its measurements. For
          the 0.7 release candidate and current project workflow, see the{" "}
          <Link
            className="text-cyan-600 dark:text-cyan-400 underline"
            href="/docs/getting-started/quick-start"
          >
            current documentation
          </Link>
          .
        </p>

        <section className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-20">
          <Stat value="1.451 MB" label="minimal standalone app" />
          <Stat value="69.9%" label="smaller than full" />
          <Stat value="1.13 MB" label="core Rust runtime" />
          <Stat value="0" label="user toolchain steps" />
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-20">
          <h2>The constraint mattered more than the linker</h2>
          <p>
            Traditional tree shaking links a fresh executable for every app.
            That would make users install Rust, Cargo, a C compiler, and target
            linkers—the opposite of Kipferl&apos;s developer-experience goal. A
            build should remain one fast command and produce one target-specific
            file with no runtime dependency.
          </p>
          <p>
            Kipferl therefore ships two prebuilt Rust runtimes. The core profile
            contains PocketPy and the dependency-light CLI surface. The full
            profile adds maintained implementations for SQLite, HTTPS, Ratatui
            input, regex, archives, crypto, timezone data, YAML, TOML, and KDL.
            Cargo features remove those complete dependency trees when we build
            core; there are no empty placeholders in the small artifact.
          </p>
        </section>

        <section className="mb-20">
          <h2 className="text-3xl md:text-4xl font-bold mb-8">
            Conservative by construction
          </h2>
          <div className="grid md:grid-cols-3 gap-5">
            {[
              [
                "1. Analyze",
                "A small lexer reads static imports while ignoring comments and quoted strings.",
              ],
              [
                "2. Select",
                "Common modules choose core; optional capabilities choose full with an explicit reason.",
              ],
              [
                "3. Package",
                "The existing loader receives the selected prebuilt runtime and transformed app.",
              ],
            ].map(([title, body]) => (
              <div
                key={title}
                className="rounded-2xl border border-gray-200 dark:border-gray-800 p-6"
              >
                <h3 className="font-bold text-lg mb-3">{title}</h3>
                <p className="text-gray-600 dark:text-gray-400 leading-relaxed">
                  {body}
                </p>
              </div>
            ))}
          </div>
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-20">
          <p>
            Dynamic imports, relative imports, <code>exec</code>, and
            <code>eval</code> deliberately fall back to full. The build log says
            which profile was chosen and why. <code>--full-runtime</code> is the
            escape hatch when application logic hides an import from static
            analysis. A false positive costs bytes; a false negative could ship
            a broken app, so the bias is intentional.
          </p>
          <pre>
            <code>{`✓ Runtime profile full (complete compatibility)\n  Full runtime: sqlite3 requires the SQLite capability`}</code>
          </pre>
        </section>

        <section className="mb-20">
          <h2 className="text-3xl md:text-4xl font-bold mb-8">
            The measured result
          </h2>
          <div className="overflow-x-auto rounded-2xl border border-gray-200 dark:border-gray-800">
            <table className="w-full text-left">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="p-4">Apple Silicon artifact</th>
                  <th className="p-4">Bytes</th>
                  <th className="p-4">Change</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
                <tr>
                  <td className="p-4">Full runtime</td>
                  <td className="p-4 font-mono">4,497,440</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Compatibility baseline
                  </td>
                </tr>
                <tr>
                  <td className="p-4">Core runtime</td>
                  <td className="p-4 font-mono">1,130,352</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    74.9% smaller runtime
                  </td>
                </tr>
                <tr>
                  <td className="p-4">Full standalone app</td>
                  <td className="p-4 font-mono">4,817,925</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Loader and minimal source included
                  </td>
                </tr>
                <tr>
                  <td className="p-4">Core standalone app</td>
                  <td className="p-4 font-mono">1,450,837</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    69.9% smaller app
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="text-sm text-gray-500 mt-4">
            Local v0.6 release-profile measurement on macOS ARM64. Source size
            and target architecture change the exact result; four-target CI
            enforces a 2.5 MB ceiling for every core runtime.
          </p>
          <h3 className="text-2xl font-bold mt-12 mb-5">
            Core runtime across every release target
          </h3>
          <div className="overflow-x-auto rounded-2xl border border-gray-200 dark:border-gray-800">
            <table className="w-full text-left">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="p-4">Target</th>
                  <th className="p-4">Full</th>
                  <th className="p-4">Core</th>
                  <th className="p-4">Reduction</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
                {[
                  ["macOS ARM64", "4,497,440", "1,130,352", "74.9%"],
                  ["macOS x86_64", "5,056,944", "1,183,140", "76.6%"],
                  ["Linux ARM64 musl", "4,738,296", "1,316,512", "72.2%"],
                  ["Linux x86_64 musl", "5,451,504", "1,349,904", "75.2%"],
                ].map(([target, full, core, reduction]) => (
                  <tr key={target}>
                    <td className="p-4 font-medium">{target}</td>
                    <td className="p-4 font-mono">{full}</td>
                    <td className="p-4 font-mono">{core}</td>
                    <td className="p-4 text-amber-600 dark:text-amber-400">
                      {reduction}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="rounded-3xl bg-gray-950 text-white p-8 md:p-12 mb-16">
          <h2 className="text-3xl font-bold mb-5">
            Small by default, complete when needed
          </h2>
          <p className="text-gray-300 text-lg leading-relaxed">
            This is a deliberately boring architecture: maintained libraries
            remain available, common apps stop paying for unused capabilities,
            and the user-facing build command stays compiler-free. More profiles
            can be added later from real import data without changing that
            contract.
          </p>
        </section>

        <section className="border-t border-gray-200 dark:border-gray-800 pt-10 flex flex-col gap-3 text-amber-600 dark:text-amber-400">
          <Link href="/docs/commands/build">Build command reference →</Link>
          <Link href="https://github.com/niklas-heer/kipferl/issues/57">
            Implementation tracker →
          </Link>
          <Link href="https://github.com/niklas-heer/kipferl/blob/main/benchmarks/tree_shaking_baseline.md">
            Reproducible benchmark report →
          </Link>
          <Link href="/blog/rust-migration">
            Rust migration retrospective →
          </Link>
        </section>
      </article>
    </main>
  );
}
