"""
Simplified shutil module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_shutil.py
"""

import os
import shutil
import sys
import time

# Test tracking
_passed = 0
_failed = 0
_errors = []
_skipped = 0

_test_dir = f"/tmp/test_shutil_{int(time.time() * 1000)}"


def path_exists(path):
    """Check if path exists - works on both CPython and PocketPy."""
    try:
        os.stat(path)
        return True
    except OSError:
        return False


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


def setup_test_dir():
    try:
        os.makedirs(_test_dir, exist_ok=True)
    except Exception:
        try:
            os.mkdir(_test_dir)
        except Exception:
            pass


def cleanup_test_dir():
    try:
        shutil.rmtree(_test_dir)
    except Exception:
        pass


def write_file(path, content):
    with open(path, "w") as f:
        f.write(content)


def read_file(path):
    with open(path, "r") as f:
        return f.read()


setup_test_dir()


# ============================================================================
# shutil.copy() tests
# ============================================================================

print("\n=== shutil.copy() tests ===")

src_file = f"{_test_dir}/src_copy.txt"
dst_file = f"{_test_dir}/dst_copy.txt"
write_file(src_file, "Hello, World!")
shutil.copy(src_file, dst_file)
test("copy basic", path_exists(dst_file))
test("copy content preserved", read_file(dst_file) == "Hello, World!")


# ============================================================================
# shutil.move() tests
# ============================================================================

print("\n=== shutil.move() tests ===")

src_file = f"{_test_dir}/src_move.txt"
dst_file = f"{_test_dir}/dst_move.txt"
write_file(src_file, "Move test content")
shutil.move(src_file, dst_file)
test("move file exists at dst", path_exists(dst_file))
test("move file gone from src", not path_exists(src_file))


# ============================================================================
# shutil.rmtree() tests
# ============================================================================

print("\n=== shutil.rmtree() tests ===")

rm_dir = f"{_test_dir}/rm_tree"
# Create nested directory structure
try:
    os.mkdir(rm_dir)
    os.mkdir(f"{rm_dir}/subdir")
except:
    pass
write_file(f"{rm_dir}/file.txt", "Content")
test("rmtree setup", path_exists(rm_dir))
shutil.rmtree(rm_dir)
test("rmtree removes dir", not path_exists(rm_dir))


# ============================================================================
# shutil.exists() tests (kipferl extension)
# ============================================================================

print("\n=== shutil.exists() tests ===")

exist_file = f"{_test_dir}/exists_test.txt"
write_file(exist_file, "Exists")
if hasattr(shutil, "exists"):
    test("exists file true", shutil.exists(exist_file))
    test("exists false", not shutil.exists(f"{_test_dir}/no_such_file.txt"))
else:
    skip("exists file true", "Kipferl extension")
    skip("exists false", "Kipferl extension")


cleanup_test_dir()


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
