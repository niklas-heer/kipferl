# Development workflow

Run these commands from the repository root. `mise.toml` and `mise.lock` are the
shared setup for Rust, Python, Node, Bun, and the diagnostic tools. The Rust pin
also appears in `rust-toolchain.toml`; `mise run doctor` checks that they agree.
Use mise 2026.9.1 or newer and a working native C compiler: Xcode Command Line
Tools on macOS, or `build-essential` on Debian/Ubuntu. PocketPy and SQLite still
compile C code.

```console
mise trust
mise install --locked rust python node bun cargo:bacon aqua:nextest-rs/nextest/cargo-nextest watchexec
mise run setup
mise run doctor
mise tasks ls
```

Trust the configuration after reviewing it. `setup` verifies the tools and C
compiler, fetches the locked Cargo dependencies, installs the website's locked
Bun dependencies, and builds the release workspace. Bacon is compiled from its
pinned upstream source and lockfile, so its first installation takes longer than
a prebuilt tool. Nix, devenv, Make, just, and pnpm are not required.

Use `mise exec -- <command>` for an individual command that needs the pinned
tools. For example, `mise exec -- python3 --version` verifies which CPython will
supply the compatibility baseline. Shell activation is optional for these
explicit commands.

## Choose the feedback loop

| Command | What it checks or runs |
| --- | --- |
| `mise run bacon` | Interactive compiler feedback across the workspace and vendored C inputs |
| `mise run watch` | Queues a debounced check, full Rust test run, and debug build after source changes |
| `mise run check` | Tool pins, generated stubs, Python helper tests, Rust formatting, all-target compilation, strict full/core Clippy, full/core nextest suites, and doctests |
| `mise run lint-audit` | Every review lint in full/core profiles, plus the source locations and reasons for explicit exceptions |
| `mise run catalog-check` | Offline catalog identities, reviewed smoke-hook hashes, popularity provenance, and generated report consistency |
| `mise run package-audit` | Resume top-1,000 screening against the pinned ranking and freshly embedded host runtime |
| `mise run refresh-runtime-assets` | Build and verify this host's full/core runtimes and loader before embedding them in the CLI |
| `mise run test` | `check`, release build, CPython compatibility, vision scenarios, recipes, and website checks |
| `mise run bench` | Statistical loader benchmarks; run separately from correctness checks |
| `mise run website-dev` | Website development server |
| `mise run seek` | Optional pinned crate, feature, and MSRV explorer, installed on demand |

Inside Bacon, `c` selects full-workspace Clippy, `t` full tests, `r` core Clippy,
`Shift-R` core tests, and `d` doctests. Compiler locations are exported to
`target/bacon-locations` for editor integration. The watch loop runs tests and
builds, but does not execute an example application. It does not replace the
full/core Clippy gates in `check`.

For a focused test, retain the relevant feature profile:

```console
mise run test-rust
mise run test-core
mise run test-doc
mise exec -- cargo nextest run --locked -p kipferl-runtime --test runtime_safety
```

Nextest runs tests in separate processes and never retries a failure. It keeps
running after failures and shows captured failure output immediately and at the
end. Tests are marked slow after 30 seconds and terminated after four such
periods, with a 10-second grace period. Descendants retaining a test's output
pipes for more than one second make the test fail. This detects leaked
processes; it does not detect all memory leaks. Rust doctests run separately
through Cargo because nextest does not execute them.

## Read the diagnostics

`mise run test-ci` runs the full and core nextest profiles sequentially and then
doctests. The reports are `target/nextest/ci/junit.xml` and
`target/nextest/ci-core/junit.xml`. CI retains these as the `rust-test-reports`
artifact even when a preceding check fails; a failure before a suite starts can
leave its report absent. Set `RUST_BACKTRACE=1` when investigating a Rust panic.

`mise run lint-audit` writes `target/lint-audit/report.md`, `diagnostics.json`,
`exceptions.json`, and full/core JSON compiler output. It fails on outstanding
findings, compiler errors, or exception-policy violations. An empty findings
list does not mean the code has no explicit lint exceptions: the exception
inventory records each reviewed invariant and its source location. CI uploads
the directory as `clippy-restriction-audit`. See the
[Rust review](rust-review.md) for the policy and fixes it uncovered.

## Know which runtime you tested

`mise run build` builds the workspace from source. Compatibility and vision
tasks explicitly execute `target/release/pocketpy-kipferl`, so they exercise the
fresh runtime. The CLI instead embeds the host's checked-in full/core runtime
assets when it is compiled. Rebuilding the CLI does not replace those assets
with the freshly compiled runtime. `kipferl run` and packaged applications can
therefore test a different runtime revision from the raw-runtime suites.

Use a fresh raw-runtime smoke test when changing native modules:

```console
mise run build-runtime
target/release/pocketpy-kipferl -c 'import math; print(math.sqrt(9))'
mise run compat
mise run vision
mise run recipes
```

Recipes check both raw-runtime execution and isolated standalone packaging.
The release pipeline must refresh embedded assets and validate the produced
packages before claiming that source changes ship in them. Native foreign-target
execution, Linux static-link checks, and the Linux AddressSanitizer/
UndefinedBehaviorSanitizer job require their CI environments; a local macOS pass
does not establish those results. Compatibility totals can vary with the host
CPython baseline, so record its version alongside counts and skipped checks.

## Optional maintenance tools

`mise run bindings` requires separately installed `bindgen-cli` and libclang
when changing PocketPy's public C API. Demo recording requires VHS. Neither is
needed for routine development. See [PocketPy maintenance](../pocketpy/README.md)
for patch verification and [benchmarking](../benchmarks/README.md) for repeatable
performance measurements. Git hooks are not installed automatically; run the
checks before opening a pull request.

## Maintain editor stubs

The 27 canonical `stubs/*.pyi` files describe Kipferl's actual module surface.
Check arguments, return types, and exported names against the runtime, including
PocketPy's embedded Python definitions. Do not copy a complete CPython API into a
stub when the runtime implements only a subset. `mise run stubs-check` validates
syntax, registration, and the generated manifest; CLI tests verify exported bytes.
These checks do not establish compatibility with every external type checker.

## Maintain PyPI package support

The CLI owns dependency resolution and installation; the embedded interpreter
and standalone loader do not fetch packages. `pep508_rs` supplies requirement
and version semantics, `ureq` performs bounded HTTPS requests with Rustls, `zip`
reads stored or Deflate-compressed wheel entries, and Serde models the strict lock
schema. These dependencies belong to the development CLI, not a packaged app's
runtime. New package artifacts are checked in staging before publication.

Compatibility JSON is compressed deterministically at build time using the
existing gzip dependency. Decoding has an 8 MiB limit per document, and parsed
evidence is cached. Tests compare the decoded values with the canonical files.
This removed about 3.70 MB from the macOS ARM64 CLI (10.85 MB to 7.15 MB before
the dotted-import asset refresh). The development CLI's CI budget is 9.25 MB
to cover native dependency resolution, TLS, wheel validation, and evidence
across release targets. Runtime and loader size budgets remain separate and
unchanged; packaged applications do not carry this package-manager code.

`mise run catalog-check` validates the checked-in compatibility evidence offline.
The normal Python test task also checks the catalog and smoke-hook hashes.
See [the package catalog guide](../compatibility/packages/README.md) for reviewed
candidate selection, evidence refresh, and the limits of each tested result.
After refreshing the catalog, rebuild the CLI to embed the new records.

`mise run build` and `mise run check` refresh this host's embedded runtime assets
from source first. The core runtime uses a separate build directory, preserving
the full runtime used by the audit. The refresh verifies nonexecuting module
compilation before replacing an asset and skips identical files. When using
Cargo directly after interpreter changes, run `mise run refresh-runtime-assets`
before CLI tests or builds. Other target assets are built and verified by their
native CI matrix jobs; a host refresh does not claim cross-platform validation.

Import changes have CPython-oracle package-tree tests covering bindings,
initialization order, cycles, failed-import cleanup, and surviving module
globals. The Linux ASan/UBSan job includes these tests alongside FFI stress
tests. The CPython oracle subprocess does not inherit sanitizer preloads;
the instrumented Kipferl subprocess does.

Keep package manager tests deterministic: Rust fixtures create wheels and seed
an offline cache, then exercise transitive resolution, extraction checks,
lock restoration, and standalone execution after source removal. Fetching real
PyPI artifacts is an explicit catalog-maintenance or smoke-testing action, not
part of every local test run.

## Prepare a release candidate

Keep `VERSION`, the Cargo workspace version, and workspace lock entries aligned.
`python3 scripts/check_release_version.py` validates them and, in a tag workflow,
requires the tag to match. Both full and core runtimes expose `--version` for
artifact verification.

After pushing a prepared commit and obtaining green CI, dispatch the Release
workflow on that branch to exercise the whole build without publishing:

```sh
gh workflow run release.yml --ref main
```

Manual runs build and check every platform but skip the release and Homebrew
jobs. Once that exact commit passes both workflows, create and push its version
tag. A tag containing `-rc.N` publishes a prerelease and leaves the stable latest
release and Homebrew formula unchanged. Use curated notes under
`.github/release-notes/<tag>.md` to explain upgrade behavior.

To promote a candidate to stable, remove the `-rc.N` suffix from all version
files and rebuild the components. Their bytes and package-lock identities can
change even when language behavior does not. Generate fresh reviewed evidence
for the final binaries; never relabel the candidate's hashes. A stable tag such
as `v0.7.1` updates GitHub's latest release and the Homebrew formula. The formula
updater downloads all four binaries and their checksum sidecars, verifies them,
and replaces the formula only after every artifact matches. After publication,
verify the public downloads, Homebrew version/checksums, and live documentation.

The Homebrew tap requires pull requests. Its automation pushes a release branch,
opens or reuses the corresponding PR, and requests a normal squash merge with
auto-merge for pending checks. It never bypasses review or branch rules. If a
merge cannot complete, the workflow summary links the pending PR instead of
claiming the formula is live. `HOMEBREW_TAP_TOKEN` needs both Contents: write and
Pull requests: write on the tap; an API `permissions.push` result alone does not
establish permission to push a protected default branch or merge a PR.

Each component job generates fresh catalog evidence from the reviewed wheel
pins against its final full runtime. macOS keeps the behavior hook sandbox;
Linux requires explicit disposable GitHub Actions execution. Only the pinned,
reviewed tzdata hook executes. Historical catalog records remain, and the broad
popularity audit retains its original runtime identity.

Before building a CLI, `prepare_release_assets.py` checks the complete set of
12 components and four catalog checksums, verifies matching tested evidence,
and restores executable permissions. Missing files cannot silently fall back
to tracked assets. `check_release_packages.py` then uses isolated project and
cache directories to test installation, missing-cache failure, locked offline
restoration, and a standalone app after deleting the project and caches. Reports
record exact artifact hashes and whether offline checks used OS network denial
(macOS) or the CLI offline flag (Linux). CI and releases share executable size
budgets through `check_release_sizes.py`.
