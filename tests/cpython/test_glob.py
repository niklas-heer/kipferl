"""
Simplified glob module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_glob.py
"""

import glob
import os
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


# Cross-platform path utilities
_sep = os.sep


def path_join(*parts):
    """Join path parts."""
    return _sep.join(parts)


def path_basename(p):
    """Get basename."""
    return p.rsplit(_sep, 1)[-1] if _sep in p else p


# Setup test directory
import time

tempdir = "/tmp/test_glob_" + str(int(time.time() * 1000) % 1000000)
try:
    os.mkdir(tempdir)
except:
    pass  # May already exist


def setup_test_files():
    # Create test files
    with open(path_join(tempdir, "file1.py"), "w") as f:
        f.write("# file1")
    with open(path_join(tempdir, "file2.py"), "w") as f:
        f.write("# file2")
    with open(path_join(tempdir, "file3.txt"), "w") as f:
        f.write("file3")

    # Create subdirectory
    dir1 = path_join(tempdir, "dir1")
    try:
        os.mkdir(dir1)
    except:
        pass
    with open(path_join(dir1, "nested1.py"), "w") as f:
        f.write("# nested1")


def cleanup_test_files():
    """Clean up test files."""

    def rmtree(path):
        try:
            for entry in os.listdir(path):
                full = path_join(path, entry)
                try:
                    os.remove(full)
                except:
                    rmtree(full)
            os.rmdir(path)
        except:
            pass

    rmtree(tempdir)


setup_test_files()


def get_basenames(results):
    return sorted([path_basename(p) for p in results])


# ============================================================================
# glob.glob() tests - Basic patterns
# ============================================================================

print("\n=== glob.glob() basic tests ===")

results = glob.glob(path_join(tempdir, "*.py"))
basenames = get_basenames(results)
test(
    "glob *.py finds python files", "file1.py" in basenames and "file2.py" in basenames
)

results = glob.glob(path_join(tempdir, "*"))
basenames = get_basenames(results)
test("glob * finds files", len(basenames) >= 3)


# ============================================================================
# glob.glob() tests - ? wildcard
# ============================================================================

print("\n=== glob.glob() ? wildcard tests ===")

results = glob.glob(path_join(tempdir, "file?.py"))
basenames = get_basenames(results)
test("glob file?.py", "file1.py" in basenames and "file2.py" in basenames)


# ============================================================================
# glob.glob() tests - recursive
# ============================================================================

print("\n=== glob.glob() recursive tests ===")

# Use positional args for PocketPy compatibility: glob(pattern, root_dir, dir_fd, recursive)
results = glob.glob(path_join(tempdir, "**", "*.py"), None, None, True)
basenames = get_basenames(results)
test("glob **/*.py recursive", "nested1.py" in basenames)


# ============================================================================
# Cleanup
# ============================================================================

cleanup_test_files()


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
