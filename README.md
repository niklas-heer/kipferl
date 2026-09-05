<p align="center">
  <img src="assets/kipferl-logo.png" alt="Kipferl logo" width="200">
</p>

<h1 align="center">Kipferl</h1>

<p align="center">
  <strong>Python CLIs. Standalone binaries. Fast startup.</strong>
</p>

<p align="center">
  <a href="https://github.com/niklas-heer/kipferl/actions/workflows/ci.yml"><img src="https://github.com/niklas-heer/kipferl/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/niklas-heer/kipferl/releases"><img src="https://github.com/niklas-heer/kipferl/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

```
╭─────────────────────────────────────────────────╮
│                                                 │
│   Write Python. Ship one native binary.         │
│   Fast startup. No Python install.              │
│                                                 │
╰─────────────────────────────────────────────────╯
```

Kipferl is a focused runtime for beautiful, fast CLI apps. You write Python-style
scripts, and Kipferl ships them as single-file binaries that start quickly.

[Documentation](https://kipferl.dev/docs) ·
[Kipferl 0.7 release story](https://kipferl.dev/blog/kipferl-0-7) ·
[Changelog](CHANGELOG.md) · [Releases](https://github.com/niklas-heer/kipferl/releases)

- Tree-shaken standalone binaries (1.4 MB in the v0.6 minimal Apple Silicon baseline;
  the complete runtime remains available when needed)
- Beautiful TUI output (boxes, tables, prompts, progress)
- Fast startup, with dated release measurements below
- Curated stdlib compatibility for CLI use cases

---

## Quickstart

**[Kipferl v0.7.2](https://github.com/niklas-heer/kipferl/releases/tag/v0.7.2)**
brings project templates, configuration defaults, completions,
compatibility-checked PyPI dependencies, and portable packaging with local
modules and resources. Follow the
[stable installation guide](https://kipferl.dev/docs/getting-started/installation#stable-release)
for Homebrew or a platform binary with checksum verification, then try the
workflow below. Read the [0.7 release story](https://kipferl.dev/blog/kipferl-0-7)
for the changes and the [upgrade guide](https://kipferl.dev/docs/guides/packages#upgrade-to-072)
before moving a project from v0.6 or the release candidate.

Homebrew follows the stable release: use `brew install niklas-heer/tap/kipferl`
or `brew update && brew upgrade kipferl`, then check `kipferl --version` reports
`v0.7.2`. The project workflow below requires 0.7; v0.6 still supports explicit
script commands such as `kipferl dev app.py` and `kipferl build app.py -o app`.

```bash
kipferl new hello --template cli
cd hello
kipferl run -- --name Ada
kipferl dev
# Press Ctrl-C when you finish editing
kipferl test
kipferl build
./dist/hello --help
```

New projects include an argument-parsing CLI, a starter test, editor stubs,
and `kipferl.json` defaults. Choose `--template api` for an HTTP client or
`--template interactive` for terminal prompts. Bare `run`, `dev`, `test`, and
`build` use the project configuration; explicit script paths still work.

The default build bundles local Python modules. Add `--asset assets` or list
resource paths in `kipferl.json` to include application data, and read resources
relative to `__file__`. Try the result from a directory containing only the
executable before sharing it. Unsupported static imports fail during the build
with guidance about the dependency.

`kipferl dev` restarts on Python, config, and template changes and keeps waiting
after the app exits. Use `--watch <path>` for additional files, `--clear` for a
fresh screen, or `--debounce <ms>` for tools that write in bursts.

Follow the [complete first-app tutorial](https://kipferl.dev/docs/getting-started/quick-start),
[project and completion reference](https://kipferl.dev/docs/commands/projects),
[portable packaging guide](https://kipferl.dev/docs/guides/packaging), or
[development reference](https://kipferl.dev/docs/commands/dev).

### PyPI packages with compatibility checks

Use `kipferl deps catalog` to see tested package versions and known blockers.
`kipferl add <requirement>` resolves pure-Python wheels and their dependencies,
checks their source with the embedded runtime, and installs accepted artifacts
inside the project. It records exact versions, wheel and installed-file hashes,
and the embedded runtime/target identity in `kipferl.lock`.
An unknown package stays **unverified**; `--allow-unverified` makes that choice
explicit without bypassing known incompatibilities. This choice is stored in
the lock for reproducible syncs; compilation alone is not a behavioral test.

```bash
kipferl deps catalog
# Requires a tested record for your exact release runtime and platform:
kipferl add 'tzdata==2025.2'
kipferl deps list
kipferl deps check
kipferl sync --locked
# After the wheels have been cached locally:
kipferl sync --locked --offline
```

The release pipeline generates a fresh catalog for each exact runtime
and platform: macOS/Linux on ARM64 and x86_64. The tested `tzdata` scope covers
version constants and four timezone data headers, not timezone conversion or
`zoneinfo`. Test your own package usage. Release verification produces
`package-catalog-<target>.json` and `package-smoke-<target>.json` evidence.

Upgrading the CLI can change its runtime hash and invalidate an existing lock.
Re-run `kipferl add` for your declared requirements, review the new lock, and
run your application tests; do not edit hashes to bypass a mismatch. Dynamic
`__import__("http.client")` now returns the root `http` module. Use
`import http.client as client` for the child module. See the
[package guide](https://kipferl.dev/docs/guides/packages) and
[0.7 upgrade notes](https://kipferl.dev/docs/guides/packages#upgrade-to-072) for details.

Commit `kipferl.json` and `kipferl.lock`. Run, test, and build use the same
verified installation; standalone builds carry the imported Python modules,
package data, and license metadata. Neither pip nor a system Python is needed
to install supported wheels. Native extensions and source builds are unsupported.
See the [package guide](https://kipferl.dev/docs/guides/packages) for the supported
requirement syntax, catalog evidence, and recovery commands. Contributors can
validate or extend the [checked-in catalog](compatibility/packages/README.md)
with exact wheel/runtime hashes and focused behavior hooks.

### Explore the popular-package audit

The refreshed 0.7.2 source audit covers the top **1,000 PyPI projects** in the
August 2026 monthly download snapshot. It reuses the selected release and
artifact pins so the comparison measures runtime changes. **44 distributions
finish compilation, including 24 with Python source; none become behaviorally
approved by this screen.** The 770 syntax blockers, 178 native-wheel constraints,
and remaining metadata/resource findings are unchanged from the previous screen.
Imports, package APIs, and dependency closures still need focused tests.

The website shows the canonical macOS ARM64 report. Each 0.7.2 release CLI embeds
a fresh report for its own exact runtime and target; `deps audit` exposes that
evidence offline and identifies a differing runtime hash in development builds.
Browse the
[searchable audit](https://kipferl.dev/docs/guides/package-audit) for top-100 and
top-1,000 summaries, package/error search, reason filters, and exact evidence.
Download counts include automation and do not measure unique users.

```bash
kipferl deps audit
kipferl deps audit --limit 100
kipferl deps audit --json
```

This audit compiles sources without importing or executing package code.
Compilation-only results remain **unverified**, and a blocker in one release is
not a verdict on every version of that project. The generated
[Markdown report](compatibility/packages/popularity-audit.md),
[JSON evidence](compatibility/packages/popularity-audit.json), and
[CSV export](compatibility/packages/popularity-audit.csv) contain the current
counts and distinguish verified artifact findings from metadata-only checks.
Verified syntax failures supply exact blockers to the installation catalog;
compilation-only successes do not create tested approvals.

See [the compatibility priorities](compatibility/packages/priorities.md) for the
most common first parser failures and candidates for focused behavior tests.
The [first language-patch rerun](compatibility/packages/language-patch-comparison.md)
increased source-bearing compilation-complete candidates from **12 to 20**.
The [dotted-import rerun](compatibility/packages/dotted-import-comparison.md)
raises that to **24**: all **170** releases that first hit dotted imports
progress, with four completing compilation and 166 reaching later blockers.
These remain unverified until dependency and behavior tests pass.

### Useful, tested recipes

Start with complete tools for [CSV summaries](examples/recipes/csv_summary.py),
[JSON APIs](examples/recipes/api_client.py),
[repository inspection](examples/recipes/repository_summary.py), and
[Markdown reports](examples/recipes/generate_report.py). The
[recipe guide](https://kipferl.dev/docs/guides/recipes) includes their exact
sources and example commands. CI checks snippet drift and runs them against
local fixtures, including a local HTTP server.

---

## Example: Nice CLI

**app.py**
```python
import tui
import input
import subprocess

tui.box("Deploying build...", title="Release", border="rounded")
result = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True)
commit = result["stdout"].decode().strip()
tui.success(f"Built commit {commit}")

features = input.multiselect("Select features:", ["Logging", "HTTP", "Config"])
if input.confirm("Deploy now?", default=True):
    tui.progress(68, 100, label="Uploading")
    tui.success(f"Deployed with {len(features)} features")
else:
    tui.warning("Canceled")
```

**Output**
```
╭─ Release ─────────────────────────────╮
│ Deploying build...                    │
╰───────────────────────────────────────╯
✓ Built commit a1b2c3d

? Select features:  ◉ Logging  ◉ HTTP  ○ Config
? Deploy now? (Y/n) y
Uploading  [███████████░░░░░░] 68%  3.2s
✓ Deployed with 2 features
```

---

## Why Kipferl

- Python ergonomics with Go-style shipping
- Compact binaries, fast startup
- Rich TUI components out of the box
- No runtime dependency chain
- Honest, curated stdlib compatibility

### From μcharm to Kipferl

Kipferl is the new name for μcharm beginning with the Rust-based 0.6 release.
The pastry name is a small nod to Bun and a literal fit for the product: Kipferl
*bakes* a Python-style CLI into one portable executable. The repository, binary,
packages, and documentation now share the same spelling, with
[kipferl.dev](https://kipferl.dev) as the public home.

The temporary `ucharm` migration aliases introduced in 0.6 end in 0.7.1.
Use the `kipferl` command and imports, `KIPFERL_*` environment variables, and
`kipferl-*` release downloads. See the [upgrade guide](https://kipferl.dev/docs/guides/packages#upgrade-to-071)
for the complete migration. The Kipferl v1 universal format keeps its original
wire bytes, so existing standalone binaries and caches remain compatible.

## Comparison

| | Python + Rich | Go TUI stack | Rust + Ratatui | **Kipferl** |
|---|:---:|:---:|:---:|:---:|
| **Startup time** | App-dependent | App-dependent | App-dependent | **7.679 ms v0.6 core baseline** |
| **Distribution size** | Interpreter + app | App-dependent | App-dependent | **1.451 MB v0.6 minimal app** |
| **Easy to write** | Yes | Medium | Hard | **Yes** |
| **Beautiful TUI** | Yes | Yes | Yes | **Yes** |

Kipferl measurements here are from the dated Apple Silicon v0.6 core baseline;
application code, bundled assets, and target change size and startup time.

---

## Features

### TUI Components

```python
import tui

print(tui.style("Bold cyan", fg="cyan", bold=True))
tui.box("Important notice", title="Notice")
tui.table([
    ["Name", "Role"],
    ["Alice", "Engineer"],
    ["Bob", "Designer"],
], headers=True)
tui.progress(50, 100, label="Downloading")
```

### Prompts

```python
import input

choice = input.select("Pick one:", ["Build", "Test", "Deploy"])
name = input.prompt("Project name:", default="my-app")
if input.confirm("Continue?", default=True):
    print("Running...")
```

### System Integration

```python
import subprocess

result = subprocess.run(["echo", "Fast!"], capture_output=True)
print(result["stdout"].decode().strip())
```

### HTTP and HTTPS

```python
import http.client as http

connection = http.HTTPSConnection("example.com")
connection.request("GET", "/")
response = connection.getresponse()
print(response.status, len(response.read()))
```

### Configuration and data files

Kipferl ships file and string APIs for the formats CLI applications commonly
need. JSON, YAML, and TOML use ordinary Python dictionaries, lists, and scalar
values. KDL uses an explicit node/entry representation so positional values,
properties, children, and type annotations survive a round trip.

| Format | Module | Read | Write | Notes |
|---|---|---|---|---|
| JSON | `json` | `load`, `loads` | `dump`, `dumps` | JSON-compatible values |
| YAML 1.2 | `yaml` | `load`, `loads`, `safe_load` | `dump`, `dumps`, `safe_dump` | JSON-compatible values; native Rust parser |
| TOML | `tomllib`, `toml` | `load`, `loads` | `toml.dump`, `toml.dumps` | `tomllib` remains read-only like CPython |
| KDL 2.0 | `kdl` | `load`, `loads` | `dump`, `dumps` | Ordered node/entry model; native Rust parser |
| XML | `xml.etree.ElementTree` | `parse`, `fromstring` | `ElementTree.write`, `tostring` | Focused ElementTree subset |
| CSV | `csv` | `reader`, `DictReader` | `writer`, `DictWriter` | Focused CPython-compatible subset |
| INI / CFG | `configparser` | `read`, `read_string` | `write` | Focused `ConfigParser` subset |

```python
import kdl
import yaml

settings = yaml.load("settings.yaml")
settings["debug"] = True
yaml.dump(settings, "settings.yaml")

document = kdl.loads('package "kipferl" version=6')
document[0]["entries"].append(kdl.property("stable", True))
kdl.dump(document, "package.kdl")
```

See the [data-format guide](https://kipferl.dev/docs/modules/data-formats) for
the KDL data model, file-object usage, and current compatibility boundaries.

---

## Standard Library Support

Kipferl targets a CLI-focused subset of CPython. See `tests/compat_report_pocketpy.md` for current compatibility and gaps.

**Essential for CLI apps:**
argparse, os, sys, time, pathlib, glob, fnmatch, subprocess, signal, json, csv,
yaml, kdl, logging, datetime, textwrap, tempfile, shutil, re, hashlib.

**Good to have:**
configparser, enum, uuid, urllib.parse, contextlib, typing, statistics,
functools, itertools, heapq.

**Nice to have:**
http.client (HTTP + HTTPS), secrets, hmac, dataclasses,
xml.etree (fromstring + basic iteration), sqlite3 (basic DB-API subset),
gzip (read), zipfile (read-only), tarfile (read-only).

---

## Installation

### Stable v0.7.2: Homebrew (macOS/Linux)

```bash
brew install niklas-heer/tap/kipferl
```

For an existing installation, run `brew update && brew upgrade kipferl` and
verify `kipferl --version` reports `v0.7.2`. If you added an RC directory to your
`PATH`, remove that entry so it does not shadow Homebrew. Read the
[upgrade guide](https://kipferl.dev/docs/guides/packages#upgrade-to-072)
for dependency-lock changes. Users with the old μcharm 0.5 formula should replace
it once:

```bash
brew uninstall --force ucharm
brew install niklas-heer/tap/kipferl
```

The deprecated `ucharm` command alias ended in 0.7.1. Update scripts to invoke
`kipferl`. See the [0.6 release story](https://kipferl.dev/blog/kipferl-0-6) for
the final artifact sizes, migration outcome, and verified release evidence.

### v0.7.2: direct download

Download the stable binary explicitly from its
[GitHub release](https://github.com/niklas-heer/kipferl/releases/tag/v0.7.2).
Choose the release asset for the machine running Kipferl:

| Platform | Asset |
| --- | --- |
| macOS Apple Silicon | `kipferl-macos-aarch64` |
| macOS Intel | `kipferl-macos-x86_64` |
| Linux x86_64 | `kipferl-linux-x86_64` |
| Linux ARM64 | `kipferl-linux-aarch64` |

Download both files **with their original names** so checksum verification
matches the filename inside the `.sha256` file. For example, on Apple Silicon:

```bash
asset=kipferl-macos-aarch64
release=https://github.com/niklas-heer/kipferl/releases/download/v0.7.2
release_dir=$(mktemp -d)
cd "$release_dir"
curl -fLO "$release/$asset"
curl -fLO "$release/$asset.sha256"
shasum -a 256 -c "$asset.sha256"
# On Linux, use: sha256sum -c "$asset.sha256"
mkdir -p "$HOME/.local/bin"
install -m 755 "$asset" "$HOME/.local/bin/kipferl"
export PATH="$HOME/.local/bin:$PATH"
kipferl --version
```

Choose the correct `asset` before running the commands. Verify the checksum
successfully before installing the binary. Add `~/.local/bin` to your shell's
`PATH` permanently if needed, and remove an earlier RC directory that would
otherwise select the old binary.

---

## Build Modes

| Mode | v0.6 reference size | Dependencies | Use case |
|------|------|--------------|----------|
| `universal` | ~1.4–5.9 MB | None | Tree-shaken production deployment |
| `executable` | ~3KB | pocketpy-kipferl | Dev machines with runtime |
| `single` | ~2KB | pocketpy-kipferl | Scripting |

```bash
# Fully standalone binary (recommended)
kipferl build app.py -o app --mode universal

# Cross-compile for another platform (downloads the target components once, with SHA-256 verification)
kipferl build app.py -o app-linux --target linux-x86_64

# Keep every optional capability when imports cannot be analyzed statically
kipferl build app.py -o app --full-runtime

# Shell wrapper (needs pocketpy-kipferl installed)
kipferl build app.py -o app --mode executable

# Just transform the Python file
kipferl build app.py -o app.py --mode single
```

The sizes above are historical v0.6 measurements; 0.7 components and bundled
package resources change the result. Check the release assets and measure your
own application rather than treating them as 0.7 size guarantees.

Universal builds inspect imports and choose the smallest prebuilt Rust runtime
that is safe for the application. JSON, CSV, XML, INI, filesystem, subprocess,
and presentation APIs fit in the core profile. Imports such as `sqlite3`,
`http.client`, `input`, YAML/TOML/KDL, regex, crypto, and archives select the
complete profile. Dynamic or relative imports also choose the complete profile
conservatively; no Rust compiler or linker is required on the user's machine.
See the [build reference](https://kipferl.dev/docs/commands/build) for the exact
rules and diagnostics.

---

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/installing-mise.html) 2026.9.1 or newer
- Git and a native C compiler: Xcode Command Line Tools on macOS, or
  `build-essential` on Debian/Ubuntu

`mise.toml` pins Rust (with Clippy, rustfmt, Rust Analyzer, and standard-library
sources), Python, Node.js, Bun, Bacon, nextest, and watchexec.
`mise.lock` records downloadable tool artifacts for macOS/Linux ARM64 and x86_64.
Cargo uses `Cargo.lock`; the website uses `website/bun.lock`.

### Quick Start

```bash
git clone https://github.com/niklas-heer/kipferl
cd kipferl

mise trust
mise install --locked rust python node bun cargo:bacon aqua:nextest-rs/nextest/cargo-nextest watchexec
mise run setup
mise run demo
mise run test
```

`mise run setup` checks the pinned tools and C compiler, fetches locked Cargo
packages, installs the website with `bun install --frozen-lockfile`, and builds
the release workspace. It is safe to rerun. `mise tasks` lists all commands;
`mise run website-dev` starts the documentation site. Shell activation is optional
when using `mise run` or `mise exec -- <command>`.

For deliberate tool upgrades, edit the exact pins in `mise.toml`, keep Rust
aligned with `rust-toolchain.toml` and Bun with `website/package.json`, then run
`mise lock --platform macos-arm64,macos-x64,linux-arm64,linux-x64`.
Review the lockfile changes and run `mise run test`. OS SDKs/linkers remain host
prerequisites; this setup pins project tools and dependencies, not the entire OS.
VHS/video tools and bindgen/libclang are optional prerequisites for recording
demos and regenerating FFI. Bacon builds from its pinned source release and
upstream Cargo.lock; the other developer tools use locked platform downloads.

The hand-authored `stubs/*.pyi` files are canonical. After adding or removing a
stub, run `mise run stubs`; `mise run check` and CI verify Python syntax and ensure the
CLI's generated include manifest has not drifted.

### Focused checks

`mise run check` runs the development/release tooling tests, stub verification,
Rust formatting, all-target/all-feature compilation and strict Clippy, and both
full and core runtime tests through nextest. `mise run test-doc` runs public API
examples through Cargo because nextest does not execute doctests.
`mise run test` also builds release binaries and runs compatibility, vision, and
documented recipe checks, including standalone executables without their sources,
and type-checks and builds the website. Run `mise run lint-audit` as well to
verify the exception inventory and require zero outstanding review findings.

See the [development guide](docs/development.md) for setup troubleshooting,
full/core feedback loops, and release-asset boundaries, and the
[benchmark guide](benchmarks/README.md) for reproducible performance work.

For Rust editing, enable your editor's Rust Analyzer integration. The pinned
toolchain includes the server and `rust-src` for standard-library navigation;
run the editor within `mise exec -- <editor> .` if it does not inherit the mise
environment. `mise exec -- rust-analyzer --version` verifies the server path.

Use the feedback tools to investigate errors and regressions:

```bash
mise run bacon       # c: strict Clippy; t: tests; r: core Clippy; d: doctests
mise run watch       # sequential check, test, build after edits; never runs the app
mise run test-ci     # full/core nextest suites with failure output and JUnit reports
mise run lint-audit  # require zero findings; inventory reviewed exceptions
mise run bench       # statistical loader/cache benchmarks with HTML reports
mise run seek        # optional interactive crate, feature, and MSRV inspection
```

Nextest retries are disabled, slow tests have deadlines, and leaked child
processes that hold output pipes open fail the run. CI uploads full/core JUnit
reports under `target/nextest/` and a separate restriction audit under
`target/lint-audit/`. Bacon exports navigable diagnostics to
`target/bacon-locations`; Criterion reports live under `target/criterion/`.

The workspace enforces every restriction in the Rust review guidance, including
pedantic and nursery checks. `mise run lint-audit` requires zero outstanding
findings across full/core profiles and records every explicit exception and its
reason in `target/lint-audit/exceptions.json` and the Markdown report. Blanket
Clippy group exceptions and missing reasons fail the audit. Test prototyping
settings remain confined to tests. See [the Rust review record](docs/rust-review.md)
for confirmed fixes and the assessment of every proposed tool and crate.

For a runtime change, iterate on its Rust integration test and compare the
matching Python fixture with the freshly built runtime:

```bash
cargo test -p kipferl-runtime --test crypto_compression_wave
cargo build --release -p kipferl-runtime
python3 tests/compat_runner.py --runtime target/release/pocketpy-kipferl --module io --verbose
```

The CLI embeds prebuilt runtime and loader assets from
`crates/kipferl-cli/assets/`; changing Rust runtime source alone does not refresh
those assets. Use the freshly built `pocketpy-kipferl` binary to test runtime
changes. CI and release builds refresh the embedded components for each target.

The executable recipe check also keeps the published examples honest:

```bash
python3 scripts/check_recipes.py --runtime target/release/pocketpy-kipferl --cli target/release/kipferl
```

This checks exact documentation snippets, runs real file and local HTTP
fixtures, and executes each standalone binary after deleting its build sources.
Use `--docs-only` for a quick snippet check.

The compatibility runner fails on runtime or CPython baseline failures, crashed
processes, and missing fixtures. `--ci` is an explicit report-only mode; omit it when gating changes.
The vision suite reports timeouts as failures and continues collecting results.
Use `--runs 1 --warmup 0` for a quick functional smoke test; use the default
sample count for startup measurements.

### Project Structure

```
kipferl/
├── crates/        # Rust CLI, runtime, loader, format, and PocketPy FFI
├── pocketpy/      # Vendored PocketPy sources and tracked patches
├── tests/         # Test suite
├── examples/      # Example apps
├── benchmarks/    # Reproducible migration and performance evidence
└── assets/        # Branding
```

---

## Compatibility and Limitations

- Kipferl is not a drop-in replacement for CPython.
- Pure-Python PyPI wheels can be installed with `kipferl add`; pip environments,
  CPython extensions, and source builds are unsupported.
- Package compatibility is recorded per artifact and runtime. Passing selected
  tests does not guarantee every API or execution path.
- See `tests/compat_report_pocketpy.md` for current parity.

## Rust architecture and status

The shipping implementation is Rust around an embedded PocketPy C runtime:

- `kipferl-cli` owns project generation, execution, and cross-target builds.
- `kipferl-runtime` provides the Python VM host and curated native modules.
- `kipferl-loader` and `kipferl-format` own the standalone application format.
- `pocketpy-sys` is the narrow, audited C FFI boundary.

The migration was compatibility-gated rather than rewritten behind a flag day.
Read [the migration plan](RUST_MIGRATION.md), the
[optimization record](benchmarks/rust_optimization_baseline.md), and the
[public retrospective](https://kipferl.dev/blog/rust-migration) for the why,
the incremental process, accepted tradeoffs, and final measurements.

Historical v0.7.0 validation on 2026-09-05 (see
[the generated compatibility report](tests/compat_report_pocketpy.md) and the
[published stable evidence](https://github.com/niklas-heer/kipferl/releases/tag/v0.7.0)):

- Rust runtime: 1,725/1,725 available checks passing (CPython 3.12.14 host)
- 52 compatibility groups: 51 with a passing baseline and no partial groups;
  the third-party `toml` baseline is unavailable on the pinned host
- 51 of the 160 modules in the standard-library inventory are targeted;
  22 dependency-related checks are explicitly skipped
- 312 full-profile and 125 core Rust tests, including 13 CPython-oracle
  dotted-import tests; 131 Python tooling tests and a clean strict audit
- All four stable release platforms passed package installation, offline restoration,
  and detached standalone smoke checks

The stable release repeated artifact checks and generated fresh per-platform
evidence. Its results identify the actual binaries; candidate hashes were not
reused for rebuilt stable components.

Historical performance measurements from the migration and v0.6.0 release:

- 7.044ms median / 7.980ms p95 native ARM64 startup and a 4,000,864-byte
  stripped runtime in the final 1,200-run migration sample

The v0.6.0 release added profile-based tree shaking without a user toolchain. A
minimal Apple Silicon app is 1,450,837 bytes with the 1,130,352-byte core
runtime, 69.9% smaller than the same app with the full runtime. Its measured
median startup is 7.679ms. See the
[reproducible baseline](benchmarks/tree_shaking_baseline.md) and
[technical deep dive](https://kipferl.dev/blog/tree-shaken-builds).

CI treats compatibility as a regression gate and uploads the full report. Each
tagged release also publishes that report plus a machine-readable verification
that the declared PocketPy patch series reproduces the vendored source from the
pristine upstream release.

## Showcase

Built something with Kipferl? Open a PR to add it here.

- (your app)

---

## FAQ

<details>
<summary>Where does the name come from?</summary>

A Kipferl is a crescent-shaped pastry. The name gives a playful nod to Bun and
fits what the tool does: it bakes a Python-style CLI into a compact standalone
binary. Before 0.6, the project was called μcharm.
</details>

<details>
<summary>Why is it so fast?</summary>

The v0.6 tree-shaken ARM64 baseline measured a 7.679ms median. Fast startup comes from:

1. No external CPython startup or virtual-environment activation
2. Built-in native modules avoid filesystem discovery
3. Minimal runtime (PocketPy is much smaller than CPython)
4. Native Rust modules (TUI components are compiled, not imported packages)
</details>

<details>
<summary>Why is the binary so small?</summary>

In the v0.6 baseline, the core ARM64 runtime was 1.13MB; full runtimes reached
5.45MB across the four release targets. Those are historical measurements,
not 0.7 size promises. The runtime stays compact because:

1. PocketPy core is small
2. The Rust release profile uses `-O2`, fat LTO, one codegen unit, overflow
   checks, symbol stripping, and aborting panics
3. Profile-based tree shaking removes unused optional dependency trees while
   keeping useful extras like `sqlite3` available
4. SQLite is statically bundled with unused extensions disabled, and HTTPS uses
   a feature-minimal Rustls/Ureq stack
5. Ratatui powers interactive selection with an inline, scrollback-preserving
   viewport and a single Crossterm backend
</details>

<details>
<summary>Why PocketPy over MicroPython?</summary>

We evaluated both and chose PocketPy for CLI tooling:

| Aspect | PocketPy | MicroPython |
|--------|----------|-------------|
| Target | General Python 3.x | Embedded/IoT |
| C API | Clean, embedding-focused | Complex, hardware-focused |
| Syntax | Python 3 subset with project compatibility tests | Subset of Python 3.4 |
| Rust host integration | Narrow C FFI | More embedded-runtime glue |
| Product fit | CLI-focused embedding | Microcontroller-focused runtime |

MicroPython excels at microcontrollers. PocketPy excels at embedding Python in applications.
</details>

<details>
<summary>What Python features are supported?</summary>

The runtime supports classes, decorators, generator functions, comprehensions, individual f-strings, `*args`/`**kwargs`, and context managers. Version 0.7 also supports dotted imports, trailing commas in import and parameter lists, and adjacent plain string and bytes literals.

Not supported:
- `async`/`await` (limited support)
- Adjacent f-string combinations and generator expressions
- Some metaclass features
- C extension packages (numpy, etc.)

See `tests/compat_report_pocketpy.md` for detailed module compatibility.
</details>

---

## Docs

- [Website documentation](https://kipferl.dev/docs)
- [Product direction](vision.md)
- [Implementation priorities](PLAN.md)
- [Rust migration record](RUST_MIGRATION.md)
- [Launch plan](LAUNCH.md)

---

## Contributing

Contributions are welcome. Areas that help the most:

- CLI ergonomics (subcommands, completions)
- Config and HTTP modules
- Unicode width correctness
- Docs and examples

---

## License

MIT License. See `LICENSE` for details.

<p align="center">
  <strong>Kipferl</strong> — Python CLIs, native speed
</p>
