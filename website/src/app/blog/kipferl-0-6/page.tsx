import type { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Kipferl 0.6: Rust-powered Python CLIs, baked smaller",
  description:
    "Kipferl 0.6 is the first stable Rust release, with tree-shaken standalone apps, Ratatui prompts, dev mode, and a broad CLI-focused runtime.",
  openGraph: {
    title: "Kipferl 0.6: Rust-powered Python CLIs, baked smaller",
    description:
      "A new name, a stable Rust foundation, and standalone Python-style CLI apps from 1.4 MB.",
    type: "article",
    publishedTime: "2026-08-05T13:36:22Z",
    url: "https://kipferl.dev/blog/kipferl-0-6",
    images: ["/blog/kipferl-0-6/opengraph-image"],
  },
  twitter: {
    card: "summary_large_image",
    title: "Kipferl 0.6: Rust-powered Python CLIs, baked smaller",
    description:
      "A new name, a stable Rust foundation, and standalone Python-style CLI apps from 1.4 MB.",
    images: ["/blog/kipferl-0-6/opengraph-image"],
  },
};

const releaseRoot =
  "https://github.com/niklas-heer/kipferl/releases/download/v0.6.0";

const assets = [
  ["macOS Apple Silicon", "kipferl-macos-aarch64", "3,727,024", "3.55 MiB"],
  ["macOS Intel", "kipferl-macos-x86_64", "4,009,384", "3.82 MiB"],
  ["Linux ARM64 (static)", "kipferl-linux-aarch64", "4,198,584", "4.00 MiB"],
  [
    "Linux x86_64 (static PIE)",
    "kipferl-linux-x86_64",
    "4,543,248",
    "4.33 MiB",
  ],
] as const;

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

export default function KipferlReleasePage() {
  return (
    <main className="min-h-screen px-6 py-16 md:py-24">
      <article className="max-w-4xl mx-auto">
        <nav className="flex items-center gap-3 text-sm text-gray-500 mb-14">
          <Link href="/" className="hover:text-cyan-500">
            Kipferl
          </Link>
          <span>/</span>
          <Link href="/blog" className="hover:text-cyan-500">
            Blog
          </Link>
        </nav>

        <header className="mb-16">
          <div className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-5">
            AUGUST 5, 2026 · RELEASE
          </div>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight leading-[1.05] mb-8">
            Kipferl 0.6: Rust-powered Python CLIs, baked smaller
          </h1>
          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 leading-relaxed">
            Our first stable release as Kipferl keeps the pleasant Python-style
            workflow, moves the foundation to Rust, and ships tree-shaken
            standalone applications from 1.4 MB—with no Python, Rust, libc, or
            musl installation on the target machine.
          </p>
        </header>

        <section className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-20">
          <Stat value="1.451 MB" label="minimal Apple Silicon app" />
          <Stat value="69.9%" label="smaller than its full build" />
          <Stat value="7.679 ms" label="measured median startup" />
          <Stat value="1,669 / 1,669" label="compatibility checks" />
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-20">
          <h2>A new name and a steadier foundation</h2>
          <p>
            Kipferl is the new name for μcharm/ucharm. The pastry name gives a
            small nod to Bun and fits the product: Kipferl bakes a focused
            Python-style CLI into one portable executable. The repository,
            command, packages, documentation, and public home now share the same
            spelling.
          </p>
          <p>
            Version 0.6 also replaces the project&apos;s Zig implementation with
            stable Rust around the same embedded PocketPy runtime. This was a
            maintenance decision, not a rejection of Zig&apos;s ability to make
            fast software. Cargo, Rust&apos;s stable language contract, explicit
            VM ownership, and a larger maintained library ecosystem let us spend
            more time on the developer experience and less on infrastructure.
          </p>
          <p>
            The migration was incremental and compatibility-gated. The
            application format remains <code>MCHARM01</code>, existing binaries
            keep running, and the temporary <code>ucharm</code> command,
            imports, environment variables, and download aliases make the
            transition deliberate rather than abrupt. Read the full{" "}
            <Link href="/blog/rust-migration">migration retrospective</Link> for
            the why, how, outcome, and tradeoffs.
          </p>
        </section>

        <section className="mb-20">
          <h2 className="text-3xl md:text-4xl font-bold mb-5">
            A tighter development loop
          </h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 leading-relaxed mb-8">
            <code>kipferl dev</code> runs immediately, watches source, config,
            and template files through native filesystem events, debounces
            editor bursts, and restores the terminal between restarts. Ratatui
            and Crossterm now power interactive select and multiselect prompts
            in a scrollback-preserving inline viewport.
          </p>
          <pre className="rounded-2xl bg-gray-950 text-gray-100 p-6 overflow-x-auto mb-10">
            <code>{`$ kipferl dev app.py\n$ kipferl build app.py -o app\n$ ./app`}</code>
          </pre>
          <div className="rounded-2xl overflow-hidden border border-gray-200 dark:border-gray-800 shadow-2xl shadow-black/20">
            <Image
              src="/demo.gif"
              alt="Kipferl terminal demo with styled output and interactive prompts"
              width={720}
              height={520}
              className="w-full"
              unoptimized
            />
          </div>
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-20">
          <h2>Useful batteries, with honest boundaries</h2>
          <p>
            Common configuration workflows cover JSON, YAML 1.2, TOML, KDL 2.0,
            XML, CSV, and INI/CFG. The CLI-focused runtime also includes SQLite,
            HTTPS, filesystem and process APIs, regex, archives, crypto, dates,
            logging, and more than 50 targeted modules. Maintained Rust crates
            back the parts where they improve correctness and ownership:
            Ratatui/Crossterm, Ureq/Rustls, Rusqlite with bundled SQLite, and
            focused parsers and archive libraries.
          </p>
          <p>
            Kipferl is intentionally not CPython with pip in a smaller box. It
            supports familiar Python syntax and a curated library surface for
            command-line applications; C extensions and arbitrary pip packages
            are outside that contract. The published compatibility report makes
            those boundaries inspectable rather than implied.
          </p>
        </section>

        <section className="mb-20">
          <h2 className="text-3xl md:text-4xl font-bold mb-6">
            Tree shaking without a user toolchain
          </h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 leading-relaxed mb-8">
            A normal build conservatively analyzes imports and selects one of
            two prebuilt Rust runtime profiles. Common scripts use core; SQLite,
            HTTPS, interactive input, YAML/TOML/KDL, regex, crypto, archives,
            dynamic imports, and other optional capabilities select full.
            <code className="mx-1">--full-runtime</code> is an explicit escape
            hatch, and neither path needs a local compiler or linker.
          </p>
          <div className="overflow-x-auto rounded-2xl border border-gray-200 dark:border-gray-800">
            <table className="w-full text-left">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="p-4">Apple Silicon app</th>
                  <th className="p-4">Bytes</th>
                  <th className="p-4">Median startup</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
                <tr>
                  <td className="p-4 font-medium">Core, selected by default</td>
                  <td className="p-4 font-mono">1,450,837</td>
                  <td className="p-4 font-mono">7.679 ms</td>
                </tr>
                <tr>
                  <td className="p-4 font-medium">Full compatibility</td>
                  <td className="p-4 font-mono">4,817,925</td>
                  <td className="p-4 font-mono">8.480 ms</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p className="text-sm text-gray-500 mt-4">
            Measured with the checked-in 2,000-run benchmark on an Apple M4 Max
            running macOS 15.6. The source, commands, distributions, and raw
            outputs are in the reproducible benchmark report.
          </p>
          <p className="text-lg text-gray-600 dark:text-gray-400 leading-relaxed mt-8">
            The 69.9% reduction comes from Cargo features removing complete
            optional dependency trees—not from dropping error handling or making
            users install Rust. The{" "}
            <Link
              href="/blog/tree-shaken-builds"
              className="text-amber-600 dark:text-amber-400 hover:underline"
            >
              tree-shaking deep dive
            </Link>{" "}
            explains the design and conservative fallbacks.
          </p>
        </section>

        <section className="mb-20">
          <h2 className="text-3xl md:text-4xl font-bold mb-6">
            Published CLI artifacts
          </h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 leading-relaxed mb-8">
            These are the exact GitHub release assets verified after
            publication. Every binary has an adjacent SHA-256 file; both Linux
            CLIs are statically linked and require no system libc or musl
            runtime.
          </p>
          <div className="overflow-x-auto rounded-2xl border border-gray-200 dark:border-gray-800">
            <table className="w-full text-left">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="p-4">Target</th>
                  <th className="p-4">CLI size</th>
                  <th className="p-4">Download</th>
                  <th className="p-4">SHA-256</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-800">
                {assets.map(([target, name, bytes, human]) => (
                  <tr key={name}>
                    <td className="p-4 font-medium">{target}</td>
                    <td className="p-4 font-mono whitespace-nowrap">
                      {human}
                      <span className="block text-xs text-gray-500">
                        {bytes} bytes
                      </span>
                    </td>
                    <td className="p-4">
                      <Link
                        href={`${releaseRoot}/${name}`}
                        className="text-cyan-600 dark:text-cyan-400 hover:underline"
                      >
                        binary
                      </Link>
                    </td>
                    <td className="p-4">
                      <Link
                        href={`${releaseRoot}/${name}.sha256`}
                        className="text-cyan-600 dark:text-cyan-400 hover:underline"
                      >
                        checksum
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="rounded-3xl bg-gray-950 text-white p-8 md:p-12 mb-20">
          <h2 className="text-3xl font-bold mb-6">Install or upgrade</h2>
          <pre className="rounded-xl bg-black/40 p-5 overflow-x-auto mb-6">
            <code>{`brew install niklas-heer/tap/kipferl\nkipferl --version`}</code>
          </pre>
          <p className="text-gray-300 leading-relaxed mb-4">
            Already using Kipferl from this tap? Run{" "}
            <code>brew upgrade kipferl</code>. If the old μcharm 0.5 formula is
            installed, replace it once:
          </p>
          <pre className="rounded-xl bg-black/40 p-5 overflow-x-auto mb-6">
            <code>{`brew uninstall --force ucharm\nbrew install niklas-heer/tap/kipferl`}</code>
          </pre>
          <p className="text-gray-300 leading-relaxed">
            Kipferl 0.6 still installs <code>ucharm</code> as a deprecated alias
            for this transition release. Direct downloads and checksum commands
            are in the installation guide.
          </p>
        </section>

        <section className="prose prose-lg dark:prose-invert max-w-none mb-16">
          <h2>What the release gate proved</h2>
          <p>
            The final tag passed 1,669/1,669 available compatibility checks and
            all 19 Vision checks. CI rebuilt the declared PocketPy patch series
            from pristine upstream, built and ran a standalone app on four
            native targets, checked every checksum, and proved that the Linux
            CLI, runtime, and loader have no dynamic interpreter. We repeated
            checksum, core/full build, execution, and Homebrew tests against the
            public release—not just the pre-tag workspace.
          </p>
          <p>
            Rust made the complete runtime larger than the old minimal Zig
            foundation, particularly after adding maintained HTTPS, SQLite,
            Ratatui, archives, and configuration parsers. Profile-based tree
            shaking lets ordinary apps stop paying that cost while preserving
            the higher-quality libraries when their capabilities are used. That
            is the 0.6 tradeoff: optimize developer experience and maintenance
            first, then make the resulting cost selective and measurable.
          </p>
        </section>

        <section className="border-t border-gray-200 dark:border-gray-800 pt-10 flex flex-col gap-3 text-cyan-600 dark:text-cyan-400">
          <Link href="/docs/getting-started/installation">
            Installation and checksum guide →
          </Link>
          <Link href="/docs/commands/dev">Development server reference →</Link>
          <Link href="/docs/modules/data-formats">Data-format guide →</Link>
          <Link href="https://github.com/niklas-heer/kipferl/releases/tag/v0.6.0">
            GitHub release and all assets →
          </Link>
          <Link href="https://github.com/niklas-heer/kipferl/blob/main/CHANGELOG.md">
            Changelog →
          </Link>
          <Link href="https://github.com/niklas-heer/kipferl/releases/download/v0.6.0/compatibility-report.md">
            Published compatibility report →
          </Link>
          <Link href="https://github.com/niklas-heer/kipferl/blob/main/benchmarks/tree_shaking_baseline.md">
            Reproducible size and startup evidence →
          </Link>
          <Link href="https://github.com/niklas-heer/kipferl/compare/v0.5.0...v0.6.0">
            Full v0.5.0…v0.6.0 diff →
          </Link>
        </section>
      </article>
    </main>
  );
}
