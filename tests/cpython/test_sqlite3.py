"""
Minimal sqlite3 module tests for kipferl compatibility testing.
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
    import sqlite3

    HAS_SQLITE3 = True
except ImportError:
    HAS_SQLITE3 = False
    print("SKIP: sqlite3 module not available")

if HAS_SQLITE3:
    print("\n=== sqlite3 basic query ===")
    try:
        conn = sqlite3.connect(":memory:")
        cur = conn.cursor()
        cur.execute("CREATE TABLE t (x INTEGER)")
        cur.execute("INSERT INTO t (x) VALUES (?)", (1,))
        cur.execute("SELECT x FROM t")
        row = cur.fetchone()
        test("fetchone returns row", row is not None)
        if row is not None:
            test("row value is 1", row[0] == 1)
        cur.close()
        conn.close()
    except Exception as e:
        test("sqlite3 memory query works", False)
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
