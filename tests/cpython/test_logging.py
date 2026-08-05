"""
Simplified logging module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on CPython's Lib/test/test_logging.py
"""

import logging
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


# ============================================================================
# Log level constants tests
# ============================================================================

print("\n=== Log level constants tests ===")

test("DEBUG constant defined", hasattr(logging, "DEBUG"))
test("INFO constant defined", hasattr(logging, "INFO"))
test("WARNING constant defined", hasattr(logging, "WARNING"))
test("ERROR constant defined", hasattr(logging, "ERROR"))
test("CRITICAL constant defined", hasattr(logging, "CRITICAL"))

test("DEBUG value is 10", logging.DEBUG == 10)
test("INFO value is 20", logging.INFO == 20)
test("WARNING value is 30", logging.WARNING == 30)
test("ERROR value is 40", logging.ERROR == 40)
test("CRITICAL value is 50", logging.CRITICAL == 50)


# ============================================================================
# getLogger() tests
# ============================================================================

print("\n=== getLogger() tests ===")

test("getLogger callable", callable(logging.getLogger))
logger = logging.getLogger("test_logger")
test("getLogger returns object", logger is not None)

# Test that getLogger returns same logger for same name
# NOTE: PocketPy's logging module doesn't cache loggers by name
logger2 = logging.getLogger("test_logger")
if hasattr(logging, "StreamHandler"):
    # CPython caches loggers
    test("getLogger returns same logger for same name", logger is logger2)
else:
    # PocketPy creates new logger instances
    skip(
        "getLogger returns same logger for same name", "PocketPy doesn't cache loggers"
    )

# Test root logger
root_logger = logging.getLogger()
test("getLogger() returns root logger", root_logger is not None)


# ============================================================================
# Logger methods tests
# ============================================================================

print("\n=== Logger methods tests ===")

logger = logging.getLogger("methods_test")
test("logger.debug exists", hasattr(logger, "debug") and callable(logger.debug))
test("logger.info exists", hasattr(logger, "info") and callable(logger.info))
test("logger.warning exists", hasattr(logger, "warning") and callable(logger.warning))
test("logger.error exists", hasattr(logger, "error") and callable(logger.error))
test(
    "logger.critical exists", hasattr(logger, "critical") and callable(logger.critical)
)

# Test setLevel and getEffectiveLevel
test(
    "logger.setLevel exists", hasattr(logger, "setLevel") and callable(logger.setLevel)
)
logger.setLevel(logging.DEBUG)
has_get_eff = hasattr(logger, "getEffectiveLevel") and callable(
    logger.getEffectiveLevel
)
test("logger.getEffectiveLevel exists", has_get_eff)
test(
    "getEffectiveLevel returns correct level",
    logger.getEffectiveLevel() == logging.DEBUG,
)


# ============================================================================
# Module-level logging functions tests
# ============================================================================

print("\n=== Module-level logging functions tests ===")

test("logging.debug exists", hasattr(logging, "debug") and callable(logging.debug))
test("logging.info exists", hasattr(logging, "info") and callable(logging.info))
test(
    "logging.warning exists", hasattr(logging, "warning") and callable(logging.warning)
)
test("logging.error exists", hasattr(logging, "error") and callable(logging.error))
test(
    "logging.critical exists",
    hasattr(logging, "critical") and callable(logging.critical),
)


# ============================================================================
# Handler tests (CPython-only features)
# ============================================================================

print("\n=== Handler tests ===")

# Handler, StreamHandler, FileHandler are CPython-specific
# PocketPy provides a simplified logging module without handlers
if hasattr(logging, "Handler"):
    test("Handler class exists", True)
    test("StreamHandler class exists", hasattr(logging, "StreamHandler"))
    test("FileHandler class exists", hasattr(logging, "FileHandler"))

    # Test StreamHandler
    stream_handler = logging.StreamHandler()
    test("StreamHandler instantiation", stream_handler is not None)
    has_set_level = hasattr(stream_handler, "setLevel") and callable(
        stream_handler.setLevel
    )
    test("StreamHandler.setLevel exists", has_set_level)
    has_set_fmt = hasattr(stream_handler, "setFormatter") and callable(
        stream_handler.setFormatter
    )
    test("StreamHandler.setFormatter exists", has_set_fmt)

    # Test adding handler to logger
    handler_logger = logging.getLogger("handler_test")
    handler_logger.addHandler(stream_handler)
    test("logger.addHandler works", True)  # If we get here, it worked
else:
    skip("Handler class exists", "PocketPy uses simplified logging")
    skip("StreamHandler class exists", "PocketPy uses simplified logging")
    skip("FileHandler class exists", "PocketPy uses simplified logging")
    skip("StreamHandler instantiation", "PocketPy uses simplified logging")
    skip("StreamHandler.setLevel exists", "PocketPy uses simplified logging")
    skip("StreamHandler.setFormatter exists", "PocketPy uses simplified logging")
    skip("logger.addHandler works", "PocketPy uses simplified logging")


# ============================================================================
# Formatter tests (CPython-only features)
# ============================================================================

print("\n=== Formatter tests ===")

if hasattr(logging, "Formatter"):
    test("Formatter class exists", True)

    formatter = logging.Formatter("%(levelname)s: %(message)s")
    test("Formatter instantiation", formatter is not None)

    # Apply formatter to handler (only if StreamHandler exists)
    if hasattr(logging, "StreamHandler"):
        stream_handler = logging.StreamHandler()
        stream_handler.setFormatter(formatter)
        test("setFormatter works", True)  # If we get here, it worked
    else:
        skip("setFormatter works", "No StreamHandler available")
else:
    skip("Formatter class exists", "PocketPy uses simplified logging")
    skip("Formatter instantiation", "PocketPy uses simplified logging")
    skip("setFormatter works", "PocketPy uses simplified logging")


# ============================================================================
# Logger hierarchy tests (CPython-only features)
# ============================================================================

print("\n=== Logger hierarchy tests ===")

parent_logger = logging.getLogger("parent")
child_logger = logging.getLogger("parent.child")
test("Child logger created", child_logger is not None)

# Logger hierarchy with parent attribute is CPython-specific
if hasattr(child_logger, "parent"):
    test("Child logger has parent attribute", True)
else:
    skip("Child logger has parent attribute", "PocketPy uses simplified logging")


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
