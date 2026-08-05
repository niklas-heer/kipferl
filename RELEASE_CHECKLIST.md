# v0.6.0 Release Checklist

This checklist separates reversible preparation from the stable tag that
publishes artifacts and updates Homebrew.

## Prepared in the release PR

- [x] Set `VERSION`, the Cargo workspace, and lockfile packages to `0.6.0`.
- [x] Add curated GitHub release notes and a changelog.
- [x] Update README, website homepage, docs, generated-project guidance,
  examples, roadmap, and release-note fallback facts for tree shaking.
- [x] Keep the long-form release post outside routed website content until the
  published artifacts can be verified.
- [x] Make Homebrew checksum downloads fail on HTTP errors.
- [x] Confirm `HOMEBREW_TAP_TOKEN` and `OPENROUTER_API_KEY` are configured.

## Pre-tag gate

- [x] Merge the release-preparation PR after every required check passes.
- [x] Confirm `main` is clean and synchronized with `origin/main`.
- [x] Confirm `v0.6.0` did not exist locally or on GitHub before tagging.
- [x] Run `cargo test --workspace` and both full/core Clippy gates on the exact
  commit to tag.
- [x] Run release-tool tests, compatibility, PocketPy patch verification, and
  the website production build on the exact commit to tag.

## Publish

From clean `main`, run `kipferl run scripts/release.py` and select **Final
v0.6.0**. The script recognizes the version files prepared by the PR, tags the
current commit, and pushes the tag. Do not create the tag from this branch.

The tag workflow must:

- [x] Build full and tree-shaken core runtimes for all four targets.
- [x] Build and smoke-test all four CLI assets.
- [x] Prove Linux CLI/runtime/loader assets have no dynamic interpreter.
- [x] Publish every binary with an adjacent SHA-256 file.
- [x] Publish compatibility and PocketPy patch-verification evidence.
- [x] Use `.github/release-notes/v0.6.0.md` for the GitHub release.
- [x] Update `niklas-heer/homebrew-tap` only after the stable release exists.
  The workflow token lacked tap write permission, so release commit `ecd46b0`
  was published with the authenticated maintainer session after verifying the
  generated formula. The workflow now fails early with the exact permission
  requirement; `HOMEBREW_TAP_TOKEN` still needs Contents write access before
  the next stable release.

## Verify after publication

- [x] Download each CLI and checksum from the GitHub release and verify all
  four hashes.
- [x] Record exact CLI asset sizes in the routed release post.
- [x] Run `--version`, a minimal core build, a full-capability build, and
  `--full-runtime` from a downloaded macOS asset.
- [x] Verify both Linux assets with `file`/`readelf` or equivalent evidence.
- [x] Install and upgrade `niklas-heer/tap/kipferl`; verify `kipferl` and the
  deprecated `ucharm` alias.
- [x] Convert `website/content/drafts/v0.6.0-release.md` into the routed release
  post, replace every explicit placeholder, add it to the blog index, and link
  it from the homepage, README, docs, and GitHub release.
- [x] Run website lint, type generation, type checks, production build, and
  mobile/desktop visual review; verify the deployed page and external links.
- [x] Update and close GitHub issue #58 only after the deployed post is live.
