# Rust Migration Host Baseline

> Historical snapshot: the measurements and implementation decisions below describe
> the recorded migration stage, not the current release. See the
> [benchmarking guide](README.md) for current commands, budgets, and validation limits.

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon. Each startup result is
from 50 measured runs after 5 warmups.

| Runtime | Bytes | Median startup | p95 startup |
| --- | ---: | ---: | ---: |
| Zig baseline | 2,313,264 | 3.960 ms | 4.689 ms |
| Rust spine | 600,496 | 2.760 ms | 3.108 ms |

The Zig baseline included Kipferl's native modules at that stage and external C
dependencies. The Rust spine includes PocketPy plus the probe native module,
but not the Kipferl module set yet. The figures establish the measurement
workflow and early feasibility; they are not a like-for-like performance claim.

Measure the current runtime with:

```console
mise run build-runtime
mise exec -- python3 benchmarks/migration_baseline.py \
  --candidate 'Current=target/release/pocketpy-kipferl' \
  --runs 50 --warmups 5 --seed 42
```

Recreating the table above requires the original saved Zig and Rust-spine
artifacts. A build of the current source includes a different module set and
cannot stand in for either historical candidate.
