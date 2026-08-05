"""
Simplified array module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_array.py
"""

import sys
from array import array

# Test tracking
_passed = 0
_failed = 0
_skipped = 0
_errors = []


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


def test_raises(name, exc_type, func, *args, **kwargs):
    global _passed, _failed, _errors
    try:
        func(*args, **kwargs)
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name} (no exception raised)")
    except exc_type:
        _passed += 1
        print(f"  PASS: {name}")
    except Exception as e:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name} (wrong exception: {type(e).__name__}: {e})")


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# ============================================================================
# array creation tests
# ============================================================================

print("\n=== array creation tests ===")

# Create empty arrays with different type codes
for typecode in ["b", "B", "h", "H", "i", "I", "l", "L", "f", "d"]:
    a = array(typecode)
    test(f"create empty array({typecode!r})", len(a) == 0)

# Create arrays from list
a = array("i", [1, 2, 3, 4, 5])
test("create from list", list(a) == [1, 2, 3, 4, 5])

# Create from tuple
a = array("i", (1, 2, 3))
test("create from tuple", list(a) == [1, 2, 3])

# Create float array
a = array("d", [1.5, 2.5, 3.5])
test("create float array", len(a) == 3)

# Test typecode attribute
a = array("i", [1, 2, 3])
test("typecode attribute", a.typecode == "i")

# Test itemsize attribute
a = array("i")
test("itemsize attribute", a.itemsize > 0)


# ============================================================================
# append tests
# ============================================================================

print("\n=== append tests ===")

a = array("i")
a.append(1)
test("append to empty", list(a) == [1])

a.append(2)
a.append(3)
test("append multiple", list(a) == [1, 2, 3])

# Append to float array
a = array("d")
a.append(1.5)
a.append(2.5)
test("append float", len(a) == 2)


# ============================================================================
# extend tests
# ============================================================================

print("\n=== extend tests ===")

a = array("i", [1, 2])
a.extend([3, 4, 5])
test("extend with list", list(a) == [1, 2, 3, 4, 5])

a = array("i", [1, 2])
a.extend((3, 4))
test("extend with tuple", list(a) == [1, 2, 3, 4])

a1 = array("i", [1, 2])
a2 = array("i", [3, 4])
a1.extend(a2)
test("extend with array", list(a1) == [1, 2, 3, 4])


# ============================================================================
# pop tests
# ============================================================================

print("\n=== pop tests ===")

a = array("i", [1, 2, 3, 4, 5])
result = a.pop()
test("pop last", result == 5)
test("pop removes element", list(a) == [1, 2, 3, 4])

result = a.pop(0)
test("pop first", result == 1)
test("pop first removes", list(a) == [2, 3, 4])

# Pop from empty array
a = array("i")
test_raises("pop empty", IndexError, a.pop)


# ============================================================================
# insert tests
# ============================================================================

print("\n=== insert tests ===")

a = array("i", [1, 3])
a.insert(1, 2)
test("insert middle", list(a) == [1, 2, 3])

a = array("i", [2, 3])
a.insert(0, 1)
test("insert start", list(a) == [1, 2, 3])


# ============================================================================
# remove tests
# ============================================================================

print("\n=== remove tests ===")

a = array("i", [1, 2, 3, 2, 4])
a.remove(2)
test("remove first occurrence", list(a) == [1, 3, 2, 4])

# Remove non-existent
a = array("i", [1, 2, 3])
test_raises("remove non-existent", ValueError, a.remove, 99)


# ============================================================================
# index tests
# ============================================================================

print("\n=== index tests ===")

a = array("i", [1, 2, 3, 2, 4])
test("index basic", a.index(2) == 1)
test("index first", a.index(1) == 0)
test("index last element", a.index(4) == 4)

# Index not found
a = array("i", [1, 2, 3])
test_raises("index not found", ValueError, a.index, 99)


# ============================================================================
# count tests
# ============================================================================

print("\n=== count tests ===")

a = array("i", [1, 2, 2, 3, 2, 4])
test("count multiple", a.count(2) == 3)
test("count single", a.count(1) == 1)
test("count zero", a.count(99) == 0)


# ============================================================================
# reverse tests
# ============================================================================

print("\n=== reverse tests ===")

a = array("i", [1, 2, 3, 4, 5])
a.reverse()
test("reverse basic", list(a) == [5, 4, 3, 2, 1])

a = array("i", [1])
a.reverse()
test("reverse single", list(a) == [1])

a = array("i")
a.reverse()
test("reverse empty", list(a) == [])


# ============================================================================
# slicing tests
# ============================================================================

print("\n=== slicing tests ===")

a = array("i", [0, 1, 2, 3, 4, 5])

# Basic slicing
test("slice [:]", list(a[:]) == [0, 1, 2, 3, 4, 5])
test("slice [1:]", list(a[1:]) == [1, 2, 3, 4, 5])
test("slice [:3]", list(a[:3]) == [0, 1, 2])
test("slice [1:4]", list(a[1:4]) == [1, 2, 3])

# Negative indices
test("slice [-1]", a[-1] == 5)
test("slice [-2:]", list(a[-2:]) == [4, 5])

# Index access
a = array("i", [1, 2, 3])
test("index [0]", a[0] == 1)
test("index [1]", a[1] == 2)
test("index [-1]", a[-1] == 3)

# Index assignment
a = array("i", [1, 2, 3])
a[1] = 20
test("index assign", list(a) == [1, 20, 3])

# Index out of range
a = array("i", [1, 2, 3])
test_raises("index out of range", IndexError, lambda: a[10])


# ============================================================================
# iteration tests
# ============================================================================

print("\n=== iteration tests ===")

a = array("i", [1, 2, 3, 4, 5])
result = []
for x in a:
    result.append(x)
test("iteration", result == [1, 2, 3, 4, 5])

# in operator
a = array("i", [1, 2, 3])
test("in operator true", 2 in a)
test("in operator false", 99 not in a)


# ============================================================================
# buffer protocol tests
# ============================================================================

print("\n=== buffer protocol tests ===")

# tobytes
a = array("i", [1, 2, 3])
b = a.tobytes()
test("tobytes returns bytes", isinstance(b, bytes))
test("tobytes length", len(b) == len(a) * a.itemsize)

# tolist
a = array("i", [1, 2, 3])
result = a.tolist()
test("tolist", result == [1, 2, 3])
test("tolist returns list", isinstance(result, list))


# ============================================================================
# operator tests
# ============================================================================

print("\n=== operator tests ===")

# Concatenation with +
a1 = array("i", [1, 2])
a2 = array("i", [3, 4])
a3 = a1 + a2
test("+ concatenation", list(a3) == [1, 2, 3, 4])

# In-place concatenation with +=
a = array("i", [1, 2])
a += array("i", [3, 4])
test("+= in-place", list(a) == [1, 2, 3, 4])

# Repetition with *
a = array("i", [1, 2])
a2 = a * 3
test("* repetition", list(a2) == [1, 2, 1, 2, 1, 2])

# Equality
a1 = array("i", [1, 2, 3])
a2 = array("i", [1, 2, 3])
a3 = array("i", [1, 2, 4])
test("== equal", a1 == a2)
test("!= not equal", a1 != a3)

# Length
a = array("i", [1, 2, 3, 4, 5])
test("len()", len(a) == 5)


# ============================================================================
# edge cases
# ============================================================================

print("\n=== edge cases ===")

# Empty array operations
a = array("i")
test("empty len", len(a) == 0)
test("empty list", list(a) == [])

# Single element array
a = array("i", [42])
test("single element", a[0] == 42)
test("single pop", a.pop() == 42)
test("after pop empty", len(a) == 0)


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
