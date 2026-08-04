# μcharm Plan and Roadmap

This is the single source of truth for priorities and next steps.

## Snapshot

- Goal: build beautiful CLI apps with Python syntax, shipped as tiny, fast binaries.
- Runtime: PocketPy with a Zig host today; the native host, modules, CLI, and loader are migrating to stable Rust.
- Language decision: Rust is the target implementation language. See `RUST_MIGRATION.md` for gates and sequencing.
- Compatibility status: `tests/compat_report_pocketpy.md` shows 51/52 targeted modules at 100% parity (1 has no baseline on the host CPython version). Refresh with `python3 tests/compat_runner.py --report`.
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

1. Record the reproducible Zig baseline and add golden/differential fixtures.
2. Prove Rust-hosted PocketPy and C dependency builds on all four release targets.
3. Port the universal format/loader, then the CLI around existing runtime assets.
4. Port the runtime and native modules without allowing compatibility to fall below the current baseline.
5. Cut CI and releases to Rust, remove Zig, then continue the product roadmap below.

## Product Roadmap After the Rust Cutover

### Phase A: Close feature gap (Vision)
- Maintain the Vision “nice-to-have” surface (`toml`/`tomllib`, `http.client`, `secrets`, `hmac`, `dataclasses`, `xml.etree`, `sqlite3`, and archive helpers).
- Keep the suite honest by expanding tests when behavior changes.

### Phase B: Developer experience
- Decide the canonical stub source (Rust registration metadata or `stubs/`) and wire `scripts/generate_stubs.py` or a CLI command to regenerate them.
- Update templates and docs to reference the canonical stubs and correct import paths.

### Phase C: Packaging + release hygiene
- Add a CI step that runs `python3 tests/compat_runner.py --report` and uploads the report as an artifact.
- Add a CI step that runs `python3 scripts/verify-pocketpy-patches.py --check-upstream`.
- Validate cross-target build support (`ucharm build --targets` and a sample `--target` build).
- Prefer native-architecture CI runners for C/Rust release builds; do not reintroduce Zig for cross-compilation.

## Backlog (ordered)

- Tree-shaking or module selection for smaller binaries.
- `ucharm dev` (watch mode / hot reload).
- Networking and formats: `http.client`, `toml`, `yaml`, `gzip`, `zipfile`, `tarfile`.
- Security: `secrets`, `hmac`.
- Concurrency: `threading`, `queue` (PocketPy threading support TBD).
- Database: `sqlite3` (likely large; consider an optional build flag / separate release flavor).
