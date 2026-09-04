# Tree-shaken Build Baseline

> Historical snapshot: the measurements and implementation decisions below describe
> the recorded migration stage, not the current release. See the
> [benchmarking guide](README.md) for current commands, budgets, and validation limits.

Recorded on 2026-08-05 for the stable v0.6 tree-shaking work tracked in
[issue #57](https://github.com/niklas-heer/kipferl/issues/57). Release runtimes
use the workspace release profile. Linux artifacts are statically linked with
musl. The first three target rows come from
[CI run 31006350747](https://github.com/niklas-heer/kipferl/actions/runs/31006350747);
the Intel macOS job succeeded on retry after a transient GitHub artifact-service
timeout.

## Architecture

Kipferl ships two prebuilt profiles so `kipferl build` does not require Rust,
Cargo, a C compiler, or a linker on the user's machine:

- `core` compiles the dependency-light CLI runtime without the `archives`,
  `crypto`, `formats`, `http`, `interactive`, `regex`, `sqlite`, and `timezone`
  Cargo features;
- `full` enables the complete compatibility surface;
- conservative static import analysis selects between them, and
  `--full-runtime` forces the complete profile.

The runtime inside the standalone application is not compressed. The CLI
stores its two embedded source runtimes as deterministic gzip streams and
decompresses the selected bytes before packaging, so application size and
startup behavior remain directly comparable with RC2.

## Four-target runtime size

| Target | Full runtime | Core runtime | Runtime reduction |
| --- | ---: | ---: | ---: |
| macOS ARM64 | 4,497,440 | 1,130,352 | 74.9% |
| macOS x86_64 | 5,056,944 | 1,183,140 | 76.6% |
| Linux ARM64 musl | 4,738,296 | 1,316,512 | 72.2% |
| Linux x86_64 musl | 5,451,504 | 1,349,904 | 75.2% |

All four core runtimes are below the 2,500,000-byte cross-target regression
ceiling. Both Linux core runtimes passed the workflow's `readelf` static-link
check and executed the core module smoke suite and a packaged universal app.

## Representative application size

Built on Apple Silicon from the checked-in CI core runtime. The differences
inside each profile are only transformed source bytes.

| Application | Selected profile | Standalone bytes |
| --- | --- | ---: |
| Minimal (`pass`) | core | 1,450,837 |
| TUI presentation (`tui`) | core | 1,450,861 |
| JSON | core | 1,450,876 |
| Interactive prompt (`input`) | full | 4,817,981 |
| YAML | full | 4,817,964 |
| HTTP/TLS | full | 4,817,973 |
| SQLite | full | 4,817,970 |
| Minimal with `--full-runtime` | full | 4,817,925 |

The default minimal application is **69.9% smaller** than its forced-full
equivalent. Optional-capability applications retain the same full runtime as
RC2 rather than replacing maintained libraries with smaller custom code.

## Startup

Measured on macOS 26.5.1 ARM64 with 100 randomized round-robin launches per
candidate after 10 warmups (seed 42):

| Application | Bytes | Median startup | p95 startup |
| --- | ---: | ---: | ---: |
| Core minimal | 1,450,837 | 7.679 ms | 8.433 ms |
| Full minimal | 4,817,925 | 8.480 ms | 9.161 ms |

Both remain below the 10 ms product goal on the measured host. Profile analysis
and deterministic gzip decompression happen during `kipferl build`, not when the
standalone application starts.

## Reproduce locally

The following compares applications packaged with the CLI's checked-in runtime
assets. It does not use a newly compiled `pocketpy-kipferl` binary. Use a temporary
directory to keep measurements separate from the workspace:

```console
mise run build
benchmark_dir="$(mktemp -d)"
printf 'pass\n' > "$benchmark_dir/minimal.py"
target/release/kipferl build "$benchmark_dir/minimal.py" \
  -o "$benchmark_dir/core-app"
target/release/kipferl build "$benchmark_dir/minimal.py" \
  -o "$benchmark_dir/full-app" --full-runtime

mise exec -- python3 benchmarks/migration_baseline.py \
  --candidate "Core app=$benchmark_dir/core-app" \
  --candidate "Full app=$benchmark_dir/full-app" \
  --runs 100 --warmups 10 --seed 42
```

To compare current runtime source profiles directly, build the core profile in a
separate target directory so it does not replace the full runtime used by other
tasks:

```console
mise run build-runtime
CARGO_TARGET_DIR=target/benchmark-core mise exec -- cargo build --locked \
  --release -p kipferl-runtime --no-default-features
mise exec -- python3 benchmarks/migration_baseline.py \
  --candidate 'Core runtime=target/benchmark-core/release/pocketpy-kipferl' \
  --candidate 'Full runtime=target/release/pocketpy-kipferl' \
  --runs 100 --warmups 10 --seed 42
```

The recorded four-target numbers describe the release matrix linked above,
where each runtime was built and executed on its target platform. New local
results are separate measurements. Packaging a foreign-target release asset
locally does not verify its execution on that target.
