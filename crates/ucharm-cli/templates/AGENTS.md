# ucharm Project

This project uses **ucharm** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Critical Context

- **Runtime**: PocketPy with native Rust modules (NOT CPython)
- **No pip packages**: Cannot use packages with C extensions (no requests, numpy, pandas)
- **Output**: Target-specific standalone binaries (about 4.3–5.3 MB)
- **50+ runtime modules** including: ansi, args, argparse, base64, tui, collections, copy, csv, dataclasses, datetime, fetch, fnmatch, functools, glob, gzip, hashlib, heapq, hmac, http.client, input, itertools, json, logging, math, operator, os, pathlib, random, re, secrets, shutil, signal, sqlite3, statistics, struct, subprocess, tarfile, tempfile, template, term, textwrap, time, toml, typing, unittest, urllib.parse, uuid, xml.etree.ElementTree, zipfile

## Import Pattern

```python
# Import the native modules directly in new code
import tui
import input

tui.box("Ready", title="Status")
choice = input.select("Next step:", ["Build", "Test", "Exit"])
```

The CLI still transforms legacy `from ucharm import ...` source for
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
ucharm run myapp.py           # Run script
ucharm build myapp.py -o app  # Build standalone binary
```

## Do NOT Suggest

- requests, httpx, aiohttp (use `fetch` module instead)
- numpy, pandas, scipy (pure Python alternatives only)
- async/await patterns (limited PocketPy support)
- Runtime type checking (use stubs for IDE only)
