# Package compatibility catalog

The catalog speeds up dependency checks with reusable evidence. It is an **allowlist/blocklist of exact artifacts on exact runtimes**, not a blanket judgment about a project. The lookup key contains the normalized distribution name, version, wheel SHA-256, runtime binary SHA-256, and operating-system/architecture target. A new version, wheel, runtime build, or target starts unverified. This deliberately includes runtime rebuilds that happen to share a release version.

`tested` means all Python sources compiled and the checked-in behavior hook passed within its stated scope. `incompatible` means this exact artifact has a demonstrated syntax, API, behavior, or native-wheel blocker. `unverified` means evidence is missing or incomplete. Source compilation and successful imports cannot establish compatibility for unexercised paths, dynamic imports, optional extras, or every use of a library. Each dependency is assessed separately before installation; a tested parent never waives a dependency's failures.

## Initial evidence

The pinned candidates include attrs, colorama, idna, inflection, more-itertools, NumPy, packaging, six, toml, and tzdata. Read `catalog.json` for the precise versions, wheel filenames/hashes, runtime hashes, compiler locations, and tested scopes. The initial results cover macOS ARM64 and both the checked-in embedded runtime and the freshly built runtime. Other targets remain unverified.

`tzdata==2025.2` is a positive resource-package example: version constants and the TZif headers of four representative timezone files pass. This does **not** add Python's `zoneinfo` API or establish timezone-conversion behavior. The other initial artifacts have concrete compiler or native-wheel blockers. Their distribution names are not permanently blocked: a future compatible artifact/runtime can have different evidence.

## Validate without downloading or executing packages

```sh
mise exec -- python3 scripts/package_catalog.py --check
mise exec -- python3 -m unittest discover -s scripts -p 'test_package_catalog.py'
```

The standard Python tooling test suite validates schema, exact hashes, unique evidence keys, and checked-in smoke hook hashes. Network access is unnecessary for these checks. Changing a tested smoke hook invalidates its old evidence until refreshed.

## Refresh evidence

Review the pinned candidates in `candidates.json`. The updater fetches release metadata and wheels from official PyPI HTTPS endpoints, verifies downloaded bytes against the published SHA-256, checks extraction paths/size limits, and compiles each Python source using the supplied runtime. It never runs setup.py or an sdist build backend.

```sh
mise exec -- python3 scripts/package_catalog.py \
  --runtime target/release/pocketpy-kipferl \
  --runtime crates/kipferl-cli/assets/pocketpy-kipferl-macos-aarch64
```

That command only compiles package source; it does not import or execute the package. Candidates without concrete compiler blockers remain unverified. It replaces the catalog with evidence for the supplied binaries, so inspect the diff before committing and include every runtime whose evidence should be retained.

After reviewing both a smoke hook and the exact wheel code it imports, explicitly enable behavior execution:

```sh
mise exec -- python3 scripts/package_catalog.py \
  --runtime target/release/pocketpy-kipferl \
  --runtime crates/kipferl-cli/assets/pocketpy-kipferl-macos-aarch64 \
  --execute-reviewed
```

Behavior execution currently requires macOS `sandbox-exec`; the updater refuses to silently run without it. Each invocation uses a separate temporary directory, a copied runtime, a cleared environment, and a timeout. The sandbox denies network access, filesystem writes, and reads from `/Users`, `/home`, and `/root`. It still permits system and temporary-file reads and is a constrained developer test runner, not a complete security boundary for arbitrary hostile code. Use a disposable environment when investigating unaudited packages. Package installation never runs these behavior hooks.

All results are tied to the binary actually invoked, including the embedded runtime when that path is supplied. To extend platform coverage, regenerate with the relevant native runtime on that target and review/merge the resulting exact records. Do not copy a tested result onto an untested platform.

## Schema

`catalog.json` has `schema_version: 1` and a `records` array. Every record requires `name`, `version`, `wheel_filename`, `wheel_sha256`, `runtime_sha256`, `target`, `status`, and human-readable `evidence`. Additional fields include the official `source_url`, `source_files_checked`, source-specific `compile_failures`, and a `smoke` object. Tested records require a checked-in hook filename, its SHA-256, and a concrete behavior scope. The Rust lookup fails closed if required fields, statuses, hashes, or evidence keys are malformed.

## Top 1,000 package screening

The [compatibility priorities](priorities.md) summarize the most common first parser failures and promising candidates for focused behavior tests. Run `mise run catalog-check` for offline evidence validation, or `mise run package-audit` to screen the pinned ranking against the freshly built runtime embedded for your host. A different runtime hash produces a separate cache identity and fresh evidence.

`popularity.json` pins the 1,000 projects with the most downloads in the upstream ranking's August 2026 window. Its recorded source uses ClickHouse; preserve the source URL, query, reporting window, retrieval time, and source hash when refreshing it. Downloads indicate popularity, not package quality or runtime compatibility.

`popularity-audit.json` records one selected **latest PyPI release per ranked project**, pinned at its recorded metadata-fetch time. The completed rerun covers all 1,000 projects on the patched embedded macOS ARM64 runtime. It found 774 exact verified wheel syntax blockers, 178 releases without a usable generic Python 3 pure wheel (including one purportedly pure wheel containing native libraries), five declared Python-version conflicts, one release without a usable wheel, 40 compilation-complete but behaviorally unverified distributions, and two audit limits. There were no network failures. Twenty of those 40 distributions contain Python source. Older releases and alternative wheels were not explored.

The [language-patch comparison](language-patch-comparison.md) preserves per-project before/after evidence. Trailing commas and adjacent plain strings/bytes were implemented, and the checker was corrected to compile in normal module mode. Compilation-complete source-bearing candidates increased from 12 to 20; 383 other packages now hit a different first blocker. Nine original `global` diagnostics came from dynamic compilation rather than missing module-level language support.

The two limits are explicit: awscli's wheel exceeded the extraction bound, and ddtrace's release-history JSON exceeded the metadata download bound. A package that contains no `.py` files, such as a stub or dependency-only distribution, remains unverified; it does not acquire a compatibility guarantee from an empty compilation pass. The audit preserves declared dependencies but does not resolve or test the entire dependency closure for each of the 1,000 projects.

Read the [summary](popularity-audit.md), [complete JSON](popularity-audit.json), or [CSV](popularity-audit.csv). The website and `kipferl deps audit` display the same canonical report. `popularity-catalog.json` contains only the exact, hash-verified syntax failures, and the CLI combines these with the original reviewed catalog while deduplicating identical evidence keys. Metadata-only observations and compilation passes never become a behavioral allowlist entry. All diagnostic and distribution-version claims remain limited to the recorded artifact/runtime/target.

Run the audit or resume the same snapshot and policy:

```sh
mise exec -- python3 scripts/package_popularity_audit.py \
  --snapshot compatibility/packages/popularity.json \
  --runtime crates/kipferl-cli/assets/pocketpy-kipferl-macos-aarch64 \
  --workers 4 --limit 1000
```

The runner uses up to six workers, official PyPI HTTPS endpoints, SHA-256 verification, safe extraction, a cleared compiler environment, and fail-fast compilation. It does not import package code, run setup scripts or build backends, or execute behavior hooks. Each artifact records how many source files were checked, its total source files, and the remaining files after the first blocker or limit. Requires-Python is interpreted against Kipferl's advertised Python 3.11.0 metadata target, using the pinned development interpreter's bundled pip packaging parser; this does not imply complete Python 3.11 language or library support.

Checkpoints and artifact downloads live under ignored `target/package-audit`. A checkpoint identity includes the ranking snapshot hash, runtime binary hash, parser version, advertised Python version, limits, and audit-policy version. Metadata is pinned before artifact work, so resuming does not silently select a newer release. A parser or policy change creates a new checkpoint identity. `--retry-network` retries network failures while retaining metadata already pinned; `--metadata-prefetch PATH` explicitly seeds new pins from previously downloaded raw PyPI JSON instead of making fresh requests. No prefetched metadata is reused implicitly.

The current policy is version 2: `runtime --check-syntax -- <source>` calls the normal module compiler without execution. Every new row includes `compilation_completed`, which is true only after all source checks finish; a failure on the last source file does not acquire that flag. The runner probes this capability before auditing and fails closed with an older runtime. `--embedded-runtime` selects the current host's CLI asset; `--runtime PATH` chooses an explicit binary.

For controlled comparisons, `--seed-metadata-from PATH` copies only metadata pins from a checkpoint with the same ranking snapshot. It never copies result records into the new runtime/policy cache. For example:

```sh
mise exec -- python3 scripts/package_popularity_audit.py \
  --embedded-runtime --seed-metadata-from /path/to/previous/checkpoint \
  --output /path/to/comparison/popularity-audit.json
```

The original checkpoint-layout migration remains restricted to its exact version-1 policy and therefore refuses migration into the current module-checker policy. Use metadata-only seeding for a new runtime or checker. `scripts/compare_package_audits.py BEFORE AFTER` validates both reports and rejects version, wheel, metadata, or ranking drift before producing the comparison.

Validate the entire checked-in report offline:

```sh
mise exec -- python3 scripts/package_popularity_audit.py --check
mise exec -- python3 -m unittest discover -s scripts -p 'test_package_popularity_audit.py'
```

The check validates report semantics, policy/cache digests, snapshot hash and ranking provenance, every rank/name/download count, source coverage, complete-result counts, and exact agreement of the generated CSV and syntax catalog with the canonical JSON. It checks consistency and provenance; it does not redownload wheels or rerun the compiler.
