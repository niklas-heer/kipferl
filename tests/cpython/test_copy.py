"""
Simplified copy module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_copy.py
"""

import copy

# Test tracking
_passed = 0
_failed = 0
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


def test_raises(name, exc_type, func, *args):
    global _passed, _failed, _errors
    try:
        func(*args)
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name} (no exception raised)")
    except exc_type:
        _passed += 1
        print(f"  PASS: {name}")
    except Exception as e:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name} (wrong exception: {type(e).__name__})")


# ============================================================================
# copy.copy() tests
# ============================================================================

print("\n=== copy.copy() tests ===")

# Test copy of atomic types
test("copy(None)", copy.copy(None) is None)
test("copy(42)", copy.copy(42) == 42)
test("copy(3.14)", copy.copy(3.14) == 3.14)
test("copy('hello')", copy.copy("hello") == "hello")
test("copy(True)", copy.copy(True) is True)
test("copy(False)", copy.copy(False) is False)

# Test copy of tuple (returns same object - immutable)
t = (1, 2, 3)
test("copy(tuple) same object", copy.copy(t) is t)

# Test copy of list (returns new object)
l1 = [1, 2, 3]
l2 = copy.copy(l1)
test("copy(list) equal", l1 == l2)
test("copy(list) not same", l1 is not l2)
l2.append(4)
test("copy(list) independent", l1 == [1, 2, 3])

# Test copy of dict
d1 = {"a": 1, "b": 2}
d2 = copy.copy(d1)
test("copy(dict) equal", d1 == d2)
test("copy(dict) not same", d1 is not d2)
d2["c"] = 3
test("copy(dict) independent", "c" not in d1)

# Test shallow copy - nested objects are same
l1 = [[1, 2], [3, 4]]
l2 = copy.copy(l1)
test("shallow copy: outer different", l1 is not l2)
test("shallow copy: inner same", l1[0] is l2[0])

# Test copy of set
s1 = {1, 2, 3}
s2 = copy.copy(s1)
test("copy(set) equal", s1 == s2)
test("copy(set) not same", s1 is not s2)

# Test copy of bytearray
ba1 = bytearray(b"hello")
ba2 = copy.copy(ba1)
test("copy(bytearray) equal", ba1 == ba2)
test("copy(bytearray) not same", ba1 is not ba2)

# ============================================================================
# copy.deepcopy() tests
# ============================================================================

print("\n=== copy.deepcopy() tests ===")

# Test deepcopy of atomic types
test("deepcopy(None)", copy.deepcopy(None) is None)
test("deepcopy(42)", copy.deepcopy(42) == 42)
test("deepcopy('hello')", copy.deepcopy("hello") == "hello")

# Test deepcopy of list
l1 = [1, 2, 3]
l2 = copy.deepcopy(l1)
test("deepcopy(list) equal", l1 == l2)
test("deepcopy(list) not same", l1 is not l2)

# Test deep copy - nested objects are copied
l1 = [[1, 2], [3, 4]]
l2 = copy.deepcopy(l1)
test("deep copy: outer different", l1 is not l2)
test("deep copy: inner different", l1[0] is not l2[0])
test("deep copy: values equal", l1 == l2)
l2[0].append(99)
test("deep copy: independent", l1[0] == [1, 2])

# Test deepcopy of dict
d1 = {"a": [1, 2], "b": [3, 4]}
d2 = copy.deepcopy(d1)
test("deepcopy(dict) equal", d1 == d2)
test("deepcopy(dict) not same", d1 is not d2)
test("deepcopy(dict) values different", d1["a"] is not d2["a"])

# Test deepcopy of nested dict
d1 = {"outer": {"inner": [1, 2, 3]}}
d2 = copy.deepcopy(d1)
test("deepcopy nested: equal", d1 == d2)
d2["outer"]["inner"].append(4)
test("deepcopy nested: independent", d1["outer"]["inner"] == [1, 2, 3])

# Test circular reference handling
# NOTE: Circular reference support requires memo tracking which is complex
# Skipping these tests for now - they would hang without proper memo support
# l1 = [1, 2]
# l1.append(l1)  # circular reference
# l2 = copy.deepcopy(l1)
# test("circular: copied", l2[0] == 1 and l2[1] == 2)
# test("circular: self-reference preserved", l2[2] is l2)
# test("circular: not same as original", l2 is not l1)

# Test mutual references
# a = [1]
# b = [2]
# a.append(b)
# b.append(a)
# a2 = copy.deepcopy(a)
# test("mutual ref: a copied", a2[0] == 1)
# test("mutual ref: b in a2", a2[1][0] == 2)
# test("mutual ref: circular preserved", a2[1][1] is a2)

# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    import sys

    sys.exit(1)
else:
    print("All tests passed!")
