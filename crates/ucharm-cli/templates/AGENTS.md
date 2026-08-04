# ucharm Project

This project uses **ucharm** - a CLI toolkit for building beautiful command-line applications with PocketPy.

## Critical Context

- **Runtime**: PocketPy with native Rust modules (NOT CPython)
- **No pip packages**: Cannot use packages with C extensions (no requests, numpy, pandas)
- **Output**: Standalone binaries (~1MB)
- **50+ runtime modules** including: ansi, args, argparse, base64, tui, collections, copy, csv, dataclasses, datetime, fetch, fnmatch, functools, glob, gzip, hashlib, heapq, hmac, http.client, input, itertools, json, logging, math, operator, os, pathlib, random, re, secrets, shutil, signal, sqlite3, statistics, struct, subprocess, tarfile, tempfile, template, term, textwrap, time, toml, typing, unittest, urllib.parse, uuid, xml.etree.ElementTree, zipfile

## Import Pattern

```python
# Use 'from ucharm import' for TUI components
from ucharm import box, success, error, select, confirm, prompt

# Or import native modules directly
import tui
import input
```

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
