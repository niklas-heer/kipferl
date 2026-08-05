"""
Simplified base64 module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_base64.py
"""

import base64
import sys

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


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# Check for available functions
has_urlsafe = hasattr(base64, "urlsafe_b64encode")


# ============================================================================
# base64.b64encode() tests
# ============================================================================

print("\n=== base64.b64encode() tests ===")

test("b64encode hello", base64.b64encode(b"Hello") == b"SGVsbG8=")
test("b64encode world", base64.b64encode(b"World") == b"V29ybGQ=")
test("b64encode empty", base64.b64encode(b"") == b"")
test("b64encode single byte", base64.b64encode(b"a") == b"YQ==")
test("b64encode two bytes", base64.b64encode(b"ab") == b"YWI=")
test("b64encode three bytes", base64.b64encode(b"abc") == b"YWJj")


# ============================================================================
# base64.b64decode() tests
# ============================================================================

print("\n=== base64.b64decode() tests ===")

test("b64decode hello", base64.b64decode(b"SGVsbG8=") == b"Hello")
test("b64decode world", base64.b64decode(b"V29ybGQ=") == b"World")
test("b64decode empty", base64.b64decode(b"") == b"")
test("b64decode single", base64.b64decode(b"YQ==") == b"a")


# ============================================================================
# Roundtrip tests
# ============================================================================

print("\n=== Roundtrip tests ===")

original = b"Test roundtrip data 12345"
test("b64 roundtrip", base64.b64decode(base64.b64encode(original)) == original)

all_bytes = bytes(list(range(256)))
encoded_all = base64.b64encode(all_bytes)
test("all bytes roundtrip", base64.b64decode(encoded_all) == all_bytes)


# ============================================================================
# base64.urlsafe_b64encode() tests
# ============================================================================

print("\n=== base64.urlsafe_b64encode() tests ===")

if has_urlsafe:
    test("urlsafe_encode hello", base64.urlsafe_b64encode(b"Hello") == b"SGVsbG8=")
    test("urlsafe_encode minus", base64.urlsafe_b64encode(b"\xfb") == b"-w==")
    test("urlsafe_encode underscore", base64.urlsafe_b64encode(b"\xfc") == b"_A==")
else:
    skip("urlsafe_encode hello", "urlsafe_b64encode not available")
    skip("urlsafe_encode minus", "urlsafe_b64encode not available")
    skip("urlsafe_encode underscore", "urlsafe_b64encode not available")


# ============================================================================
# base64.urlsafe_b64decode() tests
# ============================================================================

print("\n=== base64.urlsafe_b64decode() tests ===")

if has_urlsafe:
    test("urlsafe_decode hello", base64.urlsafe_b64decode(b"SGVsbG8=") == b"Hello")
    test("urlsafe_decode minus", base64.urlsafe_b64decode(b"-w==") == b"\xfb")
    test("urlsafe_decode underscore", base64.urlsafe_b64decode(b"_A==") == b"\xfc")
else:
    skip("urlsafe_decode hello", "urlsafe_b64decode not available")
    skip("urlsafe_decode minus", "urlsafe_b64decode not available")
    skip("urlsafe_decode underscore", "urlsafe_b64decode not available")


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
