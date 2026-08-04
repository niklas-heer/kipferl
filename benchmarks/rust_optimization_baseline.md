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

## Peak memory and interactive latency

Peak resident memory was sampled with macOS `/usr/bin/time -l`. The empty
process used a median 5,685,248 bytes under the `z` control and 5,718,016 bytes
under the accepted `s` profile over 30 runs, a 32 KiB (0.6%) increase. The
10,000-parse JSON workload used median peaks of 12,386,304 and 12,435,456 bytes
over 15 runs, a 48 KiB (0.4%) increase. This is within run-to-run allocator and
loader variance and shows no material memory regression from the profile
change.

The test-key-driven interactive path was also compared over 100 randomized
round-robin runs using `input.select` with two choices. Median wall time improved
from 6.223 ms to 5.500 ms (11.6%); p95 improved from 7.696 ms to 6.861 ms.

## Allocation and copying review

`charm.style` was the clearest avoidable allocation chain in the current TUI
core. It previously allocated a `Vec<String>`, one string per enabled attribute
or color, a joined string, and the final ANSI-prefixed string. The accepted
implementation writes every code into one lazily created `String`.

The existing golden output tests pass unchanged and the stripped ARM64 runtime
remains exactly 2,131,168 bytes. Over 60 randomized round-robin runs, a workload
that constructs 20,000 fully styled strings improves from 18.238 ms median and
19.569 ms p95 to 13.331 ms median and 14.027 ms p95. That is a 26.9% median
throughput improvement with no artifact-size cost.

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

## Terminal and TUI library spike

Crossterm 0.29.0 and Ratatui 0.30.2 were built in isolated macOS programs with
the same `s`, fat-LTO, one-codegen-unit, aborting-panic, and stripping profile as
the runtime. The control wrote representative ANSI bytes directly and was
285,936 bytes.

| Candidate | Binary | Added bytes | Unique tree entries |
| --- | ---: | ---: | ---: |
| direct ANSI control | 285,936 | — | 1 |
| Crossterm output + raw mode | 303,024 | 17,088 (6.0%) | 14 |
| Crossterm with events | 354,992 | 69,056 (24.2%) | 22 |
| Ratatui core layout + widgets, no backend | 368,608 | 82,672 (28.9%) | 53 |

The tree count includes each local spike crate. Crossterm's event path—the
relevant comparison for μcharm input—adds Mio, signal-hook, signal-hook-mio,
Rustix, parking_lot, and supporting crates. The Ratatui measurement does not
include a terminal backend, yet already adds 52 dependency entries beyond the
control.

Neither library is accepted. The current terminal surface is a small set of
exact ANSI sequences plus `/dev/tty`, process-group, timed-read, and restoration
behavior covered by byte-stream and pseudo-terminal tests. The presentation
surface deliberately preserves μcharm-specific width and rendering semantics.
The spikes add graph and artifact cost without removing those compatibility
requirements. Reconsider a library if the product grows into a stateful,
full-screen application framework rather than for the current bounded API.

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

A deeper lifetime audit found that the current `Value` wrapper represents three
different states: a borrowed callback stack slot, a process-global VM value,
and a value rooted in a `RootFrame`. Adding one lifetime parameter would not
model invalidation across allocating PocketPy calls and would provide false
confidence. The sound Rust-native direction is separate borrowed and rooted
types plus a higher-ranked callback wrapper that prevents borrowed values from
escaping. That is deferred as its own compatibility-gated refactor rather than
mixed into profile optimization; the immediate documented-unsafe enforcement
is complete.
