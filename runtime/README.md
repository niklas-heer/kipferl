# μcharm Runtime Modules

This folder contains runtime modules split into two groups:
- `runtime/ucharm/` for ucharm-native UX modules (ansi/term/tui/input/args)
- `runtime/compat/` for CPython-compat modules (argparse, csv, etc.)

Each module includes a `pocketpy.zig` binding and optional Zig core files.

## Status

- The Zig core implementations live alongside each module (e.g. `ansi.zig`).
- PocketPy bindings live under `runtime/**/pocketpy.zig`.

## Modules

### ansi
ANSI escape code generation:
- `ansi.fg(color)` - Foreground color code (name, index 0-255, or #hex)
- `ansi.bg(color)` - Background color code
- `ansi.rgb(r, g, b, bg=False)` - 24-bit true color
- `ansi.bold()`, `ansi.dim()`, `ansi.italic()`, `ansi.underline()` - Style codes
- `ansi.strikethrough()`, `ansi.reverse()`, `ansi.blink()`, `ansi.hidden()`
- `ansi.reset()` - Reset all styles

## Integration

These modules are bound into PocketPy via Zig-native bindings and registered
from `pocketpy/src/runtime.zig`.
