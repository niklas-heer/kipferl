"""
Simplified argparse module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on CPython's Lib/test/test_argparse.py
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


try:
    import argparse
except ImportError:
    print("SKIP: argparse module not available")
    sys.exit(0)


# ============================================================================
# Basic argument parsing
# ============================================================================

print("\n=== Basic argument parsing ===")

# Positional arguments
parser = argparse.ArgumentParser()
parser.add_argument("name")
args = parser.parse_args(["Alice"])
test("positional arg", args.name == "Alice")

# Optional arguments (long form)
parser = argparse.ArgumentParser()
parser.add_argument("--name")
args = parser.parse_args(["--name", "Bob"])
test("optional long arg", args.name == "Bob")

# Optional arguments (short form)
parser = argparse.ArgumentParser()
parser.add_argument("-n", "--name")
args = parser.parse_args(["-n", "Charlie"])
test("optional short arg", args.name == "Charlie")

# Default value
parser = argparse.ArgumentParser()
parser.add_argument("--name", default="Unknown")
args = parser.parse_args([])
test("default value", args.name == "Unknown")


# ============================================================================
# Argument types
# ============================================================================

print("\n=== Argument types ===")

# Integer type
parser = argparse.ArgumentParser()
parser.add_argument("--count", type=int)
args = parser.parse_args(["--count", "42"])
test("int type", args.count == 42)

# Float type
parser = argparse.ArgumentParser()
parser.add_argument("--value", type=float)
args = parser.parse_args(["--value", "3.14"])
test("float type", abs(args.value - 3.14) < 0.001)

# String type (default)
parser = argparse.ArgumentParser()
parser.add_argument("--text", type=str)
args = parser.parse_args(["--text", "hello"])
test("str type", args.text == "hello")


# ============================================================================
# Boolean flags
# ============================================================================

print("\n=== Boolean flags ===")

# store_true
parser = argparse.ArgumentParser()
parser.add_argument("--verbose", action="store_true")
args = parser.parse_args(["--verbose"])
test("store_true present", args.verbose is True)

parser = argparse.ArgumentParser()
parser.add_argument("--verbose", action="store_true")
args = parser.parse_args([])
test("store_true absent", args.verbose is False)

# store_false
parser = argparse.ArgumentParser()
parser.add_argument("--no-check", action="store_false", dest="check")
args = parser.parse_args(["--no-check"])
test("store_false present", args.check is False)

parser = argparse.ArgumentParser()
parser.add_argument("--no-check", action="store_false", dest="check")
args = parser.parse_args([])
test("store_false absent", args.check is True)


# ============================================================================
# Multiple values
# ============================================================================

print("\n=== Multiple values ===")

# nargs='*'
parser = argparse.ArgumentParser()
parser.add_argument("--files", nargs="*")
args = parser.parse_args(["--files", "a.txt", "b.txt", "c.txt"])
test("nargs * multiple", args.files == ["a.txt", "b.txt", "c.txt"])

parser = argparse.ArgumentParser()
parser.add_argument("--files", nargs="*")
args = parser.parse_args(["--files"])
test("nargs * empty", args.files == [])

# nargs='+'
parser = argparse.ArgumentParser()
parser.add_argument("files", nargs="+")
args = parser.parse_args(["a.txt", "b.txt"])
test("nargs + positional", args.files == ["a.txt", "b.txt"])

# nargs='?'
parser = argparse.ArgumentParser()
parser.add_argument("--config", nargs="?", const="default.cfg")
args = parser.parse_args(["--config"])
test("nargs ? no value", args.config == "default.cfg")

parser = argparse.ArgumentParser()
parser.add_argument("--config", nargs="?", const="default.cfg")
args = parser.parse_args(["--config", "custom.cfg"])
test("nargs ? with value", args.config == "custom.cfg")


# ============================================================================
# Choices
# ============================================================================

print("\n=== Choices ===")

parser = argparse.ArgumentParser()
parser.add_argument("--color", choices=["red", "green", "blue"])
args = parser.parse_args(["--color", "green"])
test("valid choice", args.color == "green")

# Invalid choice should raise error
parser = argparse.ArgumentParser()
parser.add_argument("--color", choices=["red", "green", "blue"])
try:
    args = parser.parse_args(["--color", "yellow"])
    test("invalid choice raises", False)
except SystemExit:
    test("invalid choice raises", True)


# ============================================================================
# Required arguments
# ============================================================================

print("\n=== Required arguments ===")

parser = argparse.ArgumentParser()
parser.add_argument("--name", required=True)
try:
    args = parser.parse_args([])
    test("required missing raises", False)
except SystemExit:
    test("required missing raises", True)


# ============================================================================
# Mutually exclusive groups
# ============================================================================

print("\n=== Mutually exclusive groups ===")

parser = argparse.ArgumentParser()
group = parser.add_mutually_exclusive_group()
group.add_argument("--verbose", action="store_true")
group.add_argument("--quiet", action="store_true")

# Only one should be allowed
args = parser.parse_args(["--verbose"])
test("mutually exclusive one", args.verbose and not args.quiet)

# Both should fail
parser = argparse.ArgumentParser()
group = parser.add_mutually_exclusive_group()
group.add_argument("--verbose", action="store_true")
group.add_argument("--quiet", action="store_true")
try:
    args = parser.parse_args(["--verbose", "--quiet"])
    test("mutually exclusive both fails", False)
except SystemExit:
    test("mutually exclusive both fails", True)


# ============================================================================
# Subparsers
# ============================================================================

print("\n=== Subparsers ===")

parser = argparse.ArgumentParser()
subparsers = parser.add_subparsers(dest="command")

# Add subparser
parser_add = subparsers.add_parser("add")
parser_add.add_argument("name")

parser_remove = subparsers.add_parser("remove")
parser_remove.add_argument("--force", action="store_true")

args = parser.parse_args(["add", "item"])
test("subparser add", args.command == "add" and args.name == "item")

args = parser.parse_args(["remove", "--force"])
test("subparser remove", args.command == "remove" and args.force)


# ============================================================================
# Help text
# ============================================================================

print("\n=== Help text ===")

parser = argparse.ArgumentParser(description="Test program")
parser.add_argument("--name", help="Your name")
test("description set", parser.description == "Test program")


# ============================================================================
# Dest customization
# ============================================================================

print("\n=== Dest customization ===")

parser = argparse.ArgumentParser()
parser.add_argument("--file-name", dest="filename")
args = parser.parse_args(["--file-name", "test.txt"])
test("custom dest", args.filename == "test.txt")


# ============================================================================
# Combined short options
# ============================================================================

print("\n=== Combined short options ===")

parser = argparse.ArgumentParser()
parser.add_argument("-v", action="store_true")
parser.add_argument("-x", action="store_true")
args = parser.parse_args(["-v", "-x"])
test("separate short opts", args.v and args.x)


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
