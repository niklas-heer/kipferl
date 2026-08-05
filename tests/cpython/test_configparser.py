"""
Simplified configparser module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_configparser.py
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


# Try to import configparser
try:
    import configparser

    HAS_CONFIGPARSER = True
except ImportError:
    HAS_CONFIGPARSER = False
    print("SKIP: configparser module not available")

if HAS_CONFIGPARSER:
    # ============================================================================
    # ConfigParser creation
    # ============================================================================

    print("\n=== ConfigParser creation ===")

    cp = configparser.ConfigParser()
    test("create ConfigParser", cp is not None)
    test("ConfigParser has read_string", hasattr(cp, "read_string"))
    test("ConfigParser has sections", hasattr(cp, "sections"))
    test("ConfigParser has get", hasattr(cp, "get"))

    # ============================================================================
    # read_string() and basic parsing
    # ============================================================================

    print("\n=== read_string() and basic parsing ===")

    simple_ini = """
[section1]
key1 = value1
key2 = value2

[section2]
option_a = hello
option_b = world
"""

    cp = configparser.ConfigParser()
    cp.read_string(simple_ini)

    test("read_string parses sections", len(cp.sections()) == 2)
    test("section1 exists", "section1" in cp.sections())
    test("section2 exists", "section2" in cp.sections())

    # ============================================================================
    # sections() method
    # ============================================================================

    print("\n=== sections() method ===")

    cp = configparser.ConfigParser()
    test("empty parser has no sections", cp.sections() == [])

    cp.read_string(simple_ini)
    sections = cp.sections()
    test("sections returns list", isinstance(sections, list))
    test("sections count", len(sections) == 2)

    # ============================================================================
    # get() method
    # ============================================================================

    print("\n=== get() method ===")

    cp = configparser.ConfigParser()
    cp.read_string(simple_ini)

    test("get key1", cp.get("section1", "key1") == "value1")
    test("get key2", cp.get("section1", "key2") == "value2")
    test("get option_a", cp.get("section2", "option_a") == "hello")

    # ============================================================================
    # getint() method
    # ============================================================================

    print("\n=== getint() method ===")

    int_ini = """
[numbers]
positive = 42
negative = -17
zero = 0
"""

    cp = configparser.ConfigParser()
    cp.read_string(int_ini)

    test("getint positive", cp.getint("numbers", "positive") == 42)
    test("getint negative", cp.getint("numbers", "negative") == -17)
    test("getint zero", cp.getint("numbers", "zero") == 0)

    # ============================================================================
    # getfloat() method
    # ============================================================================

    print("\n=== getfloat() method ===")

    float_ini = """
[floats]
pi = 3.14159
negative = -2.5
"""

    cp = configparser.ConfigParser()
    cp.read_string(float_ini)

    test("getfloat pi", abs(cp.getfloat("floats", "pi") - 3.14159) < 0.0001)
    test("getfloat negative", abs(cp.getfloat("floats", "negative") - (-2.5)) < 0.0001)

    # ============================================================================
    # getboolean() method
    # ============================================================================

    print("\n=== getboolean() method ===")

    bool_ini = """
[booleans]
yes_val = yes
no_val = no
true_val = true
false_val = false
"""

    cp = configparser.ConfigParser()
    cp.read_string(bool_ini)

    test("getboolean yes", cp.getboolean("booleans", "yes_val") is True)
    test("getboolean no", cp.getboolean("booleans", "no_val") is False)
    test("getboolean true", cp.getboolean("booleans", "true_val") is True)
    test("getboolean false", cp.getboolean("booleans", "false_val") is False)

    # ============================================================================
    # has_section() and has_option()
    # ============================================================================

    print("\n=== has_section() and has_option() ===")

    cp = configparser.ConfigParser()
    cp.read_string(simple_ini)

    test("has_section section1", cp.has_section("section1") is True)
    test("has_section missing", cp.has_section("missing") is False)
    test("has_option key1 exists", cp.has_option("section1", "key1") is True)
    test("has_option missing key", cp.has_option("section1", "missing") is False)

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
