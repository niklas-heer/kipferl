"""
Simplified fnmatch module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_fnmatch.py
"""

import fnmatch

# Test tracking
_passed = 0
_failed = 0
_errors = []


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


# ============================================================================
# fnmatch.fnmatch() tests
# ============================================================================

print("\n=== fnmatch.fnmatch() tests ===")

# Basic patterns
test("fnmatch('foo', 'foo')", fnmatch.fnmatch("foo", "foo"))
test("fnmatch('foo', 'bar')", not fnmatch.fnmatch("foo", "bar"))

# Wildcard *
test("fnmatch('foo', '*')", fnmatch.fnmatch("foo", "*"))
test("fnmatch('foo', 'f*')", fnmatch.fnmatch("foo", "f*"))
test("fnmatch('foo', '*o')", fnmatch.fnmatch("foo", "*o"))
test("fnmatch('foo', 'f*o')", fnmatch.fnmatch("foo", "f*o"))
test("fnmatch('foobar', 'foo*')", fnmatch.fnmatch("foobar", "foo*"))
test("fnmatch('foobar', '*bar')", fnmatch.fnmatch("foobar", "*bar"))
test("fnmatch('foobar', 'foo*bar')", fnmatch.fnmatch("foobar", "foo*bar"))
test("fnmatch('', '*')", fnmatch.fnmatch("", "*"))
test("fnmatch('foo', 'b*')", not fnmatch.fnmatch("foo", "b*"))

# Wildcard ?
test("fnmatch('a', '?')", fnmatch.fnmatch("a", "?"))
test("fnmatch('ab', '??')", fnmatch.fnmatch("ab", "??"))
test("fnmatch('abc', 'a?c')", fnmatch.fnmatch("abc", "a?c"))
test("fnmatch('abc', '???')", fnmatch.fnmatch("abc", "???"))
test("fnmatch('ab', '???')", not fnmatch.fnmatch("ab", "???"))
test("fnmatch('', '?')", not fnmatch.fnmatch("", "?"))

# Character sets [seq]
test("fnmatch('a', '[abc]')", fnmatch.fnmatch("a", "[abc]"))
test("fnmatch('b', '[abc]')", fnmatch.fnmatch("b", "[abc]"))
test("fnmatch('d', '[abc]')", not fnmatch.fnmatch("d", "[abc]"))
test("fnmatch('a', '[a-z]')", fnmatch.fnmatch("a", "[a-z]"))
test("fnmatch('m', '[a-z]')", fnmatch.fnmatch("m", "[a-z]"))
test("fnmatch('A', '[a-z]')", not fnmatch.fnmatch("A", "[a-z]"))

# Negated character sets [!seq]
test("fnmatch('d', '[!abc]')", fnmatch.fnmatch("d", "[!abc]"))
test("fnmatch('a', '[!abc]')", not fnmatch.fnmatch("a", "[!abc]"))
test("fnmatch('A', '[!a-z]')", fnmatch.fnmatch("A", "[!a-z]"))
test("fnmatch('a', '[!a-z]')", not fnmatch.fnmatch("a", "[!a-z]"))

# Combined patterns
test("fnmatch('foo.py', '*.py')", fnmatch.fnmatch("foo.py", "*.py"))
test("fnmatch('test_foo.py', 'test_*.py')", fnmatch.fnmatch("test_foo.py", "test_*.py"))
test("fnmatch('a1b', 'a[0-9]b')", fnmatch.fnmatch("a1b", "a[0-9]b"))
test("fnmatch('aXb', 'a[0-9]b')", not fnmatch.fnmatch("aXb", "a[0-9]b"))

# Case sensitivity (fnmatch is case-insensitive on Windows, case-sensitive on Unix)
# Our implementation follows Unix convention (case-sensitive by default)
test("fnmatch('FOO', 'foo') case", not fnmatch.fnmatch("FOO", "foo"))
test("fnmatch('foo', 'FOO') case", not fnmatch.fnmatch("foo", "FOO"))

# ============================================================================
# fnmatch.fnmatchcase() tests (always case-sensitive)
# ============================================================================

print("\n=== fnmatch.fnmatchcase() tests ===")

test("fnmatchcase('foo', 'foo')", fnmatch.fnmatchcase("foo", "foo"))
test("fnmatchcase('FOO', 'foo')", not fnmatch.fnmatchcase("FOO", "foo"))
test("fnmatchcase('foo', 'FOO')", not fnmatch.fnmatchcase("foo", "FOO"))
test("fnmatchcase('FOO', 'FOO')", fnmatch.fnmatchcase("FOO", "FOO"))
test("fnmatchcase('Foo', 'F*')", fnmatch.fnmatchcase("Foo", "F*"))
test("fnmatchcase('foo', 'F*')", not fnmatch.fnmatchcase("foo", "F*"))

# ============================================================================
# fnmatch.filter() tests
# ============================================================================

print("\n=== fnmatch.filter() tests ===")

names = ["foo", "bar", "foobar", "baz", "FOO"]

result = fnmatch.filter(names, "foo")
test("filter exact match", result == ["foo"])

result = fnmatch.filter(names, "f*")
test("filter f*", result == ["foo", "foobar"])

result = fnmatch.filter(names, "*bar")
test("filter *bar", result == ["bar", "foobar"])

result = fnmatch.filter(names, "ba?")
test("filter ba?", result == ["bar", "baz"])

result = fnmatch.filter(names, "*")
test("filter *", result == names)

result = fnmatch.filter(names, "xyz")
test("filter no match", result == [])

result = fnmatch.filter(["a.py", "b.txt", "c.py", "d.pyc"], "*.py")
test("filter *.py", result == ["a.py", "c.py"])

# ============================================================================
# fnmatch.translate() tests
# ============================================================================

print("\n=== fnmatch.translate() tests ===")

# translate() converts shell patterns to regex patterns
regex = fnmatch.translate("*")
test("translate(*)", ".*" in regex or regex.endswith(".*"))

regex = fnmatch.translate("?")
test("translate(?)", "." in regex)

regex = fnmatch.translate("*.py")
test("translate(*.py)", ".py" in regex)

# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Empty strings
test("fnmatch('', '')", fnmatch.fnmatch("", ""))
test("fnmatch('', 'a')", not fnmatch.fnmatch("", "a"))
test("fnmatch('a', '')", not fnmatch.fnmatch("a", ""))

# Special characters
test("fnmatch('[', '[')", fnmatch.fnmatch("[", "["))
test("fnmatch(']', ']')", fnmatch.fnmatch("]", "]"))

# Literal brackets (escaped by including ] first or [ last)
test("fnmatch('a', '[a]')", fnmatch.fnmatch("a", "[a]"))

# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    import sys

    sys.exit(1)
else:
    print("All tests passed!")
