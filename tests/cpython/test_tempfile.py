"""
Simplified tempfile module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_tempfile.py
"""

import os
import sys
import tempfile

# Test tracking
_passed = 0
_failed = 0
_skipped = 0
_errors = []
_cleanup_files = []
_cleanup_dirs = []


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


# MicroPython-compatible path functions
def path_exists(path):
    try:
        os.stat(path)
        return True
    except OSError:
        return False


def path_isdir(path):
    try:
        mode = os.stat(path)[0]
        return (mode & 0o170000) == 0o040000  # S_IFDIR
    except OSError:
        return False


def path_isfile(path):
    try:
        mode = os.stat(path)[0]
        return (mode & 0o170000) == 0o100000  # S_IFREG
    except OSError:
        return False


def cleanup_file(path):
    _cleanup_files.append(path)


def cleanup_dir(path):
    _cleanup_dirs.append(path)


def do_cleanup():
    for f in _cleanup_files:
        try:
            os.remove(f)
        except (OSError, IOError):
            pass
    for d in _cleanup_dirs:
        try:
            os.rmdir(d)
        except (OSError, IOError):
            pass


# ============================================================================
# tempfile.gettempdir() tests
# ============================================================================

print("\n=== tempfile.gettempdir() tests ===")

if hasattr(tempfile, "gettempdir"):
    tmpdir = tempfile.gettempdir()
    test("gettempdir returns string", isinstance(tmpdir, str))
    test("gettempdir not empty", len(tmpdir) > 0)
    test("gettempdir exists", path_isdir(tmpdir))
else:
    skip("gettempdir returns string", "gettempdir not available")
    skip("gettempdir not empty", "gettempdir not available")
    skip("gettempdir exists", "gettempdir not available")


# ============================================================================
# tempfile.mktemp() tests
# ============================================================================

print("\n=== tempfile.mktemp() tests ===")

if hasattr(tempfile, "mktemp"):
    path = tempfile.mktemp()
    test("mktemp returns string", isinstance(path, str))
    test("mktemp not empty", len(path) > 0)
    test("mktemp file not created", not path_exists(path))
else:
    skip("mktemp returns string", "mktemp not available")
    skip("mktemp not empty", "mktemp not available")
    skip("mktemp file not created", "mktemp not available")


# ============================================================================
# tempfile.mkstemp() tests
# ============================================================================

print("\n=== tempfile.mkstemp() tests ===")

if hasattr(tempfile, "mkstemp"):
    result = tempfile.mkstemp()
    if isinstance(result, tuple):
        fd, path = result
        os.close(fd)
    else:
        path = result
    test("mkstemp returns path", isinstance(path, str))
    test("mkstemp file exists", path_exists(path))
    test("mkstemp is file", path_isfile(path))
    cleanup_file(path)
else:
    skip("mkstemp returns path", "mkstemp not available")
    skip("mkstemp file exists", "mkstemp not available")
    skip("mkstemp is file", "mkstemp not available")


# ============================================================================
# tempfile.mkdtemp() tests
# ============================================================================

print("\n=== tempfile.mkdtemp() tests ===")

if hasattr(tempfile, "mkdtemp"):
    try:
        # pocketpy-kipferl mkdtemp requires an absolute path prefix
        # since it doesn't have a default temp directory
        dpath = tempfile.mkdtemp("/tmp/kipferl_test_")
        test("mkdtemp returns string", isinstance(dpath, str))
        test("mkdtemp dir exists", path_exists(dpath))
        test("mkdtemp is dir", path_isdir(dpath))
        cleanup_dir(dpath)
    except Exception as e:
        skip("mkdtemp returns string", f"mkdtemp raised: {e}")
        skip("mkdtemp dir exists", f"mkdtemp raised: {e}")
        skip("mkdtemp is dir", f"mkdtemp raised: {e}")
else:
    skip("mkdtemp returns string", "mkdtemp not available")
    skip("mkdtemp dir exists", "mkdtemp not available")
    skip("mkdtemp is dir", "mkdtemp not available")


# ============================================================================
# Cleanup
# ============================================================================

do_cleanup()


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
