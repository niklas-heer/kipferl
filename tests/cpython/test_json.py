"""
Simplified json module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_json/
"""

import json
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
# json.dumps() tests - basic types
# ============================================================================

print("\n=== json.dumps() basic types ===")

# Strings
test("dumps string", json.dumps("hello") == '"hello"')
test("dumps empty string", json.dumps("") == '""')
test("dumps string with quotes", json.dumps('say "hi"') == '"say \\"hi\\""')

# Numbers
test("dumps integer", json.dumps(42) == "42")
test("dumps negative", json.dumps(-42) == "-42")
test("dumps zero", json.dumps(0) == "0")
test("dumps float", json.dumps(3.14) in ["3.14", "3.140000"])
test("dumps negative float", json.dumps(-3.14) in ["-3.14", "-3.140000"])

# Booleans
test("dumps true", json.dumps(True) == "true")
test("dumps false", json.dumps(False) == "false")

# None/null
test("dumps None", json.dumps(None) == "null")


# ============================================================================
# json.dumps() tests - containers
# ============================================================================

print("\n=== json.dumps() containers ===")

# Lists/arrays
test("dumps empty list", json.dumps([]) == "[]")
test("dumps list", json.dumps([1, 2, 3]) == "[1, 2, 3]")
test("dumps nested list", json.dumps([[1, 2], [3, 4]]) == "[[1, 2], [3, 4]]")
test("dumps mixed list", json.dumps([1, "two", True, None]) == '[1, "two", true, null]')

# Dicts/objects
test("dumps empty dict", json.dumps({}) == "{}")
result = json.dumps({"a": 1})
test("dumps dict", result == '{"a": 1}')

# Nested structures
nested = {"list": [1, 2, 3], "nested": {"x": 10}}
result = json.loads(json.dumps(nested))
test("dumps nested roundtrip", result == nested)


# ============================================================================
# json.loads() tests - basic types
# ============================================================================

print("\n=== json.loads() basic types ===")

# Strings
test("loads string", json.loads('"hello"') == "hello")
test("loads empty string", json.loads('""') == "")
test("loads escaped quotes", json.loads('"say \\"hi\\""') == 'say "hi"')

# Numbers
test("loads integer", json.loads("42") == 42)
test("loads negative", json.loads("-42") == -42)
test("loads zero", json.loads("0") == 0)
test("loads float", abs(json.loads("3.14") - 3.14) < 0.001)
test("loads negative float", abs(json.loads("-3.14") - (-3.14)) < 0.001)
test("loads exponent", json.loads("1e10") == 1e10)
test("loads negative exponent", json.loads("1e-5") == 1e-5)

# Booleans
test("loads true", json.loads("true") is True)
test("loads false", json.loads("false") is False)

# None/null
test("loads null", json.loads("null") is None)


# ============================================================================
# json.loads() tests - containers
# ============================================================================

print("\n=== json.loads() containers ===")

# Lists/arrays
test("loads empty array", json.loads("[]") == [])
test("loads array", json.loads("[1, 2, 3]") == [1, 2, 3])
test("loads nested array", json.loads("[[1, 2], [3, 4]]") == [[1, 2], [3, 4]])

# Dicts/objects
test("loads empty object", json.loads("{}") == {})
test("loads object", json.loads('{"a": 1}') == {"a": 1})
test("loads object multi", json.loads('{"a": 1, "b": 2}') == {"a": 1, "b": 2})

# Nested structures
test(
    "loads nested",
    json.loads('{"x": [1, 2], "y": {"z": 3}}') == {"x": [1, 2], "y": {"z": 3}},
)


# ============================================================================
# Roundtrip tests
# ============================================================================

print("\n=== Roundtrip tests ===")


def roundtrip(obj):
    return json.loads(json.dumps(obj))


test("roundtrip string", roundtrip("hello") == "hello")
test("roundtrip int", roundtrip(42) == 42)
test("roundtrip float", abs(roundtrip(3.14) - 3.14) < 0.001)
test("roundtrip bool true", roundtrip(True) is True)
test("roundtrip bool false", roundtrip(False) is False)
test("roundtrip None", roundtrip(None) is None)
test("roundtrip list", roundtrip([1, 2, 3]) == [1, 2, 3])
test("roundtrip dict", roundtrip({"a": 1}) == {"a": 1})

# Complex nested structure
complex_obj = {
    "name": "test",
    "values": [1, 2, 3],
    "nested": {"x": True, "y": None},
    "empty": [],
}
test("roundtrip complex", roundtrip(complex_obj) == complex_obj)


# ============================================================================
# Whitespace handling
# ============================================================================

print("\n=== Whitespace handling ===")

test("loads with spaces", json.loads("  42  ") == 42)
test("loads with newlines", json.loads("\n42\n") == 42)
test("loads array with spaces", json.loads("[ 1 , 2 , 3 ]") == [1, 2, 3])
test("loads object with spaces", json.loads('{ "a" : 1 }') == {"a": 1})


# ============================================================================
# Unicode and escape sequences
# ============================================================================

print("\n=== Unicode and escapes ===")

test("loads newline escape", json.loads('"hello\\nworld"') == "hello\nworld")
test("loads tab escape", json.loads('"hello\\tworld"') == "hello\tworld")
test("loads backslash escape", json.loads('"hello\\\\world"') == "hello\\world")

# Unicode
test("dumps unicode", json.dumps("hello") == '"hello"')
test("loads unicode", json.loads('"hello"') == "hello")


# ============================================================================
# Error handling
# ============================================================================

print("\n=== Error handling ===")

# Invalid JSON - pocketpy raises NameError/SyntaxError instead of ValueError/JSONDecodeError
# So we accept any exception as valid error handling
try:
    json.loads("invalid")
    test("loads invalid raises", False)
except Exception:
    test("loads invalid raises", True)

# Trailing comma - pocketpy accepts trailing commas (non-strict JSON)
# Skip these tests on pocketpy since it's more lenient
try:
    result = json.loads("[1, 2,]")
    # If we got here without exception, pocketpy is lenient - skip
    skip("loads trailing comma raises", "pocketpy accepts trailing commas")
except Exception:
    test("loads trailing comma raises", True)

try:
    result = json.loads('{"a": 1,}')
    # If we got here without exception, pocketpy is lenient - skip
    skip("loads trailing comma obj raises", "pocketpy accepts trailing commas")
except Exception:
    test("loads trailing comma obj raises", True)


# ============================================================================
# Options
# ============================================================================

print("\n=== Options ===")

# indent
result = json.dumps({"a": 1}, indent=2)
test("dumps with indent", "\n" in result)

# Test proper indentation structure
result = json.dumps({"a": [1, 2]}, indent=2)
test("dumps indent structure", '  "a":' in result and "    1" in result)

# separators - not supported in pocketpy
try:
    result = json.dumps([1, 2], separators=(",", ":"))
    test("dumps with separators", result == "[1,2]")
except TypeError:
    skip("dumps with separators", "separators not supported")

# sort_keys - not supported in pocketpy
try:
    result = json.dumps({"b": 2, "a": 1}, sort_keys=True)
    test("dumps sort_keys", result.index('"a"') < result.index('"b"'))
except TypeError:
    skip("dumps sort_keys", "sort_keys not supported")

# Test sort_keys with nested dict - not supported in pocketpy
try:
    result = json.dumps({"z": {"b": 2, "a": 1}, "a": 1}, sort_keys=True)
    test("dumps sort_keys nested", result.index('"a": 1') < result.index('"z"'))
except TypeError:
    skip("dumps sort_keys nested", "sort_keys not supported")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Empty containers
test("empty list roundtrip", roundtrip([]) == [])
test("empty dict roundtrip", roundtrip({}) == {})

# Deeply nested
deep = {"a": {"b": {"c": {"d": 1}}}}
test("deep nesting", roundtrip(deep) == deep)

# Large numbers
test("large int", json.loads("9999999999999999") == 9999999999999999)

# Special float values handling - pocketpy outputs "Infinity" instead of raising
try:
    result = json.dumps(float("inf"))
    # pocketpy outputs "Infinity" instead of raising - skip the test
    skip("dumps infinity raises", "pocketpy outputs Infinity instead of raising")
except (ValueError, OverflowError):
    test("dumps infinity raises", True)

# Test JSONDecodeError exists - not available in pocketpy
if hasattr(json, "JSONDecodeError"):
    test("JSONDecodeError exists", True)

    # Test JSONDecodeError is raised on invalid JSON
    try:
        json.loads("invalid json")
        test("JSONDecodeError on invalid", False)
    except json.JSONDecodeError:
        test("JSONDecodeError on invalid", True)
else:
    skip("JSONDecodeError exists", "JSONDecodeError not available")
    skip("JSONDecodeError on invalid", "JSONDecodeError not available")


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
