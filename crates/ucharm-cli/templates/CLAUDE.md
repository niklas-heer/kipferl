# ucharm Project

This project uses **ucharm** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Key Concepts

- **PocketPy, not CPython**: This runs on PocketPy with native Rust modules, not standard Python
- **Native modules**: 50+ high-performance modules implemented in Rust (see list below)
- **No pip packages**: You cannot use pip packages that have C extensions
- **Single binary output**: Apps compile to standalone executables (~1MB)

## Available Modules

### TUI Components
- `tui` - Box, table, rule, progress bar, spinner, status messages (success/error/warning/info)
- `input` - Interactive prompts: select, multiselect, confirm, prompt, password
- `term` - Terminal control (size, raw mode, cursor, colors)
- `ansi` - ANSI escape codes for styling
- `template` - Jinja-like templating (variables, conditionals, loops)

### Networking
- `fetch` - HTTP/HTTPS client (get, post, request) with built-in TLS
- `http.client` - Low-level HTTP client

### Standard Library (Native)
- `args` - CLI argument parsing
- `argparse`, `array`, `base64`, `binascii`, `bisect`, `collections`
- `configparser`, `contextlib`, `copy`, `csv`, `dataclasses`, `datetime`
- `enum`, `errno`, `fnmatch`, `functools`, `glob`, `gzip`, `hashlib`
- `heapq`, `hmac`, `io`, `itertools`, `json`, `logging`, `math`
- `operator`, `os`, `pathlib`, `random`, `re`, `secrets`, `shutil`
- `signal`, `sqlite3`, `statistics`, `struct`, `subprocess`, `sys`
- `tarfile`, `tempfile`, `textwrap`, `time`, `toml`, `tomllib`, `typing`
- `unittest`, `urllib.parse`, `uuid`, `xml.etree.ElementTree`, `zipfile`

## Import Pattern

```python
# Use 'from ucharm import' syntax - it gets transformed automatically
from ucharm import box, success, select, confirm

# Or import native modules directly
import tui
import input
```

## Example Usage

```python
from ucharm import box, success, select

box("Welcome!", title="My App", border_color="cyan")
choice = select("Pick one:", ["Option A", "Option B", "Exit"])
success(f"You chose: {choice}")
```

## Running & Building

```bash
# Run script
ucharm run myapp.py

# Build standalone binary
ucharm build myapp.py -o myapp --mode universal
```

## What NOT to Use

- No `requests`, `httpx`, `aiohttp` (use `fetch` module instead)
- No `numpy`, `pandas` (pure Python alternatives only)
- No async/await (PocketPy has limited async support)
- No type annotations at runtime (use for IDE only via stubs)
