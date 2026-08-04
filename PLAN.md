# μcharm Plan and Roadmap

This is the single source of truth for priorities and next steps.

## Snapshot

- Goal: build beautiful CLI apps with Python syntax, shipped as tiny, fast binaries.
- Runtime: PocketPy; the Rust host, loader, CLI, and 35 fully compatible stdlib targets are implemented while the remaining native modules migrate from Zig.
- Language decision: Rust is the target implementation language. See `RUST_MIGRATION.md` for gates and sequencing.
- Compatibility status: the Rust migration runtime passes 1,285/1,668 checks (77.0%), with 35/52 targeted modules at 100% parity. Refresh with `python3 tests/compat_runner.py --runtime target/debug/pocketpy-ucharm-rs --report`.
- PocketPy vendor patches are tracked under `pocketpy/patches/` and verified via `python3 scripts/verify-pocketpy-patches.py --check-upstream`.

## Current State (from the repo)

- Native modules cover TUI (charm/input/ui), terminal + ANSI, and a growing stdlib set (copy, fnmatch, typing, csv, datetime, json, subprocess, signal, logging, etc.).
- Loader and CLI already build and run universal binaries; `cli/src/test_cmd.zig` and `tests/compat_runner.py` provide compatibility tooling.
- Stubs exist in `stubs/` and `cli/src/stubs/`; there is a generator script in `scripts/generate_stubs.py`.
- CPython tests are vendored under `tests/cpython/` and are used to track parity.

## Active Priority: Rust Migration

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

1. Finish the remaining Rust runtime module waves without regressing the 1,668-check baseline.
2. Run a four-target release candidate and cut CI, packaging, and releases fully to Rust.
3. Remove Zig after the Rust release is proven, then continue the product roadmap below.

## Product Roadmap After the Rust Cutover

### Phase A: Migration documentation and public retrospective
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

### Phase B: Close feature gap (Vision)
- Maintain the Vision “nice-to-have” surface; remaining gaps include `toml`/`tomllib`, `http.client`, `xml.etree`, and `sqlite3`.
- Keep the suite honest by expanding tests when behavior changes.

### Phase C: Developer experience
- Decide the canonical stub source (Rust registration metadata or `stubs/`) and wire `scripts/generate_stubs.py` or a CLI command to regenerate them.
- Update templates and docs to reference the canonical stubs and correct import paths.

### Phase D: Packaging + release hygiene
- Add a CI step that runs `python3 tests/compat_runner.py --report` and uploads the report as an artifact.
- Add a CI step that runs `python3 scripts/verify-pocketpy-patches.py --check-upstream`.
- Validate cross-target build support (`ucharm build --targets` and a sample `--target` build).
- Prefer native-architecture CI runners for C/Rust release builds; do not reintroduce Zig for cross-compilation.

## Backlog (ordered)

- Tree-shaking or module selection for smaller binaries.
- `ucharm dev` (watch mode / hot reload).
- Networking and formats: `http.client`, `toml`, `yaml`.
- Concurrency: `threading`, `queue` (PocketPy threading support TBD).
- Database: `sqlite3` (likely large; consider an optional build flag / separate release flavor).
