"""
Simplified bisect module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_bisect.py
"""

import sys

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


try:
    import bisect
except ImportError:
    print("SKIP: bisect module not available")
    sys.exit(0)


# ============================================================================
# bisect_left basic tests
# ============================================================================

print("\n=== bisect_left basic tests ===")

# Empty list
test("bisect_left empty", bisect.bisect_left([], 1) == 0)

# Single element
test("bisect_left single before", bisect.bisect_left([5], 3) == 0)
test("bisect_left single equal", bisect.bisect_left([5], 5) == 0)
test("bisect_left single after", bisect.bisect_left([5], 7) == 1)

# Multiple elements
a = [1, 3, 5, 7, 9]
test("bisect_left at start", bisect.bisect_left(a, 0) == 0)
test("bisect_left at end", bisect.bisect_left(a, 10) == 5)
test("bisect_left middle", bisect.bisect_left(a, 4) == 2)
test("bisect_left existing start", bisect.bisect_left(a, 1) == 0)
test("bisect_left existing middle", bisect.bisect_left(a, 5) == 2)
test("bisect_left existing end", bisect.bisect_left(a, 9) == 4)

# Duplicates - bisect_left returns leftmost position
a = [1, 2, 2, 2, 3]
test("bisect_left duplicates", bisect.bisect_left(a, 2) == 1)


# ============================================================================
# bisect_right basic tests
# ============================================================================

print("\n=== bisect_right basic tests ===")

# Empty list
test("bisect_right empty", bisect.bisect_right([], 1) == 0)

# Single element
test("bisect_right single before", bisect.bisect_right([5], 3) == 0)
test("bisect_right single equal", bisect.bisect_right([5], 5) == 1)
test("bisect_right single after", bisect.bisect_right([5], 7) == 1)

# Multiple elements
a = [1, 3, 5, 7, 9]
test("bisect_right at start", bisect.bisect_right(a, 0) == 0)
test("bisect_right at end", bisect.bisect_right(a, 10) == 5)
test("bisect_right middle", bisect.bisect_right(a, 4) == 2)
test("bisect_right existing start", bisect.bisect_right(a, 1) == 1)
test("bisect_right existing middle", bisect.bisect_right(a, 5) == 3)
test("bisect_right existing end", bisect.bisect_right(a, 9) == 5)

# Duplicates - bisect_right returns rightmost position
a = [1, 2, 2, 2, 3]
test("bisect_right duplicates", bisect.bisect_right(a, 2) == 4)


# ============================================================================
# bisect alias tests
# ============================================================================

print("\n=== bisect alias tests ===")

# bisect is alias for bisect_right
a = [1, 3, 5, 7, 9]
test("bisect is bisect_right", bisect.bisect(a, 5) == bisect.bisect_right(a, 5))
test("bisect alias middle", bisect.bisect(a, 4) == 2)
test("bisect alias existing", bisect.bisect(a, 5) == 3)


# ============================================================================
# insort_left tests
# ============================================================================

print("\n=== insort_left tests ===")

# Empty list
a = []
bisect.insort_left(a, 5)
test("insort_left empty", a == [5])

# Insert at start
a = [2, 4, 6]
bisect.insort_left(a, 1)
test("insort_left at start", a == [1, 2, 4, 6])

# Insert at end
a = [2, 4, 6]
bisect.insort_left(a, 8)
test("insort_left at end", a == [2, 4, 6, 8])

# Insert in middle
a = [2, 4, 6]
bisect.insort_left(a, 5)
test("insort_left in middle", a == [2, 4, 5, 6])

# Insert duplicate - goes to left of existing
a = [2, 4, 4, 6]
bisect.insort_left(a, 4)
test("insort_left duplicate", a == [2, 4, 4, 4, 6] and a[1] == 4)


# ============================================================================
# insort_right tests
# ============================================================================

print("\n=== insort_right tests ===")

# Empty list
a = []
bisect.insort_right(a, 5)
test("insort_right empty", a == [5])

# Insert at start
a = [2, 4, 6]
bisect.insort_right(a, 1)
test("insort_right at start", a == [1, 2, 4, 6])

# Insert at end
a = [2, 4, 6]
bisect.insort_right(a, 8)
test("insort_right at end", a == [2, 4, 6, 8])

# Insert in middle
a = [2, 4, 6]
bisect.insort_right(a, 5)
test("insort_right in middle", a == [2, 4, 5, 6])

# Insert duplicate - goes to right of existing
a = [2, 4, 4, 6]
bisect.insort_right(a, 4)
test("insort_right duplicate", a == [2, 4, 4, 4, 6] and a[3] == 4)


# ============================================================================
# insort alias tests
# ============================================================================

print("\n=== insort alias tests ===")

# insort is alias for insort_right
a = [2, 4, 6]
b = [2, 4, 6]
bisect.insort(a, 5)
bisect.insort_right(b, 5)
test("insort is insort_right", a == b)


# ============================================================================
# lo/hi bounds tests
# ============================================================================

print("\n=== lo/hi bounds tests ===")

a = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Test with lo
test("bisect_left lo", bisect.bisect_left(a, 3, lo=5) == 5)
test("bisect_right lo", bisect.bisect_right(a, 3, lo=5) == 5)

# Test with hi
test("bisect_left hi", bisect.bisect_left(a, 7, hi=5) == 5)
test("bisect_right hi", bisect.bisect_right(a, 7, hi=5) == 5)

# Test with both lo and hi
test("bisect_left lo+hi", bisect.bisect_left(a, 5, lo=2, hi=8) == 5)
test("bisect_right lo+hi", bisect.bisect_right(a, 5, lo=2, hi=8) == 6)

# insort with bounds
a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
bisect.insort_left(a, 5, lo=6)
test("insort_left with lo", a == [1, 2, 3, 4, 5, 6, 5, 7, 8, 9, 10])

a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
bisect.insort_right(a, 5, hi=3)
test("insort_right with hi", a == [1, 2, 3, 5, 4, 5, 6, 7, 8, 9, 10])


# ============================================================================
# key function tests
# ============================================================================
# NOTE: key function tests skipped - PocketPy bisect doesn't support key parameter

# print("\n=== key function tests ===")
# Skip these tests - key parameter not supported in PocketPy's bisect
skip("bisect_left with key", "key parameter not supported")
skip("bisect_right with key", "key parameter not supported")
skip("insort_left with key", "key parameter not supported")
skip("insort_right with key", "key parameter not supported")
skip("bisect_left reverse key", "key parameter not supported")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Large list
a = list(range(1000))
test("large list start", bisect.bisect_left(a, -1) == 0)
test("large list end", bisect.bisect_right(a, 1000) == 1000)
test("large list middle", bisect.bisect(a, 500) == 501)

# Floats
a = [0.1, 0.2, 0.3, 0.4, 0.5]
test("float bisect", bisect.bisect(a, 0.25) == 2)

# Strings
a = ["a", "c", "e", "g"]
test("string bisect", bisect.bisect(a, "d") == 2)

# Mixed types that are comparable
a = [1, 2.0, 3, 4.0, 5]
test("mixed int/float", bisect.bisect(a, 2.5) == 2)


# ============================================================================
# Practical usage examples
# ============================================================================

print("\n=== Practical usage examples ===")


# Grade calculation example from Python docs
# Note: PocketPy doesn't support mutable default args, so we use None pattern
def grade(score, breakpoints=None, grades="FDCBA"):
    if breakpoints is None:
        breakpoints = [60, 70, 80, 90]
    i = bisect.bisect(breakpoints, score)
    return grades[i]


test("grade F", grade(55) == "F")
test("grade D", grade(65) == "D")
test("grade C", grade(75) == "C")
test("grade B", grade(85) == "B")
test("grade A", grade(95) == "A")

# Building a sorted list
a = []
for x in [5, 1, 8, 3, 7, 2]:
    bisect.insort(a, x)
test("build sorted list", a == [1, 2, 3, 5, 7, 8])

# Finding range of values
a = [1, 2, 2, 2, 3, 4, 4, 5]
left = bisect.bisect_left(a, 2)
right = bisect.bisect_right(a, 2)
test("find range count", right - left == 3)
test("find range values", a[left:right] == [2, 2, 2])


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
