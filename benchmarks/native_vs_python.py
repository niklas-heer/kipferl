#!/usr/bin/env python3
"""
Benchmark: kipferl compat modules vs CPython stdlib

This benchmark compares our pure Python compat implementations against
CPython's native C implementations to measure the performance gap.

When native Zig modules are compiled into MicroPython, performance
should approach or exceed CPython levels.

Run with:
    python3 benchmarks/native_vs_python.py      # CPython stdlib baseline
    COMPAT=1 python3 benchmarks/native_vs_python.py  # Test compat layer on CPython
    micropython benchmarks/native_vs_python.py  # MicroPython with compat
"""

import sys
import time

# Configuration
ITERATIONS = 10000
SMALL_ITERATIONS = 1000

# Determine which modules to use
try:
    import os

    USE_COMPAT = os.getenv("COMPAT") == "1" or sys.implementation.name != "cpython"
except:
    USE_COMPAT = True

if USE_COMPAT:
    # Add project to path - handle MicroPython's limited os module
    try:
        import os.path

        base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        sys.path.insert(0, base_dir)
        sys.path.insert(0, base_dir + "/kipferl")
    except:
        # MicroPython fallback
        sys.path.insert(0, ".")
        sys.path.insert(0, "./kipferl")


def timeit(func, iterations=ITERATIONS):
    """Time a function over multiple iterations, return ms per operation."""
    # Warmup
    for _ in range(min(100, iterations // 10)):
        func()

    start = time.time()
    for _ in range(iterations):
        func()
    elapsed = time.time() - start
    return (elapsed / iterations) * 1000  # ms per operation


def format_result(name, time_ms, baseline_ms=None):
    """Format benchmark result with optional comparison."""
    ops_per_sec = 1000 / time_ms if time_ms > 0 else float("inf")
    result = f"  {name}: {time_ms:.4f} ms/op ({ops_per_sec:,.0f} ops/sec)"
    if baseline_ms is not None and baseline_ms > 0:
        ratio = time_ms / baseline_ms
        result += f" [{ratio:.1f}x]"
    return result


print("=" * 70)
print("kipferl Native Module Benchmark")
print("=" * 70)
print(f"Runtime: {sys.implementation.name} {sys.version.split()[0]}")
print(f"Mode: {'compat layer' if USE_COMPAT else 'native stdlib'}")
print(f"Iterations: {ITERATIONS:,} (small: {SMALL_ITERATIONS:,})")
print()

results = {}

# =============================================================================
# Base64 Benchmarks
# =============================================================================
print("-" * 70)
print("BASE64 ENCODING/DECODING")
print("-" * 70)

if USE_COMPAT:
    from compat import base64
else:
    import base64

test_small = b"Hello, World!"
test_1kb = b"x" * 1000
test_10kb = b"y" * 10000
encoded_1kb = base64.b64encode(test_1kb)

results["b64_encode_small"] = timeit(lambda: base64.b64encode(test_small))
print(format_result("encode 13 bytes", results["b64_encode_small"]))

results["b64_encode_1kb"] = timeit(lambda: base64.b64encode(test_1kb))
print(format_result("encode 1KB", results["b64_encode_1kb"]))

results["b64_encode_10kb"] = timeit(
    lambda: base64.b64encode(test_10kb), SMALL_ITERATIONS
)
print(format_result("encode 10KB", results["b64_encode_10kb"]))

results["b64_decode_1kb"] = timeit(lambda: base64.b64decode(encoded_1kb))
print(format_result("decode 1.3KB", results["b64_decode_1kb"]))

results["b64_urlsafe"] = timeit(lambda: base64.urlsafe_b64encode(test_1kb))
print(format_result("urlsafe encode 1KB", results["b64_urlsafe"]))

print()

# =============================================================================
# Datetime Benchmarks
# =============================================================================
print("-" * 70)
print("DATETIME OPERATIONS")
print("-" * 70)

if USE_COMPAT:
    from compat import datetime
else:
    import datetime

results["dt_create"] = timeit(lambda: datetime.datetime(2024, 6, 15, 12, 30, 45))
print(format_result("create datetime", results["dt_create"]))

results["dt_fromts"] = timeit(lambda: datetime.datetime.fromtimestamp(time.time()))
print(format_result("fromtimestamp", results["dt_fromts"]))

dt = datetime.datetime(2024, 6, 15, 12, 30, 45)
results["dt_iso"] = timeit(lambda: dt.isoformat())
print(format_result("isoformat", results["dt_iso"]))

results["dt_weekday"] = timeit(lambda: dt.weekday())
print(format_result("weekday", results["dt_weekday"]))

results["td_create"] = timeit(lambda: datetime.timedelta(days=5, hours=3, minutes=30))
print(format_result("create timedelta", results["td_create"]))

td = datetime.timedelta(days=5)
results["td_add"] = timeit(lambda: dt + td)
print(format_result("datetime + timedelta", results["td_add"]))

print()

# =============================================================================
# Fnmatch Benchmarks
# =============================================================================
print("-" * 70)
print("FNMATCH PATTERN MATCHING")
print("-" * 70)

if USE_COMPAT:
    from compat import fnmatch
else:
    import fnmatch

results["fnm_simple"] = timeit(lambda: fnmatch.fnmatch("test.py", "*.py"))
print(format_result("simple *.py", results["fnm_simple"]))

results["fnm_qmark"] = timeit(lambda: fnmatch.fnmatch("test.py", "????.py"))
print(format_result("pattern ????.py", results["fnm_qmark"]))

results["fnm_class"] = timeit(lambda: fnmatch.fnmatch("test.py", "[a-z]*.py"))
print(format_result("pattern [a-z]*.py", results["fnm_class"]))

results["fnm_complex"] = timeit(
    lambda: fnmatch.fnmatch("my_test_file_v2.py", "*test*v[0-9].py")
)
print(format_result("complex pattern", results["fnm_complex"]))

names = ["test.py", "main.py", "utils.py", "data.txt", "config.json"] * 20
results["fnm_filter"] = timeit(lambda: fnmatch.filter(names, "*.py"), SMALL_ITERATIONS)
print(format_result("filter 100 items", results["fnm_filter"]))

print()

# =============================================================================
# Statistics Benchmarks
# =============================================================================
print("-" * 70)
print("STATISTICS")
print("-" * 70)

if USE_COMPAT:
    from compat import statistics
else:
    import statistics

data_10 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
data_100 = list(range(1, 101))
data_1000 = list(range(1, 1001))

results["stats_mean_10"] = timeit(lambda: statistics.mean(data_10))
print(format_result("mean(10 items)", results["stats_mean_10"]))

results["stats_mean_100"] = timeit(lambda: statistics.mean(data_100))
print(format_result("mean(100 items)", results["stats_mean_100"]))

results["stats_median_100"] = timeit(lambda: statistics.median(data_100))
print(format_result("median(100 items)", results["stats_median_100"]))

results["stats_stdev_100"] = timeit(lambda: statistics.stdev(data_100))
print(format_result("stdev(100 items)", results["stats_stdev_100"]))

results["stats_mean_1000"] = timeit(
    lambda: statistics.mean(data_1000), SMALL_ITERATIONS
)
print(format_result("mean(1000 items)", results["stats_mean_1000"]))

print()

# =============================================================================
# Textwrap Benchmarks
# =============================================================================
print("-" * 70)
print("TEXTWRAP")
print("-" * 70)

if USE_COMPAT:
    from compat import textwrap
else:
    import textwrap

short_text = "Hello world, this is a test."
long_text = "The quick brown fox jumps over the lazy dog. " * 10
indented = "    line1\n    line2\n    line3\n    line4"

results["tw_wrap_short"] = timeit(lambda: textwrap.wrap(short_text, width=20))
print(format_result("wrap short text", results["tw_wrap_short"]))

results["tw_wrap_long"] = timeit(lambda: textwrap.wrap(long_text, width=40))
print(format_result("wrap long text", results["tw_wrap_long"]))

results["tw_fill"] = timeit(lambda: textwrap.fill(long_text, width=40))
print(format_result("fill long text", results["tw_fill"]))

results["tw_dedent"] = timeit(lambda: textwrap.dedent(indented))
print(format_result("dedent 4 lines", results["tw_dedent"]))

results["tw_indent"] = timeit(lambda: textwrap.indent(short_text, ">>> "))
print(format_result("indent text", results["tw_indent"]))

print()

# =============================================================================
# Pathlib Benchmarks
# =============================================================================
print("-" * 70)
print("PATHLIB")
print("-" * 70)

if USE_COMPAT:
    from compat.pathlib import Path
else:
    from pathlib import Path

results["path_create"] = timeit(lambda: Path("/usr/local/bin/python"))
print(format_result("create Path", results["path_create"]))

p = Path("/usr/local/bin/python.exe")
results["path_name"] = timeit(lambda: p.name)
print(format_result("Path.name", results["path_name"]))

results["path_stem"] = timeit(lambda: p.stem)
print(format_result("Path.stem", results["path_stem"]))

results["path_suffix"] = timeit(lambda: p.suffix)
print(format_result("Path.suffix", results["path_suffix"]))

results["path_parent"] = timeit(lambda: p.parent)
print(format_result("Path.parent", results["path_parent"]))

results["path_join"] = timeit(lambda: Path("/usr") / "local" / "bin")
print(format_result("Path / join", results["path_join"]))

print()

# =============================================================================
# Summary
# =============================================================================
print("-" * 70)
print("MEMORY USAGE")
print("-" * 70)

try:
    import gc

    gc.collect()
    if hasattr(gc, "mem_free"):
        # MicroPython
        print(f"  Free memory: {gc.mem_free():,} bytes")
        print(f"  Allocated: {gc.mem_alloc():,} bytes")
    else:
        # CPython
        try:
            import resource

            usage = resource.getrusage(resource.RUSAGE_SELF)
            print(f"  Max RSS: {usage.ru_maxrss // 1024:,} KB")
        except:
            print("  (memory stats not available)")
except:
    print("  (memory stats not available)")

print()
print("=" * 70)
print("Benchmark complete!")
if USE_COMPAT:
    print("Note: Using compat layer. Native Zig modules will be faster.")
print("=" * 70)
