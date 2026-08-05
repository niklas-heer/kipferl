"""
Minimal gzip module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.
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


try:
    import gzip

    HAS_GZIP = True
except ImportError:
    HAS_GZIP = False
    print("SKIP: gzip module not available")

if HAS_GZIP:
    print("\n=== gzip.compress/decompress ===")
    test("has compress", hasattr(gzip, "compress") and callable(gzip.compress))
    test("has decompress", hasattr(gzip, "decompress") and callable(gzip.decompress))
    try:
        data = b"hello kipferl"
        compressed = gzip.compress(data)
        test("compress returns bytes", isinstance(compressed, (bytes, bytearray)))
        test(
            "has gzip magic",
            len(compressed) >= 2 and compressed[0] == 0x1F and compressed[1] == 0x8B,
        )
        roundtrip = gzip.decompress(compressed)
        test("roundtrip bytes", roundtrip == data)
        test("empty roundtrip", gzip.decompress(gzip.compress(b"")) == b"")
    except Exception as e:
        test("gzip roundtrip", False)
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
