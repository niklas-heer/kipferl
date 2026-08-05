# Rust-Native Optimization Baseline

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon, with Rust 1.97.1.
Each timing is a randomized round-robin comparison with seed 42 so thermal and
run-order drift is shared across candidates. Commands are reproducible with
`benchmarks/migration_baseline.py --candidate LABEL=PATH`.

## Release profile exploration

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

This first pass established `s` with fat LTO as the better control than `z`.
The later developer-experience review broadened the acceptable runtime budget
and superseded it with the final `-O2` decision below.

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

## Final profile and value budget

The second pass compared `s`, `O2`, `O3`, thin LTO, checked overflow, and PGO.
These pre-HTTP-dependency ARM64 results use fat LTO unless stated otherwise:

| Profile | Runtime | Startup median | Fibonacci | Million-iteration loop | JSON | Style |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `s` | 2,131,168 | 5.423 ms | 142.756 ms | 30.492 ms | 32.384 ms | 13.775 ms |
| `O2` | 2,641,600 | 5.344 ms | 129.493 ms | 26.240 ms | 31.050 ms | 12.708 ms |
| `O3` | 2,872,928 | 5.344 ms | 130.268 ms | 26.285 ms | 31.005 ms | 12.836 ms |
| `s` + thin LTO | 2,284,160 | 5.437 ms | 143.397 ms | 30.613 ms | 32.716 ms | 13.646 ms |
| `O2` + overflow checks | 2,641,648 | 5.365 ms | 129.069 ms | 28.148 ms | 31.721 ms | 13.192 ms |

`O2` is accepted: it materially improves representative workloads while the
complete runtime remains within the product's value-based budget. `O3` adds
231,328 bytes over `O2` without a measurable benefit. Thin LTO is larger and
slower than fat LTO. Overflow checks add only 48 bytes to the pre-HTTPS `O2`
runtime and are accepted as a quality safeguard.

An instrumented PGO build was trained on startup, Fibonacci, loop, JSON, style,
and all 1,668 compatibility checks. It reduced the binary by 32,816 bytes but
did not improve startup; the only notable workload gain was about 4.6% for
style, while other changes were neutral or below 1%. LLVM also exhausted static
counters during the callback-heavy training run, PocketPy's C compilation did
not consume the Rust profile, and 39 Rust functions lacked profile data. PGO is
therefore rejected until a stable representative production corpus justifies
the extra release machinery.

After accepting feature-minimal Ureq/Rustls for maintained HTTP and HTTPS, the
final host artifacts are:

| Artifact | ARM64 macOS size |
| --- | ---: |
| Runtime (`pocketpy-kipferl`) | 4,000,864 bytes |
| CLI (`kipferl`, before final asset refresh) | 2,914,016 bytes |

The host runtime is approximately 4.0 MB, with a 5 MB cross-target regression
ceiling. The budget is a guardrail rather than the primary product metric: a
modest size increase is accepted when it materially improves correctness,
maintainability, testability, or user/developer experience.

The embedded runtime assets were refreshed after the `tui` interface rename:

| Release target | Runtime size |
| --- | ---: |
| macOS ARM64 | 4,000,864 bytes |
| macOS x86_64 | 4,432,008 bytes |
| Linux ARM64 (static musl) | 4,356,864 bytes |
| Linux x86_64 (static musl) | 4,831,144 bytes |

## Frozen post-cutover baseline

The final optimization baseline was repeated on the same Apple Silicon host
after the Ratatui adoption and the public `tui` namespace cutover. A 1,200-run
startup sample after 20 warmups measured **7.044 ms median** and **7.980 ms
p95**. The release artifacts on this host are:

| Artifact | Size |
| --- | ---: |
| Runtime (`pocketpy-kipferl`) | 4,000,864 bytes |
| CLI (`kipferl`, with compressed cross-target assets) | 4,796,384 bytes |
| Universal loader | 336,864 bytes |
| Minimal universal application | 4,321,388 bytes |

The representative workload corpus was repeated for 60 runs after three
warmups. Each number includes process startup and script loading:

| Workload | Median | p95 |
| --- | ---: | ---: |
| recursive Fibonacci | 164.234 ms | 182.337 ms |
| one-million-iteration loop | 40.779 ms | 55.533 ms |
| 10,000 JSON parses | 64.476 ms | 70.889 ms |
| 20,000 fully styled strings | 17.126 ms | 18.221 ms |

These are the committed absolute values for the public retrospective, not a
claim that every dependency made each interpreter workload faster. The earlier
controlled profile and allocation comparisons remain the evidence for the
accepted `O2` profile and `tui.style` optimization.

## Peak memory and interactive latency

The final 4,000,864-byte runtime used a median **6,209,536 bytes** RSS for an
empty process over 30 runs (6,291,456-byte p95). The 10,000-parse JSON workload
used a median **15,564,800 bytes** over 15 runs (15,613,952-byte p95). A real
80×24 PTY benchmark using the same cursor handshake, key injection, and cleanup
path as the integration tests completed Ratatui selection in **9.100 ms median**
and **10.186 ms p95** over 100 runs after five warmups.

The following earlier measurements isolate the release-profile change from the
later dependency additions:

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

`tui.style` was the clearest avoidable allocation chain in the current TUI
core. It previously allocated a `Vec<String>`, one string per enabled attribute
or color, a joined string, and the final ANSI-prefixed string. The accepted
implementation writes every code into one lazily created `String`.

The existing golden output tests pass unchanged and the stripped ARM64 runtime
remains exactly 2,131,168 bytes. Over 60 randomized round-robin runs, a workload
that constructs 20,000 fully styled strings improves from 18.238 ms median and
19.569 ms p95 to 13.331 ms median and 14.027 ms p95. That is a 26.9% median
throughput improvement with no artifact-size cost.

## Dependency and section audit

- Before the dependency adoptions, `Cargo.lock` contained 54 packages. The
  accepted Ureq/Rustls, archive, and Ratatui/Crossterm stacks bring the final
  graph to 162 packages. `cargo tree --duplicates` reports two unavoidable
  families in the selected upstream graphs: `hashbrown` 0.16/0.17 and `syn`
  2/3; only the target-relevant normal dependencies ship in the binary.
- `cargo audit` reports no known vulnerabilities.
- Every runtime dependency has default features disabled or an intentionally
  narrow feature set where the crate offers one. The SQLite dependency remains
  bundled and feature-minimal to preserve a standalone executable.
- The final `cargo bloat --crates` pass reports a 2.9 MiB `.text` section. Its
  largest attributions are 1.1 MiB of C/unknown symbols (primarily PocketPy and
  bundled SQLite), 451.6 KiB of `std`, 264.1 KiB of Rustls, 213.3 KiB of
  Kipferl runtime code, 182.3 KiB of the PocketPy system crate, 127.9 KiB of
  Ring, 92.6 KiB of Ureq, and 82.7 KiB of Jiff. Ratatui core contributes
  27.7 KiB and Crossterm 22.7 KiB. These estimates are directional because
  stripped C symbols cannot all be assigned precisely. The normal stripped
  release was rebuilt after instrumentation and remains 4,000,864 bytes.

The database spike retains `rusqlite` with bundled SQLite. With identical
`O2`/fat-LTO/overflow settings, a trivial control was 302,448 bytes, Rusqlite
was 1,317,120 bytes with 12 tree entries, and Turso 0.8.0-pre.2 was 9,576,960
bytes with 257 entries and 27 duplicate-version groups. Turso executed the
query successfully, but its current beta engine, artifact size, async/runtime
graph, bindgen/Clang path, ICU, tracing, crypto, parser, and index dependencies
make it a worse maintenance result for this release.

The networking spike compared feature-minimal synchronous HTTPS clients. A
trivial control was 302,432 bytes, Minreq/Rustls was 701,296 bytes with 26 tree
entries, and Ureq/Rustls was 1,463,712 bytes with 33 entries in the shared spike
workspace. Minreq is smaller, but Ureq provides the stronger maintained
protocol/configuration layer. Ureq is accepted without its gzip feature so
`http.client` returns the wire body. It removes more than 260 lines of local
socket/request/response/chunking code, preserves bounded reads and status
responses, and adds a tested `HTTPSConnection` path.

The archive spike used the same release profile. A trivial control was 302,560
bytes, `zip` 8.6.0 with only Flate2 deflate support was 385,664 bytes with 15
tree entries, and `tar` 0.4.46 without xattrs was 319,360 bytes with five
entries. Both are accepted. The complete runtime grows by 66,304 bytes over the
HTTP-only build, while maintained crates replace the local ZIP central-
directory/member parser and TAR header/member-boundary parser.

## Direct dependency decisions

| Dependency | Decision | Reason |
| --- | --- | --- |
| `flate2` | Retain, feature-minimal | Pure-Rust gzip/deflate backend, now shared by gzip and ZIP |
| `jiff` | Retain, feature-minimal | Maintained time-zone and DST correctness using the system zone database |
| `libc` | Retain | Small, standard platform ABI surface still required by terminal, process, and signal compatibility |
| RustCrypto `md-5`, `sha1`, `sha2` | Retain, feature-minimal | Maintained digest implementations; MD5/SHA-1 remain compatibility algorithms, not security choices |
| `regex-lite` | Retain | Covers the curated regex surface without the Unicode/data cost of the full `regex` crate |
| `rusqlite` + bundled SQLite | Retain | Mature DB-API substrate and standalone linkage; Turso is currently a much larger beta-engine graph |
| `ureq` + Rustls | Adopt | Transfers HTTP framing, parsing, bounded-body plumbing, TLS, and certificate roots to maintained crates |
| `zip` | Adopt, deflate only | Transfers ZIP variants, central-directory validation, CRC, and member bounds upstream |
| `tar` | Adopt, no xattrs | Transfers TAR header/path/member parsing upstream without extraction-only features |
| `ratatui` + Crossterm | Adopt, feature-minimal | Provides responsive layout, buffered rendering, inline viewports, focus styling, and TestBackend coverage for interactive selection |
| `kipferl-pocketpy-sys` + `cc` | Retain | Required local FFI boundary and build path for the embedded PocketPy C runtime |

CLI/loader dependencies remain limited to the shared format crate and the
existing RustCrypto hashes used by stable cache/integrity formats. General CLI
frameworks, full regex, async networking, alternate terminal backends, and
additional serialization layers are not adopted because they would not remove
enough product-specific code or improve the current public API.

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
relevant comparison for Kipferl input—adds Mio, signal-hook, signal-hook-mio,
Rustix, parking_lot, and supporting crates. The Ratatui measurement does not
include a terminal backend, yet already adds 52 dependency entries beyond the
control.

The initial spike was deferred because silently replacing every stateless
renderer would retain most of Kipferl's product-specific code. The later product
decision instead adopts Ratatui where its model is immediately valuable:
`input.select` and `input.multiselect` now use a real inline Ratatui viewport in
interactive terminals while preserving the public Python API.

The production integration adds 199,200 bytes to the complete `O2` runtime
(3,801,664 to 4,000,864 bytes) and brings the lockfile from 92 to 162 packages.
In return it provides buffered rendering, bounded scrolling, responsive normal
and compact layouts, a minimum-size message, visible keyboard help, semantic
focus styling, `NO_COLOR`, and reusable TestBackend infrastructure. The inline
viewport preserves shell scrollback; Crossterm handles interactive key and
resize events, while Kipferl's existing `/dev/tty`, batched legacy input,
raw-mode, and shutdown guards remain the compatibility/cleanup substrate.

Three TestBackend cases cover 80-column, compact, monochrome, and too-small
rendering. Real 80×24 PTY tests answer Crossterm's cursor query, send batched
selection and multiselection keys, verify the Ratatui border and discoverable
footer, check the returned Python values, and confirm cursor and termios
restoration. The byte-for-byte legacy renderer remains for non-interactive
sessions and deterministic Zig-compatibility fixtures.

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

The closing audit rechecked the VM owner, `RootFrame`, callback snapshots,
terminal and cursor guards, subprocess pipe ownership, SQLite userdata
destructors, and loader temporary-file cleanup. It found no additional defect
that justified changing the shipping FFI types in this batch. The existing
stress, sanitizer, PTY-restoration, concurrent-extraction, and compatibility
tests exercise those boundaries. The separate borrowed/rooted type redesign
remains worthwhile future hardening, but forcing it into the prerelease would
increase migration risk without evidence of a current failure. This closes the
bounded Rust-native optimization and safety phase.
