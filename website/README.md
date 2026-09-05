# Kipferl website

The documentation and landing site use Next.js, Fumadocs, and the repository's
pinned Node.js/Bun tools. Run these commands from the repository root after the
[mise setup](../README.md#development):

```bash
mise run website-dev
mise run website-check
```

Open [localhost:3000](http://localhost:3000) during development. `website-check`
installs dependencies from `website/bun.lock`, checks generated MDX/route types,
and builds the production site. Use Bun for dependency changes; do not add an
npm, pnpm, or Yarn lockfile.

| Location | Purpose |
| --- | --- |
| `content/docs/` | MDX documentation; `meta.json` defines navigation |
| `src/app/(home)/page.tsx` | Landing page |
| `src/app/blog/` | Dated release stories and engineering articles |
| `src/app/docs/[[...slug]]/page.tsx` | Shared documentation rendering and source-version notice |
| `src/lib/source.ts` | Content loading and machine-readable documentation |
| `src/app/api/search/route.ts` | Search index |
| `src/app/llms-full.txt/route.ts` | Complete documentation text |
| `public/` | Images and demo recording |

The homepage recording is real terminal output from the published v0.7.1 CLI.
Install VHS, ttyd, FFmpeg (including ffprobe), WebP (`cwebp`), and the browser required by VHS,
then run from the repository root:

```bash
KIPFERL_DEMO_CLI=/absolute/path/to/published/kipferl mise run demo-gif
```

The default binary path is `target/published-stable-verification/kipferl-macos-aarch64`.
Use the published binary for your host: a local rebuild can have a different
runtime hash and will not inherit the release's tested package evidence.
The task checks version 0.7.1, runs `demo.tape` in a fresh temporary workspace,
and writes `public/demos/kipferl-0.7.1.mp4` and its lossless WebP poster at 1280×800.
It requires network access to PyPI for the real `tzdata==2025.2` download.
No output is mocked; any failed command invalidates the recording.

The three chapters create/run/test the generated starter, install and check the
reviewed package, then build `scripts/demo/zones.py` as a universal executable.
The recording copies that checked-in fixture into the project and displays it.
It validates the bundled UTC file's TZif header; this is not a demonstration of
timezone conversion APIs. Finally it deletes the temporary project and its
caches and runs the executable from a separate directory. Only that temporary
workspace is removed. Update the homepage's chapter seek times after re-recording
because actual download and execution times can vary. The poster is extracted
at second 27; override `KIPFERL_DEMO_POSTER_SECOND` if that view has shifted.
The root `demo.gif` and `public/demo.gif` are refreshed from this same recording,
so historical article embeds show the current branding. Their captions identify
the newer workflow rather than implying it was available in the older release.

Documentation covers stable v0.7.1 and the current `main` checkout. Clearly
label future source-only changes, and keep the shared notice and AI-readable
export aligned with the version offered by the installation guide.
Preserve the historical scope of release articles and performance measurements;
do not imply that rebuilding source updates the CLI's embedded release assets.
The public contributor guide lives in `content/docs/guides/development.mdx`.

Recipe blocks in `content/docs/guides/recipes.mdx` mirror `examples/recipes/`.
From the repository root, run:

```bash
mise exec -- python3 scripts/check_recipes.py --docs-only
mise run recipes
```

The first checks snippet drift. The second also runs the examples and packaged
binaries with local fixtures. Keep internal links consistent with navigation;
new guide pages are automatically included in search, page metadata, and the
machine-readable docs.
The package audit page reads `../compatibility/packages/popularity-audit.json`
at build time. Keep the repository's `compatibility/` directory available when
building the site; do not copy rows into MDX or public assets. The adapter
validates coverage metadata and maps only display fields into the interactive
search/filter table. Regenerate the canonical audit before rebuilding to publish
new results. JSON, CSV, and Markdown evidence links point to the repository.

Run the website checks directly from `website/` when working on its components:

```bash
mise exec -- bun run lint
mise exec -- bun run test
mise exec -- bun run types:check
mise exec -- bun run build
```
