# Rust Migration Host Baseline

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon. Each startup result is
from 50 measured runs after 5 warmups.

| Runtime | Bytes | Median startup | p95 startup |
| --- | ---: | ---: | ---: |
| Zig baseline | 2,313,264 | 3.960 ms | 4.689 ms |
| Rust spine | 600,496 | 2.760 ms | 3.108 ms |

The Zig baseline includes μcharm's current native modules and external C
dependencies. The Rust spine includes PocketPy plus the probe native module,
but not the μcharm module set yet. The figures establish the measurement
workflow and early feasibility; they are not a like-for-like performance claim.

Refresh locally with:

```console
just build-pocketpy
just rust-build
just rust-baseline
```
