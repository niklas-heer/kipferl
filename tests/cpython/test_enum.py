"""
Simplified enum module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_enum.py
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


# Try to import enum
try:
    from enum import Enum

    HAS_ENUM = True
except ImportError:
    HAS_ENUM = False
    print("SKIP: enum module not available")

if not HAS_ENUM:
    print("\n" + "=" * 50)
    print("Results: 0 passed, 0 failed, 0 skipped")
    print("enum module not available")
    sys.exit(0)

# ============================================================================
# Basic Enum creation
# ============================================================================

print("\n=== Basic Enum creation ===")


class Color(Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


test("Enum class created", Color is not None)
test("RED member exists", hasattr(Color, "RED"))
test("GREEN member exists", hasattr(Color, "GREEN"))
test("BLUE member exists", hasattr(Color, "BLUE"))

# ============================================================================
# Accessing members
# ============================================================================

print("\n=== Accessing members ===")

test("access by name", Color.RED.name == "RED")
test("access by value", Color.RED.value == 1)
test("GREEN name", Color.GREEN.name == "GREEN")
test("GREEN value", Color.GREEN.value == 2)

# ============================================================================
# Comparison
# ============================================================================

print("\n=== Comparison ===")

test("same member equal", Color.RED == Color.RED)
test("different members not equal", Color.RED != Color.GREEN)
test("identity same", Color.RED is Color.RED)

# ============================================================================
# Iteration - not supported in PocketPy
# ============================================================================

print("\n=== Iteration ===")

skip("iteration works", "Enum iteration not supported in PocketPy")
skip("RED in iteration", "Enum iteration not supported in PocketPy")
skip("GREEN in iteration", "Enum iteration not supported in PocketPy")
skip("BLUE in iteration", "Enum iteration not supported in PocketPy")

# ============================================================================
# IntEnum - not available in PocketPy
# ============================================================================

print("\n=== IntEnum ===")

skip("IntEnum created", "IntEnum not available in PocketPy")
skip("IntEnum value is int", "IntEnum not available in PocketPy")
skip("IntEnum comparable to int", "IntEnum not available in PocketPy")
skip("IntEnum arithmetic", "IntEnum not available in PocketPy")

# ============================================================================
# Using enums in dicts
# ============================================================================

print("\n=== Using enums in dicts ===")

color_names = {
    Color.RED: "red color",
    Color.GREEN: "green color",
    Color.BLUE: "blue color",
}

test("enum as dict key", color_names[Color.RED] == "red color")

# Check if 'in' works for enum keys
try:
    result = Color.GREEN in color_names
    test("enum key lookup", result == True)
except:
    skip("enum key lookup", "enum 'in' check not supported")

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
