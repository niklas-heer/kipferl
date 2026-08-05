"""
Simplified signal module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on Python's signal module functionality.
"""

import signal
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


def skip(name, reason=""):
    global _skipped
    _skipped += 1
    msg = f"  SKIP: {name}"
    if reason:
        msg += f" ({reason})"
    print(msg)


# ============================================================================
# Signal constants tests
# ============================================================================

print("\n=== Signal constants tests ===")

test("SIGINT exists", hasattr(signal, "SIGINT"))
test("SIGTERM exists", hasattr(signal, "SIGTERM"))
test("SIGKILL exists", hasattr(signal, "SIGKILL"))

test("SIGINT is int", isinstance(signal.SIGINT, int))
test("SIGINT value", signal.SIGINT == 2)

test("SIGTERM is int", isinstance(signal.SIGTERM, int))
test("SIGTERM value", signal.SIGTERM == 15)

test("SIGKILL is int", isinstance(signal.SIGKILL, int))
test("SIGKILL value", signal.SIGKILL == 9)


# ============================================================================
# SIG_DFL and SIG_IGN tests
# ============================================================================

print("\n=== SIG_DFL and SIG_IGN tests ===")

test("SIG_DFL exists", hasattr(signal, "SIG_DFL"))
test("SIG_IGN exists", hasattr(signal, "SIG_IGN"))


# ============================================================================
# signal.getsignal() tests
# ============================================================================

print("\n=== signal.getsignal() tests ===")

test("getsignal exists", callable(signal.getsignal))
handler = signal.getsignal(signal.SIGINT)
test("getsignal(SIGINT) works", True)


# ============================================================================
# signal.signal() tests
# ============================================================================

print("\n=== signal.signal() tests ===")

test("signal function exists", callable(signal.signal))

# SIGUSR1 may not be available on all platforms (e.g., Windows)
if hasattr(signal, "SIGUSR1"):
    old_handler = signal.signal(signal.SIGUSR1, signal.SIG_IGN)
    current = signal.getsignal(signal.SIGUSR1)
    test("signal() sets SIG_IGN", current == signal.SIG_IGN or current == 1)
    signal.signal(signal.SIGUSR1, signal.SIG_DFL)
else:
    skip("signal.signal tests with SIGUSR1", "SIGUSR1 not available on this platform")


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
