# Rust Release Cutover

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon, using Rust 1.97.1.
Linux artifacts were built in native ARM64 and emulated x86_64 Debian
containers with the musl targets. GitHub's native runners remain authoritative
for the final four-target release check.

## Component artifacts

| Target | Runtime bytes | Loader bytes | CLI bytes | Linkage |
| --- | ---: | ---: | ---: | --- |
| macOS ARM64 | 1,840,304 | 320,400 | 2,583,984 | Mach-O |
| macOS x86_64 | 1,971,304 | 318,080 | 2,703,280 | Mach-O |
| Linux ARM64 | 2,313,304 | 397,264 | 3,215,360 | static ELF |
| Linux x86_64 | 2,321,680 | 417,768 | 3,268,616 | static ELF |

The macOS runtime differs by 32 bytes from the network/database-wave sample
because this table records the final public binary name and cutover rebuild.
All runtime and CLI artifacts remain below their platform gates: 2 MB runtime
on macOS, 2.5 MB runtime on static Linux, and 3.5 MB for the release CLI.

## Findings and decisions

- The first local x86_64 musl rebuild contained an ELF interpreter despite the
  musl target. It would not execute in a clean Debian container. Both Linux
  target configurations now explicitly enable static CRT linkage and pass a
  CI guard requiring `statically linked` output with no `INTERP` segment.
- Replacing the small Zig loaders with all four Rust loaders initially grew the
  macOS ARM64 CLI to 3,723,328 bytes. The final CLI embeds only its host Rust
  runtime and loader; checksum-verified cross-target component pairs are read
  locally or downloaded on demand. That reduced the CLI by 1,139,344 bytes
  (30.6%) to 2,583,984 bytes without removing cross-target builds.
- A Cargo build script derives the embedded-runtime cache key from the exact
  runtime bytes. Tagged builds therefore cannot combine a refreshed embedded
  runtime with a stale hard-coded cache identity.
- The checked-in CLI component assets are now Rust builds for all four release
  targets. The tagged-release workflow rebuilds those components, injects the
  current pairs before compiling each CLI, publishes SHA-256 files, and ships
  a Linux ARM64 CLI in addition to the existing platforms.

## Universal-application smoke

| Target | Application bytes | Result |
| --- | ---: | --- |
| macOS ARM64 | 2,161,007 | Executed SQLite query and printed `42` |
| macOS x86_64 | 2,289,687 | Executed under Rosetta and printed `42` |
| Linux ARM64 | 2,710,740 | Executed in clean Debian and printed `Hello, World!` |
| Linux x86_64 | 2,739,620 | Static layout verified; native CI execution required |

Local gates passed: `cargo fmt`, strict workspace Clippy, all workspace tests,
CLI end-to-end tests with Rust assets, optimized builds, and the full
1,668/1,668 compatibility report. The canonical CI matrix additionally runs
the universal build and execution smoke on every native release runner.
