# kipferl Project

This project uses **kipferl** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Critical Context

- **Runtime**: PocketPy with native Rust modules (NOT CPython)
- **No pip packages**: Cannot use packages with C extensions (no requests, numpy, pandas)
- **Output**: Tree-shaken, target-specific standalone binaries (about 1.4–5.9 MB)
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

## Do NOT Suggest

- requests, httpx, aiohttp (use `http.client` instead)
- numpy, pandas, scipy (pure Python alternatives only)
- async/await patterns (limited PocketPy support)
- Runtime type checking (use stubs for IDE only)
