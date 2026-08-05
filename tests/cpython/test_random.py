"""
Simplified random module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_random.py
"""

import random
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


# ============================================================================
# random.random() tests
# ============================================================================

print("\n=== random.random() tests ===")

# Basic functionality
r = random.random()
test("random returns float", isinstance(r, float))
test("random in range [0, 1)", 0 <= r < 1)

# Multiple calls return different values (usually)
values = [random.random() for _ in range(100)]
test("random produces variety", len(set(values)) > 50)

# All in range - use list comprehension instead of generator expression
test("random all in range", all([0 <= v < 1 for v in values]))


# ============================================================================
# random.randint() tests
# ============================================================================

print("\n=== random.randint() tests ===")

# Basic range
r = random.randint(1, 10)
test("randint returns int", isinstance(r, int))
test("randint in range", 1 <= r <= 10)

# Test range limits
values = [random.randint(1, 3) for _ in range(100)]
test("randint covers range", set(values) == {1, 2, 3})

# Single value range
test("randint single value", random.randint(5, 5) == 5)

# Negative range
r = random.randint(-10, -5)
test("randint negative range", -10 <= r <= -5)

# Mixed range
r = random.randint(-5, 5)
test("randint mixed range", -5 <= r <= 5)


# ============================================================================
# random.randrange() tests
# ============================================================================

print("\n=== random.randrange() tests ===")

if hasattr(random, "randrange"):
    # Single argument (0 to n-1)
    r = random.randrange(10)
    test("randrange single arg", 0 <= r < 10)

    # Two arguments (start to stop-1)
    r = random.randrange(5, 10)
    test("randrange two args", 5 <= r < 10)

    # Three arguments (start, stop, step)
    values = [random.randrange(0, 10, 2) for _ in range(50)]
    test("randrange step", all([v % 2 == 0 for v in values]))
    test("randrange step range", all([0 <= v < 10 for v in values]))
else:
    skip("randrange single arg", "randrange not available")
    skip("randrange two args", "randrange not available")
    skip("randrange step", "randrange not available")
    skip("randrange step range", "randrange not available")


# ============================================================================
# random.choice() tests
# ============================================================================

print("\n=== random.choice() tests ===")

# Basic choice
seq = [1, 2, 3, 4, 5]
c = random.choice(seq)
test("choice from list", c in seq)

# Choice from string - PocketPy only supports list/tuple
s = "abcdef"
try:
    c = random.choice(s)
    test("choice from string", c in s)
except TypeError:
    skip("choice from string", "choice only supports list/tuple")

# Choice covers all options (probability)
choices = [random.choice([1, 2, 3]) for _ in range(100)]
test("choice covers options", len(set(choices)) == 3)

# Single element
test("choice single element", random.choice([42]) == 42)

# Empty sequence raises
try:
    random.choice([])
    test("choice empty raises", False)
except IndexError:
    test("choice empty raises", True)
except ValueError:
    test("choice empty raises", True)


# ============================================================================
# random.shuffle() tests
# ============================================================================

print("\n=== random.shuffle() tests ===")

# Basic shuffle
original = [1, 2, 3, 4, 5]
shuffled = original.copy()
random.shuffle(shuffled)
test("shuffle same elements", sorted(shuffled) == sorted(original))
test("shuffle same length", len(shuffled) == len(original))

# Multiple shuffles produce different orders (usually)
results = []
for _ in range(10):
    lst = [1, 2, 3, 4, 5]
    random.shuffle(lst)
    results.append(tuple(lst))
test("shuffle produces variety", len(set(results)) > 1)

# Single element
lst = [42]
random.shuffle(lst)
test("shuffle single element", lst == [42])

# Empty list
lst = []
random.shuffle(lst)
test("shuffle empty", lst == [])


# ============================================================================
# random.sample() tests
# ============================================================================

print("\n=== random.sample() tests ===")

if hasattr(random, "sample"):
    # Basic sample
    population = [1, 2, 3, 4, 5]
    s = random.sample(population, 3)
    test("sample returns list", isinstance(s, list))
    test("sample correct length", len(s) == 3)
    test("sample unique elements", len(set(s)) == 3)
    test("sample from population", all([x in population for x in s]))

    # Sample entire population
    s = random.sample(population, 5)
    test("sample all", sorted(s) == sorted(population))

    # Sample zero elements
    s = random.sample(population, 0)
    test("sample zero", s == [])

    # Sample doesn't modify original
    original = [1, 2, 3, 4, 5]
    random.sample(original, 3)
    test("sample preserves original", original == [1, 2, 3, 4, 5])

    # Sample too many raises
    try:
        random.sample([1, 2, 3], 5)
        test("sample too many raises", False)
    except ValueError:
        test("sample too many raises", True)
else:
    skip("sample returns list", "sample not available")
    skip("sample correct length", "sample not available")
    skip("sample unique elements", "sample not available")
    skip("sample from population", "sample not available")
    skip("sample all", "sample not available")
    skip("sample zero", "sample not available")
    skip("sample preserves original", "sample not available")
    skip("sample too many raises", "sample not available")


# ============================================================================
# random.uniform() tests
# ============================================================================

print("\n=== random.uniform() tests ===")

# Basic uniform
r = random.uniform(1.0, 10.0)
test("uniform returns float", isinstance(r, float))
test("uniform in range", 1.0 <= r <= 10.0)

# Multiple values cover range
values = [random.uniform(0.0, 1.0) for _ in range(100)]
test("uniform variety", max(values) > 0.9 and min(values) < 0.1)

# Negative range
r = random.uniform(-10.0, -5.0)
test("uniform negative", -10.0 <= r <= -5.0)

# Reversed range (should still work)
r = random.uniform(10.0, 1.0)
test("uniform reversed", 1.0 <= r <= 10.0)


# ============================================================================
# random.seed() tests
# ============================================================================

print("\n=== random.seed() tests ===")

# Seeding produces reproducible results
random.seed(12345)
seq1 = [random.random() for _ in range(10)]

random.seed(12345)
seq2 = [random.random() for _ in range(10)]

test("seed reproducible", seq1 == seq2)

# Different seeds produce different results
random.seed(12345)
val1 = random.random()

random.seed(54321)
val2 = random.random()

test("different seeds differ", val1 != val2)


# ============================================================================
# random.getrandbits() tests
# ============================================================================

print("\n=== random.getrandbits() tests ===")

if hasattr(random, "getrandbits"):
    # 8 bits
    r = random.getrandbits(8)
    test("getrandbits 8 range", 0 <= r < 256)

    # 16 bits
    r = random.getrandbits(16)
    test("getrandbits 16 range", 0 <= r < 65536)

    # 1 bit
    values = [random.getrandbits(1) for _ in range(100)]
    test("getrandbits 1 values", set(values) == {0, 1})

    # 0 bits
    test("getrandbits 0", random.getrandbits(0) == 0)
else:
    skip("getrandbits 8 range", "getrandbits not available")
    skip("getrandbits 16 range", "getrandbits not available")
    skip("getrandbits 1 values", "getrandbits not available")
    skip("getrandbits 0", "getrandbits not available")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Large ranges for randint
r = random.randint(0, 10**9)
test("randint large range", 0 <= r <= 10**9)

# Many random calls
values = [random.random() for _ in range(1000)]
test("many random calls", len(values) == 1000)
test("all valid values", all([0 <= v < 1 for v in values]))


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
