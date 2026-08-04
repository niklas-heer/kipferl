import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Blog",
  description: "Engineering notes from the μcharm project.",
};

export default function BlogPage() {
  return (
    <main className="min-h-screen px-6 py-20 md:py-28">
      <div className="max-w-5xl mx-auto">
        <Link
          href="/"
          className="text-sm text-cyan-600 dark:text-cyan-400 hover:underline"
        >
          ← μcharm
        </Link>
        <div className="mt-12 mb-14 max-w-3xl">
          <p className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-4">
            ENGINEERING NOTES
          </p>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-6">
            What we learn while building μcharm
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 leading-relaxed">
            Decisions, measurements, tradeoffs, and the occasional course
            correction behind a compact runtime for Python-style CLI apps.
          </p>
        </div>

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
    </main>
  );
}
