"""
Minimal xml.etree.ElementTree tests for kipferl compatibility testing.

PocketPy doesn't support dotted import statements, so we import via __import__
and resolve attributes when available.
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
    ET = import_dotted("xml.etree.ElementTree")
    HAS_ET = True
except Exception:
    HAS_ET = False
    print("SKIP: xml.etree.ElementTree module not available")

if HAS_ET:
    print("\n=== Element/SubElement/tostring ===")
    test("has Element", hasattr(ET, "Element"))
    test("has SubElement", hasattr(ET, "SubElement"))
    test("has tostring", hasattr(ET, "tostring"))
    test("has fromstring", hasattr(ET, "fromstring"))

    try:
        root = ET.Element("root")
        child = ET.SubElement(root, "child")
        child.text = "hi"
        s = ET.tostring(root, "unicode")
        test("tostring returns str", isinstance(s, str))
        test("contains root tag", "<root>" in s and "</root>" in s)
        test("contains child tag", "<child>" in s and "</child>" in s)
        test("contains text", "hi" in s)
    except Exception as e:
        test("build xml", False)
        print(f"  ERROR: {e}")

    print("\n=== fromstring ===")
    try:
        xml = "<root><child>hi &lt;3</child></root>"
        root = ET.fromstring(xml)
        test("fromstring returns Element", hasattr(root, "tag") and root.tag == "root")
        children = list(root)
        child = children[0]
        test("child tag", child.tag == "child")
        test("child text", child.text == "hi <3")
        s2 = ET.tostring(root, "unicode")
        test("tostring after parse", "<child>" in s2 and "hi" in s2)
    except Exception as e:
        test("fromstring parse", False)
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
