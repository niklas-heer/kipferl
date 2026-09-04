# Benchmark Results: CPython vs MicroPython (Pure Python Compat)

> Archived MicroPython experiment. Kipferl now uses PocketPy with a Rust host.
> These observations and unverified projections do not describe the current runtime.
> See the [benchmarking guide](README.md) for current measurements and methodology.

## Test Environment

- **CPython**: 3.14.1
- **MicroPython**: 3.4.0
- **Iterations**: 10,000 (1,000 for slow operations)

## Results Summary

### Base64 Encoding/Decoding

| Operation | CPython | MicroPython | Slowdown |
|-----------|---------|-------------|----------|
| encode 13 bytes | 0.0001 ms | 0.0002 ms | 2x |
| encode 1KB | 0.0011 ms | 0.0057 ms | 5.2x |
| encode 10KB | 0.0099 ms | 0.0504 ms | 5.1x |
| decode 20 bytes | 0.0001 ms | 0.0003 ms | 3x |
| decode 1.3KB | 0.0015 ms | 0.0036 ms | 2.4x |
| urlsafe encode 1KB | 0.0013 ms | 0.0112 ms | 8.6x |

**Notes**: MicroPython's base64 uses `binascii` which is a native C module, so it's reasonably fast. The urlsafe variant has extra Python overhead for character replacement.

### Datetime Operations

| Operation | CPython | MicroPython | Slowdown |
|-----------|---------|-------------|----------|
| create datetime | 0.0003 ms | 0.0022 ms | 7.3x |
| fromtimestamp | 0.0007 ms | 0.0026 ms | 3.7x |
| isoformat | 0.0007 ms | 0.0103 ms | 14.7x |
| weekday | 0.0003 ms | 0.0004 ms | 1.3x |
| create timedelta | 0.0003 ms | 0.0014 ms | 4.7x |
| timedelta add | 0.0004 ms | 0.0016 ms | 4x |
| datetime + timedelta | 0.0015 ms | 0.0091 ms | 6.1x |

**Notes**: Pure Python datetime is 4-15x slower than CPython's C implementation. The `isoformat()` method is particularly slow due to string formatting.

### Fnmatch Pattern Matching

| Operation | CPython | MicroPython | Slowdown |
|-----------|---------|-------------|----------|
| simple *.py | 0.0007 ms | 0.0038 ms | 5.4x |
| pattern ????.py | 0.0005 ms | 0.0025 ms | 5x |
| pattern [a-z]*.py | 0.0009 ms | 0.0046 ms | 5.1x |
| complex pattern | 0.0017 ms | 0.0119 ms | 7x |
| no match | 0.0008 ms | 0.0058 ms | 7.3x |
| filter 100 items | 0.0755 ms | 0.4932 ms | 6.5x |

**Notes**: Our pure Python fnmatch implementation is 5-7x slower than CPython's. This is expected since it's interpreted Python vs compiled C.

## Memory Usage

| Runtime | Memory |
|---------|--------|
| CPython | ~19 MB RSS |
| MicroPython | 58 KB allocated, 2 MB free |

## Historical projections (not measurements)

The experiment proposed these improvements from native Zig modules. They were
projections, not measured results:

| Module | Expected Improvement |
|--------|---------------------|
| **base64** | 2-5x faster (Zig SIMD base64) |
| **datetime** | 5-15x faster (native time operations) |
| **fnmatch** | 5-10x faster (compiled pattern matching) |
| **glob** | 10-20x faster (native filesystem iteration) |
| **tempfile** | 10-50x faster (direct syscalls) |
| **shutil** | 10-50x faster (native file operations) |

These projections did not establish parity with CPython or a measured native
memory footprint. They should not be used as performance claims for Kipferl.

## Interpretation of the experiment

The measured pure Python compatibility operations were slower than CPython's
native C implementations. The result informed the earlier native-module
experiment; it does not measure today's PocketPy/Rust implementation.

The following footprint and startup figures were proposed targets, not results
established by the tables above:

- **Binary size**: ~700KB (vs 77MB Python installation)
- **Memory usage**: ~2MB (vs 20MB+ for CPython)
- **Startup time**: ~6ms (vs 30ms+ for Python)
