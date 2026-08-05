"""
Simplified itertools module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_itertools.py
"""

import itertools

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


# ============================================================================
# itertools.count() tests
# ============================================================================

print("\n=== itertools.count() tests ===")

# Basic count
c = itertools.count()
test("count() start", next(c) == 0)
test("count() second", next(c) == 1)
test("count() third", next(c) == 2)

# Count with start
c = itertools.count(10)
test("count(10) start", next(c) == 10)
test("count(10) second", next(c) == 11)

# Count with step
c = itertools.count(0, 2)
test("count(0, 2) start", next(c) == 0)
test("count(0, 2) second", next(c) == 2)
test("count(0, 2) third", next(c) == 4)

# Count with negative step
c = itertools.count(10, -1)
test("count(10, -1) start", next(c) == 10)
test("count(10, -1) second", next(c) == 9)


# ============================================================================
# itertools.cycle() tests
# ============================================================================

print("\n=== itertools.cycle() tests ===")

# Basic cycle
cy = itertools.cycle([1, 2, 3])
result = [next(cy) for _ in range(7)]
test("cycle basic", result == [1, 2, 3, 1, 2, 3, 1])

# Cycle string
cy = itertools.cycle("ab")
result = [next(cy) for _ in range(5)]
test("cycle string", result == ["a", "b", "a", "b", "a"])


# ============================================================================
# itertools.repeat() tests
# ============================================================================

print("\n=== itertools.repeat() tests ===")

# Repeat with times
result = list(itertools.repeat(5, 3))
test("repeat(5, 3)", result == [5, 5, 5])

# Repeat zero times
result = list(itertools.repeat(5, 0))
test("repeat(5, 0)", result == [])

# Infinite repeat (take first few)
r = itertools.repeat("x")
result = [next(r) for _ in range(3)]
test("repeat infinite", result == ["x", "x", "x"])


# ============================================================================
# itertools.chain() tests
# ============================================================================

print("\n=== itertools.chain() tests ===")

# Chain lists
result = list(itertools.chain([1, 2], [3, 4], [5]))
test("chain lists", result == [1, 2, 3, 4, 5])

# Chain empty
result = list(itertools.chain([], [1, 2], []))
test("chain with empty", result == [1, 2])

# Chain single
result = list(itertools.chain([1, 2, 3]))
test("chain single", result == [1, 2, 3])

# Chain strings
result = list(itertools.chain("ab", "cd"))
test("chain strings", result == ["a", "b", "c", "d"])


# ============================================================================
# itertools.islice() tests
# ============================================================================

print("\n=== itertools.islice() tests ===")

# islice with stop
result = list(itertools.islice(list(range(10)), 5))
test("islice stop only", result == [0, 1, 2, 3, 4])

# islice with start and stop
result = list(itertools.islice(list(range(10)), 2, 6))
test("islice start stop", result == [2, 3, 4, 5])

# islice with step
result = list(itertools.islice(list(range(10)), 0, 10, 2))
test("islice with step", result == [0, 2, 4, 6, 8])

# islice with start, stop, step
result = list(itertools.islice(list(range(20)), 1, 10, 3))
test("islice start stop step", result == [1, 4, 7])

# islice empty
result = list(itertools.islice(list(range(10)), 0))
test("islice empty", result == [])


# ============================================================================
# itertools.takewhile() tests
# ============================================================================

print("\n=== itertools.takewhile() tests ===")

# Basic takewhile
result = list(itertools.takewhile(lambda x: x < 5, [1, 2, 3, 6, 4, 1]))
test("takewhile basic", result == [1, 2, 3])

# Takewhile all pass
result = list(itertools.takewhile(lambda x: x < 10, [1, 2, 3]))
test("takewhile all pass", result == [1, 2, 3])

# Takewhile none pass
result = list(itertools.takewhile(lambda x: x < 0, [1, 2, 3]))
test("takewhile none pass", result == [])


# ============================================================================
# itertools.dropwhile() tests
# ============================================================================

print("\n=== itertools.dropwhile() tests ===")

# Basic dropwhile
result = list(itertools.dropwhile(lambda x: x < 5, [1, 2, 6, 4, 1]))
test("dropwhile basic", result == [6, 4, 1])

# Dropwhile all dropped
result = list(itertools.dropwhile(lambda x: x < 10, [1, 2, 3]))
test("dropwhile all dropped", result == [])

# Dropwhile none dropped
result = list(itertools.dropwhile(lambda x: x < 0, [1, 2, 3]))
test("dropwhile none dropped", result == [1, 2, 3])


# ============================================================================
# Combined usage
# ============================================================================

print("\n=== Combined usage ===")

# Chain with islice
result = list(itertools.islice(itertools.chain([1, 2], [3, 4, 5]), 4))
test("chain + islice", result == [1, 2, 3, 4])

# Count with takewhile - use list since takewhile needs list
result = list(
    itertools.takewhile(lambda x: x < 5, list(itertools.islice(itertools.count(), 10)))
)
test("count + takewhile", result == [0, 1, 2, 3, 4])

# Cycle with islice
result = list(itertools.islice(itertools.cycle([1, 2]), 5))
test("cycle + islice", result == [1, 2, 1, 2, 1])


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
