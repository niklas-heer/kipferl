# Rust Migration Plan

## Decision

μcharm will migrate its native implementation from Zig to stable Rust while
keeping PocketPy as the embedded Python runtime.

This is a host-language and toolchain migration, not a product rewrite. The
Python-facing API, PocketPy compatibility work, universal-binary behavior, and
product goals remain in place.

The migration should be incremental. Zig feature work is frozen except for
small correctness, security, and release-blocking fixes. New product features
resume on the Rust implementation after the relevant component has crossed its
parity gate.

## Why Rust Fits This Repository Better

- Rust has a stable language and standard-library contract. Edition changes
  are opt-in, so routine compiler updates do not require source migrations.
- PocketPy exposes a C API, which Rust can call through a small, auditable FFI
  layer without changing the interpreter.
- Cargo provides a mature dependency, workspace, test, and release model.
- The supported μcharm targets all have Rust target support. Intel macOS needs
  explicit CI attention because it is no longer a Rust Tier 1 host target.
- Rust can still produce stripped, statically linked Linux binaries. Size and
  startup must be measured rather than assumed, and are hard migration gates.
- Rust's ownership model is a good fit for the resource-heavy parts of this
  codebase: VM values, allocators, subprocess pipes, terminal state, temporary
  files, and embedded runtime extraction.

The decision does not depend on calling Zig a toy. Zig has real strengths,
especially its cross-compilation and C integration. The repository-specific
problem is that μcharm has more than 28,000 lines coupled to a pre-1.0 language
and standard library. For example, the codebase pinned to Zig 0.15.2 does not
build on Zig 0.16.0 without build API changes. That maintenance cost is now
larger than the benefit of staying.

## Current Inventory

Tracked first-party Zig code at the start of the migration:

| Area | Files | Lines | Responsibility |
| --- | ---: | ---: | --- |
| `runtime/` | 63 | 23,074 | PocketPy modules, TUI, stdlib compatibility |
| `cli/` | 11 | 2,505 | `ucharm` commands, packaging, target downloads |
| `pocketpy/` | 4 | 1,533 | VM wrapper, module registration, native build |
| `loader/` | 4 | 623 | Universal-binary extraction and execution |
| `shared/` | 1 | 454 | Shared terminal presentation helpers |
| **Total** | **83** | **28,189** | |

The runtime module layer is the dominant cost and risk. The loader is the
smallest isolated production component, while the PocketPy FFI is the critical
technical proof.

Baseline artifacts and performance must be recorded per target in Phase 0.
One local macOS `ReleaseSmall` build at the decision point produced a 2,313,264
byte PocketPy runtime and a 98,200 byte loader; these are observations, not the
cross-platform acceptance baseline.

## Target Architecture

Use one Cargo workspace with a deliberately small crate graph:

```text
Cargo.toml
rust-toolchain.toml
crates/
├── pocketpy-sys/       # C compilation and raw PocketPy declarations
├── ucharm-format/      # MCHARM01 trailer, hashes, target names
├── ucharm-runtime/     # VM ownership and native module registration
├── ucharm-loader/      # Standalone universal-binary loader
└── ucharm-cli/         # new/init/run/build/test commands
```

Runtime modules live under `ucharm-runtime/src/modules/` until there is a
measured reason to split them into more crates. Avoid recreating the current
build file's one-module-per-build-node complexity.

### Toolchain policy

- Stable Rust only; no nightly features.
- Pin an exact compiler in `rust-toolchain.toml` and update it intentionally.
- Commit `Cargo.lock` and set `rust-version` for every workspace crate.
- Use the 2024 edition unless a spike finds a concrete blocker.
- Build release binaries with LTO, one codegen unit, symbol stripping, and
  `panic = "abort"`; validate crash diagnostics before finalizing that profile.
- Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and
  `cargo test --workspace` in CI.
- Keep dependencies sparse. Prefer the standard library for the existing
  hand-written CLI parser and binary format.

### C and FFI policy

- Compile PocketPy and the existing C dependencies with the `cc` crate.
- Generate raw bindings with a pinned development tool, review them, and check
  them into `pocketpy-sys`; release builds must not require libclang.
- Keep all raw pointers and C calls inside `pocketpy-sys` and a narrow VM/value
  wrapper in `ucharm-runtime`.
- Document ownership for every PocketPy value and callback. No Rust reference
  may outlive the VM or cross a VM call that can invalidate it.
- Deny `unsafe_op_in_unsafe_fn` and explain every `unsafe` block with its
  invariant.
- Continue vendoring and verifying the PocketPy patch set before compilation.

### Cross-compilation policy

Do not adopt `cargo-zigbuild`; the completed release pipeline must not retain a
Zig dependency.

- Build macOS ARM64 natively.
- Build macOS x86_64 on macOS with an explicit x86_64 target and matching C
  compiler flags; keep it as a tested compatibility target.
- Build Linux x86_64 with musl on x86_64 Linux.
- Build Linux ARM64 with musl on an ARM64 runner. Use a conventional musl cross
  compiler only if native CI is unavailable.
- Produce hashes and artifact names identical to current releases until a
  versioned format change is intentionally introduced.

## Non-Negotiable Compatibility Gates

Each cutover PR must preserve these contracts:

1. `tests/compat_runner.py --report` stays at 100% for the currently targeted
   module set.
2. The Python imports, function signatures, return shapes, error behavior, and
   TUI output remain compatible unless a separate product change is approved.
3. A Rust CLI can consume the existing embedded loader/runtime assets during
   the transition.
4. A Rust loader can execute an existing `MCHARM01` universal binary, and a
   Zig loader can execute one produced by the Rust packager.
5. All four release targets build and pass smoke tests.
6. Median warm startup remains at or below 10 ms on the benchmark hosts.
7. A typical standalone app remains below 2 MB. A temporary exception requires
   an explicit recorded decision and a size-recovery issue before cutover.
8. No Zig-built object, compiler, or `cargo-zigbuild` step remains in the final
   release path.

Keep Zig and Rust jobs side by side until the relevant gate passes. Differential
tests should run the same Python file through both runtimes and compare exit
status, stdout, and stderr.

## Execution Plan

### Implementation status

- Phase 0 is in progress: the host baseline harness is available through
  `just rust-baseline` and the legacy CI runner is pinned for reproducibility.
- Phase 1 is in progress: the Cargo workspace builds vendored PocketPy through
  `pocketpy-sys`, a Rust-owned VM executes Python, and a probe native module
  crosses the C callback boundary.
- Phase 2 is functionally complete: `ucharm-format` encodes and decodes the
  exact `MCHARM01` wire format, and the Rust loader validates, atomically
  extracts, caches, and executes Zig-packaged payloads. Zig and Rust share
  byte-level trailer and cache-hash vectors. The measured Rust loader adds
  9.2% to a universal application while slightly improving warm startup; the
  accepted size variance is recorded in `benchmarks/loader_migration_baseline.md`.
- Phase 3 is functionally complete: `ucharm-cli` has the production command
  dispatcher plus `new`, `init`, `run`, `build`, and `test`. Valid command
  output, generated files, file modes, embedded stubs, assistant instructions,
  script transformation, argument forwarding, and runtime exit codes have
  parity with the Zig CLI.
  Unlike the legacy `run`, the Rust command embeds the matching released
  runtime on all four targets and uses a private, atomic, content-addressed
  cache. Its size and warm execution baseline is recorded in
  `benchmarks/run_migration_baseline.md`. The three Rust build modes produce
  byte-identical host artifacts, and universal cross-builds use the released
  loader/runtime pair with the shared trailer encoder. Missing cross-target
  runtimes are resolved from the versioned cache or downloaded with SHA-256
  verification. The Rust `test` command runs single files and the CPython
  compatibility runner with the source-built runtime during development and
  the embedded released runtime as a self-contained fallback.
- Phase 4 is functionally complete: the generated PocketPy bindings now expose
  the small container, type-object, and temporary-rooting surface required by
  native callbacks. Module registration is table-driven, including
  signature-based functions with defaults and keyword arguments. The complete
  `ansi`, `args`, `term`, interactive `input`, and `charm` presentation modules
  are ported with Python-level behavior, error, allocation-stress, byte-stream,
  pseudo-terminal, Unicode-width, and golden ANSI-output tests. The rooted Rust
  `args` implementation fixes the legacy alias-key corruption and SIGSEGV
  exposed by combined alias/default parsing. Rust owns raw terminal state and restores it
  during VM teardown and after every interactive input path; exact selection,
  confirmation, editing, cancellation, and password screen bytes match Zig.
  The reusable Rust TUI core preserves the Zig runtime's byte-oriented width,
  color, border, progress, spinner, and table behavior, including its historical
  keyword-binding quirks. Allocation and exception stress crosses every
  production Rust callback module. Explicit tests enforce the one-owner VM
  lifecycle, and Linux CI instruments PocketPy C with AddressSanitizer,
  UndefinedBehaviorSanitizer, and leak detection. The original `_ucharm_rust`
  proof module has been retired.
- Phase 5 is in progress: `fnmatch`, `base64`, `binascii`, `statistics`,
  `textwrap`, `heapq`, `typing`, `itertools`, `errno`, `copy`, `functools`, and
  `operator` are now joined by the first data/model wave: `collections`, `csv`,
  `dataclasses`, `datetime`, `json`, `random`, and `uuid`; and the binary-
  container wave: `array`, `struct`, and `secrets`; plus the crypto,
  compression, and archive wave: `hashlib`, `hmac`, `gzip`, `io`, `zipfile`,
  and `tarfile`; the filesystem wave: `os`, `os.path`, `pathlib`, `glob`,
  `tempfile`, and `shutil`; and the process/regex/observability wave: `re`,
  `logging`, `signal`, and `subprocess`; plus the tooling/format wave:
  `argparse`, `configparser`, `contextlib`, `unittest`, `urllib.parse`,
  `tomllib`, and `xml.etree.ElementTree`; and the core-runtime wave:
  `math`, `time`, `sys`, and the legacy ASCII string classifiers.
  The shared boundary copies PocketPy bytes into owned Rust buffers before
  allocation, roots
  exact-size byte and float results, supports equality and ordered comparison,
  mutates lists through checked slots, and constructs large string lists
  through stable global scratch registers. Removed heap values remain rooted
  across subsequent Python comparisons rather than relying on the legacy
  module's unrooted static return storage. The registrar can extend PocketPy's
  existing `base64` module and declare the `binascii.Error` and `Incomplete`
  exception aliases without custom setup code. Every migrated fixture has
  byte-for-byte Zig/Rust output parity plus native error, bounds, CRC, identity,
  allocation-stress, and cross-target smoke tests. `typing` adds table-driven
  module initialization plus native type, instance, method, and attribute
  construction while preserving its placeholder aliases, sentinel identity,
  `TypeVar` behavior, and identity decorators. `itertools` adds userdata-backed
  native iterators, GC-rooted object slots, instance checks, and a narrow
  one-argument Python callable bridge while preserving its eager legacy
  helpers and restricted iterable inputs. `errno` adds platform-native POSIX
  constants, the reverse `errorcode` mapping, safe multi-value calls into base
  exception initializers, and the ASCII `str.isupper` prerequisite previously
  supplied by the Zig monolith. `copy` adds type-constructor calls, dynamic
  optional-attribute lookup, rooted recursive container construction, and
  identity-preserving memoization. It also ports the fixture's narrow
  `bytearray` prerequisite and repairs the Zig runtime's circular-copy bus error
  and unbalanced tuple-copy stack. `functools` keeps PocketPy's correct `reduce` and
  `partial` core, then adds GC-owned comparison keys, wrapper metadata, keyword
  cache keys, recursive caching, bounded LRU eviction, counters, and clearing
  through a Rust-registered module layer. `operator` completes the first
  risk-ordered pure-module wave with GC-owned getter/caller objects, nested
  attributes, sequence helpers, length hints, and the remaining unary and
  in-place aliases. The first data/model batch completes seven fixtures at
  49/49, 24/24, 8/8, 21/21, 70/70, 46/46, and 18/18. It replaces fixed native
  result arrays with GC-owned model, iterator, parser, and formatting state,
  adds strict JSON errors/options, a minimal `StringIO` prerequisite for CSV,
  OS-backed random helpers, and UUID parsing/generation. The binary-container
  batch adds 69/69 `array`, 68/68 `struct`, and 8/8 `secrets` checks in one
  review cycle. Array values remain VM/GC-owned, while a narrow native format
  core handles checked endian conversion, float packing, mutable `bytearray`
  writes, and stable-register tuple construction. Secure tokens and unbiased
  `randbelow` reuse the OS entropy boundary. The crypto/compression wave adds
  RustCrypto-backed MD5/SHA-1/SHA-2 and HMAC, bounded pure-Rust gzip/deflate,
  full in-memory byte/text buffers, stored/deflated ZIP reads, and USTAR reads.
  It completes 106 additional compatibility checks, with CPython differential,
  allocation-stress, invalid-input, and cross-target smoke coverage. The full
  filesystem wave adds `os`, `os.path`, `pathlib`, `glob`, `tempfile`, and
  `shutil`, including real environment exposure, script `__file__`, recursive
  path lifecycle and error stress, deterministic CPython path differentials,
  and release smoke tests on both macOS architectures. Its five fixtures pass
  109/109 raw checks; the conservative CPython-baselined report gains 100
  checks. The process/regex/observability wave uses the size-oriented
  `regex-lite` engine behind PocketPy-owned `Match` and `Pattern` objects,
  PocketPy-owned logger/handler and signal state, and a Rust process boundary
  with concurrent capped stdout/stderr draining. It passes 152/152 fixture
  checks plus CPython regex/process differentials, logger output assertions,
  repeated capture/state stress, invalid-input coverage, 1 MiB-per-stream cap
  tests, and optimized dual-architecture smoke. The tooling/format wave keeps
  parser, test-runner, context-manager, URL, TOML, and XML object state owned
  by PocketPy, with narrow Rust callbacks only for UTF-8-safe URL and TOML byte
  conversion. Its seven fixtures pass 142/142 raw checks and add 137 checks to
  the conservative report, backed by CPython differential output, 250-round
  nested parser/serializer stress, malformed-input coverage, Unicode URL
  round-trips, and four-target release smoke. The core-runtime wave extends
  PocketPy's existing math, clock, argument, and recursion primitives rather
  than replacing them. Rust adds the missing hyperbolic/frexp/ldexp math
  surface; libc-backed local/UTC calendar conversion and formatting; standard
  sys metadata, module registry, interning, and streams; and the Zig runtime's
  ASCII string classifiers. Its three fixtures pass 182/182 raw checks plus
  1,000-round math/interning stress, CPython differential output, calendar
  round-trips, domain/type failures, exact stream bytes, and dual-architecture
  release smoke. The full Rust result is now 1,658/1,668 (99.4%), up from the
  original 456/1,668, with 49 of 52 targeted modules at full parity and no
  partial modules.
  Related low-risk modules now move as validated waves; standalone PRs are
  reserved for boundaries whose ownership or binary-format risk warrants them.
- The current stripped macOS runtime is 1,003,984 bytes on ARM64 and 1,081,496
  bytes on x86_64, versus 2,313,264 bytes for the legacy Zig ARM64 runtime.
  On the latest 400-run warm-start sample, native Rust measured 5.359 ms median
  and 5.912 ms p95; x86_64 Rust under Rosetta measured 12.405 ms median and
  13.653 ms p95. The native ARM64 host remains below the 10 ms gate; native
  Intel CI remains the authoritative x86_64 execution environment.

### Phase 0 — Freeze and baseline

- Tag or record the last Zig baseline commit and pin Zig 0.15.2 for maintenance.
- Record clean release sizes, startup distributions, memory use, compatibility
  results, and universal-build smoke tests on all targets.
- Add golden tests for the trailer format, CLI help/errors, and representative
  TUI output before translating those components.
- Classify every native module as pure computation, OS/terminal integration, or
  external-C-backed so the port order is explicit.

Exit gate: the baseline is reproducible in CI and contains enough fixtures to
detect behavioral drift.

### Phase 1 — Prove the Rust spine

- Create the Cargo workspace and pinned toolchain.
- Build the vendored PocketPy source from `pocketpy-sys`.
- Implement VM startup, script execution, error propagation, and one trivial
  native module through the safe wrapper.
- Produce stripped macOS ARM64 and Linux x86_64 binaries and compare startup and
  size with the baseline.
- Prove that the C build can also target macOS x86_64 and Linux ARM64 before
  broad module translation begins.

Exit gate: a Python script runs through Rust-hosted PocketPy on all release
targets, the FFI invariants are reviewed, and size/startup are within the
budget or have a concrete measured recovery plan.

### Phase 2 — Port the universal format and loader

- Implement the 48-byte `MCHARM01` trailer in `ucharm-format` with byte-level
  golden vectors shared with the Zig tests.
- Port loader validation, hashing, cache naming, extraction, permissions,
  cleanup, and `exec` behavior.
- Test corrupted trailers, truncated binaries, hash mismatches, concurrent
  launches, cache reuse, and paths containing spaces.
- Cross-test Rust and Zig packager/loader combinations.

Exit gate: the Rust loader is format-compatible and no slower or materially
larger than the recorded baseline.

### Phase 3 — Port the CLI around existing runtime assets

- Port `new`, `init`, `run`, `build`, and `test` without redesigning their UX.
- Embed the already released runtime and loader stubs first. This decouples the
  CLI migration from native module translation.
- Preserve target names, download URLs, SHA-256 verification, cache layout,
  script transformation, exit codes, and diagnostic wording.
- Add snapshot tests for help and errors plus end-to-end tests for all build
  modes.

Exit gate: the Rust CLI passes the current unit/e2e suite and can build and run
universal apps using the existing runtime assets.

### Phase 4 — Port the runtime foundation

- Port the reusable ANSI, argument, TUI, and input core logic.
- Implement a declarative module registration table so adding a module does not
  require hundreds of build-script declarations.
- Centralize PocketPy conversions for strings, bytes, lists, tuples, dicts,
  callables, exceptions, and userdata.
- Add leak checks, sanitizer runs for C code, and tests for callback/value
  lifetime boundaries.

Exit gate: representative `charm`, `input`, and compatibility modules register
through one reviewed path and survive stress/error tests.

### Phase 5 — Port native modules in risk-ordered waves

Port behavior, not Zig syntax. Every module moves with its existing CPython
fixture and a Zig-versus-Rust differential run.

1. Pure and low-OS modules: `base64`, `binascii`, `copy`, `fnmatch`, `functools`,
   `heapq`, `itertools`, `operator`, `statistics`, `textwrap`, `typing`.
2. Data/model modules: `array`, `collections`, `csv`, `datetime`, `json`, `re`,
   `struct`, `toml`, `uuid`, XML and dataclasses.
3. Filesystem and process modules: `os`, `pathlib`, `glob`, `tempfile`, `shutil`,
   `signal`, `subprocess`, logging, and terminal/input behavior.
4. External-C-backed and network modules: BearSSL/fetch/HTTP, SQLite,
   TinyTemplate, hashing/HMAC/secrets, and archive formats.
5. μcharm presentation modules and interactive golden tests.

Retire each Zig module from the Rust release only when its compatibility group
returns to 100%.

Exit gate: the complete compatibility suite and interactive/e2e suite pass on
the Rust runtime for every release target.

### Phase 6 — Cut over CI, packaging, and releases

- Switch `justfile`, CI, release workflows, updater scripts, Homebrew formula,
  docs, templates, and contributor instructions to Cargo.
- Build the CLI, runtime, and loaders from Rust/C only and refresh embedded
  stubs for all targets.
- Run a release-candidate bake with artifact install, universal build, and
  execution tests on clean machines.
- Publish one prerelease before making the Rust artifacts the default stable
  download.

Exit gate: the normal CI and release workflows contain no Zig setup and all
artifacts meet the compatibility, startup, and size gates.

### Phase 7 — Remove Zig, optimize the Rust implementation, and resume the roadmap

- Delete Zig sources, build files, caches, and stale embedded binaries only
  after the Rust release is proven. Preserve the last Zig tag and migration
  notes for archaeology.
- Update architecture documentation and contributor guidance.
- Before the public migration wrap-up, run a bounded Rust-native optimization
  and dependency review against a frozen post-cutover baseline:
  - profile startup, allocations, peak memory, interactive latency, module
    throughput, binary sections, dependency contribution, and all four release
    artifacts;
  - use Rust's ownership and type system to narrow FFI lifetimes, encode rooted
    versus borrowed PocketPy values, introduce RAII cleanup guards, validate
    handles/offsets with newtypes, and replace implicit state/error conventions
    with checked transitions and exhaustive enums where that removes a real
    failure mode;
  - investigate measured allocation/copy hot spots, module initialization,
    embedded Python execution, lazy loading, reusable buffers, and compact data
    representations without weakening the explicit VM ownership boundary;
  - audit duplicate crates and default features, attribute release size with
    `cargo-bloat`, and compare release-profile/PGO variants using the same
    compatibility, startup, size, memory, and throughput corpus;
  - spike a feature-minimal Crossterm substrate and selected Ratatui primitives
    behind the existing terminal and μcharm APIs; preserve exact golden output
    and adopt neither wholesale merely to reduce local code;
  - for future `sqlite3`, compare `rusqlite` with bundled SQLite against the
    pure-Rust Turso Database. Recheck Turso's production maturity and published
    compatibility gaps at evaluation time, and measure both as optional release
    features because database engines may dominate the tiny runtime artifact;
  - inventory focused crates for process, signals, regex, networking, archives,
    and formats, preferring the standard library or existing code whenever a
    dependency does not improve safety, correctness, maintenance, or measured
    performance enough to justify its binary and supply-chain cost;
  - keep an accept/reject decision record for every spike. Adoption requires no
    Python API, byte-output, error, compatibility, or target regression and a
    demonstrated overall benefit in the recorded matrix.
- After the Rust release is proven, complete a public documentation and
  communication pass before treating the migration as old news:
  - audit the README, website, docs, templates, examples, installation paths,
    architecture diagrams, and benchmark claims for stale Zig-era content;
  - add a website blog/migration section covering **why** μcharm moved, **how**
    the incremental rewrite worked, and **what the outcome was**;
  - publish reproducible final statistics and polished charts for compatibility
    progress, migrated surface area, binary size, startup distributions,
    four-target CI, dependencies, and migration-discovered defects;
  - link the numbers back to committed reports and benchmarks, preserve the
    final Zig tag and migration tracker, and report regressions and tradeoffs as
    explicitly as improvements.
- Resume roadmap work in this order:
  1. Rust-native optimization and dependency review;
  2. README/website/docs refresh and the migration retrospective;
  3. canonical stub generation and CI drift checking;
  4. compatibility report and PocketPy patch verification artifacts;
  5. cross-target build reliability;
  6. tree-shaking/module selection for size;
  7. `ucharm dev` watch mode;
  8. remaining networking, format, concurrency, and database work;
  9. higher-level TUI features from `vision.md`.

## Suggested PR Sequence

Keep changes reviewable and reversible:

1. Decision, baseline scripts, and golden fixtures.
2. Cargo workspace plus `pocketpy-sys` proof.
3. Rust loader and cross-format tests.
4. Rust CLI using existing assets.
5. Runtime wrapper and declarative registrar.
6. One PR per module wave, split further when a wave is too large.
7. Four-target release candidate and workflow cutover.
8. Zig removal and documentation cleanup.

Do not mix unrelated product redesigns into these PRs. A migration regression
should be attributable to one boundary or module group.

## Principal Risks

| Risk | Mitigation |
| --- | --- |
| Rust binaries exceed the size goal | Measure in Phase 1; use LTO, aborting panics, stripping, dependency review, and feature-gated heavy modules. |
| C cross-compilation is harder without Zig | Prove all four targets in Phase 1 and prefer native-architecture CI runners. |
| Unsafe PocketPy bindings introduce lifetime bugs | Confine raw FFI, document invariants, add sanitizer/leak jobs, and review the wrapper before module scaling. |
| A long dual implementation stalls the roadmap | Freeze Zig features, port in vertical slices, and enforce exit gates rather than an open-ended rewrite branch. |
| Behavioral parity drifts silently | Reuse CPython fixtures and add Zig/Rust differential output tests. |
| Intel macOS support degrades | Keep an explicit x86_64 build and smoke-test job; document its support tier honestly. |
| Universal binaries become incompatible | Freeze `MCHARM01` as v1 and test both old/new packager-loader directions. |

## Definition of Done

The migration is complete when stable releases of `ucharm`, the PocketPy
runtime, and all loader stubs are built with stable Rust plus the vendored C
sources; all current tests pass on all release targets; existing universal
binaries remain runnable; startup and size budgets are met; and the CI/release
path contains no Zig dependency.
