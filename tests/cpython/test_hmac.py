"""
Minimal hmac module tests for kipferl compatibility testing.
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
    import hmac

    HAS_HMAC = True
except ImportError:
    HAS_HMAC = False
    print("SKIP: hmac module not available")

if HAS_HMAC:
    print("\n=== hmac.new ===")
    key = b"secret-key"
    msg = b"hello world"

    try:
        h = hmac.new(key, msg, "sha256")
        hx = h.hexdigest()
        test("hexdigest returns str", isinstance(hx, str))
        test("hexdigest length", len(hx) == 64)
    except Exception as e:
        test("hmac.new works", False)
        print(f"  ERROR: {e}")

    print("\n=== compare_digest ===")
    try:
        test("compare_digest true", hmac.compare_digest("a", "a") is True)
        test("compare_digest false", hmac.compare_digest("a", "b") is False)
    except Exception as e:
        test("compare_digest works", False)
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
