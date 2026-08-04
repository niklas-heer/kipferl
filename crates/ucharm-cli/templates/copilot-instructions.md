# GitHub Copilot Instructions for ucharm

This project uses ucharm (PocketPy + native Rust modules).

## Key Facts

- PocketPy runtime, not CPython
- No pip packages with C extensions
- 50+ native modules: tui, input, term, ansi, http.client, sqlite3, json, re, etc.

## Preferred Patterns

```python
import tui
import input

tui.box("Ready", title="Status")
choice = input.select("Next step:", ["Build", "Test", "Exit"])

# Tables
tui.table([["Name", "Age"], ["Alice", "25"]], headers=True)

# Progress with elapsed time
tui.progress(5, 10, label="Loading", elapsed=2.5)
tui.progress_done()

# HTTP requests
http = __import__("http.client")
connection = http.HTTPSConnection("api.example.com")
connection.request("GET", "/data")
print(connection.getresponse().read().decode())
```

## Avoid

- requests/httpx (use `http.client` instead)
- numpy/pandas (pure Python alternatives)
- async/await
