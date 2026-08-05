"""
Simplified re module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_re.py
"""

import re
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
# re.match() tests
# ============================================================================

print("\n=== re.match() tests ===")

# Basic match
m = re.match(r"hello", "hello world")
test("match basic", m is not None)
test("match group", m.group(0) == "hello")

# Match at start only
m = re.match(r"world", "hello world")
test("match start only", m is None)

# Match with groups
m = re.match(r"(\w+) (\w+)", "hello world")
test("match groups", m is not None)
test("match group 0", m.group(0) == "hello world")
test("match group 1", m.group(1) == "hello")
test("match group 2", m.group(2) == "world")

# No match
m = re.match(r"xyz", "hello")
test("match no match", m is None)


# ============================================================================
# re.search() tests
# ============================================================================

print("\n=== re.search() tests ===")

# Basic search
m = re.search(r"world", "hello world")
test("search basic", m is not None)
test("search group", m.group(0) == "world")

# Search anywhere in string
m = re.search(r"o", "hello")
test("search middle", m is not None)
test("search start", m.start(0) == 4)

# Search with groups
m = re.search(r"(\d+)", "abc 123 def")
test("search groups", m is not None)
test("search group 1", m.group(1) == "123")

# No match
m = re.search(r"xyz", "hello")
test("search no match", m is None)


# ============================================================================
# re.findall() tests
# ============================================================================

print("\n=== re.findall() tests ===")

# Find all occurrences
result = re.findall(r"\d+", "a1b2c3d4")
test("findall basic", result == ["1", "2", "3", "4"])

# No matches
result = re.findall(r"\d+", "no numbers here")
test("findall no match", result == [])

# With groups (returns groups)
result = re.findall(r"(\w)(\d)", "a1b2c3")
test("findall groups", result == [("a", "1"), ("b", "2"), ("c", "3")])

# Single group (returns list of strings)
result = re.findall(r"(\d+)", "a1b22c333")
test("findall single group", result == ["1", "22", "333"])


# ============================================================================
# re.sub() tests
# ============================================================================

print("\n=== re.sub() tests ===")

# Basic substitution
result = re.sub(r"\d+", "X", "a1b2c3")
test("sub basic", result == "aXbXcX")

# No match - unchanged
result = re.sub(r"\d+", "X", "abc")
test("sub no match", result == "abc")

# Count argument (use positional arg for pocketpy compatibility)
result = re.sub(r"\d+", "X", "a1b2c3", 2)
test("sub count", result == "aXbXc3")

# Empty replacement
result = re.sub(r"\d+", "", "a1b2c3")
test("sub empty replacement", result == "abc")

# Replace with backreference
result = re.sub(r"(\w+)", r"[\1]", "hello world")
test("sub backreference", result == "[hello] [world]")


# ============================================================================
# re.split() tests
# ============================================================================

print("\n=== re.split() tests ===")

# Basic split
result = re.split(r"\s+", "hello world foo")
test("split basic", result == ["hello", "world", "foo"])

# Split on digits
result = re.split(r"\d+", "a1b2c3d")
test("split digits", result == ["a", "b", "c", "d"])

# No match - single element
result = re.split(r"x", "hello")
test("split no match", result == ["hello"])

# Split with maxsplit (use positional arg for pocketpy compatibility)
result = re.split(r"\s+", "a b c d", 2)
test("split maxsplit", result == ["a", "b", "c d"])

# Empty string
result = re.split(r"\s+", "")
test("split empty", result == [""])


# ============================================================================
# re.compile() tests
# ============================================================================

print("\n=== re.compile() tests ===")

# Compile pattern
pattern = re.compile(r"\d+")
test("compile returns pattern", pattern is not None)

# Use compiled pattern
m = pattern.match("123abc")
test("compile match", m is not None and m.group(0) == "123")

m = pattern.search("abc123def")
test("compile search", m is not None and m.group(0) == "123")

result = pattern.findall("a1b2c3")
test("compile findall", result == ["1", "2", "3"])

# Pattern.sub and Pattern.split - check if available
if hasattr(pattern, "sub"):
    result = pattern.sub("X", "a1b2c3")
    test("compile sub", result == "aXbXcX")
else:
    skip("compile sub", "Pattern.sub not available in pocketpy")

if hasattr(pattern, "split"):
    result = pattern.split("a1b2c3d")
    test("compile split", result == ["a", "b", "c", "d"])
else:
    skip("compile split", "Pattern.split not available in pocketpy")


# ============================================================================
# Match object tests
# ============================================================================

print("\n=== Match object tests ===")

m = re.match(r"(\w+) (\w+)", "hello world extra")

# group()
test("match.group(0)", m.group(0) == "hello world")
test("match.group(1)", m.group(1) == "hello")
test("match.group(2)", m.group(2) == "world")

# group() without argument
test("match.group()", m.group() == "hello world")

# groups()
test("match.groups()", m.groups() == ("hello", "world"))

# start() and end()
test("match.start()", m.start(0) == 0)
test("match.end()", m.end(0) == 11)
test("match.start(1)", m.start(1) == 0)
test("match.end(1)", m.end(1) == 5)

# span()
test("match.span()", m.span(0) == (0, 11))
test("match.span(1)", m.span(1) == (0, 5))


# ============================================================================
# Character classes
# ============================================================================

print("\n=== Character classes ===")

# \d - digits
test("\\d matches digit", re.match(r"\d", "5") is not None)
test("\\d no match letter", re.match(r"\d", "a") is None)

# \w - word characters
test("\\w matches letter", re.match(r"\w", "a") is not None)
test("\\w matches digit", re.match(r"\w", "5") is not None)
test("\\w matches underscore", re.match(r"\w", "_") is not None)
test("\\w no match space", re.match(r"\w", " ") is None)

# \s - whitespace
test("\\s matches space", re.match(r"\s", " ") is not None)
test("\\s matches tab", re.match(r"\s", "\t") is not None)
test("\\s no match letter", re.match(r"\s", "a") is None)

# Character set
test("[abc] matches", re.match(r"[abc]", "b") is not None)
test("[abc] no match", re.match(r"[abc]", "d") is None)

# Negated set
test("[^abc] matches", re.match(r"[^abc]", "d") is not None)
test("[^abc] no match", re.match(r"[^abc]", "a") is None)

# Range
test("[a-z] matches", re.match(r"[a-z]", "m") is not None)
test("[0-9] matches", re.match(r"[0-9]", "5") is not None)


# ============================================================================
# Quantifiers
# ============================================================================

print("\n=== Quantifiers ===")

# * - zero or more
test("* matches zero", re.match(r"ab*c", "ac") is not None)
test("* matches one", re.match(r"ab*c", "abc") is not None)
test("* matches many", re.match(r"ab*c", "abbbc") is not None)

# + - one or more
test("+ no match zero", re.match(r"ab+c", "ac") is None)
test("+ matches one", re.match(r"ab+c", "abc") is not None)
test("+ matches many", re.match(r"ab+c", "abbbc") is not None)

# ? - zero or one
test("? matches zero", re.match(r"ab?c", "ac") is not None)
test("? matches one", re.match(r"ab?c", "abc") is not None)
test("? no match many", re.match(r"ab?c$", "abbc") is None)

# {n} - exactly n
test("{2} matches", re.match(r"ab{2}c", "abbc") is not None)
test("{2} no match", re.match(r"ab{2}c", "abc") is None)


# ============================================================================
# Anchors
# ============================================================================

print("\n=== Anchors ===")

# ^ - start of string
test("^ at start", re.match(r"^hello", "hello world") is not None)

# $ - end of string
test("$ at end", re.search(r"world$", "hello world") is not None)
test("$ not at end", re.search(r"hello$", "hello world") is None)


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Empty pattern
m = re.match(r"", "hello")
test("empty pattern matches", m is not None)

# Empty string
m = re.match(r".*", "")
test("empty string matches", m is not None)

# Special characters
m = re.match(r"\.", ".")
test("escaped dot matches", m is not None)
m = re.match(r"\.", "a")
test("escaped dot no match", m is None)


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
