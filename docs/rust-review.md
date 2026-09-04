# Rust review: restrictions, diagnostics, and development workflow

Reviewed 2026-09-05 against the existing architecture, all workspace manifests,
Cargo.lock, stable Rust 1.97.1 / MSRV 1.97, CI, tests, and the user's complete Rust
guidance. Mise remains authoritative. Nix/devenv is excluded by request.

## Confirmed findings and changes

| Finding | Change and evidence |
| --- | --- |
| A 64-byte, non-ASCII downloaded SHA256 token could index a UTF-8 string at an invalid boundary and panic. | Decode validated ASCII hexadecimal bytes. Regression includes a Unicode token with the formerly accepted byte length. |
| Corrupt embedded gzip and missing build assets used panic paths. | Propagate contextual I/O errors. CLI production now rejects expects and explicit panic paths. |
| String command dispatch had an impossible `unreachable!` branch. | Use an exhaustive Run/Dev/Build enum; invalid command states cannot enter that function. |
| `HTTPConnection(..., timeout=1e300)` caused SIGABRT when Rust's duration conversion panicked inside an extern-C callback. | Use fallible duration conversion and checked platform deadlines. Subprocess tests verify catchable Python errors and no process abort. |
| Logarithm/power-based frexp/ldexp produced zero or infinity for representable extreme/subnormal values. | Use existing platform libm, check exponent conversions and range errors, and test bit-preserving round trips including signed zero. No new runtime dependency. |
| atanh accepted -1 and rejected NaN; negative fractional timestamps truncated toward zero. | Correct endpoint/NaN behavior and floor timestamps; test gmtime(-0.25) before the epoch. |
| VM metadata setters could panic or truncate lengths at the C ABI boundary. | Introduce structured ContextError and fallible set_argv/set_file. Update callers and boundary tests. |
| Loader cache comparison used 128 KiB of local stack arrays. | Allocate bounded buffers on the heap; a regression validates a populated cache on a 64 KiB worker stack. |
| Format/loader byte and size operations relied on manually proven indexing, casts, and arithmetic. | Use fixed-width chunks, checked ranges/conversions, and checked subtraction. Preserve golden MCHARM01 bytes/cache keys and all format error behavior. |

Additional findings from clearing the full audit:

| Finding | Change and evidence |
| --- | --- |
| Logo padding underflowed for a long prerelease version. | Compute the box width from both title and tagline; regression exercises a long title. |
| UI progress multiplication overflowed at large u32 values; median addition overflowed for two finite f64::MAX values. | Widen progress arithmetic and use f64::midpoint; boundary regressions cover both. |
| Regex replacement scanning could step into a Unicode continuation byte after a backslash. | Advance by complete characters and test Unicode replacement input. |
| Huge bytearray lengths could panic during allocation; negative islice starts could overflow iteration arithmetic. | Validate lengths and reserve fallibly; reject negative slice parameters with Python errors. |
| Finite values outside the f32 range were silently packed as infinity. | Reject overflow before struct packing while preserving representable IEEE rounding; boundary regression covers the Python error. |
| User-defined comparisons and predicates can mutate lists during heap/iterator operations; deepcopy hooks can mutate source containers. | Keep callback operands rooted, snapshot source values where required, and check accesses after callbacks. Regression fixtures exercise reentrant mutation. |

Core text wrapping and filename matching use checked slice/iterator operations.
Differential checks against the previous implementation covered 16,807 input
strings at seven widths for three wrapping operations, and 32,768 patterns against
14 strings. This preserves existing byte-width and Unicode matching behavior.

The reported strftime suspicion was investigated and rejected: the existing
Jiff formatting path is lenient. It was not rewritten merely to satisfy a theory.

## Every proposed lint

`Cargo.toml` now denies both lint groups and every restriction below across all
workspace crates. `mise run lint-audit` checks full/core profiles and all targets,
deduplicates compiler diagnostics, and fails on outstanding findings. CI uploads
Markdown, JSON, and original compiler output under `target/lint-audit/`.

| Lints | Enforcement |
| --- | --- |
| pedantic, nursery | Workspace deny, priority -1 so individual reviewed exceptions remain possible. |
| unwrap_used, expect_used, indexing_slicing, arithmetic_side_effects | Workspace deny. Input-derived operations use checked conversions, slices, iterators, or structured errors. |
| unreachable, unimplemented, todo, panic, panic_in_result_fn, exit | Workspace deny. Binaries return ExitCode and callbacks use Python exceptions. |
| unchecked_time_subtraction, string_slice, as_conversions | Workspace deny. Time, byte boundaries, and ABI conversions are checked or have a specific documented mathematical invariant. |

Existing undocumented-unsafe-block and unsafe-operation checks remain. The
format crate forbids unsafe code entirely. `clippy.toml` permits unwrap, expect,
panic, and indexing in tests without relaxing production scopes.

The audit honors narrowly scoped `#[expect(..., reason = "...")]` declarations
and inventories **every explicit allow/expect attribute**, including test helpers
and generated bindings, in `exceptions.json` and the Markdown report. Missing
Clippy reasons or blanket group exceptions fail. Typical reviewed exceptions are
fixed native slot layouts, bounded algorithm indices, IEEE-754 rounding required
by Python semantics, and failing test/benchmark setup on infrastructure errors.
Generated bindings have two individual lint exceptions preserving upstream C
field names and documentation. These declarations are deliberate exceptions,
not claims that the flagged operations were removed. Rust reports stale unused
expectations, so the normal warning-free Clippy gate catches obsolete annotations.

## Every developer-tool recommendation

| Tool | Decision and concrete use |
| --- | --- |
| rustfmt / Clippy / Rust Analyzer | Pinned through the existing Rust toolchain; rust-src supports editor navigation. Full/core CI gates remain. |
| Bacon 3.25.0 | Added compiler, Clippy, nextest full/core, and doctest jobs; error-location export and key navigation. Verified a real compiler error followed by edit/recovery in a disposable fixture. |
| cargo-nextest 0.9.143 | Added isolated processes, zero retries, slow-test deadlines, leaked-output-pipe detection, JUnit failure output, and CI artifacts. Deliberate assertion, timeout, and leaked-child fixtures verified diagnostics. Doctests still run through Cargo. |
| watchexec 2.7.1 | Added pinned serial check/test/build loop covering crates, PocketPy, embedded assets, and configuration. Verified repeated edits trigger serial runs. Never executes an application. |
| cargo-generate | Evaluated, not installed: no established Rust scaffold template exists here. Kipferl already generates its Python application templates; cargo-generate targets a different workflow. Revisit if a native module/plugin template is introduced. |
| cargo-seek 0.2.0 | Added as an optional mise task for interactive crate, feature, and MSRV inspection; installed/version-checked without adding application dependencies. |
| Criterion 0.8.2 | Added only under loader dev-dependencies, with defaults disabled and cargo_bench_support, plotters, html_reports enabled. Benchmarks inspect bounded samples and validate full warm-cache payloads at 1 KiB and 1 MiB per payload. No Rayon or async benchmark feature. |
| Local Git hooks | Not installed: CI and shared mise tasks enforce checks without mutating contributors' Git configuration. |
| Nightly | Not introduced: all adopted tools, benchmarks, and checks work on pinned stable Rust. |
| Nix/devenv/rust-overlay | Excluded as requested. |

Bacon publishes no official binary archives for this version, so mise builds the
exact release using its upstream Cargo.lock. The release lock references yanked
bisync 0.3.0; it was not silently replaced with an unreviewed tool dependency
update. Nextest, watchexec, and cargo-seek have checksum-locked downloads for all
four macOS/Linux ARM64/x86_64 host combinations. No dependencies mutate on shell
entry. Optional VHS/libclang tools remain specific to recording/regeneration.

## Every crate recommendation

| Candidate | Assessment against this project |
| --- | --- |
| color-eyre | Evaluated for CLI diagnostics. The CLI deliberately returns/prints I/O errors and then execs a separate interpreter; installing a hook alone would not improve Python tracebacks or branches already converted to exit statuses. Fixed actual context-loss/panic paths directly. A future unified Rust error-reporting redesign can reconsider it. |
| itertools | Existing standard iterators express the reviewed transformations. No specific awkward operation justified another dependency. |
| rayon | Runtime VM ownership is single-threaded and FFI values cannot be shared freely. No measured CPU-bound Rust workload justified parallel scheduling; Criterion also excludes Rayon. |
| serde + derive | serde_json already handles project config and JSON; manual bounded config validation provides current exact path/unknown-field diagnostics. No new serializable data model requires derive. |
| clap + derive | A serious future candidate: eight commands, help text, and three completion definitions have duplication. However, switching parsers would require a deliberate exact-help/error compatibility migration. This pass fixes input validation and invalid dispatch states without simultaneously replacing the public CLI contract. |
| Chrono / Jiff | Keep existing Jiff with minimal std/system-timezone features. Time fixes reuse it; adding Chrono would duplicate the capability. |
| cmd_lib | Existing std::process::Command provides exact argument boundaries, exec, capture, status, and checked errors. A shell-like wrapper offers no identified advantage at these boundaries. |
| utoipa | No Rust HTTP server/OpenAPI endpoint surface exists to describe. |
| reqwest + rustls | Existing ureq + rustls supplies the synchronous Python HTTP API. Adding another HTTP/TLS stack would duplicate it; the concrete timeout bug was fixed at the current boundary. |
| sqlx | Existing bundled rusqlite implements Python's synchronous sqlite3 API with runtime SQL. There is no fixed application schema or compile-time query set for sqlx to validate. |
| Leptos + Trunk | The product is a Python CLI runtime; documentation already uses Next.js. No requested Rust web frontend. |
| Dioxus | No cross-platform graphical application surface to implement. |
| Tauri | No desktop webview application requirement; the runtime already uses terminal UI components. |

Cargo's duplicate-dependency inspection found transitive version splits under
existing parsers/TUI crates (including hashbrown, syn, unicode-width, winnow),
not redundant direct application libraries that could safely be removed in this
pass. Lockfile changes add Criterion's development dependency graph; there is no
unrelated dependency upgrade. Criterion's declared MSRV (1.86) fits this workspace.

## API states and ownership

A concrete command enum replaces invalid string dispatch in the CLI. VM lifecycle
already uses an owned active capability, atomic process lifecycle, !Send/!Sync,
and irreversible finalization through Drop. Generic Initialized/Finalized types
would not make the current operations safer. Raw Value/RootFrame lifetime and
rooting design deserves a dedicated FFI design review, not superficial typestate.
The loader validates layout before execution paths; hash sampling remains clearly
documented as a legacy identifier rather than an integrity guarantee.

## Verification and boundaries

Run `mise run check`, `mise run lint-audit`, `mise run bench`, and `mise run test`.
The first uses nextest plus explicit Cargo doctests; the last additionally verifies
release runtime compatibility, vision, portable recipes, and website builds.
Benchmarks produce machine-local statistical evidence, not universal timing claims.
Linux sanitizer and other-platform link/build validation require the existing CI
matrix, especially for the libm boundary. Checked-in embedded release runtime
assets remain separate from runtime source changes; release CI rebuilds them.

Final local verification: `mise run check` passed with 244 full-profile nextest
tests, 96 core tests, 2 doctests, and 51 Python tooling tests. Formatting, generator
drift, compilation, and strict full/core Clippy passed. `mise run lint-audit`
reports **zero outstanding diagnostics**, **135 explicit exception declarations**,
and **zero exception-policy violations**. Its report distinguishes allow from
expect and records each declaration's location, individual lints, and reason.

The original 1,394 diagnostics have been fixed or individually reviewed and
justified; they were not 1,394 confirmed bugs. No blanket Clippy group suppression
is permitted. The audit parser has regression coverage for hidden attributes in
strings/comments, inline attributes, nested cfg_attr reasons, Rust string escapes,
and broad warning-group suppression.

A fresh release workspace build passed, followed by 1,725 available compatibility
checks against mise-pinned CPython 3.12.14, 20 vision tests, and all four recipes (runtime/CLI/source-deleted binary).
The compatibility runner explicitly reports 22 dependency-related skips. Its 52
test groups include the external `toml` package: 51 groups intersect the
160-module standard-library inventory, leaving 109 inventory modules untargeted.
Passing all available checks measures the tested subset, not complete CPython
coverage. All four
Criterion cases completed. Website type/build checks passed earlier in this work.
No preexisting Cargo.lock package version was removed or replaced by this pass.

Maintainer references: [Clippy lint catalog](https://rust-lang.github.io/rust-clippy/master/),
[Bacon configuration](https://dystroy.org/bacon/config/),
[nextest configuration](https://nexte.st/docs/configuration/reference/),
[nextest JUnit](https://nexte.st/docs/machine-readable/junit/),
[Criterion](https://github.com/criterion-rs/criterion.rs),
[mise Cargo tools](https://mise.jdx.dev/dev-tools/backends/cargo.html), and
[cargo-seek](https://github.com/tareqimbasher/cargo-seek).

For setup, daily feedback loops, report locations, and the distinction between
fresh source runtimes and embedded release assets, see the
[development guide](development.md). Historical measurements and current
performance procedures live in the [benchmarking guide](../benchmarks/README.md).
