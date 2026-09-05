# kipferl Project

This project uses **kipferl** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Key Concepts

- **PocketPy, not CPython**: This runs on PocketPy with native Rust modules, not standard Python
- **Native modules**: 50+ high-performance modules implemented in Rust (see list below)
- **Dependencies**: Use native modules, compatible local Python files, or pure-Python wheels installed with `kipferl add`. Check `kipferl deps catalog` for tested versions and blockers. Commit `kipferl.json` and `kipferl.lock`; restore with `kipferl sync --locked`. CPython extensions and source builds are unsupported
- **Single binary output**: Apps build into target-specific standalone executables; size depends on runtime profile and bundled assets

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
- `heapq`, `hmac`, `io`, `itertools`, `json`, `kdl`, `logging`, `math`
- `operator`, `os`, `pathlib`, `random`, `re`, `secrets`, `shutil`
- `signal`, `sqlite3`, `statistics`, `struct`, `subprocess`, `sys`
- `tarfile`, `tempfile`, `textwrap`, `time`, `toml`, `tomllib`, `typing`
- `unittest`, `urllib.parse`, `uuid`, `xml.etree.ElementTree`, `yaml`, `zipfile`

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

# Keep every optional capability when imports are dynamic
kipferl build myapp.py -o myapp --full-runtime
```

## Project Workflow

For projects containing `kipferl.json`, use `kipferl run`, `kipferl dev`,
`kipferl test`, and `kipferl build`; explicit script paths override the entry.
Keep local modules in the project. Include resources with `--asset <path>` or
configuration assets, then resolve them relative to `__file__`. Test the built
executable away from the source directory before sharing it.

## Runtime Compatibility

Use the installed Kipferl runtime to verify behavior; the standard-library
surface is curated and editor stubs are not a CPython compatibility guarantee.
In runtimes built from the current source:

- `subprocess.run` returns a dictionary. Captured output retains the first 1 MiB
  per stream while draining the rest; uncaptured streams are discarded.
- HTTP timeouts must be finite, nonnegative, and fit the platform clock, or
  `None`. Invalid timeouts raise `ValueError` when making the request.
- `bytearray(n)` zero-filled allocation is limited to 64 MiB. `islice` requires
  nonnegative start/stop and a positive step; it returns a list.
- Comparison and predicate callbacks should leave input collection lengths
  unchanged; detected mutation raises `RuntimeError`. `deepcopy` snapshots
  container entries before invoking custom hooks.
- Numeric range failures in `math.ldexp` and f32 `struct.pack` use `ValueError`.
  Do not assume CPython's `OverflowError` is available.

Runtime fixes require a matching runtime build; rebuilding the CLI alone does
not update its checked-in embedded runtime assets. See the repository README
for source-development and release instructions.

## What NOT to Use

- No `requests`, `httpx`, `aiohttp` (use `http.client` instead)
- No `numpy`, `pandas` (pure Python alternatives only)
- No async/await (PocketPy has limited async support)
- No type annotations at runtime (use for IDE only via stubs)
