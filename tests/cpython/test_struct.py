"""
Simplified struct module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_struct.py
"""

import struct
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
# struct.pack() basic types
# ============================================================================

print("\n=== struct.pack() basic types ===")

# Byte (b/B)
test("pack byte b", struct.pack("b", -1) == b"\xff")
test("pack byte B", struct.pack("B", 255) == b"\xff")
test("pack byte B 0", struct.pack("B", 0) == b"\x00")

# Short (h/H)
test("pack short h", len(struct.pack("h", 1)) == 2)
test("pack short H", len(struct.pack("H", 1)) == 2)

# Int (i/I)
test("pack int i", len(struct.pack("i", 1)) == 4)
test("pack int I", len(struct.pack("I", 1)) == 4)

# Long (l/L)
test("pack long l", len(struct.pack("l", 1)) >= 4)
test("pack long L", len(struct.pack("L", 1)) >= 4)

# Long long (q/Q)
test("pack longlong q", len(struct.pack("q", 1)) == 8)
test("pack longlong Q", len(struct.pack("Q", 1)) == 8)

# Float (f)
packed_f = struct.pack("f", 3.14)
test("pack float f", len(packed_f) == 4)

# Double (d)
packed_d = struct.pack("d", 3.14)
test("pack double d", len(packed_d) == 8)


# ============================================================================
# struct.unpack() basic types
# ============================================================================

print("\n=== struct.unpack() basic types ===")

# Byte
test("unpack byte b", struct.unpack("b", b"\xff")[0] == -1)
test("unpack byte B", struct.unpack("B", b"\xff")[0] == 255)
test("unpack byte B 0", struct.unpack("B", b"\x00")[0] == 0)

# Short
packed = struct.pack("h", 12345)
test("unpack short h", struct.unpack("h", packed)[0] == 12345)

packed = struct.pack("H", 65535)
test("unpack short H", struct.unpack("H", packed)[0] == 65535)

# Int
packed = struct.pack("i", 123456789)
test("unpack int i", struct.unpack("i", packed)[0] == 123456789)

packed = struct.pack("I", 4294967295)
test("unpack int I", struct.unpack("I", packed)[0] == 4294967295)

# Float roundtrip
packed = struct.pack("f", 3.14)
unpacked = struct.unpack("f", packed)[0]
test("unpack float f", abs(unpacked - 3.14) < 0.001)

# Double roundtrip
packed = struct.pack("d", 3.14159265358979)
unpacked = struct.unpack("d", packed)[0]
test("unpack double d", abs(unpacked - 3.14159265358979) < 1e-10)


# ============================================================================
# Multiple values
# ============================================================================

print("\n=== Multiple values ===")

# Pack multiple
packed = struct.pack("bbb", 1, 2, 3)
test("pack multiple bytes", packed == b"\x01\x02\x03")

unpacked = struct.unpack("bbb", packed)
test("unpack multiple bytes", unpacked == (1, 2, 3))

# Mixed types
packed = struct.pack("bhd", 1, 1000, 3.14)
unpacked = struct.unpack("bhd", packed)
test("pack/unpack mixed", unpacked[0] == 1 and unpacked[1] == 1000)
test("pack/unpack mixed float", abs(unpacked[2] - 3.14) < 0.001)


# ============================================================================
# Byte order
# ============================================================================

print("\n=== Byte order ===")

# Native (@ or =)
native = struct.pack("=I", 0x12345678)
test("native byte order", len(native) == 4)

# Little endian (<)
little = struct.pack("<I", 0x12345678)
test("little endian", little == b"\x78\x56\x34\x12")

# Big endian (>)
big = struct.pack(">I", 0x12345678)
test("big endian", big == b"\x12\x34\x56\x78")

# Network byte order (!)
network = struct.pack("!I", 0x12345678)
test("network byte order", network == b"\x12\x34\x56\x78")

# Unpack with byte order
test("unpack little endian", struct.unpack("<I", b"\x78\x56\x34\x12")[0] == 0x12345678)
test("unpack big endian", struct.unpack(">I", b"\x12\x34\x56\x78")[0] == 0x12345678)


# ============================================================================
# struct.calcsize()
# ============================================================================

print("\n=== struct.calcsize() ===")

test("calcsize b", struct.calcsize("b") == 1)
test("calcsize B", struct.calcsize("B") == 1)
test("calcsize h", struct.calcsize("h") == 2)
test("calcsize H", struct.calcsize("H") == 2)
test("calcsize i", struct.calcsize("i") == 4)
test("calcsize I", struct.calcsize("I") == 4)
test("calcsize q", struct.calcsize("q") == 8)
test("calcsize Q", struct.calcsize("Q") == 8)
test("calcsize f", struct.calcsize("f") == 4)
test("calcsize d", struct.calcsize("d") == 8)

# Multiple
test("calcsize bbb", struct.calcsize("bbb") == 3)
test("calcsize bhd", struct.calcsize("bhd") >= 11)  # May have padding


# ============================================================================
# Repeat counts
# ============================================================================

print("\n=== Repeat counts ===")

# Pack with count
packed = struct.pack("3b", 1, 2, 3)
test("pack 3b", packed == b"\x01\x02\x03")

# Unpack with count
unpacked = struct.unpack("3b", b"\x01\x02\x03")
test("unpack 3b", unpacked == (1, 2, 3))

# calcsize with count
test("calcsize 3b", struct.calcsize("3b") == 3)
test("calcsize 4I", struct.calcsize("4I") == 16)


# ============================================================================
# Padding (x)
# ============================================================================

print("\n=== Padding ===")

# Pack with padding
packed = struct.pack("bxb", 1, 2)
test("pack with padding", len(packed) == 3)
test("pack padding value", packed[1:2] == b"\x00")

# Unpack with padding
unpacked = struct.unpack("bxb", b"\x01\x00\x02")
test("unpack with padding", unpacked == (1, 2))

# calcsize with padding
test("calcsize bxb", struct.calcsize("bxb") == 3)


# ============================================================================
# Struct object
# ============================================================================

print("\n=== Struct object ===")

# Create Struct object
s = struct.Struct("bhd")
test("Struct object", s is not None)
test("Struct size", s.size >= 11)

# Pack with Struct
packed = s.pack(1, 1000, 3.14)
test("Struct pack", len(packed) == s.size)

# Unpack with Struct
unpacked = s.unpack(packed)
test("Struct unpack", unpacked[0] == 1 and unpacked[1] == 1000)


# ============================================================================
# pack_into() and unpack_from()
# ============================================================================

print("\n=== pack_into/unpack_from ===")

buffer = bytearray(10)
struct.pack_into("bb", buffer, 0, 1, 2)
test("pack_into", buffer[0:2] == b"\x01\x02")

struct.pack_into("bb", buffer, 5, 3, 4)
test("pack_into offset", buffer[5:7] == b"\x03\x04")

data = b"\x00\x00\x01\x02\x03"
unpacked = struct.unpack_from("bb", data, 2)
test("unpack_from", unpacked == (1, 2))


# ============================================================================
# Error handling
# ============================================================================

print("\n=== Error handling ===")

# Too few arguments
try:
    struct.pack("bb", 1)
    test("pack too few args raises", False)
except struct.error:
    test("pack too few args raises", True)

# Value out of range
try:
    struct.pack("b", 200)  # b is signed, max 127
    test("pack out of range raises", False)
except struct.error:
    test("pack out of range raises", True)

# Unpack wrong size
try:
    struct.unpack("I", b"\x00\x00")  # Need 4 bytes
    test("unpack wrong size raises", False)
except struct.error:
    test("unpack wrong size raises", True)


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Zero
test("pack zero byte", struct.pack("b", 0) == b"\x00")
test("pack zero int", struct.pack("i", 0) == b"\x00\x00\x00\x00")

# Max values
test("pack max unsigned byte", struct.pack("B", 255) == b"\xff")

# Negative values
test("pack negative byte", struct.pack("b", -128) == b"\x80")

# Empty format
packed = struct.pack("")
test("pack empty format", packed == b"")
test("calcsize empty", struct.calcsize("") == 0)


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
