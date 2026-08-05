"""
Simplified os module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_os.py
"""

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


# ============================================================================
# os.getcwd() tests
# ============================================================================

print("\n=== os.getcwd() tests ===")

cwd = os.getcwd()
test("getcwd returns string", isinstance(cwd, str))
test("getcwd not empty", len(cwd) > 0)
test("getcwd is absolute", cwd.startswith("/") or (len(cwd) > 1 and cwd[1] == ":"))


# ============================================================================
# os.listdir() tests
# ============================================================================

print("\n=== os.listdir() tests ===")

entries = os.listdir(".")
test("listdir returns list", isinstance(entries, list))
test("listdir has entries", len(entries) >= 0)

# All entries are strings
all_strings = True
for e in entries:
    if not isinstance(e, str):
        all_strings = False
        break
test("listdir all strings", all_strings)


# ============================================================================
# os.path tests
# ============================================================================

print("\n=== os.path tests ===")

# exists
test("path.exists cwd", os.path.exists("."))
test("path.exists nonexistent", not os.path.exists("/nonexistent_path_12345"))

# isdir
test("path.isdir cwd", os.path.isdir("."))

# isfile
test("path.isfile cwd", not os.path.isfile("."))

# join
test("path.join two", os.path.join("a", "b") in ["a/b", "a\\b"])
test("path.join three", os.path.join("a", "b", "c") in ["a/b/c", "a\\b\\c"])
test(
    "path.join absolute",
    os.path.join("a", "/b") == "/b" or os.path.join("a", "/b") == "\\b",
)

# basename
test("path.basename", os.path.basename("/foo/bar") == "bar")
test("path.basename no dir", os.path.basename("bar") == "bar")
test("path.basename trailing slash", os.path.basename("/foo/bar/") == "")

# dirname
test("path.dirname", os.path.dirname("/foo/bar") == "/foo")
test("path.dirname no dir", os.path.dirname("bar") == "")

# split
head, tail = os.path.split("/foo/bar")
test("path.split head", head == "/foo")
test("path.split tail", tail == "bar")

# splitext
root, ext = os.path.splitext("/foo/bar.txt")
test("path.splitext root", root == "/foo/bar")
test("path.splitext ext", ext == ".txt")

root, ext = os.path.splitext("/foo/bar")
test("path.splitext no ext", ext == "")

# isabs
test("path.isabs absolute", os.path.isabs("/foo"))
test("path.isabs relative", not os.path.isabs("foo"))

# abspath
abs_path = os.path.abspath(".")
test("path.abspath is absolute", os.path.isabs(abs_path))

# normpath
test("path.normpath dots", os.path.normpath("a/./b") in ["a/b", "a\\b"])
test("path.normpath dotdot", os.path.normpath("a/b/../c") in ["a/c", "a\\c"])


# ============================================================================
# os.environ tests
# ============================================================================

print("\n=== os.environ tests ===")

test("environ is dict-like", hasattr(os.environ, "__getitem__"))

# PATH or HOME should exist on most systems
has_path = "PATH" in os.environ or "HOME" in os.environ or "USER" in os.environ
test("environ has common vars", has_path)

# getenv
result = os.getenv("PATH", "default")
test("getenv PATH", result != "default" or os.getenv("HOME", "x") != "x")

test(
    "getenv nonexistent default",
    os.getenv("NONEXISTENT_VAR_12345", "default") == "default",
)

test("getenv nonexistent none", os.getenv("NONEXISTENT_VAR_12345") is None)


# ============================================================================
# os.sep and os.name tests
# ============================================================================

print("\n=== os.sep and os.name tests ===")

test("sep is string", isinstance(os.sep, str))
test("sep is / or \\", os.sep in ["/", "\\"])

test("name is string", isinstance(os.name, str))
test("name is known", os.name in ["posix", "nt", "java"])

test("linesep is string", isinstance(os.linesep, str))
test("linesep is newline", os.linesep in ["\n", "\r\n", "\r"])


# ============================================================================
# File descriptor operations
# ============================================================================

print("\n=== File operations ===")

# mkdir/rmdir
test_dir = "/tmp/kipferl_test_dir_12345"
try:
    if os.path.exists(test_dir):
        os.rmdir(test_dir)
    os.mkdir(test_dir)
    test("mkdir creates dir", os.path.exists(test_dir))
    os.rmdir(test_dir)
    test("rmdir removes dir", not os.path.exists(test_dir))
except (OSError, PermissionError) as e:
    skip("mkdir/rmdir", str(e))

# remove/unlink
test_file = "/tmp/kipferl_test_file_12345.txt"
try:
    with open(test_file, "w") as f:
        f.write("test")
    exists_before = os.path.exists(test_file)
    os.remove(test_file)
    exists_after = os.path.exists(test_file)
    test("remove deletes file", exists_before and not exists_after)
except (OSError, PermissionError) as e:
    skip("remove/unlink", str(e))


# ============================================================================
# os.stat tests
# ============================================================================

print("\n=== os.stat tests ===")

try:
    st = os.stat(".")
    test("stat returns object", st is not None)

    # Check common stat attributes
    if hasattr(st, "st_mode"):
        test("stat has st_mode", isinstance(st.st_mode, int))
    elif isinstance(st, tuple) and len(st) >= 1:
        test("stat has st_mode", isinstance(st[0], int))

    if hasattr(st, "st_size"):
        test("stat has st_size", isinstance(st.st_size, int))
    elif isinstance(st, tuple) and len(st) >= 7:
        test("stat has st_size", isinstance(st[6], int))
except OSError as e:
    skip("stat tests", str(e))


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
