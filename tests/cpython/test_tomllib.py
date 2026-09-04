"""
Minimal tomllib tests for kipferl compatibility testing.
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
    import tomllib

    HAS_TOMLLIB = True
except ImportError:
    HAS_TOMLLIB = False
    print("SKIP: tomllib module not available")

if HAS_TOMLLIB:
    print("\n=== tomllib.loads ===")
    data = "a=1\nb='x'\n[tool]\nname='kipferl'\n"
    try:
        obj = tomllib.loads(data)
        test("loads returns dict", isinstance(obj, dict))
        test("parses int", obj.get("a") == 1)
        test("parses string", obj.get("b") == "x")
        test(
            "parses table",
            isinstance(obj.get("tool"), dict) and obj["tool"].get("name") == "kipferl",
        )
    except Exception as e:
        test("loads parses", False)
        print(f"  ERROR: {e}")

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
