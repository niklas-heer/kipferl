"""
Minimal dataclasses tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.
"""

import sys

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
    import dataclasses

    HAS_DATACLASSES = True
except ImportError:
    HAS_DATACLASSES = False
    print("SKIP: dataclasses module not available")

if HAS_DATACLASSES:
    print("\n=== dataclass decorator ===")

    @dataclasses.dataclass
    class Point:
        x: int
        y: int = 2

    p = Point(1)
    test("dataclass creates __init__", hasattr(Point, "__init__"))
    test("dataclass field assignment", p.x == 1 and p.y == 2)
    test("is_dataclass(class)", dataclasses.is_dataclass(Point) is True)
    test("is_dataclass(instance)", dataclasses.is_dataclass(p) is True)
    test("has __dataclass_fields__", hasattr(Point, "__dataclass_fields__"))

    print("\n=== repr/eq basics ===")
    p2 = Point(1, 2)
    test("dataclass equality", p == p2)
    r = repr(p)
    test("repr contains class name", "Point" in r)
    test("repr contains x", "x" in r)

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
