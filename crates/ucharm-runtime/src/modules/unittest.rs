use ucharm_pocketpy_sys as ffi;

use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r#"
class SkipTest(Exception):
    pass

class TestResult:
    def __init__(self):
        self.testsRun = 0
        self.failures = []
        self.errors = []
        self.skipped = []
        self.expectedFailures = []
        self.unexpectedSuccesses = []

    def wasSuccessful(self):
        return len(self.failures) == 0 and len(self.errors) == 0

class _AssertRaisesContext:
    def __init__(self, expected):
        self.expected = expected

    def __enter__(self):
        return self

    def __exit__(self, *args):
        if len(args) == 0 or args[0] is None:
            raise AssertionError('did not raise')
        error = args[0]
        if isinstance(error, self.expected):
            return True
        return False

def _mark_skip(func, active, reason):
    if active:
        func.__unittest_skip__ = True
        func.__unittest_skip_why__ = reason
    return func

class _SkipDecorator:
    def __init__(self, active, reason):
        self.active = active
        self.reason = reason

    def __call__(self, func):
        return _mark_skip(func, self.active, self.reason)

def skip(reason):
    return _SkipDecorator(True, reason)

def skipIf(condition, reason):
    return _SkipDecorator(condition, reason)

def skipUnless(condition, reason):
    return _SkipDecorator(not condition, reason)

def expectedFailure(func):
    func.__unittest_expected_failure__ = True
    return func

class TestCase:
    def __init__(self, methodName='runTest'):
        self._testMethodName = methodName

    def fail(self, msg=None):
        raise AssertionError(msg or 'test failed')

    def assertTrue(self, expr, msg=None):
        if not expr: self.fail(msg)

    def assertFalse(self, expr, msg=None):
        if expr: self.fail(msg)

    def assertEqual(self, first, second, msg=None):
        if first != second: self.fail(msg)

    def assertNotEqual(self, first, second, msg=None):
        if first == second: self.fail(msg)

    def assertIs(self, first, second, msg=None):
        if first is not second: self.fail(msg)

    def assertIsNot(self, first, second, msg=None):
        if first is second: self.fail(msg)

    def assertIsNone(self, value, msg=None):
        if value is not None: self.fail(msg)

    def assertIsNotNone(self, value, msg=None):
        if value is None: self.fail(msg)

    def assertIn(self, member, container, msg=None):
        if member not in container: self.fail(msg)

    def assertNotIn(self, member, container, msg=None):
        if member in container: self.fail(msg)

    def assertIsInstance(self, obj, cls, msg=None):
        if not isinstance(obj, cls): self.fail(msg)

    def assertNotIsInstance(self, obj, cls, msg=None):
        if isinstance(obj, cls): self.fail(msg)

    def assertGreater(self, first, second, msg=None):
        if not first > second: self.fail(msg)

    def assertLess(self, first, second, msg=None):
        if not first < second: self.fail(msg)

    def assertGreaterEqual(self, first, second, msg=None):
        if not first >= second: self.fail(msg)

    def assertLessEqual(self, first, second, msg=None):
        if not first <= second: self.fail(msg)

    def assertAlmostEqual(self, first, second, places=7, msg=None):
        if round(abs(first - second), places) != 0: self.fail(msg)

    def assertRaises(self, expected, *args, **kwargs):
        if len(args) == 0:
            return _AssertRaisesContext(expected)
        callable_obj = args[0]
        try:
            callable_obj(*args[1:], **kwargs)
        except expected:
            return None
        raise AssertionError('did not raise')

    def skipTest(self, reason):
        raise SkipTest(reason)

    def run(self, result=None):
        if result is None:
            result = TestResult()
        result.testsRun += 1
        method = getattr(self, self._testMethodName)
        original = getattr(type(self), self._testMethodName)
        if getattr(original, '__unittest_skip__', False):
            result.skipped.append(getattr(original, '__unittest_skip_why__', 'skipped'))
            return result
        expected = getattr(original, '__unittest_expected_failure__', False)
        if hasattr(self, 'setUp'):
            self.setUp()
        succeeded = False
        try:
            method()
            succeeded = True
        except SkipTest:
            result.skipped.append('skipped')
        except AssertionError:
            if expected:
                result.expectedFailures.append('expectedFailure')
            else:
                result.failures.append('failure')
        except Exception:
            result.errors.append('error')
        if succeeded and expected:
            result.unexpectedSuccesses.append('unexpectedSuccess')
        if hasattr(self, 'tearDown'):
            self.tearDown()
        return result

class TestSuite:
    def __init__(self, tests=None):
        self._tests = [] if tests is None else list(tests)

    def addTest(self, test):
        self._tests.append(test)

    def countTestCases(self):
        return len(self._tests)

    def run(self, result):
        for test in self._tests:
            test.run(result)
        return result

class TestLoader:
    def loadTestsFromTestCase(self, test_case_class):
        suite = TestSuite()
        for name in dir(test_case_class):
            if name.startswith('test'):
                suite.addTest(test_case_class(name))
        return suite
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"unittest",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, SOURCE) {
        // SAFETY: module initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded unittest module");
    }
}
