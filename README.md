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

- Standalone, target-specific binaries (about 4.8–5.8 MB for a minimal app)
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
that write files in bursts. Run `kipferl dev --help` for the complete interface.

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
| **Binary size** | 80MB+ | 2-3MB | 2-5MB | **~5MB** |
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

### Rust release candidate (recommended for testing)

The renamed Rust release is available as `v0.6.0-rc.2`. Pick the CLI matching your
machine; each download is a standalone executable that embeds its matching
runtime.

```bash
# macOS (Apple Silicon)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0-rc.2/kipferl-macos-aarch64 -o kipferl
chmod +x kipferl

# macOS (Intel)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0-rc.2/kipferl-macos-x86_64 -o kipferl
chmod +x kipferl

# Linux (x86_64, static musl)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0-rc.2/kipferl-linux-x86_64 -o kipferl
chmod +x kipferl

# Linux (ARM64, static musl)
curl -L https://github.com/niklas-heer/kipferl/releases/download/v0.6.0-rc.2/kipferl-linux-aarch64 -o kipferl
chmod +x kipferl
```

Move the downloaded file somewhere on your `PATH`, then confirm it with
`kipferl --version`.

### Homebrew stable (macOS/Linux)

The previous μcharm `v0.5` formula remains available during the RC. After
Kipferl `v0.6` is promoted to stable, install the renamed formula with:

```bash
brew install niklas-heer/tap/kipferl
```

Until then, existing users can continue using `brew install
ucharmdev/tap/ucharm`. Kipferl 0.6 keeps `ucharm` as a deprecated command alias
for one release cycle.

---

## Build Modes

| Mode | Size | Dependencies | Use case |
|------|------|--------------|----------|
| `universal` | ~4.8-5.8MB | None | Production deployment |
| `executable` | ~3KB | pocketpy-kipferl | Dev machines with runtime |
| `single` | ~2KB | pocketpy-kipferl | Scripting |

```bash
# Fully standalone binary (recommended)
kipferl build app.py -o app --mode universal

# Cross-compile for another platform (downloads the target components once, with SHA-256 verification)
kipferl build app.py -o app-linux --target linux-x86_64

# Shell wrapper (needs pocketpy-kipferl installed)
kipferl build app.py -o app --mode executable

# Just transform the Python file
kipferl build app.py -o app.py --mode single
```

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

The current native ARM64 median is 7.044ms. Fast startup comes from:

1. No external CPython startup or virtual-environment activation
2. No import machinery (modules compiled into the binary)
3. Minimal runtime (PocketPy is much smaller than CPython)
4. Native Rust modules (TUI components are compiled, not imported packages)
</details>

<details>
<summary>Why is the binary so small?</summary>

About 4MB for the optimized ARM64 runtime (SQLite, HTTPS, and Ratatui enabled)
because:

1. PocketPy core is small
2. The Rust release profile uses `-O2`, fat LTO, one codegen unit, overflow
   checks, symbol stripping, and aborting panics
3. Curated stdlib surface (no bloat) while still bundling useful extras like `sqlite3`
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
