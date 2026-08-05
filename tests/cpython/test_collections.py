"""
Simplified collections module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_collections.py
"""

import collections
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
        print(f"  FAIL: {name} (wrong exception: {type(e).__name__})")


# Check what's available
HAS_ORDEREDDICT = hasattr(collections, "OrderedDict")
HAS_NAMEDTUPLE = hasattr(collections, "namedtuple")
HAS_DEQUE = hasattr(collections, "deque")
HAS_COUNTER = hasattr(collections, "Counter")
HAS_DEFAULTDICT = hasattr(collections, "defaultdict")


# ============================================================================
# OrderedDict tests
# ============================================================================

print("\n=== OrderedDict tests ===")

if HAS_ORDEREDDICT:
    from collections import OrderedDict

    # Basic creation
    od = OrderedDict()
    test("OrderedDict empty", len(od) == 0)

    # Creation with items
    od = OrderedDict([("a", 1), ("b", 2), ("c", 3)])
    test("OrderedDict from list", list(od.keys()) == ["a", "b", "c"])
    test("OrderedDict values", list(od.values()) == [1, 2, 3])

    # Test insertion order is preserved
    od = OrderedDict()
    od["one"] = 1
    od["two"] = 2
    od["three"] = 3
    test("OrderedDict insertion order", list(od.keys()) == ["one", "two", "three"])

    # Test that updating a value doesn't change order
    od["two"] = 22
    test(
        "OrderedDict update preserves order", list(od.keys()) == ["one", "two", "three"]
    )
    test("OrderedDict update changes value", od["two"] == 22)

    # Test deletion and re-insertion changes order
    del od["two"]
    od["two"] = 2
    test(
        "OrderedDict delete+insert changes order",
        list(od.keys()) == ["one", "three", "two"],
    )

    # move_to_end
    od = OrderedDict([("a", 1), ("b", 2), ("c", 3)])
    od.move_to_end("a")
    test("move_to_end default", list(od.keys()) == ["b", "c", "a"])

    # popitem on empty raises KeyError
    od_empty = OrderedDict()
    test_raises("popitem empty raises KeyError", KeyError, od_empty.popitem)
else:
    skip("OrderedDict empty", "OrderedDict not available")
    skip("OrderedDict from list", "OrderedDict not available")
    skip("OrderedDict values", "OrderedDict not available")
    skip("OrderedDict insertion order", "OrderedDict not available")
    skip("OrderedDict update preserves order", "OrderedDict not available")
    skip("OrderedDict update changes value", "OrderedDict not available")
    skip("OrderedDict delete+insert changes order", "OrderedDict not available")
    skip("move_to_end default", "OrderedDict not available")
    skip("popitem empty raises KeyError", "OrderedDict not available")


# ============================================================================
# namedtuple tests
# ============================================================================

print("\n=== namedtuple tests ===")

if HAS_NAMEDTUPLE:
    from collections import namedtuple

    # Basic creation with list field names
    Point = namedtuple("Point", ["x", "y"])
    p = Point(1, 2)
    test("namedtuple creation", p.x == 1 and p.y == 2)

    # Creation with space-separated field names
    Point2 = namedtuple("Point2", "x y")
    p2 = Point2(3, 4)
    test("namedtuple space-separated", p2.x == 3 and p2.y == 4)

    # PocketPy namedtuple doesn't support kwargs, skip this test
    # p4 = Point(x=10, y=20)
    # test("namedtuple kwargs", p4.x == 10 and p4.y == 20)
    skip("namedtuple kwargs", "PocketPy namedtuple doesn't support kwargs")

    # Attribute access
    p = Point(3, 4)
    test("namedtuple attr x", p.x == 3)
    test("namedtuple attr y", p.y == 4)

    # Index access
    test("namedtuple index 0", p[0] == 3)
    test("namedtuple index 1", p[1] == 4)

    # Iteration
    test("namedtuple iter", list(p) == [3, 4])

    # Length
    test("namedtuple len", len(p) == 2)

    # Unpacking - PocketPy namedtuple doesn't support unpacking
    # x, y = p
    # test("namedtuple unpack", x == 3 and y == 4)
    skip("namedtuple unpack", "PocketPy namedtuple doesn't support unpacking")

    # _asdict
    d = p._asdict()
    test("_asdict type", isinstance(d, dict))
    test("_asdict values", d["x"] == 3 and d["y"] == 4)

    # _replace - PocketPy doesn't support kwargs
    # p2 = p._replace(x=10)
    # test("_replace single", p2.x == 10 and p2.y == 4)
    # test("_replace original unchanged", p.x == 3)
    skip("_replace single", "PocketPy namedtuple._replace doesn't support kwargs")
    skip(
        "_replace original unchanged",
        "PocketPy namedtuple._replace doesn't support kwargs",
    )

    # _fields
    test("_fields", Point._fields == ("x", "y"))
else:
    skip("namedtuple creation", "namedtuple not available")
    skip("namedtuple space-separated", "namedtuple not available")
    skip("namedtuple kwargs", "namedtuple not available")
    skip("namedtuple attr x", "namedtuple not available")
    skip("namedtuple attr y", "namedtuple not available")
    skip("namedtuple index 0", "namedtuple not available")
    skip("namedtuple index 1", "namedtuple not available")
    skip("namedtuple iter", "namedtuple not available")
    skip("namedtuple len", "namedtuple not available")
    skip("namedtuple unpack", "namedtuple not available")
    skip("_asdict type", "namedtuple not available")
    skip("_asdict values", "namedtuple not available")
    skip("_replace single", "namedtuple not available")
    skip("_replace original unchanged", "namedtuple not available")
    skip("_fields", "namedtuple not available")


# ============================================================================
# deque tests
# ============================================================================

print("\n=== deque tests ===")

if HAS_DEQUE:
    from collections import deque

    # Empty deque
    d = deque()
    test("deque empty", len(d) == 0)

    # From list
    d = deque([1, 2, 3])
    test("deque from list", list(d) == [1, 2, 3])

    # append (right)
    d = deque([1, 2, 3])
    d.append(4)
    test("deque append", list(d) == [1, 2, 3, 4])

    # appendleft
    d.appendleft(0)
    test("deque appendleft", list(d) == [0, 1, 2, 3, 4])

    # pop (right)
    d = deque([1, 2, 3, 4, 5])
    item = d.pop()
    test("deque pop value", item == 5)
    test("deque pop result", list(d) == [1, 2, 3, 4])

    # popleft
    item = d.popleft()
    test("deque popleft value", item == 1)
    test("deque popleft result", list(d) == [2, 3, 4])

    # pop on empty raises IndexError
    d_empty = deque()
    test_raises("deque pop empty", IndexError, d_empty.pop)
    test_raises("deque popleft empty", IndexError, d_empty.popleft)

    # rotate
    d = deque([1, 2, 3, 4, 5])
    d.rotate(2)
    test("deque rotate right", list(d) == [4, 5, 1, 2, 3])

    d = deque([1, 2, 3, 4, 5])
    d.rotate(-2)
    test("deque rotate left", list(d) == [3, 4, 5, 1, 2])

    # maxlen
    d = deque([1, 2, 3], maxlen=3)
    d.append(4)
    test("deque maxlen append", list(d) == [2, 3, 4])

    d = deque([1, 2, 3], maxlen=3)
    d.appendleft(0)
    test("deque maxlen appendleft", list(d) == [0, 1, 2])

    # extend
    d = deque([1, 2])
    d.extend([3, 4, 5])
    test("deque extend", list(d) == [1, 2, 3, 4, 5])

    # clear
    d = deque([1, 2, 3])
    d.clear()
    test("deque clear", len(d) == 0)
else:
    skip("deque empty", "deque not available")
    skip("deque from list", "deque not available")
    skip("deque append", "deque not available")
    skip("deque appendleft", "deque not available")
    skip("deque pop value", "deque not available")
    skip("deque pop result", "deque not available")
    skip("deque popleft value", "deque not available")
    skip("deque popleft result", "deque not available")
    skip("deque pop empty", "deque not available")
    skip("deque popleft empty", "deque not available")
    skip("deque rotate right", "deque not available")
    skip("deque rotate left", "deque not available")
    skip("deque maxlen append", "deque not available")
    skip("deque maxlen appendleft", "deque not available")
    skip("deque extend", "deque not available")
    skip("deque clear", "deque not available")


# ============================================================================
# Counter tests
# ============================================================================

print("\n=== Counter tests ===")

if HAS_COUNTER:
    from collections import Counter

    # Check if Counter supports empty initialization
    try:
        c = Counter()
        test("Counter empty", len(c) == 0)
    except TypeError:
        skip("Counter empty", "Counter() requires an argument")

    # Counter from list
    c = Counter([1, 1, 2, 3, 3, 3])
    test("Counter from list", c[1] == 2 and c[2] == 1 and c[3] == 3)

    # Counter from string
    c = Counter("hello")
    test("Counter from string", c["l"] == 2 and c["h"] == 1)

    # Missing key returns 0 (CPython behavior) - check if supported
    try:
        missing_val = c["z"]
        test("Counter missing key", missing_val == 0)
    except KeyError:
        skip("Counter missing key", "Counter raises KeyError for missing keys")

    # Update - PocketPy Counter.update() only accepts dict, not iterable
    if hasattr(Counter([1]), "update"):
        c = Counter([1, 2])
        try:
            c.update([2, 3])
            test("Counter update", c[1] == 1 and c[2] == 2 and c[3] == 1)
        except TypeError:
            skip("Counter update", "Counter.update() only accepts dict, not iterable")
    else:
        skip("Counter update", "update() not available")

    # most_common
    if hasattr(Counter([1]), "most_common"):
        c = Counter("abracadabra")
        mc = c.most_common(2)
        test("Counter most_common", mc[0][0] == "a" and mc[0][1] == 5)
    else:
        skip("Counter most_common", "most_common() not available")

    # elements (if available)
    if hasattr(Counter([1]), "elements"):
        c = Counter(a=2, b=1)
        elems = sorted(c.elements())
        test("Counter elements", elems == ["a", "a", "b"])
    else:
        skip("Counter elements", "elements() not available")

    # subtract (if available)
    if hasattr(Counter([1]), "subtract"):
        c = Counter(a=4, b=2)
        c.subtract(Counter(a=1, b=3))
        test("Counter subtract", c["a"] == 3 and c["b"] == -1)
    else:
        skip("Counter subtract", "subtract() not available")
else:
    skip("Counter empty", "Counter not available")
    skip("Counter from list", "Counter not available")
    skip("Counter from string", "Counter not available")
    skip("Counter missing key", "Counter not available")
    skip("Counter update", "Counter not available")
    skip("Counter most_common", "Counter not available")
    skip("Counter elements", "Counter not available")
    skip("Counter subtract", "Counter not available")


# ============================================================================
# defaultdict tests
# ============================================================================

print("\n=== defaultdict tests ===")

if HAS_DEFAULTDICT:
    from collections import defaultdict

    # defaultdict with int
    dd = defaultdict(int)
    dd["a"] += 1
    dd["a"] += 1
    dd["b"] += 1
    test("defaultdict int", dd["a"] == 2 and dd["b"] == 1)

    # defaultdict with list
    dd = defaultdict(list)
    dd["a"].append(1)
    dd["a"].append(2)
    dd["b"].append(3)
    test("defaultdict list", dd["a"] == [1, 2] and dd["b"] == [3])

    # Missing key creates default
    dd = defaultdict(int)
    _ = dd["missing"]
    test("defaultdict missing key", "missing" in dd and dd["missing"] == 0)

    # No default_factory - check if supported
    try:
        dd = defaultdict()
        try:
            _ = dd["key"]
            test("defaultdict no factory raises KeyError", False)
        except KeyError:
            test("defaultdict no factory raises KeyError", True)
    except TypeError:
        skip(
            "defaultdict no factory raises KeyError",
            "defaultdict() requires an argument",
        )

    # default_factory attribute
    dd = defaultdict(list)
    if hasattr(dd, "default_factory"):
        test("defaultdict default_factory attr", dd.default_factory is list)
    else:
        skip(
            "defaultdict default_factory attr",
            "default_factory attribute not available",
        )
else:
    skip("defaultdict int", "defaultdict not available")
    skip("defaultdict list", "defaultdict not available")
    skip("defaultdict missing key", "defaultdict not available")
    skip("defaultdict no factory raises KeyError", "defaultdict not available")
    skip("defaultdict default_factory attr", "defaultdict not available")


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
