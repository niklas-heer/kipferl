"""
Simplified functools module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_functools.py
"""

import functools

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


def skip(name):
    """Skip a test."""
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name}")


# Check if reduce supports initial value by testing it
def _reduce_supports_initial():
    try:
        functools.reduce(lambda x, y: x + y, [1, 2], 0)
        return True
    except TypeError:
        return False


_has_reduce_initial = _reduce_supports_initial()
_has_partial = hasattr(functools, "partial")
_has_cmp_to_key = hasattr(functools, "cmp_to_key")
_has_wraps = hasattr(functools, "wraps")
_has_lru_cache = hasattr(functools, "lru_cache")
_has_cache = hasattr(functools, "cache")


# ============================================================================
# functools.reduce() tests
# ============================================================================

print("\n=== functools.reduce() tests ===")

# Basic reduction
test("reduce add", functools.reduce(lambda x, y: x + y, [1, 2, 3, 4]) == 10)
test("reduce multiply", functools.reduce(lambda x, y: x * y, [1, 2, 3, 4]) == 24)

# reduce with initial value (not supported in pocketpy)
if _has_reduce_initial:
    test(
        "reduce with initial", functools.reduce(lambda x, y: x + y, [1, 2, 3], 10) == 16
    )
else:
    skip("reduce with initial")

# Single element
test("reduce single", functools.reduce(lambda x, y: x + y, [5]) == 5)

if _has_reduce_initial:
    test(
        "reduce single with initial",
        functools.reduce(lambda x, y: x + y, [5], 10) == 15,
    )
else:
    skip("reduce single with initial")

# Empty with initial
if _has_reduce_initial:
    test(
        "reduce empty with initial", functools.reduce(lambda x, y: x + y, [], 42) == 42
    )
else:
    skip("reduce empty with initial")

# String concatenation
test("reduce strings", functools.reduce(lambda x, y: x + y, ["a", "b", "c"]) == "abc")

# Max/min
test(
    "reduce max", functools.reduce(lambda x, y: x if x > y else y, [3, 1, 4, 1, 5]) == 5
)
test(
    "reduce min", functools.reduce(lambda x, y: x if x < y else y, [3, 1, 4, 1, 5]) == 1
)


# ============================================================================
# functools.partial() tests
# ============================================================================

print("\n=== functools.partial() tests ===")

if _has_partial:

    def add(a, b, c=0):
        return a + b + c

    # Basic partial
    p = functools.partial(add, 1)
    test("partial basic", p(2) == 3)
    test("partial with remaining", p(2, c=10) == 13)

    # Partial with multiple args
    p2 = functools.partial(add, 1, 2)
    test("partial two args", p2() == 3)
    test("partial two args with kwarg", p2(c=5) == 8)

    # Partial with keyword
    p3 = functools.partial(add, c=10)
    test("partial with kwarg", p3(1, 2) == 13)

    # Nested partial
    p4 = functools.partial(functools.partial(add, 1), 2)
    test("nested partial", p4() == 3)

    # Partial attributes
    test("partial.func", p.func == add)
    test("partial.args", p.args == (1,))
    test("partial.keywords", p.keywords == {})
else:
    skip("partial basic")
    skip("partial with remaining")
    skip("partial two args")
    skip("partial two args with kwarg")
    skip("partial with kwarg")
    skip("nested partial")
    skip("partial.func")
    skip("partial.args")
    skip("partial.keywords")


# ============================================================================
# functools.cmp_to_key() tests
# ============================================================================

print("\n=== functools.cmp_to_key() tests ===")

if _has_cmp_to_key:

    def compare(a, b):
        if a < b:
            return -1
        elif a > b:
            return 1
        return 0

    # Sort with cmp_to_key
    nums = [3, 1, 4, 1, 5, 9, 2, 6]
    sorted_nums = sorted(nums, key=functools.cmp_to_key(compare))
    test("cmp_to_key ascending", sorted_nums == [1, 1, 2, 3, 4, 5, 6, 9])

    # Reverse comparison
    def reverse_compare(a, b):
        return -compare(a, b)

    sorted_desc = sorted(nums, key=functools.cmp_to_key(reverse_compare))
    test("cmp_to_key descending", sorted_desc == [9, 6, 5, 4, 3, 2, 1, 1])

    # String comparison
    words = ["banana", "Apple", "cherry"]

    def case_insensitive_cmp(a, b):
        return compare(a.lower(), b.lower())

    sorted_words = sorted(words, key=functools.cmp_to_key(case_insensitive_cmp))
    test("cmp_to_key strings", sorted_words == ["Apple", "banana", "cherry"])
else:
    skip("cmp_to_key ascending")
    skip("cmp_to_key descending")
    skip("cmp_to_key strings")


# ============================================================================
# functools.wraps() tests
# ============================================================================

print("\n=== functools.wraps() tests ===")

if _has_wraps:

    def original_func():
        """Original docstring."""
        pass

    original_func.__name__ = "original_func"
    original_func.__doc__ = "Original docstring."

    def decorator(f):
        @functools.wraps(f)
        def wrapper(*args, **kwargs):
            return f(*args, **kwargs)

        return wrapper

    decorated = decorator(original_func)
    test("wraps preserves __name__", decorated.__name__ == "original_func")
    test("wraps preserves __doc__", decorated.__doc__ == "Original docstring.")

    # Test wraps with custom function
    def greet(name):
        """Greet someone by name."""
        return f"Hello, {name}!"

    greet.__doc__ = "Greet someone by name."

    @decorator
    def greet_wrapped(name):
        """Greet someone by name."""
        return f"Hello, {name}!"

    greet_wrapped.__doc__ = "Greet someone by name."
    test("wraps on decorated function", greet_wrapped("World") == "Hello, World!")
else:
    skip("wraps preserves __name__")
    skip("wraps preserves __doc__")
    skip("wraps on decorated function")


# ============================================================================
# functools.lru_cache() tests
# ============================================================================

print("\n=== functools.lru_cache() tests ===")

if _has_lru_cache:
    call_count = 0

    @functools.lru_cache(maxsize=32)
    def fibonacci(n):
        global call_count
        call_count += 1
        if n < 2:
            return n
        return fibonacci(n - 1) + fibonacci(n - 2)

    # Test basic functionality
    call_count = 0
    result = fibonacci(10)
    test("lru_cache fibonacci(10)", result == 55)
    test(
        "lru_cache reduces calls", call_count == 11
    )  # Should only call once per unique n

    # Test cache hit
    call_count = 0
    result2 = fibonacci(10)
    test("lru_cache cache hit", result2 == 55)
    test("lru_cache no new calls on hit", call_count == 0)

    # Test cache_info if available
    if hasattr(fibonacci, "cache_info"):
        info = fibonacci.cache_info()
        test("lru_cache cache_info exists", info is not None)
    else:
        skip("lru_cache cache_info exists")

    # Test cache_clear if available
    if hasattr(fibonacci, "cache_clear"):
        fibonacci.cache_clear()
        call_count = 0
        fibonacci(5)
        test("lru_cache cache_clear works", call_count == 6)
    else:
        skip("lru_cache cache_clear works")

    # lru_cache without maxsize (unbounded)
    @functools.lru_cache(maxsize=None)
    def factorial(n):
        if n <= 1:
            return 1
        return n * factorial(n - 1)

    test("lru_cache unbounded", factorial(5) == 120)
    test("lru_cache unbounded larger", factorial(10) == 3628800)

    # lru_cache with no arguments (default maxsize=128)
    @functools.lru_cache()
    def square(x):
        return x * x

    test("lru_cache default maxsize", square(5) == 25)
    test("lru_cache default cached", square(5) == 25)
else:
    skip("lru_cache fibonacci(10)")
    skip("lru_cache reduces calls")
    skip("lru_cache cache hit")
    skip("lru_cache no new calls on hit")
    skip("lru_cache cache_info exists")
    skip("lru_cache cache_clear works")
    skip("lru_cache unbounded")
    skip("lru_cache unbounded larger")
    skip("lru_cache default maxsize")
    skip("lru_cache default cached")


# ============================================================================
# functools.cache() tests (Python 3.9+, alias for lru_cache(maxsize=None))
# ============================================================================

print("\n=== functools.cache() tests ===")

if _has_cache:
    cache_call_count = 0

    @functools.cache
    def cached_double(x):
        global cache_call_count
        cache_call_count += 1
        return x * 2

    cache_call_count = 0
    test("cache basic", cached_double(5) == 10)
    test("cache called once", cache_call_count == 1)

    # Should hit cache
    cached_double(5)
    test("cache hit", cache_call_count == 1)

    # New value should increment
    cached_double(10)
    test("cache new value", cache_call_count == 2)
else:
    skip("cache basic")
    skip("cache called once")
    skip("cache hit")
    skip("cache new value")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Reduce with error on empty
try:
    functools.reduce(lambda x, y: x + y, [])
    test("reduce empty raises", False)
except TypeError:
    test("reduce empty raises", True)

# Partial called with wrong args
if _has_partial:

    def add_for_partial(a, b, c=0):
        return a + b + c

    try:
        p = functools.partial(add_for_partial)
        p()  # Missing required args
        test("partial missing args raises", False)
    except TypeError:
        test("partial missing args raises", True)
else:
    skip("partial missing args raises")


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
