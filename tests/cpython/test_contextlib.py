"""
Simplified contextlib module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_contextlib.py
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


import contextlib
from contextlib import closing, contextmanager, nullcontext, suppress

# ============================================================================
# contextmanager decorator tests
# ============================================================================

print("\n=== contextmanager tests ===")


@contextmanager
def simple_context():
    yield "value"


with simple_context() as val:
    result = val
test("contextmanager basic yield", result == "value")

# contextmanager with setup and teardown
setup_called = False
teardown_called = False


@contextmanager
def setup_teardown():
    global setup_called, teardown_called
    setup_called = True
    yield "middle"
    teardown_called = True


with setup_teardown() as val:
    test("contextmanager setup called", setup_called)
    test("contextmanager teardown not yet called", not teardown_called)

test("contextmanager teardown called after", teardown_called)

cleaned_up = False


@contextmanager
def guarded():
    global cleaned_up
    try:
        yield
    finally:
        cleaned_up = True


try:
    with guarded():
        raise ValueError("boom")
except ValueError:
    pass
test("contextmanager cleanup on exception", cleaned_up)

# ============================================================================
# suppress() tests
# ============================================================================

print("\n=== suppress() tests ===")

result = "raised"
with suppress(ValueError):
    raise ValueError("ignored")
    result = "unreachable"
test("suppress ValueError", result == "raised")

try:
    with suppress(ValueError):
        raise KeyError("kept")
    test("suppress doesn't catch other types", False)
except KeyError:
    test("suppress doesn't catch other types", True)

result = "success"
with suppress(ValueError):
    result = "completed"
test("suppress no exception", result == "completed")

# ============================================================================
# closing() tests
# ============================================================================

print("\n=== closing() tests ===")


class Closeable:
    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True


obj = Closeable()
with closing(obj) as c:
    test("closing yields object", c is obj)
    test("closing not closed during", not obj.closed)

test("closing called close", obj.closed)

obj = Closeable()
try:
    with closing(obj):
        raise ValueError("boom")
except ValueError:
    pass
test("closing called close on exception", obj.closed)

# ============================================================================
# nullcontext() tests
# ============================================================================

print("\n=== nullcontext() tests ===")

with nullcontext() as val:
    pass
test("nullcontext no arg yields None", val is None)

with nullcontext("hello") as val:
    pass
test("nullcontext with value", val == "hello")

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
