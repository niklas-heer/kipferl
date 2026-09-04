# GitHub Copilot Instructions for kipferl

This project uses kipferl (PocketPy + native Rust modules).

## Key Facts

- PocketPy runtime, not CPython
- Use native modules or compatible local Python files; there is no pip environment
- 50+ native modules: tui, input, term, ansi, http.client, sqlite3, json, yaml, toml, kdl, re, etc.

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
connection = http.HTTPSConnection("api.example.com", timeout=10)
connection.request("GET", "/data")
print(connection.getresponse().read().decode())
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

## Avoid

- requests/httpx (use `http.client` instead)
- numpy/pandas (pure Python alternatives)
- async/await
