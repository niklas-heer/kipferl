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

Documentation tracks the current `main` checkout, including unreleased work.
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
