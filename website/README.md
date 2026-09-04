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
