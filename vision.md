# kipferl Vision

This document captures the product vision, the CLI experience we want, and the
minimum module surface needed to make kipferl a great choice for shipping CLI
apps as tiny, fast, standalone binaries.

## Vision

kipferl makes it easy to build great command-line apps with Python syntax and
ship them as single-file binaries that start instantly and work anywhere.

We will not chase full CPython or pip compatibility. The goal is a focused,
curated runtime optimized for CLI applications: beautiful output, solid IO,
predictable packaging, and fast startup.

## What It Should Feel Like

- A Pythonic developer experience with very low friction.
- Tiny binaries with instant startup and no external runtime dependencies.
- Clean, beautiful output by default (tables, progress, prompts).
- Reliable scripting primitives (files, subprocess, paths, env).
- Clear limitations and a predictable compatibility story.

## Example CLIs (what "nice" looks like)

### Simple status + table

```python
import tui
import subprocess

tui.box("Deploying build...", title="Release", border="rounded")
result = subprocess.run(["git", "rev-parse", "--short", "HEAD"], capture_output=True)
commit = result["stdout"].decode().strip()
tui.success(f"Built commit {commit}")

tui.table(
    [
        ["Artifact", "Size", "Time"],
        ["app-linux", "900KB", "6ms"],
        ["app-macos", "910KB", "7ms"],
    ],
    headers=True,
)
```

### Interactive flow

```python
import input
import tui

tui.rule("Project Setup")
name = input.prompt("Project name:")
features = input.multiselect("Select features:", ["Logging", "HTTP", "Config"])
if input.confirm("Create project now?", default=True):
    tui.success(f"Created {name} with {len(features)} features")
else:
    tui.warning("Canceled")
```

### Progress + subprocess

```python
import tui
import subprocess

tui.progress(0, 100, label="Uploading")
result = subprocess.run(["/usr/bin/scp", "dist/app", "prod:/apps/"], capture_output=True)
if result["returncode"] == 0:
    tui.success("Upload complete")
else:
    tui.error("Upload failed")
```

## Example Output (what it should look like)

```
╭─ Release ─────────────────────────────╮
│ Deploying build...                    │
╰───────────────────────────────────────╯
✓ Built commit a1b2c3d

┌───────────┬───────┬──────┐
│ Artifact  │ Size  │ Time │
├───────────┼───────┼──────┤
│ app-linux │ 900KB │ 6ms  │
│ app-macos │ 910KB │ 7ms  │
└───────────┴───────┴──────┘
```

```
? Project name: fastship
? Select features:  ◉ Logging  ◉ HTTP  ○ Config
? Create project now? (Y/n) y
✓ Created fastship with 2 features
```

```
Uploading  [███████████░░░░░░] 68%  3.2s
```

## Gold Set (must be great for CLI apps)

### Core CLI APIs

- argument parsing and help output
- subcommands and groups
- shell completion generation
- config loading (ini/toml at minimum)
- logging with levels and formatting
- robust subprocess API
- structured output: tables, boxes, progress, spinners

### Runtime Guarantees

- fast startup (< 10 ms)
- small binaries (< 2MB range)
- predictable behavior across macOS and Linux
- clear error messages and exit codes

## Standard Library Support (tiered)

### Essential

These are required for real-world CLI usage and should be high parity.

- argparse
- os, sys, time
- pathlib, glob, fnmatch
- subprocess, signal
- json, csv
- logging
- datetime
- textwrap
- tempfile, shutil
- re
- hashlib (subset OK, but stable)

### Good to have

These unlock common workflows and popular CLI libraries.

- configparser
- enum
- uuid
- urllib.parse
- contextlib
- typing (runtime stubs)
- statistics, functools, itertools, heapq

### Nice to have

Useful for some apps but not required for most CLIs.

- toml
- http.client
- gzip, zipfile, tarfile
- secrets, hmac
- dataclasses
- xml.etree
- sqlite3 (large - is there an efficient way?)

### Probably will not need

Low value for typical CLI apps or too heavy to justify.

- multiprocessing
- decimal, fractions
- tkinter, curses
- site, venv, distutils

## Positioning

kipferl is for CLI tools that want:

- Python ergonomics
- small, portable binaries
- fast startup
- beautiful terminal UX

It is not a drop-in replacement for CPython or pip.

## Success Metrics

- Minimal standalone application stays near the current 1.4 MB tree-shaken
  baseline, while the complete runtime remains below the 5.75 MB ceiling
  range and starts in <= 10ms on the measured native baseline.
- 90%+ parity for essential modules listed above.
- polished UX for prompts, tables, progress, and error output.
- at least one production-grade sample CLI app in the repo.

## Runtime Decision (PocketPy)

PocketPy is the runtime base for kipferl.

Why:
- Velocity: the Rust host passes 1,669/1,669 available checks in the curated
  CPython-compatibility suite (see `tests/compat_report_pocketpy.md`).
- Extension workflow: PocketPy exposes a narrow embedding API that the Rust
  host wraps behind one audited FFI boundary.
- Product fit: the final Apple Silicon baseline starts in 7.044ms median and
  retains useful headroom under the 10ms target while bundling SQLite, HTTPS,
  maintained archive readers, and Ratatui-backed interaction.

Decision implications:
- Continue investing in compatibility-gated Rust modules on PocketPy.
- MicroPython is not part of the repo anymore; keep historical comparisons for context only.

## Future Features (Wishlist)

Features inspired by popular CLI frameworks (Rich, Inquirer, BubbleTea, listr2) that would enhance kipferl.

### High Priority

| Feature | Description | Inspiration |
|---------|-------------|-------------|
| `tui.tree()` | Hierarchical tree display for file structures, dependencies, nested data | Rich Tree |
| Fuzzy select | Filter choices by typing in `input.select()` | Inquirer/Questionary |
| Task list | Show multiple tasks with status (pending/running/done/failed) | listr2 |

### Medium Priority

| Feature | Description | Inspiration |
|---------|-------------|-------------|
| Column layout | Display content in multiple columns | Rich Columns |
| Autocomplete prompt | Text input with tab-completion suggestions | Inquirer |
| Multiple progress bars | Show several concurrent progress bars | Rich Progress |
| Table enhancements | Cell alignment, row highlighting, alternating colors | Rich Table |

### Lower Priority

| Feature | Description | Inspiration |
|---------|-------------|-------------|
| File picker | Interactive directory navigation and file selection | BubbleTea filepicker |
| Syntax highlighting | Language-aware code coloring | Rich Syntax |
| Markdown rendering | Render markdown in terminal | Rich Markdown |
| Paginated output | Page through long output with scrolling | Rich Pager |

### Example: Tree Output

```
📁 project/
├── 📄 main.py
├── 📁 src/
│   ├── 📄 utils.py
│   └── 📄 config.py
└── 📄 README.md
```

### Example: Task List

```
  ◉ Installing dependencies
  ◉ Running tests
  ◐ Building artifacts...
  ○ Deploying to production
```

### Example: Fuzzy Select

```
? Choose a file: py
  > main.py
    utils.py
    config.py
```

## Reference TUI Tooling (inspiration)

- Bubble Tea (Go): https://github.com/charmbracelet/bubbletea
- Bubbles (Go components): https://github.com/charmbracelet/bubbles
- Lip Gloss (Go styling): https://github.com/charmbracelet/lipgloss
- Rich (Python formatting): https://github.com/Textualize/rich
- Typer (Python CLI DX): https://github.com/tiangolo/typer
- Click (Python CLI): https://github.com/pallets/click
- Ratatui (Rust TUI): https://github.com/ratatui/ratatui
