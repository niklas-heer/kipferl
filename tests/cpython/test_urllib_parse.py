"""
Simplified urllib.parse module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_urlparse.py
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


# Try to import urllib.parse
try:
    from urllib.parse import quote, unquote, urlencode, urljoin, urlparse, urlunparse

    HAS_URLLIB_PARSE = True
except ImportError:
    HAS_URLLIB_PARSE = False
    print("SKIP: urllib.parse module not available")

if HAS_URLLIB_PARSE:
    # ============================================================================
    # urlparse() tests
    # ============================================================================

    print("\n=== urlparse() tests ===")

    result = urlparse("http://www.example.com:80/path?query=value#fragment")
    test("urlparse scheme", result.scheme == "http")
    test("urlparse netloc", result.netloc == "www.example.com:80")
    test("urlparse path", result.path == "/path")
    test("urlparse query", result.query == "query=value")
    test("urlparse fragment", result.fragment == "fragment")

    result = urlparse("https://example.com")
    test("urlparse https", result.scheme == "https")
    test("urlparse simple netloc", result.netloc == "example.com")

    result = urlparse("/path/to/file")
    test("urlparse path-only", result.path == "/path/to/file")
    test("urlparse path-only scheme", result.scheme == "")

    # ============================================================================
    # urlunparse() tests
    # ============================================================================

    print("\n=== urlunparse() tests ===")

    parts = ("http", "www.example.com", "/path", "", "query=value", "fragment")
    expected = "http://www.example.com/path?query=value#fragment"
    test("urlunparse basic", urlunparse(parts) == expected)

    # Roundtrip
    original = "http://www.example.com:8080/path?query=value#section"
    parsed = urlparse(original)
    test("urlunparse roundtrip", urlunparse(parsed) == original)

    # ============================================================================
    # urljoin() tests
    # ============================================================================

    print("\n=== urljoin() tests ===")

    result = urljoin("http://example.com/path/", "file.html")
    test("urljoin relative", result == "http://example.com/path/file.html")

    result = urljoin("http://example.com/path/", "/other.html")
    test("urljoin absolute", result == "http://example.com/other.html")

    result = urljoin("http://example.com/", "http://other.com/")
    test("urljoin full url", result == "http://other.com/")

    # ============================================================================
    # quote() and unquote() tests
    # ============================================================================

    print("\n=== quote() and unquote() tests ===")

    test("quote simple", quote("hello") == "hello")
    test("quote space", quote("hello world") == "hello%20world")
    test("quote special", quote("a=b&c=d") == "a%3Db%26c%3Dd")

    test("unquote simple", unquote("hello") == "hello")
    test("unquote space", unquote("hello%20world") == "hello world")
    test("unquote special", unquote("a%3Db%26c%3Dd") == "a=b&c=d")

    # Roundtrip
    original = "Hello, World! Special chars: =&?#"
    test("quote unquote roundtrip", unquote(quote(original)) == original)

    # ============================================================================
    # urlencode() tests
    # ============================================================================

    print("\n=== urlencode() tests ===")

    params = {"key": "value"}
    test("urlencode single", urlencode(params) == "key=value")

    params = [("a", "1"), ("b", "2")]
    test("urlencode tuples", urlencode(params) == "a=1&b=2")

    params = {"key": "hello world"}
    test("urlencode spaces", urlencode(params) == "key=hello+world")

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
