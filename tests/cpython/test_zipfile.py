"""
Minimal zipfile module tests for kipferl compatibility testing.
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
    import zipfile

    HAS_ZIPFILE = True
except ImportError:
    HAS_ZIPFILE = False
    print("SKIP: zipfile module not available")

if HAS_ZIPFILE:
    print("\n=== zipfile.ZipFile ===")
    test(
        "has is_zipfile",
        hasattr(zipfile, "is_zipfile") and callable(zipfile.is_zipfile),
    )
    test("has ZipFile", hasattr(zipfile, "ZipFile"))
    try:
        test(
            "is_zipfile missing returns False",
            zipfile.is_zipfile("__kipferl_missing.zip") is False,
        )
    except Exception as e:
        test("is_zipfile handles missing", False)
        print(f"  ERROR: {e}")

    ZIP_BYTES = b"PK\x03\x04\x14\x00\x00\x00\x00\x00\xf9\xb6\x94[\xac*\x93\xd8\x02\x00\x00\x00\x02\x00\x00\x00\x05\x00\x00\x00a.txthiPK\x03\x04\x14\x00\x00\x00\x00\x00\xf9\xb6\x94[\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\t\x00\x00\x00empty.txtPK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00\xf9\xb6\x94[\xac*\x93\xd8\x02\x00\x00\x00\x02\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x80\x01\x00\x00\x00\x00a.txtPK\x01\x02\x14\x03\x14\x00\x00\x00\x00\x00\xf9\xb6\x94[\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\t\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x80\x01%\x00\x00\x00empty.txtPK\x05\x06\x00\x00\x00\x00\x02\x00\x02\x00j\x00\x00\x00L\x00\x00\x00\x00\x00"
    path = "__kipferl_test.zip"
    try:
        with open(path, "wb") as f:
            f.write(ZIP_BYTES)
        test("is_zipfile returns True", zipfile.is_zipfile(path) is True)
        z = zipfile.ZipFile(path, "r")
        names = z.namelist()
        test("namelist returns list", isinstance(names, list))
        test("contains a.txt", "a.txt" in names)
        test("read a.txt", z.read("a.txt") == b"hi")
        z.close()
        try:
            import os

            os.remove(path)
        except Exception:
            pass
    except Exception as e:
        test("ZipFile read", False)
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
