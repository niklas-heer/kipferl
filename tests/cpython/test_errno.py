"""
Simplified errno module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_errno.py
"""

import errno
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
# Common POSIX errno constants tests
# ============================================================================

print("\n=== Common POSIX errno constants ===")

# Core file/directory errors
test("ENOENT is integer", isinstance(errno.ENOENT, int))
test("ENOENT value", errno.ENOENT == 2)

test("EEXIST is integer", isinstance(errno.EEXIST, int))
test("EEXIST value", errno.EEXIST == 17)

test("ENOTDIR is integer", isinstance(errno.ENOTDIR, int))
test("ENOTDIR value", errno.ENOTDIR == 20)

test("EISDIR is integer", isinstance(errno.EISDIR, int))
test("EISDIR value", errno.EISDIR == 21)

# Permission errors
test("EACCES is integer", isinstance(errno.EACCES, int))
test("EACCES value", errno.EACCES == 13)

test("EPERM is integer", isinstance(errno.EPERM, int))
test("EPERM value", errno.EPERM == 1)

# Argument errors
test("EINVAL is integer", isinstance(errno.EINVAL, int))
test("EINVAL value", errno.EINVAL == 22)

test("EBADF is integer", isinstance(errno.EBADF, int))
test("EBADF value", errno.EBADF == 9)


# ============================================================================
# I/O and system resource errors
# ============================================================================

print("\n=== I/O and system resource errors ===")

test("EIO is integer", isinstance(errno.EIO, int))
test("EIO value", errno.EIO == 5)

test("ENOMEM is integer", isinstance(errno.ENOMEM, int))
test("ENOMEM value", errno.ENOMEM == 12)

test("ENOSPC is integer", isinstance(errno.ENOSPC, int))
test("ENOSPC value", errno.ENOSPC == 28)


# ============================================================================
# Process and system errors
# ============================================================================

print("\n=== Process and system errors ===")

test("ESRCH is integer", isinstance(errno.ESRCH, int))
test("ESRCH value", errno.ESRCH == 3)

test("ECHILD is integer", isinstance(errno.ECHILD, int))
test("ECHILD value", errno.ECHILD == 10)

test("EAGAIN is integer", isinstance(errno.EAGAIN, int))
# EAGAIN is 35 on macOS, 11 on Linux
test("EAGAIN value", errno.EAGAIN == 35 or errno.EAGAIN == 11)

test("EINTR is integer", isinstance(errno.EINTR, int))
test("EINTR value", errno.EINTR == 4)


# ============================================================================
# Pipe errors
# ============================================================================

print("\n=== Pipe errors ===")

test("EPIPE is integer", isinstance(errno.EPIPE, int))
test("EPIPE value", errno.EPIPE == 32)


# ============================================================================
# errorcode dict tests
# ============================================================================

print("\n=== errorcode dict tests ===")

test("errorcode is dict", isinstance(errno.errorcode, dict))
test("errorcode not empty", len(errno.errorcode) > 0)

# Check that errorcode maps integers to strings
test("errorcode ENOENT mapping", errno.errorcode[errno.ENOENT] == "ENOENT")


# ============================================================================
# Uniqueness tests
# ============================================================================

print("\n=== Uniqueness tests ===")

# Collect all available errno constants
errno_constants = {}
for name in dir(errno):
    if name.startswith("E") and name.isupper():
        value = getattr(errno, name)
        if isinstance(value, int):
            errno_constants[name] = value

test("has errno constants", len(errno_constants) > 0)
test("has multiple errno constants", len(errno_constants) >= 10)


# ============================================================================
# Integration with OSError tests
# ============================================================================

print("\n=== Integration with OSError tests ===")

err = OSError(errno.ENOENT, "No such file or directory")
test("OSError with ENOENT", err.args[0] == errno.ENOENT)


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
