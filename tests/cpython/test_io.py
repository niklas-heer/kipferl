"""
Simplified io module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_io.py
"""

import io
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
# io.BytesIO tests
# ============================================================================

print("\n=== io.BytesIO tests ===")

# Create empty BytesIO
b = io.BytesIO()
test("BytesIO empty", b.getvalue() == b"")

# Create with initial data
b = io.BytesIO(b"hello")
test("BytesIO initial data", b.getvalue() == b"hello")

# Write to BytesIO
b = io.BytesIO()
b.write(b"hello")
test("BytesIO write", b.getvalue() == b"hello")

# Multiple writes
b = io.BytesIO()
b.write(b"hel")
b.write(b"lo")
test("BytesIO multiple writes", b.getvalue() == b"hello")

# Read from BytesIO
b = io.BytesIO(b"hello")
test("BytesIO read all", b.read() == b"hello")

b = io.BytesIO(b"hello")
test("BytesIO read partial", b.read(3) == b"hel")
test("BytesIO read rest", b.read() == b"lo")

# Seek and tell
b = io.BytesIO(b"hello")
test("BytesIO tell start", b.tell() == 0)
b.read(2)
test("BytesIO tell after read", b.tell() == 2)
b.seek(0)
test("BytesIO seek start", b.tell() == 0)
b.seek(3)
test("BytesIO seek middle", b.tell() == 3)
test("BytesIO read after seek", b.read() == b"lo")

# Seek modes
b = io.BytesIO(b"hello")
b.seek(0, 2)  # Seek to end
test("BytesIO seek end", b.tell() == 5)
b.seek(-2, 2)  # Seek 2 bytes from end
test("BytesIO seek from end", b.tell() == 3)
b.seek(1, 1)  # Seek 1 byte from current
test("BytesIO seek from current", b.tell() == 4)

# getvalue after writes
b = io.BytesIO()
b.write(b"abc")
b.seek(0)
b.write(b"X")
test("BytesIO getvalue after overwrite", b.getvalue() == b"Xbc")

# Close
b = io.BytesIO(b"hello")
b.close()
test("BytesIO closed", b.closed)


# ============================================================================
# io.StringIO tests
# ============================================================================

print("\n=== io.StringIO tests ===")

# Create empty StringIO
s = io.StringIO()
test("StringIO empty", s.getvalue() == "")

# Create with initial data
s = io.StringIO("hello")
test("StringIO initial data", s.getvalue() == "hello")

# Write to StringIO
s = io.StringIO()
s.write("hello")
test("StringIO write", s.getvalue() == "hello")

# Multiple writes
s = io.StringIO()
s.write("hel")
s.write("lo")
test("StringIO multiple writes", s.getvalue() == "hello")

# Read from StringIO
s = io.StringIO("hello")
test("StringIO read all", s.read() == "hello")

s = io.StringIO("hello")
test("StringIO read partial", s.read(3) == "hel")
test("StringIO read rest", s.read() == "lo")

# Seek and tell
s = io.StringIO("hello")
test("StringIO tell start", s.tell() == 0)
s.read(2)
test("StringIO tell after read", s.tell() == 2)
s.seek(0)
test("StringIO seek start", s.tell() == 0)

# readline
s = io.StringIO("line1\nline2\nline3")
test("StringIO readline 1", s.readline() == "line1\n")
test("StringIO readline 2", s.readline() == "line2\n")
test("StringIO readline 3", s.readline() == "line3")

# readlines
s = io.StringIO("line1\nline2\nline3")
lines = s.readlines()
test("StringIO readlines", lines == ["line1\n", "line2\n", "line3"])

# writelines
s = io.StringIO()
s.writelines(["line1\n", "line2\n", "line3"])
test("StringIO writelines", s.getvalue() == "line1\nline2\nline3")

# Close
s = io.StringIO("hello")
s.close()
test("StringIO closed", s.closed)


# ============================================================================
# BytesIO additional tests
# ============================================================================

print("\n=== BytesIO additional tests ===")

# readline
b = io.BytesIO(b"line1\nline2\nline3")
test("BytesIO readline 1", b.readline() == b"line1\n")
test("BytesIO readline 2", b.readline() == b"line2\n")
test("BytesIO readline 3", b.readline() == b"line3")

# readlines
b = io.BytesIO(b"line1\nline2\nline3")
lines = b.readlines()
test("BytesIO readlines", lines == [b"line1\n", b"line2\n", b"line3"])

# writelines
b = io.BytesIO()
b.writelines([b"line1\n", b"line2\n", b"line3"])
test("BytesIO writelines", b.getvalue() == b"line1\nline2\nline3")

# truncate
b = io.BytesIO(b"hello world")
b.truncate(5)
test("BytesIO truncate", b.getvalue() == b"hello")

# Read empty
b = io.BytesIO(b"")
test("BytesIO read empty", b.read() == b"")

# Read beyond end
b = io.BytesIO(b"hi")
b.read()
test("BytesIO read at end", b.read() == b"")


# ============================================================================
# Context manager tests
# ============================================================================

print("\n=== Context manager tests ===")

# BytesIO as context manager
with io.BytesIO(b"hello") as b:
    data = b.read()
test("BytesIO context manager read", data == b"hello")
test("BytesIO context manager closed", b.closed)

# StringIO as context manager
with io.StringIO("hello") as s:
    data = s.read()
test("StringIO context manager read", data == "hello")
test("StringIO context manager closed", s.closed)


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Empty operations
b = io.BytesIO()
test("BytesIO read empty buffer", b.read() == b"")
test("BytesIO readline empty buffer", b.readline() == b"")
test("BytesIO readlines empty buffer", b.readlines() == [])

# Large data
large_data = b"x" * 10000
b = io.BytesIO(large_data)
test("BytesIO large data", b.read() == large_data)

# Binary data with null bytes
binary_data = b"\x00\x01\x02\x00\x03"
b = io.BytesIO(binary_data)
test("BytesIO binary with nulls", b.read() == binary_data)

# Seek past end
b = io.BytesIO(b"hello")
b.seek(100)
test("BytesIO seek past end", b.tell() == 100)
test("BytesIO read after seek past end", b.read() == b"")

# Write after seek past end
b = io.BytesIO(b"hello")
b.seek(10)
b.write(b"X")
result = b.getvalue()
test("BytesIO write past end length", len(result) == 11)


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
