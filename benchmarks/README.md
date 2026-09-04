# Benchmarking Kipferl

The other Markdown files in this directory preserve measurements from specific
migration stages. Their runtime contents, dependencies, artifact sizes, and size
ceilings may differ from the current project. Keep those observations dated;
record new measurements with the revision, toolchain, host, profile, and workload
rather than overwriting the historical tables.

## Loader microbenchmarks

From the repository root:

```console
mise run bench
```

The pinned Criterion benchmark measures bounded bundle inspection and validation
of an already populated loader cache. Each operation uses fixtures with 1 KiB
and 1 MiB runtime and Python payloads, giving four cases. Fixture setup happens
outside the timed operations. Failures abort measurement instead of producing
success-shaped timings. HTML results are written under `target/criterion/`.

These results separate inspection overhead from warm cache validation. They do
not measure process startup, cold cache extraction, Python execution, or foreign
platform performance. Run benchmarks without competing builds or test suites;
compare the same toolchain and host before attributing a difference to code.
Criterion is a development-only dependency of the loader benchmark.

## Runtime startup and script execution

Build and measure the fresh host runtime explicitly:

```console
mise run build-runtime
mise exec -- python3 benchmarks/migration_baseline.py \
  --candidate 'Current=target/release/pocketpy-kipferl' \
  --runs 100 --warmups 10 --seed 42
```

The default workload is `-c pass`. Use `--code '...'` for another expression or
`--script path/to/workload.py` for script wall time. Repeat `--candidate LABEL=PATH`
to compare saved binaries; every candidate must support the same workload. The
runner warms each candidate, randomizes round-robin order using the supplied
seed, requires every process to succeed, and reports file size, median, and p95.
Keep copies of the compared binaries and record the source revisions that
produced them. The script's default candidate pair references the old Zig
migration artifacts, so supply `--candidate` for current work.

These are warm host-process measurements, including launch and exit overhead.
They are not cold-start measurements or a promise about other machines. The
10 ms startup objective used in earlier reports is a product goal, not a hard
timing gate in CI.

## Standalone applications and release sizes

The CLI embeds checked-in runtime assets; a newly built raw runtime is not
implicitly used by `kipferl build`. When comparing packaged applications, record
the embedded asset revision as well as the CLI revision. See the updated
[tree-shaking procedure](tree_shaking_baseline.md#reproduce-locally) and the
[development guide](../docs/development.md#know-which-runtime-you-tested).

The current CI release matrix enforces these uncompressed artifact limits,
strictly below the listed number of bytes:

| Artifact | Limit |
| --- | ---: |
| Full runtime | 5,750,000 |
| Core runtime | 2,500,000 |
| Loader | 1,000,000 |
| CLI with embedded runtimes | 6,750,000 |

The source of truth is the size-budget step in
[CI](../.github/workflows/ci.yml). These limits are regression budgets, not
measurements. Cross-target release acceptance also requires native smoke tests
and static-link checks for Linux; local benchmarking alone does not establish
that a release works on all four targets.
