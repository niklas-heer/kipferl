"""
Tests for the fetch module (kipferl's requests-like HTTP client).

Note: These tests require network access to httpbin.org or similar.
Tests that require network are skipped if SKIP_NETWORK_TESTS=1.
"""

import os
import sys

_passed = 0
_failed = 0
_errors = []
_skipped = 0

SKIP_NETWORK = os.environ.get("SKIP_NETWORK_TESTS", "0") == "1"


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


try:
    import fetch

    HAS_FETCH = True
except ImportError:
    HAS_FETCH = False
    print("SKIP: fetch module not available")

if HAS_FETCH:
    print("\n=== Module attributes ===")
    test("has get function", hasattr(fetch, "get"))
    test("has post function", hasattr(fetch, "post"))
    test("has request function", hasattr(fetch, "request"))

    print("\n=== Function signatures ===")
    # Test that functions are callable
    test("get is callable", callable(fetch.get))
    test("post is callable", callable(fetch.post))
    test("request is callable", callable(fetch.request))

    if SKIP_NETWORK:
        print("\n=== Network tests ===")
        skip("GET request", "SKIP_NETWORK_TESTS=1")
        skip("POST request", "SKIP_NETWORK_TESTS=1")
        skip("POST with JSON", "SKIP_NETWORK_TESTS=1")
        skip("Custom headers", "SKIP_NETWORK_TESTS=1")
    else:
        print("\n=== Network tests (httpbin.org) ===")

        # Test basic GET
        try:
            r = fetch.get("https://httpbin.org/get")
            test("GET returns dict", isinstance(r, dict))
            test("GET has status", "status" in r)
            test("GET status is 200", r.get("status") == 200)
            test("GET has body", "body" in r)
            test("GET body is bytes", isinstance(r.get("body"), bytes))
            test("GET has headers", "headers" in r)
        except Exception as e:
            test("GET request", False)
            print(f"  ERROR: {e}")

        # Test POST with data
        try:
            r = fetch.post("https://httpbin.org/post", data=b"hello")
            test("POST returns dict", isinstance(r, dict))
            test("POST status is 200", r.get("status") == 200)
        except Exception as e:
            test("POST request", False)
            print(f"  ERROR: {e}")

        # Test POST with JSON
        try:
            r = fetch.post("https://httpbin.org/post", json={"key": "value"})
            test("POST JSON status is 200", r.get("status") == 200)
            body = r.get("body", b"").decode()
            test("POST JSON body contains key", '"key"' in body)
        except Exception as e:
            test("POST with JSON", False)
            print(f"  ERROR: {e}")

        # Test custom headers
        try:
            r = fetch.get(
                "https://httpbin.org/headers", headers={"X-Custom": "test123"}
            )
            test("Custom header status is 200", r.get("status") == 200)
            body = r.get("body", b"").decode()
            test("Custom header in response", "test123" in body)
        except Exception as e:
            test("Custom headers", False)
            print(f"  ERROR: {e}")

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
else:
    print("All tests passed!")
