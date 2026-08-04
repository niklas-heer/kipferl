import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "From Zig to Rust",
  description:
    "Why μcharm moved to Rust, how the incremental migration worked, and the measured outcome.",
};

const evidenceRoot = "https://github.com/ucharmdev/ucharm/blob/main/benchmarks";

function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div className="rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900/60 p-5">
      <div className="text-2xl md:text-3xl font-bold text-cyan-600 dark:text-cyan-400 mb-1">
        {value}
      </div>
      <div className="text-sm text-gray-600 dark:text-gray-400">{label}</div>
    </div>
  );
}

function Phase({
  number,
  title,
  children,
}: React.PropsWithChildren<{ number: string; title: string }>) {
  return (
    <div className="grid md:grid-cols-[5rem_1fr] gap-4 md:gap-8 py-8 border-t border-gray-200 dark:border-gray-800">
      <div className="font-mono text-cyan-600 dark:text-cyan-400">{number}</div>
      <div>
        <h3 className="text-xl font-bold mb-3">{title}</h3>
        <div className="text-gray-700 dark:text-gray-300 leading-relaxed space-y-4">
          {children}
        </div>
      </div>
    </div>
  );
}

export default function RustMigrationPage() {
  return (
    <main className="min-h-screen px-6 py-16 md:py-24">
      <article className="max-w-4xl mx-auto">
        <nav className="flex items-center gap-3 text-sm text-gray-500 mb-14">
          <Link href="/" className="hover:text-cyan-500">
            μcharm
          </Link>
          <span>/</span>
          <Link href="/blog" className="hover:text-cyan-500">
            Blog
          </Link>
        </nav>

        <header className="mb-16">
          <div className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-5">
            AUGUST 2026 · MIGRATION RETROSPECTIVE
          </div>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight leading-[1.05] mb-8">
            From Zig to Rust without stopping the roadmap
          </h1>
          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 leading-relaxed">
            μcharm changed its implementation language, runtime host, release
            pipeline, and dependency strategy—without changing its promise:
            write a focused Python CLI and ship one fast native file.
          </p>
        </header>

        <section className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-20">
          <Stat value="1,669 / 1,669" label="compatibility checks" />
          <Stat value="7.044 ms" label="median startup" />
          <Stat value="3.98–4.77 MB" label="release runtimes" />
          <Stat value="4 targets" label="native CI matrix" />
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-20">
          <p className="text-sm font-mono text-cyan-600 dark:text-cyan-400">
            THE WHY
          </p>
          <h2>A foundation should reduce product risk</h2>
          <p>
            Zig helped μcharm prove that a small, embedded Python runtime could
            produce genuinely pleasant standalone CLI applications. But the
            project increasingly paid for a pre-1.0 language surface, changing
            build conventions, a smaller library ecosystem, and more custom
            systems code than the product needed to own.
          </p>
          <p>
            The concern was not whether Zig could produce fast binaries. It
            could. The concern was the long-term cost of maintaining a CLI,
            loader, VM host, terminal UI, TLS, SQLite integration, archives,
            process management, and release tooling while the foundation kept
            moving. Ecosystem predictability and governance are engineering
            inputs too.
          </p>
          <p>
            Rust offered a stable language and Cargo workflow, a larger pool of
            maintained libraries, explicit ownership at the PocketPy boundary,
            first-class test tooling, and mature cross-target release support.
            That aligned better with the real goal: spend maintenance effort on
            the developer experience, not on rebuilding infrastructure.
          </p>
        </section>

        <section className="mb-20">
          <p className="text-sm font-mono text-cyan-600 dark:text-cyan-400 mb-4">
            THE HOW
          </p>
          <h2 className="text-3xl md:text-4xl font-bold mb-8">
            Replace one boundary at a time
          </h2>
          <Phase number="01" title="Freeze the contract">
            <p>
              We recorded startup, size, loader behavior, binary trailers, and
              compatibility before porting. Existing Zig artifacts became golden
              inputs instead of a vague reference implementation.
            </p>
          </Phase>
          <Phase number="02" title="Prove the Rust spine">
            <p>
              A Cargo workspace built the vendored PocketPy C runtime, owned the
              VM lifecycle, and crossed one narrow FFI boundary. Format and
              loader crates came next, preserving the existing on-disk format.
            </p>
          </Phase>
          <Phase number="03" title="Port behavior in compatibility waves">
            <p>
              The CLI and runtime modules moved in risk-ordered batches. Every
              batch ran the same Python under CPython and μcharm, comparing
              status, stdout, and stderr. The suite expanded from the original
              456-check baseline to 1,669 passing checks.
            </p>
          </Phase>
          <Phase number="04" title="Use the ecosystem deliberately">
            <p>
              We accepted Ratatui/Crossterm for interactive selection,
              Ureq/Rustls for maintained HTTPS, Rusqlite with bundled SQLite,
              and focused ZIP/TAR crates. We rejected options whose size,
              maturity, or dependency cost exceeded their maintenance value.
            </p>
          </Phase>
          <Phase number="05" title="Cut over, release, then delete">
            <p>
              Four-target CI built the runtime, loader, CLI, and a real
              standalone app. Only after `v0.6.0-rc.1` passed public-asset
              checksum and execution smokes did we remove the archived Zig
              tree—85,310 obsolete tracked lines in one recoverable Git commit.
            </p>
          </Phase>
        </section>

        <section className="mb-20">
          <p className="text-sm font-mono text-cyan-600 dark:text-cyan-400 mb-4">
            THE OUTCOME
          </p>
          <h2 className="text-3xl md:text-4xl font-bold mb-8">
            Better ownership, with explicit tradeoffs
          </h2>
          <div className="overflow-x-auto rounded-2xl border border-gray-200 dark:border-gray-800">
            <table className="w-full text-left">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="p-4">Measure</th>
                  <th className="p-4">Result</th>
                  <th className="p-4">What it means</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
                <tr>
                  <td className="p-4 font-medium">Compatibility</td>
                  <td className="p-4">1,669 / 1,669</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    All available checks pass; one host-unavailable TOML
                    baseline remains outside the denominator.
                  </td>
                </tr>
                <tr>
                  <td className="p-4 font-medium">Startup</td>
                  <td className="p-4">7.044 ms median, 7.980 ms p95</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Slower than the 4.332 ms legacy baseline after adding the
                    maintained feature stack, but still inside the 10 ms goal.
                  </td>
                </tr>
                <tr>
                  <td className="p-4 font-medium">Runtime size</td>
                  <td className="p-4">3.98–4.77 MB across RC targets</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Larger than the 2.31 MB Zig ARM64 baseline; the budget now
                    values correctness and maintainability over the smallest
                    possible artifact.
                  </td>
                </tr>
                <tr>
                  <td className="p-4 font-medium">Memory</td>
                  <td className="p-4">6.21 MB median empty-process RSS</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Measured on Apple Silicon; the JSON workload used 15.56 MB.
                  </td>
                </tr>
                <tr>
                  <td className="p-4 font-medium">Interactive selection</td>
                  <td className="p-4">9.100 ms median PTY round trip</td>
                  <td className="p-4 text-gray-600 dark:text-gray-400">
                    Ratatui adds a responsive, tested viewport while preserving
                    terminal state and shell scrollback.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <section className="rounded-3xl bg-gray-950 text-white p-8 md:p-12 mb-16">
          <h2 className="text-3xl font-bold mb-5">The result we wanted</h2>
          <p className="text-gray-300 text-lg leading-relaxed mb-6">
            The rewrite did not win every microbenchmark, and we do not want to
            pretend it did. It produced a more stable foundation, removed
            bespoke protocol and archive code, made unsafe boundaries explicit,
            improved the interactive experience, and kept the product inside its
            startup and size budgets. That is a better long-term trade.
          </p>
          <p className="text-gray-300 text-lg leading-relaxed">
            The next work is product work again: better stubs, cross-target
            reliability, tree-shaking, watch mode, and higher-level terminal
            experiences.
          </p>
        </section>

        <section className="border-t border-gray-200 dark:border-gray-800 pt-10">
          <h2 className="text-2xl font-bold mb-5">Reproduce the numbers</h2>
          <div className="flex flex-col gap-3 text-cyan-600 dark:text-cyan-400">
            <Link href={`${evidenceRoot}/rust_optimization_baseline.md`}>
              Rust optimization baseline →
            </Link>
            <Link href={`${evidenceRoot}/release_cutover.md`}>
              Release cutover report →
            </Link>
            <Link href="https://github.com/ucharmdev/ucharm/blob/main/tests/compat_report_pocketpy.md">
              Compatibility report →
            </Link>
            <Link href="https://github.com/ucharmdev/ucharm/issues/13">
              Migration tracker →
            </Link>
            <Link href="https://github.com/ucharmdev/ucharm/releases/tag/v0.6.0-rc.1">
              v0.6.0-rc.1 release →
            </Link>
          </div>
        </section>
      </article>
    </main>
  );
}
