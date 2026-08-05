# kipferl Project

This project uses **kipferl** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Key Concepts

- **PocketPy, not CPython**: This runs on PocketPy with native Rust modules, not standard Python
- **Native modules**: 50+ high-performance modules implemented in Rust (see list below)
- **No pip packages**: You cannot use pip packages that have C extensions
- **Single binary output**: Apps compile to target-specific standalone executables (about 4.3–5.3 MB)

## Available Modules

### TUI Components
- `tui` - Box, table, rule, progress bar, spinner, status messages (success/error/warning/info)
- `input` - Interactive prompts: select, multiselect, confirm, prompt, password
- `term` - Terminal control (size, raw mode, cursor, colors)
- `ansi` - ANSI escape codes for styling

### Networking
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
# Import native modules directly in new code
import tui
import input
```

The CLI still transforms legacy `from kipferl import ...` source for
compatibility.

## Example Usage

```python
import tui
import input

tui.box("Welcome!", title="My App", border_color="cyan")
choice = input.select("Pick one:", ["Option A", "Option B", "Exit"])
tui.success(f"You chose: {choice}")
```

## Running & Building

```bash
# Run script
kipferl run myapp.py

# Build standalone binary
kipferl build myapp.py -o myapp --mode universal
```

## What NOT to Use

- No `requests`, `httpx`, `aiohttp` (use `http.client` instead)
- No `numpy`, `pandas` (pure Python alternatives only)
- No async/await (PocketPy has limited async support)
- No type annotations at runtime (use for IDE only via stubs)
