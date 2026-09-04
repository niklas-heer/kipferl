use std::process::{Command, Output};

#[expect(
    clippy::expect_used,
    reason = "The regression harness must fail if the child runtime cannot be launched."
)]
fn run(source: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pocketpy-kipferl"))
        .args(["-c", source])
        .output()
        .expect("run runtime in an isolated process")
}

#[cfg(feature = "http")]
#[test]
fn invalid_http_timeouts_raise_python_errors_without_aborting_the_process() {
    // Finite values can overflow either Duration or the platform's Instant.
    // Validation happens before the request, so this never needs a server.
    let output = run(r"
from http.client import HTTPConnection
for timeout in [1e300, 1e19, float('inf'), float('nan'), -1.0]:
    try:
        HTTPConnection('127.0.0.1', 1, timeout).request('GET', '/')
        raise AssertionError('invalid timeout was accepted')
    except ValueError:
        pass
print('all timeout errors caught')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"all timeout errors caught\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn float_boundary_operations_match_cpython_and_preserve_python_errors() {
    let source = r"
import math
for value in [1e308, 5e-324, 2.2250738585072014e-308, 1.0, -1e308, -5e-324]:
    mantissa, exponent = math.frexp(value)
    assert 0.5 <= abs(mantissa) < 1.0
    assert math.ldexp(mantissa, exponent) == value
assert math.ldexp(0.5, 1024) == 8.98846567431158e307
assert math.ldexp(1e308, -1074) > 0.0
assert math.ldexp(1.0, -9223372036854775807) == 0.0
assert math.isnan(math.atanh(float('nan')))
for value in [-1.0, 1.0]:
    try:
        math.atanh(value)
        raise AssertionError('domain error was not raised')
    except ValueError:
        pass
try:
    math.ldexp(1.0, 1024)
    raise AssertionError('overflow was not raised')
except Exception:
    pass
print('float boundaries passed')
";
    let output = run(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"float boundaries passed\n");
    assert!(output.stderr.is_empty(), "{output:?}");
    let reference = Command::new("python3")
        .args(["-c", source])
        .output()
        .expect("run CPython differential oracle");
    assert!(reference.status.success(), "{reference:?}");
    assert_eq!(output.stdout, reference.stdout);
}

#[cfg(feature = "timezone")]
#[test]
fn fractional_timestamps_round_down_before_the_unix_epoch() {
    let source = "import time\nassert tuple(time.gmtime(-0.25)) == (1969, 12, 31, 23, 59, 59, 2, 365, 0)\nassert tuple(time.gmtime(-1.25)) == (1969, 12, 31, 23, 59, 58, 2, 365, 0)\n";
    let output = run(source);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn native_callbacks_handle_reentrant_collection_mutations() {
    let output = run(r"
import heapq, itertools, copy
heap = []
class MutatingComparison:
    def __lt__(self, other):
        heap.clear()
        return False
heap.append(MutatingComparison())
try:
    heapq.heappush(heap, MutatingComparison())
    raise AssertionError('heap mutation was accepted')
except RuntimeError:
    pass
class RaisingComparison:
    def __lt__(self, other):
        heap.clear()
        raise ValueError('original comparison error')
heap = [RaisingComparison(), RaisingComparison()]
try:
    heapq.nsmallest(1, heap)
    raise AssertionError('comparison exception was lost')
except ValueError:
    pass
items = [1, 2, 3]
def predicate(value):
    items.clear()
    return True
try:
    itertools.takewhile(predicate, items)
    raise AssertionError('predicate mutation was accepted')
except RuntimeError:
    pass
source_list = []
class ListHook:
    def __deepcopy__(self, memo):
        source_list.clear()
        return 'copied'
source_list.append(ListHook())
source_list.append('tail')
assert copy.deepcopy(source_list) == ['copied', 'tail']
source_dict = {}
class DictHook:
    def __deepcopy__(self, memo):
        source_dict.clear()
        return 'copied'
source_dict['first'] = DictHook()
source_dict['second'] = 'tail'
assert copy.deepcopy(source_dict) == {'first': 'copied', 'second': 'tail'}
print('mutating callbacks handled')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"mutating callbacks handled\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn extreme_native_sizes_and_indices_raise_catchable_errors() {
    let output = run(r"
import itertools, struct
try:
    struct.pack('f', 1e300)
    raise AssertionError('float overflow was accepted')
except ValueError:
    pass
try:
    bytearray(9223372036854775807)
    raise AssertionError('huge allocation was accepted')
except ValueError:
    pass
try:
    itertools.islice([1], -9223372036854775807, 1)
    raise AssertionError('negative slice was accepted')
except ValueError:
    pass
print('extreme inputs rejected')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"extreme inputs rejected\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[cfg(feature = "regex")]
#[test]
fn regex_replacements_preserve_utf8_after_escaped_characters() {
    let source = r"
import re
assert re.sub('x', '\\μtail', 'x') == '\\μtail'
assert re.sub('(x)', 'μ\\1é', 'x') == 'μxé'
assert re.sub('x', '\\n', 'x') == '\n'
print('unicode replacement passed')
";
    let output = run(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"unicode replacement passed\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}
