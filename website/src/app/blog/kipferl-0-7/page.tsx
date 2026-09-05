import type { Metadata } from "next";
import Link from "next/link";

const title = "Kipferl 0.7: packages with evidence";
const description =
  "Try the new project workflow, compatibility-checked PyPI dependencies, and native dotted imports in Kipferl v0.7.0.";

export const metadata: Metadata = {
  title,
  description,
  openGraph: {
    title,
    description,
    type: "article",
    publishedTime: "2026-09-05T14:07:05Z",
    url: "https://kipferl.dev/blog/kipferl-0-7",
  },
};

const releaseUrl = "https://github.com/niklas-heer/kipferl/releases/tag/v0.7.0";
const downloadRoot =
  "https://github.com/niklas-heer/kipferl/releases/download/v0.7.0";
const assets = [
  ["macOS Apple Silicon", "kipferl-macos-aarch64"],
  ["macOS Intel", "kipferl-macos-x86_64"],
  ["Linux ARM64", "kipferl-linux-aarch64"],
  ["Linux x86_64", "kipferl-linux-x86_64"],
] as const;

export default function StableReleasePage() {
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
        <header className="mb-12">
          <p className="font-mono text-sm text-cyan-600 dark:text-cyan-400 mb-5">
            SEPTEMBER 5, 2026 · STABLE RELEASE
          </p>
          <h1 className="text-5xl md:text-7xl font-bold tracking-tight leading-[1.05] mb-8">
            {title}
          </h1>
          <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 leading-relaxed">
            Start a project, check its PyPI dependencies, and ship an executable
            that carries its modules and resources. Version 0.7.0 is the stable
            release of the project and package workflow introduced in RC1.
          </p>
        </header>

        <aside className="mb-14 rounded-2xl border border-cyan-500/30 bg-cyan-500/5 p-6 text-gray-700 dark:text-gray-300">
          <strong>Install stable 0.7.0 with Homebrew.</strong> Run{" "}
          <code>brew install niklas-heer/tap/kipferl</code>, or{" "}
          <code>brew update</code> and <code>brew upgrade kipferl</code> for an
          existing installation. The{" "}
          <Link
            className="text-cyan-600 dark:text-cyan-400 underline"
            href="/docs/getting-started/installation#stable-release"
          >
            installation guide
          </Link>{" "}
          also provides direct downloads and checksum verification.
        </aside>

        <div className="prose prose-lg dark:prose-invert max-w-none">
          <h2>A complete project workflow</h2>
          <p>
            CLI, API, and interactive starters now include project
            configuration, editor support, tests, and a README. Local modules
            and resources travel with your application, and project commands use
            the same configuration during development, testing, and packaging.
          </p>
          <pre>
            <code>{`kipferl new hello --template cli
cd hello
kipferl run
kipferl test
kipferl build
./dist/hello --help`}</code>
          </pre>
          <p>
            The{" "}
            <Link href="/docs/getting-started/quick-start">quick start</Link>{" "}
            walks through this workflow. The{" "}
            <Link href="/docs/guides/recipes">tested recipes</Link> show
            complete applications and their standalone builds.
          </p>

          <h2>PyPI dependencies with explicit compatibility checks</h2>
          <p>
            The new package manager resolves dependencies, verifies wheel
            contents, and compiles their Python sources before publishing an
            installation. It checks an offline catalog of evidence for the exact
            wheel, runtime, and platform. The lock records those identities for
            reproducible restoration.
          </p>
          <pre>
            <code>{`kipferl deps catalog
kipferl add 'tzdata==2025.2'
kipferl deps check
kipferl sync --locked --offline`}</code>
          </pre>
          <p>
            Each of the four release CLIs includes a fresh, reviewed positive
            record for tzdata 2025.2. Its test covers version constants and four
            representative TZif resource headers. This establishes resource
            loading for those checks; it does not provide zoneinfo or establish
            timezone conversion behavior. Offline restoration needs the wheels
            already present in the project cache.
          </p>
          <p>
            Pure Python wheels are supported. Native extensions, source builds,
            extras, and environment markers remain unsupported. Missing runtime
            APIs and untested application paths can still prevent a library from
            working. <code>--allow-unverified</code> lets you evaluate packages
            without matching positive evidence; it cannot bypass a demonstrated
            incompatibility. The{" "}
            <Link href="/docs/guides/packages">package guide</Link> explains
            these decisions and their recovery steps.
          </p>

          <h2>More Python package syntax</h2>
          <p>
            Native dotted imports now initialize parent packages, bind roots and
            aliases correctly, and handle circular imports and failed-import
            retries. Relative from-imports work, along with trailing commas in
            parenthesized imports and parameter lists and adjacent plain string
            and bytes literals.
          </p>
          <pre>
            <code>{`import http.client
import urllib.parse as urls

print(urls.quote("hello world"))`}</code>
          </pre>
          <p>
            These changes address frequent blockers in our pinned 1,000-package
            screen. Dotted imports moved all 170 affected first failures
            forward: four packages completed compilation, while 166 exposed
            later blockers. Across the full screen, 44 releases completed
            compilation, including 24 containing Python source. Compilation is
            only one compatibility check; those numbers are not a count of
            working libraries.
          </p>
          <p>
            The <Link href="/docs/guides/package-audit">searchable audit</Link>{" "}
            retains its original package versions and recorded runtime hash.
            Fresh per-platform release catalogs provide separate evidence for
            the binaries in this release.
          </p>

          <h2>Two upgrade changes to review</h2>
          <ul>
            <li>
              <strong>Dynamic imports return the root by default.</strong>{" "}
              <code>__import__("http.client")</code> now returns{" "}
              <code>http</code>. To keep a child-module alias, use{" "}
              <code>import http.client as http</code>, or pass a nonempty
              positional fromlist to the dynamic import. Nonzero dynamic import
              levels, namespace packages, and custom import finders remain
              unsupported.
            </li>
            <li>
              <strong>Locks belong to an exact runtime and platform.</strong>{" "}
              After changing runtimes, rerun <code>kipferl add</code> for your
              declared requirements, review the resulting lock, run your tests,
              and commit the configuration and lock together. Do not edit hashes
              to bypass a mismatch. Repeat <code>--allow-unverified</code> only
              for dependencies you intentionally accept and test.
            </li>
          </ul>
          <p>
            Follow the{" "}
            <Link href="/docs/guides/packages#upgrade-to-070">
              upgrade notes
            </Link>{" "}
            before updating an existing application.
          </p>

          <h2>Testing the binaries you download</h2>
          <p>
            CI and the release pipeline passed on macOS Apple Silicon, macOS
            Intel, Linux ARM64, and Linux x86_64. The pipeline verifies exact
            component versions and checksums, generates runtime-specific catalog
            evidence, and tests online installation, locked offline restoration,
            and standalone execution after deleting the project and caches.
            macOS offline checks deny network access through sandbox-exec; Linux
            exercises the CLI’s offline mode on disposable runners.
          </p>
          <p>
            The stable release compatibility run passed 1,725 available checks,
            with 22 explicit dependency skips. The published stable Apple
            Silicon download also passed checksum verification and a fresh
            package-install, offline-restore, and detached-executable smoke
            test.
          </p>
          <p>
            The <a href={releaseUrl}>GitHub release</a> includes executables and
            checksums, per-platform catalogs and package smoke reports, plus
            compatibility, Vision, and vendor-patch verification reports. See
            the{" "}
            <a href="https://github.com/niklas-heer/kipferl/actions/runs/33970383103">
              stable release workflow
            </a>{" "}
            for the build and validation logs.
          </p>

          <h2>Download v0.7.0</h2>
          <div className="overflow-x-auto">
            <table>
              <thead>
                <tr>
                  <th>Platform</th>
                  <th>CLI</th>
                  <th>Checksum</th>
                </tr>
              </thead>
              <tbody>
                {assets.map(([platform, asset]) => (
                  <tr key={asset}>
                    <td>{platform}</td>
                    <td>
                      <a href={`${downloadRoot}/${asset}`}>Download</a>
                    </td>
                    <td>
                      <a href={`${downloadRoot}/${asset}.sha256`}>SHA-256</a>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p>
            Use the{" "}
            <Link href="/docs/getting-started/installation#stable-release">
              installation guide
            </Link>{" "}
            for checksum commands and upgrade instructions. Try the release with
            your project and report reproducible problems through{" "}
            <a href="https://github.com/niklas-heer/kipferl/issues">
              GitHub issues
            </a>
            , including the version, platform, and smallest example that fails.
          </p>
        </div>
      </article>
    </main>
  );
}
