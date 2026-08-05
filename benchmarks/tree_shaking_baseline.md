# Tree-shaken Build Baseline

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

```console
cargo build --release -p kipferl-runtime
cp target/release/pocketpy-kipferl /tmp/pocketpy-kipferl-full
cargo build --release -p kipferl-runtime --no-default-features
cp target/release/pocketpy-kipferl /tmp/pocketpy-kipferl-core

cargo build --release -p kipferl-cli
printf 'pass\n' > /tmp/kipferl-minimal.py
target/release/kipferl build /tmp/kipferl-minimal.py -o /tmp/kipferl-core-app
target/release/kipferl build /tmp/kipferl-minimal.py \
  -o /tmp/kipferl-full-app --full-runtime

python3 benchmarks/migration_baseline.py \
  --candidate 'Core app=/tmp/kipferl-core-app' \
  --candidate 'Full app=/tmp/kipferl-full-app' \
  --runs 100 --warmups 10
```

The four-target numbers are authoritative because each runtime is built and
executed natively by the release matrix. Local cross-builds package those exact
checked-in release artifacts rather than approximating a foreign target.
