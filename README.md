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

- Tiny, portable binaries (target < 2MB for typical CLIs)
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
import charm
import input
import subprocess

charm.box("Deploying build...", title="Release", border="rounded")
result = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True)
commit = result["stdout"].decode().strip()
charm.success(f"Built commit {commit}")

features = input.multiselect("Select features:", ["Logging", "HTTP", "Config"])
if input.confirm("Deploy now?", default=True):
    charm.progress(68, 100, label="Uploading")
    charm.success(f"Deployed with {len(features)} features")
else:
    charm.warning("Canceled")
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

| | Python + Rich | Go + Charm | Rust + Ratatui | **μcharm** |
|---|:---:|:---:|:---:|:---:|
| **Startup time** | 100ms+ | ~10-20ms | ~2-10ms | **~ 3ms** |
| **Binary size** | 80MB+ | 2-3MB | 2-5MB | **< 2MB** |
| **Easy to write** | Yes | Medium | Hard | **Yes** |
| **Beautiful TUI** | Yes | Yes | Yes | **Yes** |

---

## Features

### TUI Components

```python
import charm

print(charm.style("Bold cyan", fg="cyan", bold=True))
charm.box("Important notice", title="Notice")
charm.table([
    ["Name", "Role"],
    ["Alice", "Engineer"],
    ["Bob", "Designer"],
], headers=True)
charm.progress(50, 100, label="Downloading")
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
toml/tomllib, http.client (no TLS), secrets, hmac, dataclasses,
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
```

---

## Build Modes

| Mode | Size | Dependencies | Use case |
|------|------|--------------|----------|
| `universal` | ~0.9-2.0MB | None | Production deployment |
| `executable` | ~3KB | pocketpy-ucharm | Dev machines with runtime |
| `single` | ~2KB | pocketpy-ucharm | Scripting |

```bash
# Fully standalone binary (recommended)
ucharm build app.py -o app --mode universal

# Cross-compile for another platform (downloads a small target runtime once, with sha256 verification)
ucharm build app.py -o app-linux --target linux-x86_64

# Shell wrapper (needs pocketpy-ucharm installed)
ucharm build app.py -o app --mode executable

# Just transform the Python file
ucharm build app.py -o app.py --mode single
```

---

## Development

### Prerequisites

- [Zig](https://ziglang.org/) 0.15+
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
├── cli/           # Zig CLI tool
├── loader/        # Universal binary loader
├── runtime/       # Runtime Zig modules
├── tests/         # Test suite
├── examples/      # Example apps
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

- Rust migration runtime: 1,437/1,668 tests passing (86.2%)
- 52 targeted modules, with 39 at 100% parity on the Rust host
- 4.558ms median native ARM64 startup and a 970,064-byte stripped runtime in
  the latest 400-run migration sample

## Showcase

Built something with μcharm? Open a PR to add it here.

- (your app)

---

## FAQ

<details>
<summary>Where does the name come from?</summary>

μcharm started as “MicroPython + charm-like libraries” → **μcharm** (official name) → **ucharm** (ASCII-friendly).
</details>

<details>
<summary>Why is it so fast?</summary>

~3ms startup vs CPython’s ~15ms comes from:

1. No interpreter overhead (PocketPy embeds into one native binary)
2. No import machinery (modules compiled into the binary)
3. Minimal runtime (PocketPy is much smaller than CPython)
4. Native Zig modules (TUI components are Zig, not Python)
</details>

<details>
<summary>Why is the binary so small?</summary>

~1-2MB universal binaries (sqlite enabled) because:

1. PocketPy core is small
2. Zig modules compile small
3. Curated stdlib surface (no bloat) while still bundling useful extras like `sqlite3`
4. `-Doptimize=ReleaseSmall` strips unused code
</details>

<details>
<summary>Why PocketPy over MicroPython?</summary>

We evaluated both and chose PocketPy for CLI tooling:

| Aspect | PocketPy | MicroPython |
|--------|----------|-------------|
| Target | General Python 3.x | Embedded/IoT |
| C API | Clean, embedding-focused | Complex, hardware-focused |
| Syntax | Full Python 3.x | Subset of Python 3.4 |
| Zig integration | Excellent | More glue |
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
