"use client";

import Image from "next/image";
import Link from "next/link";
import { useState } from "react";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      type="button"
      onClick={copy}
      className="text-gray-400 hover:text-white transition-colors"
      title="Copy to clipboard"
    >
      {copied ? (
        <svg
          aria-hidden="true"
          className="w-4 h-4 text-green-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M5 13l4 4L19 7"
          />
        </svg>
      ) : (
        <svg
          aria-hidden="true"
          className="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
      )}
    </button>
  );
}

function StatCard({
  value,
  label,
  suffix = "",
}: {
  value: string;
  label: string;
  suffix?: string;
}) {
  return (
    <div className="text-center">
      <div className="text-4xl md:text-5xl font-bold bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
        {value}
        <span className="text-2xl">{suffix}</span>
      </div>
      <div className="text-gray-500 dark:text-gray-400 mt-1">{label}</div>
    </div>
  );
}

function FeatureCard({
  icon,
  title,
  description,
  gradient,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  gradient: string;
}) {
  return (
    <div className="group relative p-6 rounded-2xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900/50 hover:border-cyan-500/50 transition-all duration-300 hover:shadow-lg hover:shadow-cyan-500/10">
      <div className={`inline-flex p-3 rounded-xl ${gradient} mb-4`}>
        {icon}
      </div>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <p className="text-gray-600 dark:text-gray-400 text-sm leading-relaxed">
        {description}
      </p>
    </div>
  );
}

function ComparisonRow({
  feature,
  kipferl,
  python,
  node,
}: {
  feature: string;
  kipferl: string;
  python: string;
  node: string;
}) {
  return (
    <tr className="border-b border-gray-200 dark:border-gray-800">
      <td className="py-4 px-4 font-medium">{feature}</td>
      <td className="py-4 px-4 text-center">
        <span className="text-green-500 font-semibold">{kipferl}</span>
      </td>
      <td className="py-4 px-4 text-center text-gray-500">{python}</td>
      <td className="py-4 px-4 text-center text-gray-500">{node}</td>
    </tr>
  );
}

export default function HomePage() {
  return (
    <div className="flex flex-col overflow-hidden">
      {/* Hero Section */}
      <section className="relative py-24 md:py-32 px-6">
        {/* Background gradient */}
        <div className="absolute inset-0 -z-10 overflow-hidden">
          <div className="absolute top-0 left-1/4 w-96 h-96 bg-cyan-500/20 rounded-full blur-3xl" />
          <div className="absolute top-20 right-1/4 w-96 h-96 bg-blue-500/20 rounded-full blur-3xl" />
          <div className="absolute -top-40 right-0 w-80 h-80 bg-purple-500/10 rounded-full blur-3xl" />
        </div>

        <div className="max-w-5xl mx-auto text-center">
          {/* Logo */}
          <div className="mb-8">
            <Image
              src="/kipferl-logo.png"
              alt="Kipferl logo"
              width={80}
              height={80}
              className="mx-auto"
              priority
            />
          </div>

          {/* Badge */}
          <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-600 dark:text-cyan-400 text-sm font-medium mb-8">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-cyan-500"></span>
            </span>
            Rust v0.6.0-rc.2 is available now
          </div>

          <h1 className="text-5xl md:text-7xl font-bold mb-6 tracking-tight leading-[1.1]">
            Build CLI apps that
            <br />
            <span className="bg-gradient-to-r from-cyan-400 via-blue-500 to-purple-500 bg-clip-text text-transparent">
              developers love
            </span>
          </h1>

          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 mb-10 max-w-3xl mx-auto leading-relaxed">
            Python syntax. Standalone binaries around 5–6 MB.
            <span className="text-gray-900 dark:text-white font-medium">
              {" "}
              Instant startup.
            </span>
            <br className="hidden md:block" />
            Ship beautiful command-line tools without the bloat.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
            <Link
              href="/docs/getting-started/installation"
              className="group px-8 py-4 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-semibold rounded-xl transition-all duration-200 shadow-lg shadow-cyan-500/25 hover:shadow-xl hover:shadow-cyan-500/30 hover:-translate-y-0.5"
            >
              Get Started
              <span className="inline-block ml-2 group-hover:translate-x-1 transition-transform">
                →
              </span>
            </Link>
            <Link
              href="https://github.com/niklas-heer/kipferl"
              className="group px-8 py-4 border border-gray-300 dark:border-gray-700 hover:border-gray-400 dark:hover:border-gray-600 font-semibold rounded-xl transition-all duration-200 hover:bg-gray-50 dark:hover:bg-gray-800/50"
            >
              <svg
                aria-hidden="true"
                className="inline-block w-5 h-5 mr-2 -mt-0.5"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
              </svg>
              Star on GitHub
            </Link>
          </div>

          {/* Install command */}
          <div className="inline-flex items-center gap-4 px-5 py-3 bg-gray-900 dark:bg-gray-950 rounded-xl font-mono text-sm border border-gray-800">
            <span className="text-cyan-400">$</span>
            <code className="text-gray-100">
              curl -LO …/v0.6.0-rc.2/kipferl-macos-aarch64
            </code>
            <CopyButton text="curl -LO https://github.com/niklas-heer/kipferl/releases/download/v0.6.0-rc.2/kipferl-macos-aarch64" />
          </div>
        </div>
      </section>

      {/* Demo GIF Section */}
      <section className="py-20 px-6 bg-gradient-to-b from-gray-50 to-white dark:from-gray-900/50 dark:to-gray-950">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              See it in action
            </h2>
            <p className="text-gray-600 dark:text-gray-400 text-lg">
              Beautiful output and interactive prompts, built-in.
            </p>
          </div>
          <div className="rounded-xl overflow-hidden border border-gray-200 dark:border-gray-800 shadow-2xl shadow-black/20">
            <Image
              src="/demo.gif"
              alt="kipferl demo showing CLI app with interactive prompts and beautiful output"
              width={720}
              height={520}
              className="w-full"
              unoptimized
            />
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-20 px-6">
        <div className="max-w-4xl mx-auto">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12">
            <StatCard value="7.044" suffix="ms" label="Median startup" />
            <StatCard value="4.8" suffix="MB" label="Minimal app" />
            <StatCard value="1,669" label="Compat checks" />
            <StatCard value="4" label="Release targets" />
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 px-6 bg-gradient-to-b from-white to-gray-50 dark:from-gray-950 dark:to-gray-900/50">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              Everything you need
            </h2>
            <p className="text-gray-600 dark:text-gray-400 text-lg max-w-2xl mx-auto">
              A complete toolkit for building modern CLI applications, without
              the complexity.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-yellow-400 to-orange-500"
              title="Instant Startup"
              description="A measured 7.044 ms median and 7.980 ms p95 on Apple Silicon, without virtual-environment activation."
            />
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-green-400 to-emerald-500"
              title="Standalone Binaries"
              description="Target-specific executables around 4.8–5.8 MB. Linux releases are static musl binaries with no dynamic libc dependency."
            />
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-blue-400 to-indigo-500"
              title="Python Syntax"
              description="Write in familiar Python. No new language to learn—just better, faster tooling for the CLI."
            />
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-purple-400 to-pink-500"
              title="Beautiful Output"
              description="Tables, boxes, progress bars, spinners, and rich colors. All built-in, zero dependencies."
            />
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-cyan-400 to-blue-500"
              title="Interactive Prompts"
              description="Select, multiselect, confirm, and password inputs with smooth keyboard navigation."
            />
            <FeatureCard
              icon={
                <svg
                  aria-hidden="true"
                  className="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                  />
                </svg>
              }
              gradient="bg-gradient-to-br from-red-400 to-rose-500"
              title="50+ Runtime Modules"
              description="HTTP, SQLite, JSON, regex, subprocess, and more—hosted by Rust with compatibility tests against CPython."
            />
          </div>
        </div>
      </section>

      {/* Comparison Section */}
      <section className="py-20 px-6">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              How it compares
            </h2>
            <p className="text-gray-600 dark:text-gray-400 text-lg">
              kipferl vs traditional CLI tooling
            </p>
          </div>

          <div className="overflow-x-auto rounded-xl border border-gray-200 dark:border-gray-800">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr className="border-b border-gray-200 dark:border-gray-800">
                  <th className="py-4 px-4 text-left font-semibold">Feature</th>
                  <th className="py-4 px-4 text-center font-semibold text-cyan-500">
                    kipferl
                  </th>
                  <th className="py-4 px-4 text-center font-semibold">
                    Python + Click
                  </th>
                  <th className="py-4 px-4 text-center font-semibold">
                    Node.js
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-950">
                <ComparisonRow
                  feature="Cold start"
                  kipferl="7.044ms"
                  python="Runtime-dependent"
                  node="Runtime-dependent"
                />
                <ComparisonRow
                  feature="Binary size"
                  kipferl="~4.8MB"
                  python="Runtime + app"
                  node="Runtime + app"
                />
                <ComparisonRow
                  feature="Dependencies"
                  kipferl="None"
                  python="pip + venv"
                  node="node_modules"
                />
                <ComparisonRow
                  feature="Distribution"
                  kipferl="Single file"
                  python="Complex"
                  node="Complex"
                />
                <ComparisonRow
                  feature="TUI built-in"
                  kipferl="Yes"
                  python="Requires Rich"
                  node="Requires libs"
                />
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="py-20 px-6 bg-gradient-to-b from-gray-50 to-white dark:from-gray-900/50 dark:to-gray-950">
        <div className="max-w-5xl mx-auto">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-3xl md:text-4xl font-bold mb-6">
                Simple, expressive API
              </h2>
              <p className="text-gray-600 dark:text-gray-400 text-lg mb-6 leading-relaxed">
                Everything you need to build great CLI apps in a clean, Pythonic
                interface. No boilerplate, no configuration—just write code and
                ship.
              </p>
              <ul className="space-y-3">
                {[
                  "Beautiful boxes and tables",
                  "Interactive prompts with keyboard nav",
                  "Progress bars and spinners",
                  "Colored status messages",
                  "HTTP requests with built-in TLS",
                ].map((item) => (
                  <li key={item} className="flex items-center gap-3">
                    <svg
                      aria-hidden="true"
                      className="w-5 h-5 text-green-500 flex-shrink-0"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                    <span className="text-gray-700 dark:text-gray-300">
                      {item}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
            <div className="rounded-xl overflow-hidden border border-gray-200 dark:border-gray-800 shadow-2xl shadow-black/10">
              <div className="bg-gray-900 px-4 py-3 flex items-center gap-2 border-b border-gray-800">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-red-500" />
                  <div className="w-3 h-3 rounded-full bg-yellow-500" />
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                </div>
                <span className="text-sm text-gray-400 ml-2 font-mono">
                  app.py
                </span>
              </div>
              <pre className="bg-gray-950 p-6 overflow-x-auto text-sm leading-relaxed">
                <code className="text-gray-100 font-mono">
                  <span className="text-rose-400">import</span>
                  {" tui\n"}
                  <span className="text-rose-400">import</span>
                  {" input\n\n"}
                  <span className="text-gray-500"># Beautiful boxes</span>
                  {"\ntui.box("}
                  <span className="text-amber-300">{'"Deploying..."'}</span>
                  {", title="}
                  <span className="text-amber-300">{'"Release"'}</span>
                  {")\n\n"}
                  <span className="text-gray-500"># Interactive prompts</span>
                  {"\nenv = input.select("}
                  <span className="text-amber-300">{'"Environment:"'}</span>
                  {", ["}
                  <span className="text-amber-300">{'"dev"'}</span>
                  {", "}
                  <span className="text-amber-300">{'"prod"'}</span>
                  {"])\n\n"}
                  <span className="text-gray-500"># Formatted tables</span>
                  {"\ntui.table([\n    ["}
                  <span className="text-amber-300">{'"Artifact"'}</span>
                  {", "}
                  <span className="text-amber-300">{'"Size"'}</span>
                  {"],\n    ["}
                  <span className="text-amber-300">{'"app"'}</span>
                  {", "}
                  <span className="text-amber-300">{'"4.8MB"'}</span>
                  {"],\n], headers="}
                  <span className="text-purple-400">True</span>
                  {")\n\n"}
                  {"tui.success("}
                  <span className="text-amber-300">f"Deployed to </span>
                  <span className="text-gray-100">{"{env}"}</span>
                  <span className="text-amber-300">!"</span>
                  {")"}
                </code>
              </pre>
            </div>
          </div>
        </div>
      </section>

      {/* Migration story */}
      <section className="py-20 px-6 bg-gray-950 text-white">
        <div className="max-w-5xl mx-auto grid lg:grid-cols-[1.15fr_0.85fr] gap-12 items-center">
          <div>
            <div className="text-cyan-400 font-mono text-sm mb-4">
              ENGINEERING RETROSPECTIVE
            </div>
            <h2 className="text-3xl md:text-5xl font-bold mb-6 leading-tight">
              Why we rewrote Kipferl in Rust
            </h2>
            <p className="text-gray-300 text-lg leading-relaxed mb-8">
              We migrated the CLI, loader, runtime host, and native modules
              incrementally—keeping the application format stable and using
              compatibility as the release gate. The result is easier to
              maintain, fully tested on four targets, and still compact enough
              to ship as one file.
            </p>
            <Link
              href="/blog/rust-migration"
              className="inline-flex items-center font-semibold text-cyan-400 hover:text-cyan-300 transition-colors"
            >
              Read the migration story <span className="ml-2">→</span>
            </Link>
          </div>
          <div className="grid grid-cols-2 gap-4">
            {[
              ["1,669 / 1,669", "compatibility checks"],
              ["85,310", "obsolete lines removed"],
              ["7.044 ms", "median startup"],
              ["4", "native release targets"],
            ].map(([value, label]) => (
              <div
                key={label}
                className="rounded-2xl border border-gray-800 bg-gray-900 p-5"
              >
                <div className="text-2xl font-bold text-white mb-1">
                  {value}
                </div>
                <div className="text-sm text-gray-400">{label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24 px-6 relative overflow-hidden">
        {/* Background gradient */}
        <div className="absolute inset-0 -z-10">
          <div className="absolute bottom-0 left-1/4 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl" />
          <div className="absolute bottom-20 right-1/4 w-96 h-96 bg-blue-500/10 rounded-full blur-3xl" />
        </div>

        <div className="max-w-3xl mx-auto text-center">
          <h2 className="text-4xl md:text-5xl font-bold mb-6">
            Ready to build something
            <br />
            <span className="bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
              beautiful?
            </span>
          </h2>
          <p className="text-xl text-gray-600 dark:text-gray-400 mb-10">
            Get started in under a minute. Ship your first CLI app today.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              href="/docs/getting-started/installation"
              className="group px-8 py-4 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-semibold rounded-xl transition-all duration-200 shadow-lg shadow-cyan-500/25 hover:shadow-xl hover:shadow-cyan-500/30 hover:-translate-y-0.5"
            >
              Read the Docs
              <span className="inline-block ml-2 group-hover:translate-x-1 transition-transform">
                →
              </span>
            </Link>
            <Link
              href="https://github.com/niklas-heer/kipferl"
              className="px-8 py-4 border border-gray-300 dark:border-gray-700 font-semibold rounded-xl transition-all duration-200 hover:bg-gray-50 dark:hover:bg-gray-800/50"
            >
              View Examples
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 px-6 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900/30">
        <div className="max-w-5xl mx-auto">
          <div className="flex flex-col md:flex-row justify-between items-center gap-6">
            <div className="flex items-center gap-2">
              <span className="text-2xl font-bold">
                <span className="text-cyan-500">u</span>charm
              </span>
              <span className="text-gray-400 dark:text-gray-500">|</span>
              <span className="text-gray-600 dark:text-gray-400">
                Beautiful CLI apps, tiny binaries.
              </span>
            </div>
            <div className="flex items-center gap-8">
              <Link
                href="/docs"
                className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
              >
                Docs
              </Link>
              <Link
                href="/blog"
                className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
              >
                Blog
              </Link>
              <Link
                href="https://github.com/niklas-heer/kipferl"
                className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
              >
                GitHub
              </Link>
              <Link
                href="https://github.com/niklas-heer/kipferl/issues"
                className="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
              >
                Issues
              </Link>
            </div>
          </div>
          <div className="mt-8 pt-8 border-t border-gray-200 dark:border-gray-800 text-center text-sm text-gray-500">
            Built with Rust and PocketPy. Open source under the MIT license.
          </div>
        </div>
      </footer>
    </div>
  );
}
