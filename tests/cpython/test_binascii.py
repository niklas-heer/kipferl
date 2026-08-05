"""
Simplified binascii module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_binascii.py
"""

import binascii
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
# binascii.hexlify() tests
# ============================================================================

print("\n=== binascii.hexlify() tests ===")

test("hexlify empty", binascii.hexlify(b"") == b"")
test("hexlify single byte", binascii.hexlify(b"\x00") == b"00")
test("hexlify ff byte", binascii.hexlify(b"\xff") == b"ff")
test("hexlify hello", binascii.hexlify(b"hello") == b"68656c6c6f")
test("hexlify ascii", binascii.hexlify(b"ABC") == b"414243")
test("hexlify binary", binascii.hexlify(b"\xde\xad\xbe\xef") == b"deadbeef")

# Long strings
long_bytes = bytes(list(range(256)))
long_hex = binascii.hexlify(long_bytes)
test("hexlify long length", len(long_hex) == 512)
test("hexlify long starts", long_hex[:8] == b"00010203")


# ============================================================================
# binascii.unhexlify() tests
# ============================================================================

print("\n=== binascii.unhexlify() tests ===")

test("unhexlify empty", binascii.unhexlify(b"") == b"")
test("unhexlify single", binascii.unhexlify(b"00") == b"\x00")
test("unhexlify ff", binascii.unhexlify(b"ff") == b"\xff")
test("unhexlify FF uppercase", binascii.unhexlify(b"FF") == b"\xff")
test("unhexlify hello", binascii.unhexlify(b"68656c6c6f") == b"hello")
test("unhexlify mixed case", binascii.unhexlify(b"DeAdBeEf") == b"\xde\xad\xbe\xef")

# Roundtrip
test("roundtrip hello", binascii.unhexlify(binascii.hexlify(b"hello")) == b"hello")
test(
    "roundtrip binary", binascii.unhexlify(binascii.hexlify(b"\x00\xff")) == b"\x00\xff"
)

# Error cases
try:
    binascii.unhexlify(b"a")  # Odd-length string
    test("unhexlify odd length raises", False)
except ValueError:
    test("unhexlify odd length raises", True)
except Exception:
    test("unhexlify odd length raises", True)

try:
    binascii.unhexlify(b"gg")  # Invalid hex
    test("unhexlify invalid hex raises", False)
except ValueError:
    test("unhexlify invalid hex raises", True)
except Exception:
    test("unhexlify invalid hex raises", True)


# ============================================================================
# binascii.b2a_base64() tests
# ============================================================================

print("\n=== binascii.b2a_base64() tests ===")

test("b2a_base64 empty", binascii.b2a_base64(b"") in (b"\n", b""))
test("b2a_base64 hello", binascii.b2a_base64(b"hello") == b"aGVsbG8=\n")
test("b2a_base64 a", binascii.b2a_base64(b"a") == b"YQ==\n")
test("b2a_base64 ab", binascii.b2a_base64(b"ab") == b"YWI=\n")
test("b2a_base64 abc", binascii.b2a_base64(b"abc") == b"YWJj\n")

# Known test vectors (from RFC 4648)
test("b2a_base64 f", binascii.b2a_base64(b"f") == b"Zg==\n")
test("b2a_base64 fo", binascii.b2a_base64(b"fo") == b"Zm8=\n")
test("b2a_base64 foo", binascii.b2a_base64(b"foo") == b"Zm9v\n")
test("b2a_base64 foobar", binascii.b2a_base64(b"foobar") == b"Zm9vYmFy\n")


# ============================================================================
# binascii.a2b_base64() tests
# ============================================================================

print("\n=== binascii.a2b_base64() tests ===")

test("a2b_base64 empty", binascii.a2b_base64(b"") == b"")
test("a2b_base64 hello", binascii.a2b_base64(b"aGVsbG8=") == b"hello")
test("a2b_base64 a", binascii.a2b_base64(b"YQ==") == b"a")
test("a2b_base64 ab", binascii.a2b_base64(b"YWI=") == b"ab")
test("a2b_base64 abc", binascii.a2b_base64(b"YWJj") == b"abc")

# With newline
test("a2b_base64 with newline", binascii.a2b_base64(b"aGVsbG8=\n") == b"hello")

# Known test vectors (from RFC 4648)
test("a2b_base64 Zg==", binascii.a2b_base64(b"Zg==") == b"f")
test("a2b_base64 Zm8=", binascii.a2b_base64(b"Zm8=") == b"fo")
test("a2b_base64 Zm9v", binascii.a2b_base64(b"Zm9v") == b"foo")
test("a2b_base64 Zm9vYmFy", binascii.a2b_base64(b"Zm9vYmFy") == b"foobar")


# ============================================================================
# Roundtrip tests
# ============================================================================

print("\n=== Roundtrip tests ===")

test("base64 roundtrip empty", binascii.a2b_base64(binascii.b2a_base64(b"")) == b"")
test(
    "base64 roundtrip hello",
    binascii.a2b_base64(binascii.b2a_base64(b"hello")) == b"hello",
)
test(
    "base64 roundtrip binary",
    binascii.a2b_base64(binascii.b2a_base64(b"\x00\xff")) == b"\x00\xff",
)

# Long roundtrip
long_data = bytes(list(range(256)))
test(
    "base64 roundtrip long",
    binascii.a2b_base64(binascii.b2a_base64(long_data)) == long_data,
)


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Single bytes roundtrip
for i in range(0, 256, 32):
    b = bytes([i])
    test(f"roundtrip byte {i}", binascii.unhexlify(binascii.hexlify(b)) == b)

# Boundary values
test("hexlify boundary 0x0f", binascii.hexlify(b"\x0f") == b"0f")
test("hexlify boundary 0x10", binascii.hexlify(b"\x10") == b"10")
test("hexlify boundary 0x7f", binascii.hexlify(b"\x7f") == b"7f")
test("hexlify boundary 0x80", binascii.hexlify(b"\x80") == b"80")

# b2a_hex alias
test("b2a_hex alias", binascii.b2a_hex(b"hello") == binascii.hexlify(b"hello"))

# a2b_hex alias
test(
    "a2b_hex alias",
    binascii.a2b_hex(b"68656c6c6f") == binascii.unhexlify(b"68656c6c6f"),
)


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
