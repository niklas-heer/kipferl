<p align="center">
  <img src="assets/logo.png" alt="μcharm logo" width="200">
</p>

<h1 align="center">μcharm</h1>

<p align="center">
  <strong>Python CLIs. Tiny binaries. Instant startup.</strong>
</p>

<p align="center">
  <a href="https://github.com/ucharmdev/ucharm/actions/workflows/ci.yml"><img src="https://github.com/ucharmdev/ucharm/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/ucharmdev/ucharm/releases"><img src="https://github.com/ucharmdev/ucharm/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

```
╭─────────────────────────────────────────────────╮
│                                                 │
│   Write Python. Ship a tiny binary.             │
│   <= 10ms startup. No runtime deps.             │
│                                                 │
╰─────────────────────────────────────────────────╯
```

μcharm is a focused runtime for beautiful, fast CLI apps. You write Python-style
scripts, and μcharm ships them as single-file binaries that start instantly.

- Tiny, portable binaries (about 4–4.9 MB; 5 MB release-target ceiling)
- Beautiful TUI output (boxes, tables, prompts, progress)
- Fast startup (<= 10ms on macOS/Linux)
- Curated stdlib compatibility for CLI use cases

---

## Quickstart

```bash
# Run a script
ucharm run app.py

# Build a standalone binary
ucharm build app.py -o app
./app
```

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

## Why μcharm

- Python ergonomics with Go-style shipping
- Tiny binaries, instant startup
- Rich TUI components out of the box
- No runtime dependency chain
- Honest, curated stdlib compatibility

## Comparison

| | Python + Rich | Go TUI stack | Rust + Ratatui | **μcharm** |
|---|:---:|:---:|:---:|:---:|
| **Startup time** | 100ms+ | ~10-20ms | ~2-10ms | **~ 3ms** |
| **Binary size** | 80MB+ | 2-3MB | 2-5MB | **~4MB** |
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

### Templating

```python
import template

src = "{% for p in posts %}- {{p.title}}\\n{% end %}"
print(template.render(src, {"posts": [{"title": "a"}, {"title": "b"}]}))
```

### HTTP (fetch)

```python
import fetch

r = fetch.get("https://example.com/", verify=True)
print(r["status"], len(r["body"]))
```

---

## Standard Library Support

μcharm targets a CLI-focused subset of CPython. See `tests/compat_report_pocketpy.md` for current compatibility and gaps.

**Essential for CLI apps:**
argparse, os, sys, time, pathlib, glob, fnmatch, subprocess, signal, json, csv,
logging, datetime, textwrap, tempfile, shutil, re, hashlib.

**Good to have:**
configparser, enum, uuid, urllib.parse, contextlib, typing, statistics,
functools, itertools, heapq.

**Nice to have:**
toml/tomllib, http.client (HTTP + HTTPS), secrets, hmac, dataclasses,
xml.etree (fromstring + basic iteration), sqlite3 (basic DB-API subset),
gzip (read), zipfile (read-only), tarfile (read-only).

---

## Installation

### Homebrew (macOS/Linux)

```bash
brew install ucharmdev/tap/ucharm
```

### Direct Download

```bash
# macOS (Apple Silicon)
curl -L https://github.com/ucharmdev/ucharm/releases/latest/download/ucharm-macos-aarch64 -o ucharm
chmod +x ucharm

# macOS (Intel)
curl -L https://github.com/ucharmdev/ucharm/releases/latest/download/ucharm-macos-x86_64 -o ucharm
chmod +x ucharm

# Linux
curl -L https://github.com/ucharmdev/ucharm/releases/latest/download/ucharm-linux-x86_64 -o ucharm
chmod +x ucharm

# Linux (ARM64)
curl -L https://github.com/ucharmdev/ucharm/releases/latest/download/ucharm-linux-aarch64 -o ucharm
chmod +x ucharm
```

---

## Build Modes

| Mode | Size | Dependencies | Use case |
|------|------|--------------|----------|
| `universal` | ~2.1-2.8MB | None | Production deployment |
| `executable` | ~3KB | pocketpy-ucharm | Dev machines with runtime |
| `single` | ~2KB | pocketpy-ucharm | Scripting |

```bash
# Fully standalone binary (recommended)
ucharm build app.py -o app --mode universal

# Cross-compile for another platform (downloads the target components once, with SHA-256 verification)
ucharm build app.py -o app-linux --target linux-x86_64

# Shell wrapper (needs pocketpy-ucharm installed)
ucharm build app.py -o app --mode executable

# Just transform the Python file
ucharm build app.py -o app.py --mode single
```

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) using the pinned stable toolchain
- [just](https://github.com/casey/just) (optional, recommended)

### Quick Start

```bash
git clone https://github.com/ucharmdev/ucharm
cd ucharm

just setup
just demo
just test
```

### Project Structure

```
ucharm/
├── crates/        # Rust CLI, runtime, loader, format, and PocketPy FFI
├── pocketpy/      # Vendored PocketPy sources and tracked patches
├── tests/         # Test suite
├── examples/      # Example apps
├── benchmarks/    # Reproducible migration and performance evidence
└── assets/        # Branding
```

---

## Compatibility and Limitations

- μcharm is not a drop-in replacement for CPython.
- No pip or C-extension support.
- Pure-Python packages may work if compatible with PocketPy.
- See `tests/compat_report_pocketpy.md` for current parity.

## Status

Current compatibility summary (from `tests/compat_report_pocketpy.md`):

- Rust runtime: 1,669/1,669 available tests passing (100%)
- 52 targeted modules, with 51 at 100% parity, no partial modules, and one
  host-unavailable `toml` baseline
- 6.986ms median / 8.024ms p95 native ARM64 startup and a 1,840,336-byte
  stripped runtime in the latest 400-run migration sample

## Showcase

Built something with μcharm? Open a PR to add it here.

- (your app)

---

## FAQ

<details>
<summary>Where does the name come from?</summary>

The `μ` signals the project's compact-runtime focus; **ucharm** is the
ASCII-friendly spelling used for commands, packages, and repository paths.
</details>

<details>
<summary>Why is it so fast?</summary>

The current native ARM64 median is 6.986ms. Fast startup comes from:

1. No interpreter overhead (PocketPy embeds into one native binary)
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
| Binary size | ~400KB | ~600KB |
| Startup | ~3ms | ~2ms |

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

- `vision.md` for product direction
- `PLAN.md` for implementation priorities
- `LAUNCH.md` for go-to-market

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
  <strong>μcharm</strong> — Python CLIs, native speed
</p>
