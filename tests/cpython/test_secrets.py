"""
Minimal secrets module tests for kipferl compatibility testing.
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
    import secrets

    HAS_SECRETS = True
except ImportError:
    HAS_SECRETS = False
    print("SKIP: secrets module not available")

if HAS_SECRETS:
    print("\n=== token helpers ===")
    b = secrets.token_bytes(16)
    test("token_bytes returns bytes", isinstance(b, (bytes, bytearray)))
    test("token_bytes length", len(b) == 16)

    h = secrets.token_hex(16)
    test("token_hex returns str", isinstance(h, str))
    test("token_hex length", len(h) == 32)

    u = secrets.token_urlsafe(16)
    test("token_urlsafe returns str", isinstance(u, str))
    test("token_urlsafe non-empty", len(u) > 0)

    print("\n=== randbelow/choice ===")
    r = secrets.randbelow(10)
    test("randbelow range", isinstance(r, int) and 0 <= r < 10)

    c = secrets.choice([1, 2, 3])
    test("choice returns element", c in (1, 2, 3))

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
