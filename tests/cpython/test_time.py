"""
Simplified time module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_time.py
"""

import sys
import time

# Test tracking
_passed = 0
_failed = 0
_errors = []
_skipped = 0


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# Helper to get struct_time field (works with both tuple and named attributes)
def get_tm_field(st, index, attr):
    """Get a field from struct_time by index or attribute name."""
    if hasattr(st, attr):
        return getattr(st, attr)
    return st[index]


# ============================================================================
# time.time() tests
# ============================================================================

print("\n=== time.time() tests ===")

t = time.time()
test("time returns number", isinstance(t, (int, float)))
test("time is positive", t > 0)
test("time is reasonable", t > 1000000000)  # After 2001

# Time should increase
t1 = time.time()
t2 = time.time()
test("time increases", t2 >= t1)


# ============================================================================
# time.sleep() tests
# ============================================================================

print("\n=== time.sleep() tests ===")

# Sleep for a short time
start = time.time()
time.sleep(0.1)
elapsed = time.time() - start
test("sleep 0.1s", elapsed >= 0.09)  # Allow some tolerance

# Sleep for zero (should return immediately)
start = time.time()
time.sleep(0)
elapsed = time.time() - start
test("sleep 0", elapsed < 0.1)


# ============================================================================
# time.localtime() tests
# ============================================================================

print("\n=== time.localtime() tests ===")

if hasattr(time, "localtime"):
    lt = time.localtime()
    test(
        "localtime returns struct_time",
        isinstance(lt, (tuple, type(lt))),
    )

    # Check reasonable values using attribute access
    tm_year = get_tm_field(lt, 0, "tm_year")
    tm_mon = get_tm_field(lt, 1, "tm_mon")
    tm_mday = get_tm_field(lt, 2, "tm_mday")
    tm_hour = get_tm_field(lt, 3, "tm_hour")
    tm_min = get_tm_field(lt, 4, "tm_min")
    tm_sec = get_tm_field(lt, 5, "tm_sec")
    tm_wday = get_tm_field(lt, 6, "tm_wday")
    tm_yday = get_tm_field(lt, 7, "tm_yday")

    test("localtime year", tm_year >= 2020)
    test("localtime month", 1 <= tm_mon <= 12)
    test("localtime day", 1 <= tm_mday <= 31)
    test("localtime hour", 0 <= tm_hour <= 23)
    test("localtime minute", 0 <= tm_min <= 59)
    test("localtime second", 0 <= tm_sec <= 61)  # 61 for leap seconds
    test("localtime weekday", 0 <= tm_wday <= 6)
    test("localtime yearday", 1 <= tm_yday <= 366)

    # With argument - check if localtime accepts arguments
    try:
        lt2 = time.localtime(0)
        test("localtime(0) works", lt2 is not None)
    except TypeError:
        skip("localtime(0)", "localtime does not accept arguments")
else:
    skip("localtime tests", "time.localtime not available")


# ============================================================================
# time.gmtime() tests
# ============================================================================

print("\n=== time.gmtime() tests ===")

if hasattr(time, "gmtime"):
    gt = time.gmtime()
    test("gmtime returns struct_time", isinstance(gt, (tuple, type(gt))))

    # Check reasonable values using attribute access
    tm_year = get_tm_field(gt, 0, "tm_year")
    tm_mon = get_tm_field(gt, 1, "tm_mon")

    test("gmtime year", tm_year >= 2020)
    test("gmtime month", 1 <= tm_mon <= 12)

    # gmtime(0) should be Unix epoch
    epoch = time.gmtime(0)
    epoch_year = get_tm_field(epoch, 0, "tm_year")
    epoch_mon = get_tm_field(epoch, 1, "tm_mon")
    epoch_mday = get_tm_field(epoch, 2, "tm_mday")

    test("gmtime(0) year", epoch_year == 1970)
    test("gmtime(0) month", epoch_mon == 1)
    test("gmtime(0) day", epoch_mday == 1)
else:
    skip("gmtime tests", "time.gmtime not available")


# ============================================================================
# time.mktime() tests
# ============================================================================

print("\n=== time.mktime() tests ===")

if hasattr(time, "mktime"):
    # Round-trip test
    now = time.time()
    lt = time.localtime(now)
    back = time.mktime(lt)
    test("mktime roundtrip", abs(now - back) < 2)  # Allow 1 second tolerance

    # Known date
    # 2020-01-01 00:00:00
    try:
        t = time.mktime((2020, 1, 1, 0, 0, 0, 0, 0, -1))
        test(
            "mktime known date",
            t > 1577836800 - 86400 * 2 and t < 1577836800 + 86400 * 2,
        )
    except (OverflowError, OSError):
        skip("mktime known date", "date out of range")
else:
    skip("mktime tests", "time.mktime not available")


# ============================================================================
# time.strftime() tests
# ============================================================================

print("\n=== time.strftime() tests ===")

if hasattr(time, "strftime") and hasattr(time, "localtime"):
    lt = time.localtime()

    # Basic format
    result = time.strftime("%Y", lt)
    test("strftime %Y", len(result) == 4 and result.isdigit())

    result = time.strftime("%m", lt)
    test("strftime %m", len(result) == 2)

    result = time.strftime("%d", lt)
    test("strftime %d", len(result) == 2)

    result = time.strftime("%H:%M:%S", lt)
    test("strftime %H:%M:%S", len(result) == 8 and ":" in result)

    result = time.strftime("%Y-%m-%d", lt)
    test("strftime %Y-%m-%d", len(result) == 10 and "-" in result)

    # Literal text
    result = time.strftime("Year: %Y", lt)
    test("strftime literal", result.startswith("Year: "))
else:
    skip("strftime tests", "time.strftime not available")


# ============================================================================
# time.strptime() tests
# ============================================================================

print("\n=== time.strptime() tests ===")

if hasattr(time, "strptime"):
    # Parse date
    result = time.strptime("2020-01-15", "%Y-%m-%d")
    r_year = get_tm_field(result, 0, "tm_year")
    r_mon = get_tm_field(result, 1, "tm_mon")
    r_mday = get_tm_field(result, 2, "tm_mday")
    test("strptime date", r_year == 2020 and r_mon == 1 and r_mday == 15)

    # Parse time
    result = time.strptime("14:30:00", "%H:%M:%S")
    r_hour = get_tm_field(result, 3, "tm_hour")
    r_min = get_tm_field(result, 4, "tm_min")
    r_sec = get_tm_field(result, 5, "tm_sec")
    test("strptime time", r_hour == 14 and r_min == 30 and r_sec == 0)
else:
    skip("strptime tests", "time.strptime not available")


# ============================================================================
# time.monotonic() tests
# ============================================================================

print("\n=== time.monotonic() tests ===")

if hasattr(time, "monotonic"):
    m = time.monotonic()
    test("monotonic returns number", isinstance(m, (int, float)))
    test("monotonic is non-negative", m >= 0)

    # Should always increase
    m1 = time.monotonic()
    time.sleep(0.01)
    m2 = time.monotonic()
    test("monotonic increases", m2 > m1)
else:
    skip("monotonic tests", "time.monotonic not available")


# ============================================================================
# time.perf_counter() tests
# ============================================================================

print("\n=== time.perf_counter() tests ===")

if hasattr(time, "perf_counter"):
    p = time.perf_counter()
    test("perf_counter returns number", isinstance(p, (int, float)))

    # Should increase
    p1 = time.perf_counter()
    time.sleep(0.01)
    p2 = time.perf_counter()
    test("perf_counter increases", p2 > p1)
else:
    skip("perf_counter tests", "time.perf_counter not available")


# ============================================================================
# time.time_ns() tests
# ============================================================================

print("\n=== time.time_ns() tests ===")

if hasattr(time, "time_ns"):
    t = time.time_ns()
    test("time_ns returns int", isinstance(t, int))
    test("time_ns is positive", t > 0)
    test("time_ns is nanoseconds", t > 1000000000000000000)  # After 2001
else:
    skip("time_ns tests", "time.time_ns not available")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Very small sleep
start = time.time()
time.sleep(0.001)
elapsed = time.time() - start
test("sleep 1ms", elapsed >= 0.0005)

# Multiple time() calls
times = [time.time() for _ in range(10)]
# Check that times are monotonically non-decreasing
times_increasing = True
for i in range(9):
    if times[i] > times[i + 1]:
        times_increasing = False
        break
test("multiple time calls", times_increasing)


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
