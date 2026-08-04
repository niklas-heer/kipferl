# μcharm Plan and Roadmap

This is the single source of truth for priorities and next steps.

## Snapshot

- Goal: build beautiful CLI apps with Python syntax, shipped as tiny, fast binaries.
- Runtime: PocketPy; the Rust host, loader, CLI, and 51 fully compatible stdlib targets are implemented. The release-path cutover is the remaining migration boundary.
- Language decision: Rust is the target implementation language. See `RUST_MIGRATION.md` for gates and sequencing.
- Compatibility status: the Rust runtime passes 1,668/1,668 available checks (100%), with 51/52 targeted modules at 100% parity, no partial modules, and one host-unavailable `toml` baseline. Refresh with `python3 tests/compat_runner.py --runtime target/release/pocketpy-ucharm --report`.
- PocketPy vendor patches are tracked under `pocketpy/patches/` and verified via `python3 scripts/verify-pocketpy-patches.py --check-upstream`.

## Current State (from the repo)

- Native modules cover TUI (charm/input/ui), terminal + ANSI, and a growing stdlib set (copy, fnmatch, typing, csv, datetime, json, subprocess, signal, logging, etc.).
- The Rust loader and CLI build and run universal binaries; the Rust CLI tests and `tests/compat_runner.py` provide compatibility tooling.
- Stubs exist in `stubs/` and `cli/src/stubs/`; there is a generator script in `scripts/generate_stubs.py`.
- CPython tests are vendored under `tests/cpython/` and are used to track parity.

## Active Priority: Rust Migration

- Phase 5 implementation is complete at 1,668/1,668 available checks.
- Phase 6 cuts the canonical CI, release workflow, embedded assets, public
  binary names, local commands, and contributor guidance over to Rust/Cargo.
- Freeze new Zig feature work; accept only small correctness, security, and release-blocking fixes.
- Preserve PocketPy, the Python-facing API, `MCHARM01` universal binaries, and all current compatibility tests.
- Establish the Rust/PocketPy FFI and four-target build proof before translating the large runtime module surface.
- Port in releasable slices: loader, CLI using existing assets, runtime foundation, then module waves.
- Keep Zig and Rust differential tests until each component crosses its parity, size, startup, and target gates.
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
- Align stubs with the PocketPy import surface and regenerate with a single command.
- Update templates/examples to reference the PocketPy runtime and current modules.
- Add CI target for PocketPy compatibility report generation.

## What to Focus on Next

1. Merge the Rust-only CI/release cutover after its four native targets build,
   execute universal applications, and satisfy static-link and size gates.
2. Publish and bake one Rust prerelease using the rebuilt CLI/runtime/loader
   assets and Homebrew path.
3. Remove the archived Zig implementation after that release is proven, then
   continue the product roadmap below.

## Product Roadmap After the Rust Cutover

### Phase A: Rust-native optimization and dependency review
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
  [`cargo-bloat`](https://github.com/RazrFalcon/cargo-bloat). Compare `z` versus
  `s`, LTO choices, and profile-guided optimization using the same startup,
  size, and throughput corpus; keep the existing size-oriented release profile
  as the control.
- Run isolated library spikes behind the existing Python API and golden tests:
  - compare the current terminal/raw-mode/event code with a feature-minimal
    [`Crossterm`](https://github.com/crossterm-rs/crossterm) substrate, and
    compare selected layout/widget primitives with
    [`Ratatui`](https://ratatui.rs/) rather than replacing μcharm's presentation
    model wholesale;
  - retain the accepted feature-minimal `rusqlite` plus statically bundled
    SQLite implementation. The August 2026 Turso spike added 211 lockfile
    packages and produced a 5,420,336-byte runtime, so the pure-Rust engine was
    rejected for this release. Recheck Turso's maturity, compatibility, and
    artifact cost after the cutover; the measurements and decision are in
    `benchmarks/network_database_wave.md`;
  - inventory focused crates for remaining process, signal, regex, networking,
    archive, and format modules, disabling default features and rejecting a
    dependency when the standard library or current implementation is clearer.
- Record each spike as an accept/reject decision with compatibility, safety,
  maintenance, license, dependency, size, startup, memory, and throughput data.
  A library is adopted only when it improves the overall engineering result;
  “more Rust” or fewer local lines is not sufficient.
- Exit gate: no Python API, byte-output, error, compatibility, or release-target
  regression; every accepted change has a measured benefit and the final
  baseline is committed for the public retrospective.

### Phase B: Migration documentation and public retrospective
- Revisit the README, website, and all user/contributor documentation once the
  Rust release is proven. Remove stale Zig architecture, commands, examples,
  screenshots, performance claims, and download instructions.
- Add a migration section to the website and publish a polished retrospective,
  either as a three-part series or one long-form article with clear sections:
  1. **Why** — repository-specific maintenance, ownership, ecosystem, and
     governance reasons, while acknowledging Zig's strengths.
  2. **How** — the incremental architecture, compatibility gates, FFI safety,
     differential tests, four-target CI, and release cutover.
  3. **Outcome** — what improved, what regressed, what remains, and what we
     would do differently.
- Finish with reproducible, well-presented statistics and charts: compatibility
  over time, migrated modules and lines, binary sizes, startup distributions,
  CI/target coverage, defects found during migration, dependencies, and the
  final Zig-versus-Rust artifact comparison. Link every headline number to its
  committed benchmark or compatibility source.
- Preserve the migration issue, plan, final Zig tag, and major PRs as an
  engineering case study rather than erasing the project history.

### Phase C: Close feature gap (Vision)
- Maintain the Vision “nice-to-have” surface. `tomllib`, `http.client`,
  `xml.etree`, and the basic `sqlite3` DB-API subset are now present; remaining
  work includes the separate third-party `toml` package, HTTPS/TLS, YAML, and
  deeper API coverage where product demand justifies it.
- Keep the suite honest by expanding tests when behavior changes.

### Phase D: Developer experience
- Decide the canonical stub source (Rust registration metadata or `stubs/`) and wire `scripts/generate_stubs.py` or a CLI command to regenerate them.
- Update templates and docs to reference the canonical stubs and correct import paths.

### Phase E: Packaging + release hygiene
- Add a CI step that runs `python3 tests/compat_runner.py --report` and uploads the report as an artifact.
- Add a CI step that runs `python3 scripts/verify-pocketpy-patches.py --check-upstream`.
- Validate cross-target build support (`ucharm build --targets` and a sample `--target` build).
- Prefer native-architecture CI runners for C/Rust release builds; do not reintroduce Zig for cross-compilation.

## Backlog (ordered)

- Tree-shaking or module selection for smaller binaries.
- `ucharm dev` (watch mode / hot reload).
- Networking and formats: HTTPS/TLS, third-party `toml`, and YAML.
- Concurrency: `threading`, `queue` (PocketPy threading support TBD).
- Database: expand the current bounded `sqlite3` subset only behind compatibility and artifact-size gates.
