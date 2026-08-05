"""
Simplified unittest module tests for kipferl compatibility testing.
Works on both CPython and pocketpy-kipferl.

Based on CPython's Lib/test/test_unittest/
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
    import unittest
except ImportError:
    print("SKIP: unittest module not available")
    sys.exit(0)


# ============================================================================
# Basic TestCase
# ============================================================================

print("\n=== Basic TestCase ===")


class SimpleTestCase(unittest.TestCase):
    def test_pass(self):
        self.assertTrue(True)

    def test_fail(self):
        self.assertTrue(False)


# Create and run a single test
tc = SimpleTestCase("test_pass")
result = tc.run()
test("single test run", result.testsRun == 1)
test("single test success", result.wasSuccessful())

tc = SimpleTestCase("test_fail")
result = tc.run()
test("single test failure", len(result.failures) == 1)


# ============================================================================
# Assert methods
# ============================================================================

print("\n=== Assert methods ===")


class AssertTestCase(unittest.TestCase):
    def runTest(self):
        pass


tc = AssertTestCase()

# assertTrue/assertFalse
tc.assertTrue(True)
test("assertTrue passes", True)

try:
    tc.assertTrue(False)
    test("assertTrue fails", False)
except AssertionError:
    test("assertTrue fails", True)

tc.assertFalse(False)
test("assertFalse passes", True)

# assertEqual/assertNotEqual
tc.assertEqual(1, 1)
test("assertEqual passes", True)

try:
    tc.assertEqual(1, 2)
    test("assertEqual fails", False)
except AssertionError:
    test("assertEqual fails", True)

tc.assertNotEqual(1, 2)
test("assertNotEqual passes", True)

# assertIs/assertIsNot
a = []
tc.assertIs(a, a)
test("assertIs passes", True)

tc.assertIsNot([], [])
test("assertIsNot passes", True)

# assertIsNone/assertIsNotNone
tc.assertIsNone(None)
test("assertIsNone passes", True)

tc.assertIsNotNone(42)
test("assertIsNotNone passes", True)

# assertIn/assertNotIn
tc.assertIn(1, [1, 2, 3])
test("assertIn passes", True)

tc.assertNotIn(4, [1, 2, 3])
test("assertNotIn passes", True)

# assertIsInstance/assertNotIsInstance
tc.assertIsInstance([], list)
test("assertIsInstance passes", True)

tc.assertNotIsInstance([], dict)
test("assertNotIsInstance passes", True)

# assertGreater/assertLess
tc.assertGreater(5, 3)
test("assertGreater passes", True)

tc.assertLess(3, 5)
test("assertLess passes", True)

tc.assertGreaterEqual(5, 5)
test("assertGreaterEqual passes", True)

tc.assertLessEqual(5, 5)
test("assertLessEqual passes", True)

# assertAlmostEqual
tc.assertAlmostEqual(1.0000001, 1.0000002, places=5)
test("assertAlmostEqual passes", True)


# ============================================================================
# assertRaises
# ============================================================================

print("\n=== assertRaises ===")


class RaisesTestCase(unittest.TestCase):
    def runTest(self):
        pass


tc = RaisesTestCase()


# With callable
def raise_value_error():
    raise ValueError("test error")


tc.assertRaises(ValueError, raise_value_error)
test("assertRaises callable", True)

# With context manager
try:
    with tc.assertRaises(ValueError):
        raise ValueError("test")
    test("assertRaises context manager", True)
except:
    test("assertRaises context manager", False)

# Should fail when exception not raised
try:
    tc.assertRaises(ValueError, lambda: None)
    test("assertRaises no exception fails", False)
except AssertionError:
    test("assertRaises no exception fails", True)


# ============================================================================
# setUp and tearDown
# ============================================================================

print("\n=== setUp and tearDown ===")

setup_called = False
teardown_called = False


class SetupTeardownTestCase(unittest.TestCase):
    def setUp(self):
        global setup_called
        setup_called = True

    def tearDown(self):
        global teardown_called
        teardown_called = True

    def test_method(self):
        pass


tc = SetupTeardownTestCase("test_method")
tc.run()
test("setUp called", setup_called)
test("tearDown called", teardown_called)


# ============================================================================
# TestSuite
# ============================================================================

print("\n=== TestSuite ===")


class SuiteTestCase1(unittest.TestCase):
    def test_one(self):
        pass


class SuiteTestCase2(unittest.TestCase):
    def test_two(self):
        pass


suite = unittest.TestSuite()
suite.addTest(SuiteTestCase1("test_one"))
suite.addTest(SuiteTestCase2("test_two"))

result = unittest.TestResult()
suite.run(result)
test("suite runs all tests", result.testsRun == 2)
test("suite all pass", result.wasSuccessful())


# ============================================================================
# TestLoader
# ============================================================================

print("\n=== TestLoader ===")


class LoaderTestCase(unittest.TestCase):
    def test_alpha(self):
        pass

    def test_beta(self):
        pass

    def helper_method(self):
        pass


loader = unittest.TestLoader()
suite = loader.loadTestsFromTestCase(LoaderTestCase)
test("loader finds test methods", suite.countTestCases() == 2)


# ============================================================================
# Skip decorators
# ============================================================================

print("\n=== Skip decorators ===")


class SkipTestCase(unittest.TestCase):
    @unittest.skip("testing skip")
    def test_skipped(self):
        self.fail("should not run")

    @unittest.skipIf(True, "condition true")
    def test_skip_if(self):
        self.fail("should not run")

    @unittest.skipUnless(False, "condition false")
    def test_skip_unless(self):
        self.fail("should not run")

    def test_not_skipped(self):
        pass


tc = SkipTestCase("test_skipped")
result = tc.run()
test("skip decorator works", len(result.skipped) == 1)

tc = SkipTestCase("test_skip_if")
result = tc.run()
test("skipIf decorator works", len(result.skipped) == 1)

tc = SkipTestCase("test_skip_unless")
result = tc.run()
test("skipUnless decorator works", len(result.skipped) == 1)

tc = SkipTestCase("test_not_skipped")
result = tc.run()
test("non-skipped test runs", result.wasSuccessful())


# ============================================================================
# skipTest method
# ============================================================================

print("\n=== skipTest method ===")


class SkipTestMethodCase(unittest.TestCase):
    def test_skip_in_test(self):
        self.skipTest("skipping programmatically")
        self.fail("should not reach here")


tc = SkipTestMethodCase("test_skip_in_test")
result = tc.run()
test("skipTest method works", len(result.skipped) == 1)


# ============================================================================
# expectedFailure
# ============================================================================

print("\n=== expectedFailure ===")


class ExpectedFailureCase(unittest.TestCase):
    @unittest.expectedFailure
    def test_expected_fail(self):
        self.fail("expected to fail")

    @unittest.expectedFailure
    def test_unexpected_success(self):
        pass


tc = ExpectedFailureCase("test_expected_fail")
result = tc.run()
test("expectedFailure works", len(result.expectedFailures) == 1)

tc = ExpectedFailureCase("test_unexpected_success")
result = tc.run()
test("unexpected success detected", len(result.unexpectedSuccesses) == 1)


# ============================================================================
# TestResult
# ============================================================================

print("\n=== TestResult ===")

result = unittest.TestResult()
test("result initial testsRun", result.testsRun == 0)
test("result initial successful", result.wasSuccessful())

result.testsRun = 5
result.failures.append(("test", "error"))
test("result with failure not successful", not result.wasSuccessful())


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
