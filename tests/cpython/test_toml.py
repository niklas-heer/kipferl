"""
Minimal toml module tests for kipferl compatibility testing.
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
    import toml

    HAS_TOML = True
except Exception:
    HAS_TOML = False
    skip("toml import", "module not available (not stdlib on CPython)")

if HAS_TOML:
    print("\n=== toml.loads/dumps/load ===")
    test("has loads", hasattr(toml, "loads") and callable(toml.loads))
    test("has dumps", hasattr(toml, "dumps") and callable(toml.dumps))
    test("has load", hasattr(toml, "load") and callable(toml.load))

    try:
        doc = b'a = 1\n[sec]\nb = "x"\narr = [1, 2]\n'
        d = toml.loads(doc)
        test("loads parses root key", d["a"] == 1)
        test("loads parses table", d["sec"]["b"] == "x")
        test("loads parses array", d["sec"]["arr"][1] == 2)

        s = toml.dumps(d)
        test("dumps returns str", isinstance(s, str))
        test("dumps contains table header", "[sec]" in s)

        path = "__kipferl_toml_test.toml"
        with open(path, "w") as f:
            f.write('a = 2\n[sec]\nb = "y"\n')
        d2 = toml.load(path)
        test("load reads file", d2["a"] == 2 and d2["sec"]["b"] == "y")
        try:
            import os

            os.remove(path)
        except Exception:
            pass
    except Exception as e:
        test("toml operations", False)
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
