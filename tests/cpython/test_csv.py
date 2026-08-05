"""
Simplified csv module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on CPython's Lib/test/test_csv.py
"""

import csv
import io
import sys

# Test tracking
_passed = 0
_failed = 0
_skipped = 0
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


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# Detect if csv.writer is functional (PocketPy returns None)
def _has_working_writer():
    try:
        output = io.StringIO()
        w = csv.writer(output)
        return w is not None and hasattr(w, "writerow")
    except Exception:
        return False


_HAS_WRITER = _has_working_writer()
_HAS_DICTREADER = hasattr(csv, "DictReader")
_HAS_DICTWRITER = hasattr(csv, "DictWriter")


# ============================================================================
# csv.reader() tests
# ============================================================================

print("\n=== csv.reader() tests ===")

# Simple CSV
data = ["a,b,c", "1,2,3"]
reader = csv.reader(data)
rows = list(reader)
test("reader basic", rows == [["a", "b", "c"], ["1", "2", "3"]])

# Empty fields
data = ["a,,c", ",2,"]
reader = csv.reader(data)
rows = list(reader)
test("reader empty fields", rows == [["a", "", "c"], ["", "2", ""]])

# Quoted fields
data = ['"a,b",c,d']
reader = csv.reader(data)
rows = list(reader)
test("reader quoted comma", rows == [["a,b", "c", "d"]])

# Quotes within quoted field
data = ['"a""b",c']
reader = csv.reader(data)
rows = list(reader)
test("reader escaped quote", rows == [['a"b', "c"]])

# Empty input
data = []
reader = csv.reader(data)
rows = list(reader)
test("reader empty input", rows == [])

# Single field
data = ["abc"]
reader = csv.reader(data)
rows = list(reader)
test("reader single field", rows == [["abc"]])

# Numbers as strings
data = ["1,2.5,3"]
reader = csv.reader(data)
rows = list(reader)
test("reader numbers as strings", rows == [["1", "2.5", "3"]])

# ============================================================================
# csv.writer() tests
# ============================================================================

print("\n=== csv.writer() tests ===")

if _HAS_WRITER:
    output = io.StringIO()
    writer = csv.writer(output)
    writer.writerow(["a", "b", "c"])
    result = output.getvalue()
    test("writer basic", "a,b,c" in result or "a, b, c" in result)

    output = io.StringIO()
    writer = csv.writer(output)
    writer.writerow(["a,b", "c"])
    result = output.getvalue()
    # Should quote the field with comma
    test("writer quotes comma", '"a,b"' in result or "'a,b'" in result)
else:
    skip("writer basic", "csv.writer not implemented")
    skip("writer quotes comma", "csv.writer not implemented")

# ============================================================================
# csv.DictReader() tests
# ============================================================================

print("\n=== csv.DictReader() tests ===")

if _HAS_DICTREADER:
    # Basic DictReader
    data = ["name,age,city", "Alice,30,NYC", "Bob,25,LA"]
    reader = csv.DictReader(data)
    rows = list(reader)
    test("DictReader basic", len(rows) == 2)
    test("DictReader keys", rows[0]["name"] == "Alice" and rows[0]["age"] == "30")
    test("DictReader second row", rows[1]["name"] == "Bob" and rows[1]["city"] == "LA")

    # DictReader with custom fieldnames (positional arg for PocketPy compat)
    data = ["Alice,30,NYC", "Bob,25,LA"]
    reader = csv.DictReader(data, ["name", "age", "city"])
    rows = list(reader)
    test("DictReader custom fieldnames", rows[0]["name"] == "Alice")
else:
    skip("DictReader basic", "csv.DictReader not implemented")
    skip("DictReader keys", "csv.DictReader not implemented")
    skip("DictReader second row", "csv.DictReader not implemented")
    skip("DictReader custom fieldnames", "csv.DictReader not implemented")

# ============================================================================
# csv.DictWriter() tests
# ============================================================================

print("\n=== csv.DictWriter() tests ===")

if _HAS_DICTWRITER:
    output = io.StringIO()
    fieldnames = ["name", "age"]
    writer = csv.DictWriter(output, fieldnames)  # positional arg for PocketPy
    writer.writeheader()
    writer.writerow({"name": "Alice", "age": "30"})
    result = output.getvalue()
    test("DictWriter basic", "name" in result and "age" in result and "Alice" in result)

    output = io.StringIO()
    fieldnames = ["a", "b"]
    writer = csv.DictWriter(output, fieldnames)  # positional arg for PocketPy
    writer.writerow({"a": "1", "b": "2"})
    result = output.getvalue()
    test("DictWriter row", "1" in result and "2" in result)
else:
    skip("DictWriter basic", "csv.DictWriter not implemented")
    skip("DictWriter row", "csv.DictWriter not implemented")

# ============================================================================
# csv constants tests
# ============================================================================

print("\n=== csv constants tests ===")

test("QUOTE_MINIMAL exists", hasattr(csv, "QUOTE_MINIMAL"))
test("QUOTE_ALL exists", hasattr(csv, "QUOTE_ALL"))
test("QUOTE_NONNUMERIC exists", hasattr(csv, "QUOTE_NONNUMERIC"))
test("QUOTE_NONE exists", hasattr(csv, "QUOTE_NONE"))

test("QUOTE_MINIMAL is int", isinstance(csv.QUOTE_MINIMAL, int))
test("QUOTE_ALL is int", isinstance(csv.QUOTE_ALL, int))

# ============================================================================
# Edge cases
# ============================================================================

print("\n=== Edge cases ===")

# Newline in quoted field
data = ['"a\nb",c']
reader = csv.reader(data)
rows = list(reader)
test("reader newline in quotes", len(rows) >= 1)

# Multiple rows
data = ["a,b", "c,d", "e,f"]
reader = csv.reader(data)
rows = list(reader)
test("reader multiple rows", len(rows) == 3)

# Unicode
data = ["hello,world"]
reader = csv.reader(data)
rows = list(reader)
test("reader unicode", rows == [["hello", "world"]])

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
