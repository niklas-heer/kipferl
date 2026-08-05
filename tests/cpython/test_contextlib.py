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

# contextmanager with exception - SKIPPED
# PocketPy's with statement doesn't call __exit__ with exception info,
# so exception handling in context managers doesn't work properly.
skip("contextmanager cleanup on exception", "PocketPy with statement limitation")

# ============================================================================
# suppress() tests
# ============================================================================

print("\n=== suppress() tests ===")

# suppress requires __exit__ to receive exception info and return True to suppress
# PocketPy's with statement doesn't support this, so skip exception-related tests
skip("suppress ValueError", "PocketPy with statement limitation")
skip("suppress doesn't catch other types", "PocketPy with statement limitation")

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

# closing on exception - SKIPPED
# PocketPy's with statement doesn't call __exit__ when exception is raised
skip("closing called close on exception", "PocketPy with statement limitation")

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
