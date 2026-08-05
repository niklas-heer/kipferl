"""
Simplified sys module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_sys.py
"""

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
# sys.version tests
# ============================================================================

print("\n=== sys.version tests ===")

test("version exists", hasattr(sys, "version"))
test("version is string", isinstance(sys.version, str))
test("version not empty", len(sys.version) > 0)

test("version_info exists", hasattr(sys, "version_info"))
test(
    "version_info has major",
    hasattr(sys.version_info, "major") or len(sys.version_info) >= 1,
)
# Check major version is reasonable (3 for Python 3)
if isinstance(sys.version_info, tuple):
    major = sys.version_info[0]
else:
    major = sys.version_info.major
test("version_info major", major >= 1)


# ============================================================================
# sys.platform tests
# ============================================================================

print("\n=== sys.platform tests ===")

test("platform exists", hasattr(sys, "platform"))
test("platform is string", isinstance(sys.platform, str))
test("platform not empty", len(sys.platform) > 0)
platform_values = [
    "linux",
    "darwin",
    "win32",
    "cygwin",
    "freebsd",
    "rp2",
    "esp32",
    "pyboard",
    "unix",
]
test("platform known", sys.platform in platform_values)


# ============================================================================
# sys.path tests
# ============================================================================

print("\n=== sys.path tests ===")

test("path exists", hasattr(sys, "path"))
test("path is list", isinstance(sys.path, list))


# ============================================================================
# sys.modules tests
# ============================================================================

print("\n=== sys.modules tests ===")

test("modules exists", hasattr(sys, "modules"))
test("modules is dict", isinstance(sys.modules, dict))
test("sys in modules", "sys" in sys.modules)
test("modules not empty", len(sys.modules) > 0)


# ============================================================================
# sys.argv tests
# ============================================================================

print("\n=== sys.argv tests ===")

test("argv exists", hasattr(sys, "argv"))
test("argv is list", isinstance(sys.argv, list))


# ============================================================================
# sys.stdin/stdout/stderr tests
# ============================================================================

print("\n=== sys.stdin/stdout/stderr tests ===")

test("stdin exists", hasattr(sys, "stdin"))
test("stdout exists", hasattr(sys, "stdout"))
test("stderr exists", hasattr(sys, "stderr"))

test("stdout not None", sys.stdout is not None)
test("stdout has write", hasattr(sys.stdout, "write"))

test("stderr not None", sys.stderr is not None)
test("stderr has write", hasattr(sys.stderr, "write"))


# ============================================================================
# sys.exit() tests
# ============================================================================

print("\n=== sys.exit() tests ===")

test("exit exists", hasattr(sys, "exit"))
test("exit is callable", callable(sys.exit))

# Test that SystemExit is raised (but don't actually exit)
try:
    sys.exit(0)
    test("exit raises SystemExit", False)
except SystemExit as e:
    test("exit raises SystemExit", True)
    test("exit code", e.args[0] == 0 if e.args else True)


# ============================================================================
# sys.maxsize tests
# ============================================================================

print("\n=== sys.maxsize tests ===")

test("maxsize exists", hasattr(sys, "maxsize"))
test("maxsize is int", isinstance(sys.maxsize, int))
test("maxsize is large", sys.maxsize > 2**30)  # At least 32-bit


# ============================================================================
# sys.byteorder tests
# ============================================================================

print("\n=== sys.byteorder tests ===")

test("byteorder exists", hasattr(sys, "byteorder"))
test("byteorder is string", isinstance(sys.byteorder, str))
test("byteorder value", sys.byteorder in ["little", "big"])


# ============================================================================
# sys.implementation tests
# ============================================================================

print("\n=== sys.implementation tests ===")

test("implementation exists", hasattr(sys, "implementation"))
test("implementation has name", hasattr(sys.implementation, "name"))
test("implementation name is string", isinstance(sys.implementation.name, str))
impl_names = ["cpython", "micropython", "pocketpy", "pypy", "jython", "ironpython"]
test("implementation known", sys.implementation.name in impl_names)


# ============================================================================
# sys.executable tests
# ============================================================================

print("\n=== sys.executable tests ===")

test("executable exists", hasattr(sys, "executable"))
test("executable is string", isinstance(sys.executable, str))


# ============================================================================
# sys.getrecursionlimit()/setrecursionlimit() tests
# ============================================================================

print("\n=== Recursion limit tests ===")

test("getrecursionlimit exists", hasattr(sys, "getrecursionlimit"))
limit = sys.getrecursionlimit()
test("getrecursionlimit returns int", isinstance(limit, int))
test("getrecursionlimit positive", limit > 0)

test("setrecursionlimit exists", hasattr(sys, "setrecursionlimit"))
# Save original
original = sys.getrecursionlimit()
# Set new limit
sys.setrecursionlimit(500)
test("setrecursionlimit works", sys.getrecursionlimit() == 500)
# Restore original
sys.setrecursionlimit(original)
test("setrecursionlimit restore", sys.getrecursionlimit() == original)


# ============================================================================
# sys.getsizeof() tests
# ============================================================================

print("\n=== sys.getsizeof() tests ===")

test("getsizeof exists", hasattr(sys, "getsizeof"))
test("getsizeof int", sys.getsizeof(0) > 0)
test("getsizeof str", sys.getsizeof("hello") > 0)
test("getsizeof list", sys.getsizeof([]) > 0)
test("getsizeof dict", sys.getsizeof({}) > 0)
test("getsizeof bigger list", sys.getsizeof([1, 2, 3]) >= sys.getsizeof([]))


# ============================================================================
# sys.intern() tests
# ============================================================================

print("\n=== sys.intern() tests ===")

test("intern exists", hasattr(sys, "intern"))
s = sys.intern("hello")
test("intern returns string", isinstance(s, str))
test("intern same value", s == "hello")

# Interned strings should be identical
s1 = sys.intern("test_string")
s2 = sys.intern("test_string")
test("intern same identity", s1 is s2)


# ============================================================================
# sys.flags tests
# ============================================================================

print("\n=== sys.flags tests ===")

test("flags exists", hasattr(sys, "flags"))


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    # Don't exit here since we tested exit above
else:
    print("All tests passed!")
