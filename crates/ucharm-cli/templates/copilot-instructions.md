# GitHub Copilot Instructions for ucharm

This project uses ucharm (PocketPy + native Rust modules).

## Key Facts

- PocketPy runtime, not CPython
- No pip packages with C extensions
- 50+ native modules: tui, input, term, ansi, fetch, template, sqlite3, json, re, etc.

## Preferred Patterns

```python
from ucharm import box, table, success, select, confirm
import tui

# Tables
tui.table([["Name", "Age"], ["Alice", "25"]], headers=True)

# Progress with elapsed time
tui.progress(5, 10, label="Loading", elapsed=2.5)
tui.progress_done()

# HTTP requests
import fetch
resp = fetch.get("https://api.example.com/data")
print(resp["body"].decode())

# Templating
import template
html = template.render("Hello {{name}}!", {"name": "World"})
```

## Avoid

- requests/httpx (use `fetch` module instead)
- numpy/pandas (pure Python alternatives)
- async/await
