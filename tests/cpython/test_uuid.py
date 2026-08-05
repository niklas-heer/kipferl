"""
Simplified uuid module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_uuid.py
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


# Try to import uuid
try:
    import uuid

    HAS_UUID = True
except ImportError:
    HAS_UUID = False
    print("SKIP: uuid module not available")

if HAS_UUID:
    # ============================================================================
    # uuid4() tests
    # ============================================================================

    print("\n=== uuid4() tests ===")

    u = uuid.uuid4()
    test("uuid4 returns UUID", type(u).__name__ == "UUID")
    test("uuid4 has version", hasattr(u, "version"))
    test("uuid4 version is 4", u.version == 4)

    # UUID4 uniqueness
    uuids = [uuid.uuid4() for _ in range(10)]
    unique_strs = set([str(u) for u in uuids])
    test("uuid4 produces unique values", len(unique_strs) == 10)

    # ============================================================================
    # UUID from string
    # ============================================================================

    print("\n=== UUID from string ===")

    u = uuid.UUID("12345678-1234-5678-1234-567812345678")
    test("UUID from string", str(u) == "12345678-1234-5678-1234-567812345678")

    # ============================================================================
    # UUID attributes
    # ============================================================================

    print("\n=== UUID attributes ===")

    u = uuid.UUID("12345678-1234-5678-1234-567812345678")

    test("UUID has hex", hasattr(u, "hex"))
    test("UUID hex is string", isinstance(u.hex, str))
    test("UUID hex length", len(u.hex) == 32)

    test("UUID has bytes", hasattr(u, "bytes"))
    test("UUID bytes is bytes", isinstance(u.bytes, bytes))
    test("UUID bytes length", len(u.bytes) == 16)

    test("UUID has int", hasattr(u, "int"))
    test("UUID int is int", isinstance(u.int, int))

    # ============================================================================
    # String representation
    # ============================================================================

    print("\n=== String representation ===")

    u = uuid.UUID("12345678-1234-5678-1234-567812345678")
    s = str(u)
    test("str length", len(s) == 36)
    test("str has hyphens", s.count("-") == 4)
    test("str value", s == "12345678-1234-5678-1234-567812345678")

    # ============================================================================
    # Comparison
    # ============================================================================

    print("\n=== Comparison ===")

    u1 = uuid.UUID("12345678-1234-5678-1234-567812345678")
    u2 = uuid.UUID("12345678-1234-5678-1234-567812345678")
    u3 = uuid.UUID("12345678-1234-5678-1234-567812345679")

    test("equal UUIDs", u1 == u2)
    test("different UUIDs", u1 != u3)

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
