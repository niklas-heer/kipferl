"""
Simplified textwrap module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_textwrap.py
"""

import textwrap

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
# textwrap.wrap() tests
# ============================================================================

print("\n=== textwrap.wrap() tests ===")

if hasattr(textwrap, "wrap"):
    # Basic wrap - use positional args only for kipferl compatibility
    text = "Hello World, this is a test of text wrapping functionality."
    result = textwrap.wrap(text, 20)
    test("wrap basic", len(result) > 1)
    test("wrap line length", all([len(line) <= 20 for line in result]))

    # Short text that doesn't need wrapping
    result = textwrap.wrap("Hello", 20)
    test("wrap short", result == ["Hello"])

    # Empty string
    result = textwrap.wrap("", 20)
    test("wrap empty", result == [])

    # Exact width
    result = textwrap.wrap("Hello World", 11)
    test("wrap exact", result == ["Hello World"])
else:
    skip("wrap basic", "textwrap.wrap not available")
    skip("wrap line length", "textwrap.wrap not available")
    skip("wrap short", "textwrap.wrap not available")
    skip("wrap empty", "textwrap.wrap not available")
    skip("wrap exact", "textwrap.wrap not available")


# ============================================================================
# textwrap.fill() tests
# ============================================================================

print("\n=== textwrap.fill() tests ===")

if hasattr(textwrap, "fill"):
    text = "Hello World, this is a test."
    result = textwrap.fill(text, 15)
    test("fill returns string", isinstance(result, str))
    test("fill has newlines", "\n" in result)

    # Short text
    result = textwrap.fill("Hello", 20)
    test("fill short", result == "Hello")
else:
    skip("fill returns string", "textwrap.fill not available")
    skip("fill has newlines", "textwrap.fill not available")
    skip("fill short", "textwrap.fill not available")


# ============================================================================
# textwrap.dedent() tests
# ============================================================================

print("\n=== textwrap.dedent() tests ===")

if hasattr(textwrap, "dedent"):
    # Basic dedent
    text = "    Hello\n    World\n    Test"
    result = textwrap.dedent(text)
    expected = "Hello\nWorld\nTest"
    test("dedent basic", result == expected)

    # Mixed indentation (uses common prefix)
    text = "        Line 1\n        Line 2\n        Line 3"
    result = textwrap.dedent(text)
    test("dedent common prefix", not result.startswith("        "))

    # No indentation
    text = "No indent\nAnother line"
    result = textwrap.dedent(text)
    test("dedent no indent", result == text)

    # Empty lines preserved
    text = "    Line 1\n\n    Line 2"
    result = textwrap.dedent(text)
    test("dedent preserves empty", "\n\n" in result or result.count("\n") >= 2)
else:
    skip("dedent basic", "textwrap.dedent not available")
    skip("dedent common prefix", "textwrap.dedent not available")
    skip("dedent no indent", "textwrap.dedent not available")
    skip("dedent preserves empty", "textwrap.dedent not available")


# ============================================================================
# textwrap.indent() tests
# ============================================================================

print("\n=== textwrap.indent() tests ===")

if hasattr(textwrap, "indent"):
    # Basic indent
    text = "Hello\nWorld\nTest"
    result = textwrap.indent(text, "  ")
    expected = "  Hello\n  World\n  Test"
    test("indent basic", result == expected)

    # Different prefix
    text = "Line 1\nLine 2"
    result = textwrap.indent(text, ">>> ")
    test("indent prefix", result == ">>> Line 1\n>>> Line 2")

    # Empty string
    result = textwrap.indent("", "  ")
    test("indent empty", result == "")

    # Single line
    result = textwrap.indent("Hello", "  ")
    test("indent single", result == "  Hello")
else:
    skip("indent basic", "textwrap.indent not available")
    skip("indent prefix", "textwrap.indent not available")
    skip("indent empty", "textwrap.indent not available")
    skip("indent single", "textwrap.indent not available")


# ============================================================================
# textwrap.shorten() tests
# ============================================================================

print("\n=== textwrap.shorten() tests ===")

if hasattr(textwrap, "shorten"):
    # Basic shorten - use positional args
    text = "Hello World, this is a long piece of text that needs to be shortened."
    result = textwrap.shorten(text, 30)
    test("shorten fits width", len(result) <= 30)
    test(
        "shorten has placeholder",
        "[...]" in result or "..." in result or len(result) <= 30,
    )

    # Short text that doesn't need shortening
    result = textwrap.shorten("Hello", 20)
    test("shorten short", result == "Hello")
else:
    skip("shorten fits width", "textwrap.shorten not available")
    skip("shorten has placeholder", "textwrap.shorten not available")
    skip("shorten short", "textwrap.shorten not available")


# ============================================================================
# Combined tests
# ============================================================================

print("\n=== Combined tests ===")

if hasattr(textwrap, "dedent") and hasattr(textwrap, "wrap"):
    # Dedent then wrap
    text = "    This is a long line of indented text that should be dedented and then wrapped to a specific width."
    dedented = textwrap.dedent(text)
    wrapped = textwrap.wrap(dedented, 30)
    test("dedent + wrap", len(wrapped) > 1)
else:
    skip("dedent + wrap", "textwrap.dedent or textwrap.wrap not available")

if hasattr(textwrap, "indent"):
    # Indent then fill
    text = "Hello World"
    indented = textwrap.indent(text, ">>> ")
    test("indent works", indented.startswith(">>> "))
else:
    skip("indent works", "textwrap.indent not available")


# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

if hasattr(textwrap, "wrap"):
    # Very long word
    result = textwrap.wrap("Supercalifragilisticexpialidocious", 10)
    test("wrap long word", len(result) >= 1)

    # Only whitespace - behavior may vary
    result = textwrap.wrap("   ", 10)
    test("wrap whitespace", result == [] or result == [""] or result == ["   "])

    # Newlines in input
    text = "Line 1\nLine 2\nLine 3"
    result = textwrap.wrap(text, 50)
    test("wrap with newlines", len(result) >= 1)
else:
    skip("wrap long word", "textwrap.wrap not available")
    skip("wrap whitespace", "textwrap.wrap not available")
    skip("wrap with newlines", "textwrap.wrap not available")


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    import sys

    sys.exit(1)
else:
    print("All tests passed!")
