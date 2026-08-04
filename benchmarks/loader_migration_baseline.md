# Rust Loader Migration Baseline

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon. The Zig CLI packaged
the PocketPy runtime and Python fixture in both executables; only the loader
stub changed. Startup results use 50 measured cache-hit runs after 5 warmups.

| Loader | Stub bytes | Universal bytes | Median startup | p95 startup |
| --- | ---: | ---: | ---: | ---: |
| Zig 0.15.2 | 98,216 | 2,409,578 | 5.642 ms | 6.492 ms |
| Rust 1.97.1 | 320,352 | 2,631,714 | 5.451 ms | 5.913 ms |

The Rust universal executable is 222,136 bytes (9.2%) larger because the
loader uses Rust's standard library. Warm startup is within the existing
budget and was slightly faster in this host sample. CI keeps the stripped Rust
loader below 1 MB while later loader work looks for stable-toolchain size wins.

The compatibility run also verifies that the Rust loader can execute the real
runtime and transformed Python payload produced by the Zig packager.
