"""
Simplified statistics module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_statistics.py
"""

import statistics

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


def approx_equal(a, b, tol=1e-9):
    """Check if two floats are approximately equal."""
    return abs(a - b) < tol


def has_attr(mod, name):
    """Check if module has an attribute."""
    try:
        getattr(mod, name)
        return True
    except AttributeError:
        return False


# ============================================================================
# statistics.mean() tests
# ============================================================================

print("\n=== statistics.mean() tests ===")

test("mean integers", statistics.mean([1, 2, 3, 4, 5]) == 3)
test("mean floats", approx_equal(statistics.mean([1.0, 2.0, 3.0]), 2.0))
test("mean single", statistics.mean([5]) == 5)
test("mean negative", statistics.mean([-1, -2, -3]) == -2)
test("mean mixed", approx_equal(statistics.mean([1, 2, 3, 4]), 2.5))


# ============================================================================
# statistics.median() tests
# ============================================================================

print("\n=== statistics.median() tests ===")

test("median odd", statistics.median([1, 3, 5]) == 3)
test("median even", statistics.median([1, 2, 3, 4]) == 2.5)
test("median single", statistics.median([7]) == 7)
test("median unsorted", statistics.median([3, 1, 2]) == 2)
test("median floats", approx_equal(statistics.median([1.5, 2.5, 3.5]), 2.5))


# ============================================================================
# statistics.median_low() and median_high() tests
# ============================================================================

print("\n=== statistics.median_low/high() tests ===")

if has_attr(statistics, "median_low"):
    test("median_low odd", statistics.median_low([1, 3, 5]) == 3)
    test("median_low even", statistics.median_low([1, 2, 3, 4]) == 2)
    test("median_low single", statistics.median_low([7]) == 7)
else:
    skip("median_low odd", "median_low not available")
    skip("median_low even", "median_low not available")
    skip("median_low single", "median_low not available")

if has_attr(statistics, "median_high"):
    test("median_high odd", statistics.median_high([1, 3, 5]) == 3)
    test("median_high even", statistics.median_high([1, 2, 3, 4]) == 3)
    test("median_high single", statistics.median_high([7]) == 7)
else:
    skip("median_high odd", "median_high not available")
    skip("median_high even", "median_high not available")
    skip("median_high single", "median_high not available")


# ============================================================================
# statistics.mode() tests
# ============================================================================

print("\n=== statistics.mode() tests ===")

test("mode single mode", statistics.mode([1, 1, 2, 3]) == 1)
test("mode all same", statistics.mode([5, 5, 5]) == 5)

# PocketPy's mode only supports numeric data
try:
    result = statistics.mode(["a", "b", "a"])
    test("mode strings", result == "a")
except TypeError:
    skip("mode strings", "mode only supports numeric data")


# ============================================================================
# statistics.stdev() tests (sample standard deviation)
# ============================================================================

print("\n=== statistics.stdev() tests ===")

# stdev of [1, 2, 3, 4, 5]
# mean = 3, variance = [(1-3)^2 + (2-3)^2 + (3-3)^2 + (4-3)^2 + (5-3)^2] / 4 = 10/4 = 2.5
# stdev = sqrt(2.5) ≈ 1.5811
result = statistics.stdev([1, 2, 3, 4, 5])
test("stdev basic", approx_equal(result, 1.5811388300841898, 1e-6))

# stdev of [2, 4, 4, 4, 5, 5, 7, 9]
result = statistics.stdev([2, 4, 4, 4, 5, 5, 7, 9])
test("stdev mixed", approx_equal(result, 2.138089935299395, 1e-6))


# ============================================================================
# statistics.variance() tests (sample variance)
# ============================================================================

print("\n=== statistics.variance() tests ===")

result = statistics.variance([1, 2, 3, 4, 5])
test("variance basic", approx_equal(result, 2.5))

result = statistics.variance([2, 4, 4, 4, 5, 5, 7, 9])
test("variance mixed", approx_equal(result, 4.571428571428571, 1e-6))


# ============================================================================
# statistics.pstdev() tests (population standard deviation)
# ============================================================================

print("\n=== statistics.pstdev() tests ===")

# pstdev of [1, 2, 3, 4, 5]
# pvariance = 10/5 = 2, pstdev = sqrt(2) ≈ 1.4142
result = statistics.pstdev([1, 2, 3, 4, 5])
test("pstdev basic", approx_equal(result, 1.4142135623730951, 1e-6))


# ============================================================================
# statistics.pvariance() tests (population variance)
# ============================================================================

print("\n=== statistics.pvariance() tests ===")

result = statistics.pvariance([1, 2, 3, 4, 5])
test("pvariance basic", approx_equal(result, 2.0))

result = statistics.pvariance([2, 4, 4, 4, 5, 5, 7, 9])
test("pvariance mixed", approx_equal(result, 4.0))


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Empty data
try:
    statistics.mean([])
    test("mean empty raises", False)
except Exception:
    test("mean empty raises", True)

# Single value stdev
try:
    statistics.stdev([5])
    test("stdev single raises", False)
except Exception:
    test("stdev single raises", True)


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    import sys

    sys.exit(1)
else:
    print("All tests passed!")
