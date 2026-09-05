# Changelog

All notable user-facing changes are documented here.

## Unreleased

## [0.7.2] - 2026-09-05

### Fixed

- Reaudit all 1,000 pinned popular projects on the current runtime and update
  the CLI's embedded full report and exact syntax-blocker catalog.
- Bind each release CLI to a fresh audit of its own native runtime and target.
  Validate complete coverage, metadata pins, hashes, and derived exports before
  packaging, and verify the embedded report in published-binary smoke tests.
- Publish per-platform audit JSON, CSV, and syntax catalogs with checksums.
- Refresh the documentation and website with the rerun and comparison: the
  candidate retains 44 compilation-complete distributions, 24 with Python
  source, and no changed first blockers. These remain behaviorally unverified.

### Development

- Check in the original 999 available package metadata pins and explicitly
  record the missing ddtrace pin. Reuse verified downloads, never old compiler
  results, when producing release audits.

## [0.7.1] - 2026-09-05

### Fixed

- Replace the old line-art project banner with the canonical Kipferl banner.
- Complete the previously announced rename transition: use `kipferl` imports,
  commands, download names, and `KIPFERL_*` environment variables. Retire the
  deprecated command/download aliases and test-key variable.
- Check branding in source, filenames, and the obsolete line-art logo in CI.
- Refresh the website and all demo formats using the published 0.7.1 CLI.

### Upgrade notes

- Update old project imports and shell configuration to the canonical names
  before upgrading. The v1 universal executable bytes remain compatible;
  existing standalone applications continue to run.
- Runtime-bound dependency locks need refreshing after the runtime changes.
  Re-run `kipferl add` for declared requirements, review the lock, and test.

## [0.7.0] - 2026-09-05

### Added

- Native dotted imports with parent initialization, root/alias binding, circular
  imports, failure retries, and preserved globals from failed attempts.
- Exact per-platform release catalogs and package smoke reports verify online
  installation, locked offline restoration, and detached standalone execution.
- Trailing commas in parenthesized imports and function/lambda parameters,
  and adjacent plain strings/bytes, including default values.
- A reproducible top-1,000 package audit, ranked CLI/website views, and a
  before/after comparison. The first language patches increase source-bearing
  compilation-complete candidates from 12 to 20; dotted imports raise this to
  24 (44 total including zero-source distributions). These remain unverified
  and tied to the historical audit runtime, separately from release evidence.

- PyPI pure-Python wheel dependencies with `kipferl add`, exact artifact/runtime
  locks, offline `sync --locked`, compatibility catalog and installed-file checks.
  The initial catalog contains ten distribution versions with scoped test or
  blocker evidence. Unknown artifacts require explicit acceptance; native
  extensions, source builds, extras, and environment markers are unsupported.
- Installed package imports, resources, and license metadata flow through run,
  test, and standalone builds; new editor configurations include package paths.
- Portable builds include local Python modules and packages, follow their
  dependencies when selecting a runtime, and support explicit `--asset` files
  and directories. Build-time checks catch unsupported imports and syntax.
- `kipferl new --template cli|api|interactive` creates a runnable project with
  editor stubs, a README, tests, and a `kipferl.json` configuration.
- Project defaults let `run`, `dev`, `build`, and `test` work without repeating
  paths, including from nested project directories.
- Ordinary `test_*.py` project tests and Bash, Zsh, and Fish completions.
- Four executable recipes for CSV summaries, HTTP API clients, repository
  summaries, and report generation, checked against their documentation in CI.

### Changed

- Dynamic `__import__("http.client")` returns root `http`, following Python.
  Use a child-module alias or a nonempty positional fromlist when needed.
- Dependency locks identify exact runtime bytes and target. After an upgrade,
  rerun `kipferl add` for declared requirements, review the lock, and test the app.

### Fixed

- Package and bundle syntax checks use nonexecuting module compilation.
  Generated wrappers preserve module globals and original traceback filenames.

- Prevent callback-driven heap and iterator mutations from using invalid values;
  preserve comparator exceptions and snapshot containers before deepcopy hooks.
- Handle Unicode regex replacement escapes, oversized byte arrays, and negative
  iterator ranges without panicking. Reject finite values overflowing struct f32.
- Avoid overflowing median/progress calculations and long-version logo padding.
  Validate terminal dimensions and cap rule output at one million columns.

- Reject malformed non-ASCII download checksums without panicking. Report corrupt
  embedded runtimes and missing build assets with contextual errors.
- Reject extreme HTTP timeouts without aborting across FFI. Correct frexp/ldexp
  at float boundaries, atanh endpoint/NaN handling, and negative fractional times.
- Return structured VM metadata errors for C ABI length overflow. Remove
  unchecked format/loader slicing and move cache buffers off small worker stacks.

- Runtime errors identify original Python files and line numbers; application
  arguments and `sys.exit()` statuses survive development and packaged runs.
- Validate complete cached runtime and script contents, repairing same-size
  corruption and stale standalone payloads beyond the legacy hash sample.
- Build outputs are written atomically, preserving the previous artifact on
  failure and replacing output symlinks without changing their targets.
- Enforce the script size limit while reading, handle multiline legacy imports,
  and quote generated project names safely in Python source.
- Correct in-memory buffer seeking, line hints, closed-state errors, integer
  arguments, and empty/sparse writes; avoid quadratic work when appending.
- Preserve captured subprocess output after `wait()`, honor `Popen(text=True)`,
  and return the actual negative signal number for terminated processes.
- Compatibility checks now reject crashed processes, missing fixtures, and
  invalid runtimes. Vision reports retain timeout and benchmark failures,
  support single-sample runs, and include the supported TOML fixture.

### Development

- Add pinned Bacon, nextest, watchexec, and optional cargo-seek through mise;
  configure failure navigation, zero retries, slow/leaked-process detection,
  and CI JUnit reports. Add Criterion 0.8.2 as a loader-only development dependency.
- Enforce every requested Clippy restriction, pedantic, and nursery across the
  workspace. Require a clean full/core audit and publish all narrowly justified
  exceptions with their reasons; reject blanket groups and unexplained exceptions.

- Include Rust Analyzer and standard-library sources in the pinned setup;
  explicitly check all Rust features and targets in the shared CI task.
- Reject debug/placeholder macros and unchecked time subtraction in workspace
  Clippy. Forbid unsafe code in the format crate and document error contracts
  with executable API examples.
- Replace the justfile with mise tasks, exact development tool pins, a
  multiplatform tool lockfile, and shared local/CI setup for Rust and the website.

- Add regression coverage across CLI, loader, runtime, and test runners,
  including 56 additional I/O and subprocess compatibility checks.
- Correct CPython fixture inputs and skip Kipferl-only extensions on the host,
  so compatibility baselines complete on the CI Python version.
- Run development and release tooling tests in both `mise run check` and CI.
- Document focused runtime checks and the prebuilt CLI asset workflow.

## [0.6.0] - 2026-08-05

### Added

- Profile-based tree shaking selects a small core runtime or the complete
  runtime from static imports without requiring a user Rust toolchain.
- `--full-runtime` provides an explicit conservative override for universal
  builds.
- `kipferl dev` watches project and extra paths with native filesystem events,
  debounces editor bursts, and restores terminal state between restarts.
- Ratatui-backed `input.select` and `input.multiselect` provide responsive,
  accessible interaction while preserving terminal scrollback.
- Maintained Rust parsers provide YAML 1.2, TOML, and KDL 2.0 alongside the
  existing JSON, XML, CSV, and INI/CFG APIs.
- HTTPS, SQLite, archive, cryptographic, filesystem, process, and terminal
  capabilities are available through the curated runtime.
- macOS ARM64/x86_64 and static-musl Linux ARM64/x86_64 release assets include
  adjacent SHA-256 checksums.

### Changed

- Renamed the project, repository, binary, packages, and public site from
  μcharm/ucharm to Kipferl.
- Reimplemented the production CLI, universal loader, PocketPy host, and native
  modules in stable Rust; PocketPy remains the embedded C runtime.
- Replaced handwritten HTTP and archive internals with feature-minimal,
  maintained Rust libraries while preserving the Python-facing API.
- Updated the README, website, command documentation, templates, examples,
  benchmarks, and migration retrospective for the Rust architecture.

### Performance

- The four tree-shaken core runtimes are 1,130,352–1,349,904 bytes, 72.2–76.6%
  smaller than their full-runtime counterparts.
- A minimal Apple Silicon standalone app is 1,450,837 bytes, 69.9% smaller than
  the 4,817,925-byte full-runtime build.
- The measured Apple Silicon core app starts in 7.679 ms median and 8.433 ms
  p95 over 100 runs.

### Compatibility

- Passes 1,669/1,669 available compatibility checks, with 51 fully compatible
  targeted modules and one host-unavailable TOML baseline.
- Preserves the Kipferl v1 standalone application format.
- Accepts legacy `from ucharm ...` source and environment variables, publishes
  temporary `ucharm-*` assets, and installs a deprecated `ucharm` command alias
  for the 0.6 transition.

[0.7.0]: https://github.com/niklas-heer/kipferl/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/niklas-heer/kipferl/compare/v0.5.0...v0.6.0

[0.7.1]: https://github.com/niklas-heer/kipferl/compare/v0.7.0...v0.7.1

[0.7.2]: https://github.com/niklas-heer/kipferl/compare/v0.7.1...v0.7.2
