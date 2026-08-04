# Rust-Native Optimization Baseline

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon, with Rust 1.97.1.
Each timing is a randomized round-robin comparison with seed 42 so thermal and
run-order drift is shared across candidates. Commands are reproducible with
`benchmarks/migration_baseline.py --candidate LABEL=PATH`.

## Release profile decision

All candidates retain one codegen unit, aborting panics, symbol stripping, and
the existing Cargo dependency/features graph.

| Profile | ARM64 runtime | Startup median | Startup p95 |
| --- | ---: | ---: | ---: |
| `z` + fat LTO (control) | 1,840,336 bytes | 6.060 ms | 6.460 ms |
| `s` + fat LTO | 2,131,168 bytes | 5.396 ms | 5.658 ms |
| `z` + thin LTO | 2,057,200 bytes | 6.090 ms | 6.437 ms |

The startup corpus uses 400 launches per candidate after ten warmups. The
`s`/fat-LTO build adds 290,832 bytes (15.8%) but improves median startup by
11.0%. Thin LTO is both larger and no faster than the control, so it is
rejected.

The same candidates were measured over 60 runs of representative existing
workloads after three warmups:

| Workload | `z` + fat LTO | `s` + fat LTO | Improvement | Thin LTO |
| --- | ---: | ---: | ---: | ---: |
| recursive Fibonacci | 270.122 ms | 136.736 ms | 49.4% | 269.540 ms |
| one-million-iteration loop | 63.627 ms | 32.384 ms | 49.1% | 63.483 ms |
| 10,000 JSON parses | 63.243 ms | 50.225 ms | 20.6% | 62.852 ms |

The speedup is large enough to justify the size tradeoff under the revised
policy. `opt-level = "s"` with fat LTO is accepted. The practical runtime target
is 2.5 MB, with 3 MB retained as the static-Linux regression ceiling. This is a
performance-aware budget, not a target to fill.

Measured `s` artifacts:

| Target | Runtime | Loader | Linkage |
| --- | ---: | ---: | --- |
| macOS ARM64 | 2,131,168 bytes | 320,352 bytes | Mach-O |
| macOS x86_64 | 2,148,764 bytes | 322,160 bytes | Mach-O |
| Linux ARM64 musl | 2,378,840 bytes | 397,264 bytes | static ELF, no `INTERP` |

Linux ARM64 is effectively unchanged from the `z` build while preserving the
runtime speed-oriented profile choice. Native CI remains authoritative for the
Linux x86_64 and final CLI-with-embedded-runtime figures.

The complete host CLI is 2,880,976 bytes with the ARM64 runtime embedded. The
generated macOS x86_64 universal application is 2,471,096 bytes and executes
under Rosetta; the Linux ARM64 universal application is 2,776,276 bytes and
executes in a clean Debian ARM64 container. These application sizes include the
loader, embedded runtime, trailer, and test payload rather than only the runtime
shown above.

## Dependency and section audit

- `Cargo.lock` contains 54 packages and `cargo tree --duplicates` reports no
  duplicate versions.
- `cargo audit` reports no known vulnerabilities.
- Every runtime dependency has default features disabled or an intentionally
  narrow feature set where the crate offers one. The SQLite dependency remains
  bundled and feature-minimal to preserve a standalone executable.
- `cargo bloat --crates` attributes the 1.2 MiB control `.text` section mainly
  to 630.7 KiB of C/unknown symbols (PocketPy and bundled SQLite), 247.0 KiB of
  `std`, 110.8 KiB of μcharm runtime code, 69.8 KiB of Jiff, 26.7 KiB of
  `regex-lite`, and 21.7 KiB attributed directly to `libsqlite3-sys`. These
  estimates are directional because stripped C symbols cannot all be assigned
  precisely.

No dependency replacement is accepted in this pass. The graph is already
small, has no duplicated versions, and the previously measured pure-Rust Turso
spike remains substantially worse for size and dependency count.

## Rust safety audit

The runtime already uses RAII for VM shutdown, PocketPy temporary root frames,
terminal raw mode, cursor restoration, and loader cache cleanup. The remaining
quality gap was enforcement: 77 unsafe FFI blocks did not have a Clippy-visible
safety rationale even where surrounding prose described the invariant.

The workspace now denies `clippy::undocumented_unsafe_blocks`. Every current
PocketPy callback-stack conversion, raw value access, global register use, and
FFI operation has an immediately attached safety statement. Strict workspace
Clippy, tests, sanitizers, and compatibility remain the behavioral guards; the
new lint prevents unexplained unsafe code from entering future migration work.
