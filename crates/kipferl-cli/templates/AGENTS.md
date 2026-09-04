# kipferl Project

This project uses **kipferl** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Critical Context

- **Runtime**: PocketPy with native Rust modules (NOT CPython)
- **Dependencies**: Use native modules or compatible local Python files; pip packages and CPython extensions are not installed into the runtime
- **Output**: Target-specific standalone binaries; size depends on runtime profile and bundled assets
- **50+ runtime modules** including: ansi, args, argparse, base64, tui, collections, configparser, copy, csv, dataclasses, datetime, fnmatch, functools, glob, gzip, hashlib, heapq, hmac, http.client, input, itertools, json, kdl, logging, math, operator, os, pathlib, random, re, secrets, shutil, signal, sqlite3, statistics, struct, subprocess, tarfile, tempfile, term, textwrap, time, toml, tomllib, typing, unittest, urllib.parse, uuid, xml.etree.ElementTree, yaml, zipfile

## Import Pattern

```python
# Import the native modules directly in new code
import tui
import input

tui.box("Ready", title="Status")
choice = input.select("Next step:", ["Build", "Test", "Exit"])
```

The CLI still transforms legacy `from kipferl import ...` source for
compatibility, but generated projects use the explicit modules.

## Available TUI Functions

- `box(content, title=None, border="rounded", border_color=None)` - Draw a box
- `table(rows, headers=False, border="square", border_color=None)` - Display formatted table
- `rule(title=None, color=None, width=80)` - Horizontal divider
- `progress(current, total, label=None, width=40, elapsed=None)` - Progress bar
- `spinner(frame, message=None, color=None)` - Animated spinner
- `progress_done()` - Complete progress/spinner line
- `success(msg)`, `error(msg)`, `warning(msg)`, `info(msg)` - Status messages
- `select(prompt, choices)` -> str - Interactive selection
- `multiselect(prompt, choices)` -> list - Multiple selection
- `confirm(prompt, default=True)` -> bool - Yes/no prompt
- `prompt(message, default=None)` -> str - Text input
- `password(message)` -> str - Hidden input

## Running & Building

```bash
kipferl run myapp.py           # Run script
kipferl build myapp.py -o app  # Build standalone binary
kipferl build myapp.py -o app --full-runtime  # Disable tree shaking when imports are dynamic
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

## Do NOT Suggest

- requests, httpx, aiohttp (use `http.client` instead)
- numpy, pandas, scipy (pure Python alternatives only)
- async/await patterns (limited PocketPy support)
- Runtime type checking (use stubs for IDE only)
