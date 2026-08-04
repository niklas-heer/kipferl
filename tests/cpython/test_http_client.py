"""
Minimal http.client tests for ucharm compatibility testing.

PocketPy doesn't support dotted import statements (e.g. `import http.client`),
so we import via __import__ and resolve attributes when available.
"""

import sys

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


def import_dotted(name):
    m = __import__(name)
    parts = name.split(".")
    cur = m
    for p in parts[1:]:
        if hasattr(cur, p):
            cur = getattr(cur, p)
        else:
            return m
    return cur


try:
    http_client = import_dotted("http.client")
    HAS_HTTP_CLIENT = True
except Exception:
    HAS_HTTP_CLIENT = False
    print("SKIP: http.client module not available")

if HAS_HTTP_CLIENT:
    print("\n=== HTTPConnection basics ===")
    test("has HTTPConnection", hasattr(http_client, "HTTPConnection"))
    test("has HTTPSConnection", hasattr(http_client, "HTTPSConnection"))
    test("has HTTPResponse", hasattr(http_client, "HTTPResponse"))

    try:
        conn = http_client.HTTPConnection("example.com", 80)
        test("connection has host attr", hasattr(conn, "host"))
        test("connection has port attr", hasattr(conn, "port"))
        if hasattr(conn, "host"):
            test("host is str", isinstance(conn.host, str))
        if hasattr(conn, "port"):
            test("port is int", isinstance(conn.port, int))
        test("has request method", hasattr(conn, "request"))
        test("has getresponse method", hasattr(conn, "getresponse"))
    except Exception as e:
        test("construct HTTPConnection", False)
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
