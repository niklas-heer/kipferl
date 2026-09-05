import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Blog",
  description: "Engineering notes from the Kipferl project.",
};

export default function BlogPage() {
  return (
    <main className="min-h-screen px-6 py-20 md:py-28">
      <div className="max-w-5xl mx-auto">
        <Link
          href="/"
          className="text-sm text-cyan-600 dark:text-cyan-400 hover:underline"
        >
          ← Kipferl
        </Link>
        <div className="mt-12 mb-14 max-w-3xl">
          <p className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-4">
            ENGINEERING NOTES
          </p>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-6">
            What we learn while building Kipferl
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 leading-relaxed">
            Decisions, measurements, tradeoffs, and the occasional course
            correction behind a compact runtime for Python-style CLI apps.
          </p>
        </div>

        <div className="space-y-6">
          <Link
            href="/blog/kipferl-0-7-rc-1"
            className="group block rounded-3xl border border-cyan-500/30 bg-gradient-to-br from-cyan-500/15 via-transparent to-blue-500/15 p-8 md:p-12 hover:border-cyan-500/60 transition-colors"
          >
            <div className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-5">
              SEPTEMBER 5, 2026 · RELEASE CANDIDATE
            </div>
            <h2 className="text-3xl md:text-5xl font-bold mb-5 group-hover:text-cyan-600 dark:group-hover:text-cyan-400 transition-colors">
              Kipferl 0.7 RC1: packages with evidence
            </h2>
            <p className="text-lg text-gray-600 dark:text-gray-400 max-w-3xl leading-relaxed mb-8">
              A project workflow, compatibility-checked PyPI dependencies,
              native dotted imports, and a release pipeline that tests what you
              download.
            </p>
            <span className="font-semibold text-cyan-600 dark:text-cyan-400">
              Explore the RC and upgrade notes →
            </span>
          </Link>
          <Link
            href="/blog/kipferl-0-6"
            className="group block rounded-3xl border border-gray-200 dark:border-gray-800 bg-gradient-to-br from-cyan-500/15 via-transparent to-purple-500/15 p-8 md:p-12 hover:border-cyan-500/50 transition-colors"
          >
            <div className="font-mono text-sm text-gray-500 mb-5">
              AUGUST 5, 2026 · RELEASE
            </div>
            <h2 className="text-3xl md:text-5xl font-bold mb-5 group-hover:text-cyan-600 dark:group-hover:text-cyan-400 transition-colors">
              Kipferl 0.6: Rust-powered Python CLIs, baked smaller
            </h2>
            <p className="text-lg text-gray-600 dark:text-gray-400 max-w-3xl leading-relaxed mb-8">
              A new name, a stable Rust foundation, a better terminal
              experience, and tree-shaken standalone apps from 1.4 MB.
            </p>
            <span className="font-semibold text-cyan-600 dark:text-cyan-400">
              Read the release story →
            </span>
          </Link>

          <Link
            href="/blog/tree-shaken-builds"
            className="group block rounded-3xl border border-gray-200 dark:border-gray-800 bg-gradient-to-br from-amber-500/10 via-transparent to-cyan-500/10 p-8 md:p-12 hover:border-amber-500/50 transition-colors"
          >
            <div className="font-mono text-sm text-gray-500 mb-5">
              AUGUST 2026 · BUILD ENGINEERING
            </div>
            <h2 className="text-3xl md:text-5xl font-bold mb-5 group-hover:text-amber-600 dark:group-hover:text-amber-400 transition-colors">
              A 1.4 MB Kipferl without putting Rust on your machine
            </h2>
            <p className="text-lg text-gray-600 dark:text-gray-400 max-w-3xl leading-relaxed mb-8">
              How conservative import analysis and prebuilt Rust profiles cut a
              minimal standalone app by 69.9% while preserving the full runtime
              as a safe fallback.
            </p>
            <span className="font-semibold text-amber-600 dark:text-amber-400">
              Read the implementation story →
            </span>
          </Link>

          <Link
            href="/blog/rust-migration"
            className="group block rounded-3xl border border-gray-200 dark:border-gray-800 bg-gradient-to-br from-cyan-500/10 via-transparent to-blue-500/10 p-8 md:p-12 hover:border-cyan-500/50 transition-colors"
          >
            <div className="font-mono text-sm text-gray-500 mb-5">
              AUGUST 2026 · MIGRATION RETROSPECTIVE
            </div>
            <h2 className="text-3xl md:text-5xl font-bold mb-5 group-hover:text-cyan-600 dark:group-hover:text-cyan-400 transition-colors">
              From Zig to Rust without stopping the roadmap
            </h2>
            <p className="text-lg text-gray-600 dark:text-gray-400 max-w-3xl leading-relaxed mb-8">
              Why we changed foundations, how compatibility gates made an
              incremental rewrite possible, and what the public Rust release
              candidate actually costs and improves.
            </p>
            <span className="font-semibold text-cyan-600 dark:text-cyan-400">
              Read the story →
            </span>
          </Link>
        </div>
      </div>
    </main>
  );
}
