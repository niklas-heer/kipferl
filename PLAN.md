# Kipferl Plan and Roadmap

This is the single source of truth for priorities and next steps.

## Snapshot

- Goal: build beautiful CLI apps with Python syntax, shipped as compact, fast,
  standalone binaries.
- Runtime: PocketPy; the Rust host, loader, CLI, 51 standard-library compatibility
  groups, Cargo-first release path, and tree-shaken build profiles are
  implemented. RC2 is proven and stable `v0.6.0` has been published.
- Language decision: Rust is the target implementation language. See `RUST_MIGRATION.md` for gates and sequencing.
- Current source compatibility: 1,725/1,725 available checks on pinned CPython
  3.12.14 (2026-09-05), with 51 passing groups and one unavailable third-party
  `toml` baseline. The 52 groups cover 51 of the 160 stdlib inventory modules.
  Refresh with `mise run compat`; totals vary with the baseline Python version.
- PocketPy vendor patches are tracked under `pocketpy/patches/` and verified via `python3 scripts/verify-pocketpy-patches.py --check-upstream`.

## Current State (from the repo)

- Native modules cover TUI presentation (`tui`) and interaction (`input`),
  terminal + ANSI, and a growing stdlib set (copy, fnmatch, typing, csv,
  datetime, json, subprocess, signal, logging, etc.).
- The Rust loader and CLI build and run universal binaries; the Rust CLI tests and `tests/compat_runner.py` provide compatibility tooling.
- Canonical project stubs live in `stubs/`; embedded release components and
  generated-project templates live under `crates/kipferl-cli/`.
- CPython tests are vendored under `tests/cpython/` and are used to track parity.

## Stable v0.6.0 Release

- Phase 5 implementation is complete at 1,669/1,669 available checks.
- Phase 6 has cut the canonical CI, release workflow, embedded assets, public
  binary names, local commands, and contributor guidance over to Rust/Cargo.
- The Rust optimization, safety, native CI, and prerelease gates are complete.
- Profile-based tree shaking, four-target core assets, release packaging,
  documentation, reproducible measurements, and the technical post are
  complete under issue #57.
- `kipferl dev` provides a Rust-native watch/restart loop with debounced native
  filesystem events, extra watch paths, terminal clearing, and terminal-state
  restoration for interactive applications.
- Common configuration formats are now a first-class runtime surface: JSON,
  YAML 1.2, TOML, KDL 2.0, XML, CSV, and INI/CFG have documented string and/or
  file APIs. YAML, TOML, and KDL use maintained Rust parsers; generated stubs
  and `kipferl dev` cover the new modules and extensions.
- The archived Zig implementation has been removed from the working tree; use
  final Zig tag `v0.5.0` for archaeology.
- Preserve PocketPy, the Python-facing API, `MCHARM01` universal binaries, and all current compatibility tests.
- Establish the Rust/PocketPy FFI and four-target build proof before translating the large runtime module surface.
- Port in releasable slices: loader, CLI using existing assets, runtime foundation, then module waves.
- Preserve the frozen v0.5.0 golden and differential fixtures that define
  compatibility, even though the archived implementation is no longer built.
- Do not retain Zig indirectly through `cargo-zigbuild` in the final release pipeline.

The detailed inventory, architecture, acceptance gates, PR sequence, and risks are in `RUST_MIGRATION.md`.

## Completed Runtime Decision: PocketPy

### Phase 1: Runtime switch + tooling (COMPLETE)
- PocketPy is now the default and only runtime.
- MicroPython has been removed from the project.
- `tests/compat_runner.py` defaults to PocketPy.

### Phase 2: Module parity + API surface (COMPLETE FOR CURRENT TARGET SET)
- Native modules remain the standard during the Rust port; avoid introducing temporary Python fallbacks that change behavior.
- Targeted stdlib parity is now at 100% for the current module set (see `tests/compat_report_pocketpy.md`).
- Maintain parity by re-running `python3 tests/compat_runner.py --report` after runtime changes.

### Phase 3: DX + packaging alignment
- **Complete:** align stubs with the PocketPy import surface and regenerate
  with a single command.
- **Complete:** templates and examples reference the PocketPy runtime and the
  direct `tui`/`input` module interface.
- **Complete:** CI generates and uploads the PocketPy compatibility report,
  fails on compatibility regressions, and publishes a machine-readable proof
  that the declared PocketPy patches replay onto pristine upstream and exactly
  reconstruct the vendored source.

## Current Source Improvements (unreleased)

- Configurable projects: cli/api/interactive templates, `kipferl.json` defaults,
  source-aware diagnostics, project tests, and Bash/Zsh/Fish completions.
- Portable packaging: local modules/packages, explicit assets, static-import
  diagnostics, atomic output replacement, and validation of cached payloads.
- PyPI pure-wheel resolution, exact artifact/runtime locks, offline restoration,
  and installed-file verification. A versioned compatibility catalog records
  ten distribution versions on two macOS ARM64 runtime builds; broader target
  coverage and requirement extras/markers remain future work.
- Four documented recipes verified against local fixtures and standalone apps
  with their build sources removed.
- Mise pins the toolchain and developer tools. Bacon, nextest, watchexec, and
  optional cargo-seek provide feedback; Criterion measures the loader/cache.
- Every requested Rust restriction plus pedantic/nursery is enforced across the
  workspace. The audit inventories every explicit exception and requires zero findings.
- Runtime regression fixes cover Unicode escapes, oversized allocations,
  numeric boundaries, and callbacks that mutate collections. See
  [the review record](docs/rust-review.md) for the exact findings and validation.

These changes are implemented on main; existing published v0.6.0 downloads and
checked-in runtime assets are separate from the current runtime sources.

## What to Focus on Next

1. Pass native four-target and sanitizer CI for the updated source.
2. Rebuild and verify full/core embedded components through the release workflow
   before publishing these features and safety fixes in a new version.
3. Verify the Homebrew token's repository write permission before that stable
   release; do not infer that the earlier permission problem has been resolved.
4. Expand compatibility from actual application requirements. Keep the
   [development workflow](docs/development.md), recipes, and editor stubs aligned.
5. Measure future optimizations against dated baselines; do not advertise old
   artifact sizes or timings as measurements of the latest source.

## Product Roadmap After the Rust Cutover

### Phase A: Rust-native optimization and dependency review
- The completed profile review accepts `opt-level = 2`, fat LTO, one codegen
  unit, checked integer overflow, aborting panics, and symbol stripping. On the
  pre-HTTPS ARM64 runtime, `-O2` improved representative interpreter workloads
  by 7-14% over `s` for about 510 KiB. `-O3` added another 231 KiB without a
  measurable win, thin LTO regressed size and speed, and PGO's small,
  corpus-specific gains did not justify its training/toolchain burden.
- Treat 5.75 MB as the cross-target runtime regression ceiling, not as a goal
  to fill at the expense of developer experience, correctness, or
  maintainability. The configuration-format increase qualifies because
  maintained parsers replace incomplete handwritten behavior for three
  user-facing formats. The refreshed runtimes range from 4,497,440 bytes on
  ARM64 macOS to 5,451,504 bytes on x86_64 Linux; the corresponding CLIs range
  from 5,394,608 to 6,656,720 bytes.
- Ratatui 0.30.2 with its Crossterm backend is accepted for real interactive
  `input.select` and `input.multiselect` sessions. It uses an inline viewport to
  preserve scrollback, bounded list scrolling, visible keyboard help, semantic
  focus styling, `NO_COLOR`, responsive compact/minimum-size modes, and
  TestBackend plus PTY coverage. Crossterm owns interactive key and resize event
  parsing; the legacy renderer remains for non-interactive sessions and the
  deterministic v0.5.0-compatibility harness. Build future stateful screen APIs
  on the same renderer rather than introducing another terminal abstraction.
- The public presentation module is `tui`, with no legacy module alias. Runtime
  registration, generated-project stubs, CLI transforms, tests, examples, and
  documentation use the same name. The frozen `MCHARM01` binary format remains
  unchanged until a separately versioned format migration is justified.
- The first allocation cleanup replaces `tui.style`'s temporary vector,
  per-code strings, join, and final format with one lazy output buffer. It keeps
  the ARM64 runtime byte size unchanged and improves a 20,000-call style
  workload by 26.9% at the median with exact golden-output parity.
- Treat borrowed callback values, VM globals, and rooted values as distinct
  states in any future FFI type redesign. A single lifetime on the current
  `Value` wrapper is insufficient because PocketPy allocations can invalidate
  unrooted slots; implement the split only as a dedicated compatibility-gated
  refactor.
- Freeze a reproducible post-cutover baseline before optimizing: compatibility,
  startup distributions, peak memory and allocations, interactive latency,
  representative module throughput, binary sections, dependency contribution,
  and all four release artifacts. Use profiles and measurements rather than
  line count or assumptions about whether custom code or a crate is faster.
- Audit the Rust architecture for places where the type system can remove
  runtime failure modes: narrower FFI lifetimes and ownership states, RAII
  guards for VM/terminal/file cleanup, newtypes for validated handles and
  offsets, exhaustive error enums, checked conversions, and APIs that make
  unrooted PocketPy values or invalid state transitions hard to express.
- Profile allocation, copying, module initialization, and embedded Python
  execution. Evaluate borrowing, reusable buffers, lazy module initialization,
  static data, and more compact representations only where the profiler shows
  a meaningful gain and the PocketPy ownership rules remain explicit.
- Audit Cargo features and duplicate dependencies with `cargo tree` and analyze
  release contribution with
  [`cargo-bloat`](https://github.com/RazrFalcon/cargo-bloat). The completed
  profile comparison covers `s`, `O2`, `O3`, fat/thin LTO, overflow checks, and
  PGO against the same startup and throughput corpus.
- Run isolated library spikes behind the existing Python API and golden tests:
  - accept [`Ratatui`](https://ratatui.rs/) with its Crossterm backend for
    interactive selection. The first production slice keeps the PocketPy API
    stable while replacing rendering/layout and retaining Kipferl's tested raw
    input and cleanup guards;
  - retain feature-minimal `rusqlite` plus statically bundled SQLite. A current
    Turso 0.8.0-pre.2 spike with public defaults disabled produced a 9,576,960
    byte binary and 257 dependency-tree entries, so it increases size and
    maintenance/supply-chain surface while its database engine remains beta;
  - accept feature-minimal Ureq/Rustls for `http.client`. It removes the local
    socket, framing, response-parser, and chunk-decoder implementation, retains
    the 8 MB body cap, and adds HTTPS. With `-O2` the complete ARM64 runtime is
    3,735,360 bytes before the archive-library adoption and a real TLS request
    succeeds;
  - accept feature-minimal `zip` and `tar` readers. They replace handwritten
    central-directory, chunk, header, and member-boundary parsing for about
    66 KiB in the pre-Ratatui runtime;
  - inventory focused crates for remaining process, signal, regex, and format
    modules, disabling default features and rejecting a dependency when
    the standard library or current implementation is clearer.
- Record each spike as an accept/reject decision with compatibility, safety,
  maintenance, license, dependency, size, startup, memory, and throughput data.
  A library is adopted only when it improves the overall engineering result;
  “more Rust” or fewer local lines is not sufficient.
- Exit gate: no Python API, byte-output, error, compatibility, or release-target
  regression; every accepted change has a measured benefit and the final
  baseline is committed for the public retrospective. **Complete:** the final
  host baseline, four release sizes, dependency attribution, memory, PTY
  latency, workload corpus, safety audit, and accept/reject decisions are
  recorded in `benchmarks/rust_optimization_baseline.md`.

### Phase B: Migration documentation and public retrospective
- **Engineering rename complete:** the project is now Kipferl, the repository
  lives at `niklas-heer/kipferl`, and the Rust packages, binaries, release
  assets, site, docs, templates, and visual identity use the new name. Package
  namespaces were available when checked, and `kipferl.dev` is now registered
  as the public home. Complete formal legal clearance before the public stable
  launch. Keep the compatibility aliases for the documented 0.6 window; the
  frozen `MCHARM01` format remains unchanged.
- **Complete:** the README, website, user docs, generated-project templates,
  and examples now use the Rust architecture, direct `tui`/`input` imports,
  measured performance/size claims, and real architecture-specific RC assets.
- **Complete:** the website has a migration section and a long-form article
  with clear sections:
  1. **Why** — repository-specific maintenance, ownership, ecosystem, and
     governance reasons, while acknowledging Zig's strengths.
  2. **How** — the incremental architecture, compatibility gates, FFI safety,
     differential tests, four-target CI, and release cutover.
  3. **Outcome** — what improved, what regressed, what remains, and what we
     would do differently.
- **Complete for the RC:** the article presents reproducible statistics as
  responsive cards and an explicit outcome table: the final compatibility
  result, removed surface area, runtime sizes, startup distribution, memory,
  interactive latency, and four-target coverage. Headline numbers link to
  committed benchmark, compatibility, issue, and release sources. Add
  longitudinal charts later only when more than the migration endpoints exist;
  do not manufacture a trend from two samples.
- **Complete:** preserve the migration issue, plan, final Zig tag, and major
  PRs as an engineering case study rather than erasing the project history.

### Phase C: Close feature gap (Vision)
- Maintain the Vision “nice-to-have” surface. `tomllib`, `http.client`,
  `xml.etree`, HTTPS/TLS through `http.client`, and the basic `sqlite3` DB-API
  subset are now present. The writable `toml` compatibility module, YAML 1.2,
  and KDL 2.0 are also complete with native Rust parsers, file/string APIs,
  stubs, docs, and round-trip tests. Deeper XML/CSV/INI API coverage remains
  demand-driven and its current boundaries are documented.
- Keep the suite honest by expanding tests when behavior changes.

### Phase D: Developer experience
- **Implemented on main (unreleased):** the project/template/configuration/test/
  completion workflow and pinned mise diagnostic tooling described above.
- **Complete:** root `stubs/*.pyi` files are the single source of truth.
  `scripts/generate_stubs.py` validates their syntax and runtime registration,
  deterministically generates the Rust include manifest, supports exact
  directory exports, and fails CI on drift. The CLI now installs the entire
  canonical set instead of maintaining a partial handwritten list. Stale
  `fetch` and `template` editor/documentation APIs were removed because the
  Rust runtime does not register those modules; supported networking examples
  now use `http.client`.
- **Complete:** templates and docs reference the canonical stubs and direct
  `tui`/`input` import paths.
- **Complete:** `kipferl dev` runs a script immediately, watches its project
  directory through `notify`'s native macOS/Linux backends, debounces editor
  event bursts, supports repeatable `--watch` paths and `--clear`, remains
  alive after script exit, and restores terminal settings between interactive
  restarts. Parser, filtering, help, and real edit/restart behavior are tested.

### Phase E: Packaging + release hygiene
- **Implemented on main (unreleased):** local module/package and resource
  bundling, static-import diagnostics, atomic writes, and complete cache checks.
- **Complete:** CI runs the full compatibility report as a regression gate and
  uploads it as an artifact. Tagged releases regenerate and publish that report.
- **Complete:** CI checks the PocketPy anchors against pristine upstream,
  replays every declared patch, requires the result to match the vendored
  source byte for byte, and publishes the JSON verification record in tagged
  releases.
- **Complete:** `kipferl build --targets` is integration-tested, every embedded
  target is packaged and trailer-verified, and native-architecture CI builds
  and executes a sample `--target` universal application on all four targets.
- Prefer native-architecture CI runners for C/Rust release builds; do not reintroduce Zig for cross-compilation.

## Backlog (ordered)

- Restore automatic Homebrew promotion before the next stable release by giving
  `HOMEBREW_TAP_TOKEN` Contents write access to `niklas-heer/homebrew-tap`.
- Concurrency: `threading`, `queue` (PocketPy threading support TBD).
- Database: expand the current bounded `sqlite3` subset only behind compatibility and artifact-size gates.
