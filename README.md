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
│   <= 10ms startup. No runtime deps.             │
│                                                 │
╰─────────────────────────────────────────────────╯
```

Kipferl is a focused runtime for beautiful, fast CLI apps. You write Python-style
scripts, and Kipferl ships them as single-file binaries that start instantly.

[Documentation](https://kipferl.dev/docs) ·
[Kipferl 0.6 release story](https://kipferl.dev/blog/kipferl-0-6) ·
[Changelog](CHANGELOG.md) · [Releases](https://github.com/niklas-heer/kipferl/releases)

- Tree-shaken standalone binaries (1.4 MB for a minimal app on Apple Silicon;
  the complete runtime remains available when needed)
- Beautiful TUI output (boxes, tables, prompts, progress)
- Fast startup (<= 10ms on macOS/Linux)
- Curated stdlib compatibility for CLI use cases

---

## Quickstart

```bash
# Develop with automatic restart on every edit
kipferl dev app.py

# Build a standalone binary
kipferl build app.py -o app
./app
```

`kipferl dev` watches Python, config, and template files under the script
directory—including JSON, YAML, TOML, XML, CSV, KDL, and INI/CFG—and keeps
waiting after the program exits. Use `--watch <path>` for additional files or
directories (including other file types),
`--clear` for a clean screen on each restart, or `--debounce <ms>` for tools
that write files in bursts. See the complete [`kipferl dev` reference](https://kipferl.dev/docs/commands/dev)
for every option, default, watched path, and restart behavior.

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

Existing 0.5 users have a gentle transition. The 0.6 release accepts old
`from ucharm ...` imports and environment variables, publishes temporary
`ucharm-*` download aliases, and installs `ucharm` as a deprecated command alias.
The `MCHARM01` application format is deliberately unchanged, so existing
standalone binaries and caches remain compatible.

## Comparison

| | Python + Rich | Go TUI stack | Rust + Ratatui | **Kipferl** |
|---|:---:|:---:|:---:|:---:|
| **Startup time** | 100ms+ | ~10-20ms | ~2-10ms | **~8ms** |
| **Binary size** | 80MB+ | 2-3MB | 2-5MB | **~1.4–5.9MB** |
| **Easy to write** | Yes | Medium | Hard | **Yes** |
| **Beautiful TUI** | Yes | Yes | Yes | **Yes** |

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
http = __import__("http.client")

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

### Homebrew (macOS/Linux)

```bash
brew install niklas-heer/tap/kipferl
```

Existing Kipferl users can update with `brew upgrade kipferl`. Users with the
old μcharm 0.5 formula should replace it once:

```bash
brew uninstall --force ucharm
brew install niklas-heer/tap/kipferl
```

Kipferl 0.6 installs `ucharm` as a deprecated command alias for one release
cycle. See the [0.6 release story](https://kipferl.dev/blog/kipferl-0-6) for
the final artifact sizes, migration outcome, and verified release evidence.

### Direct download

Kipferl `v0.6.0` is available for four native targets. Pick the CLI matching
the machine where Kipferl itself will run:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0/kipferl-macos-aarch64 -o kipferl
chmod +x kipferl

# macOS (Intel)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0/kipferl-macos-x86_64 -o kipferl
chmod +x kipferl

# Linux (x86_64, static musl)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0/kipferl-linux-x86_64 -o kipferl
chmod +x kipferl

# Linux (ARM64, static musl)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0/kipferl-linux-aarch64 -o kipferl
chmod +x kipferl
```

Every binary has an adjacent `.sha256` file in the release. Download both and
verify with `shasum -a 256 -c kipferl-macos-aarch64.sha256` on Apple Silicon
macOS or `sha256sum -c kipferl-linux-x86_64.sha256` on x86_64 Linux, adjusting
the asset name for the selected target. Then move the binary onto your `PATH`
and confirm it with `kipferl --version`.

---

## Build Modes

| Mode | Size | Dependencies | Use case |
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

- [Rust](https://rustup.rs/) using the pinned stable toolchain
- [just](https://github.com/casey/just) (optional, recommended)

### Quick Start

```bash
git clone https://github.com/niklas-heer/kipferl
cd kipferl

just setup
just demo
just test
```

The hand-authored `stubs/*.pyi` files are canonical. After adding or removing a
stub, run `just stubs`; `just check` and CI verify Python syntax and ensure the
CLI's generated include manifest has not drifted.

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
- No pip or C-extension support.
- Pure-Python packages may work if compatible with PocketPy.
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

Current compatibility summary (from `tests/compat_report_pocketpy.md`):

- Rust runtime: 1,669/1,669 available tests passing (100%)
- 52 targeted modules, with 51 at 100% parity, no partial modules, and one
  host-unavailable `toml` baseline
- 7.044ms median / 7.980ms p95 native ARM64 startup and a 4,000,864-byte
  stripped runtime in the final 1,200-run migration sample

The stable build adds profile-based tree shaking without a user toolchain. A
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

The measured tree-shaken ARM64 median is 7.679ms. Fast startup comes from:

1. No external CPython startup or virtual-environment activation
2. No import machinery (modules compiled into the binary)
3. Minimal runtime (PocketPy is much smaller than CPython)
4. Native Rust modules (TUI components are compiled, not imported packages)
</details>

<details>
<summary>Why is the binary so small?</summary>

The core ARM64 runtime is 1.13MB; optional capabilities select a full runtime
up to 5.45MB across the four release targets. It stays compact because:

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
| Syntax | Full Python 3.x | Subset of Python 3.4 |
| Rust host integration | Narrow C FFI | More embedded-runtime glue |
| Product fit | CLI-focused embedding | Microcontroller-focused runtime |

MicroPython excels at microcontrollers. PocketPy excels at embedding Python in applications.
</details>

<details>
<summary>What Python features are supported?</summary>

Most Python 3.x syntax works: classes, decorators, generators, comprehensions, f-strings, `*args`/`**kwargs`, context managers, and more.

Not supported:
- `async`/`await` (limited support)
- Implicit string concatenation (`"a" "b"`)
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
