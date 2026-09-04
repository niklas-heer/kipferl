# CLI `run` Migration Baseline

> Historical snapshot: the measurements and implementation decisions below describe
> the recorded migration stage, not the current release. See the
> [benchmarking guide](README.md) for current commands, budgets, and validation limits.

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon. Both CLIs used the
released `pocketpy-kipferl-macos-aarch64` runtime and ran
`tests/vision/scripts/t_errno.py` from a warm cache. Timings are 30 measured
runs after five warmups.

| CLI | Stripped size | Median | p95 |
| --- | ---: | ---: | ---: |
| Zig 0.15.2 | 2,783,944 bytes | 4.894 ms | 5.162 ms |
| Rust 1.97.1 | 2,698,208 bytes | 4.705 ms | 6.264 ms |

The Rust CLI is 3.1% smaller on this host and has comparable warm execution
latency. The Rust implementation also embeds the appropriate released runtime
for all four supported targets; the legacy `run` command only selected the
macOS ARM64 asset.

Direct differential checks cover help and error output, transformed script
bytes, application output, argument forwarding, and failure exit status.
