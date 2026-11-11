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

// PocketPy 2.2 registers `stdc` (malloc, free, memcpy, raw reads and writes by
// integer address) and `gc.untrack`, after which a collection frees objects the
// untracked container still references. Scripts can reach neither, nor
// `gc.setup_debug_callback`, whose raising callback aborts the process.
#[test]
fn raw_memory_and_gc_untracking_are_not_scriptable() {
    let output = run(r"
import sys
try:
    import stdc
    raise AssertionError('stdc was importable')
except ImportError as error:
    print(error)
assert 'stdc' not in sys.modules
import gc
assert not hasattr(gc, 'untrack') and not hasattr(gc, 'track')
assert not hasattr(gc, 'setup_debug_callback')
items = [[1, 2, 3] for _ in range(5)]
assert gc.is_tracked(items)
gc.collect()
junk = [[9, 9, 9, 9] for _ in range(1000)]
assert items[0] == [1, 2, 3] and items[4] == [1, 2, 3]
print('raw memory access stays unavailable')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        b"No module named 'stdc'\nraw memory access stays unavailable\n"
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

// `pkpy.TValue[int].__new__(list, address)` built a list whose object pointer
// was the given integer, so any list operation read or wrote that address. The
// concrete TValue types and stdc's memory types stayed reachable through
// `object.__subclasses__()`, and pickle.loads copied raw values from its input.
#[test]
fn forged_values_are_not_scriptable() {
    let output = run(r"
import pkpy, pickle
assert not hasattr(pkpy, 'TValue')
names = [c.__name__ for c in object.__subclasses__()]
assert not [n for n in names if n.startswith('TValue')], names
memory = [c for c in object.__subclasses__() if c.__name__ == 'Memory'][0]
for c in memory.__subclasses__():
    assert not hasattr(c, 'read') and not hasattr(c, 'write') and not hasattr(c, 'array'), c
    try:
        c()
        raise AssertionError('stdc value was created')
    except TypeError:
        pass
forged = b'PK99(list)\n0\n*' + bytes([99, 0, 1, 0, 0, 0, 0, 0, 16] + [0] * 15) + b'-'
for data in [forged, b'PK\n0\n\x1f\x15\x7f-', b'PK\n1\n\x00\x05-', b'PK\n0\n\x1d\x08ab',
             b'PK1(' + b'x' * 300 + b'.y)\n0\n-', b'PK\n0\n' + b'\x03' * 20000 + b'-']:
    try:
        pickle.loads(data)
        raise AssertionError('malformed pickle was accepted')
    except (ValueError, ImportError) as error:
        print(type(error).__name__)
class Point:
    def __init__(self, x):
        self.x = x
value = [1, 2.5, True, None, 'text', b'raw', (1, 2), {'k': [3]}, Point(4)]
copy = pickle.loads(pickle.dumps(value))
assert copy[:8] == value[:8] and copy[8].x == 4
print('value forging stays unavailable')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        b"ValueError\nValueError\nValueError\nValueError\nImportError\nValueError\n\
value forging stays unavailable\n"
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

// C methods and constructors trusted their first argument's type: `dict.clear([1])`
// read a list as a dict, `dict.__new__(list)` made a list with a dict's payload, and
// a class could borrow another type's C magic method. Each now raises TypeError.
#[test]
fn native_methods_reject_foreign_receivers() {
    let output = run(r"
import random, io, enum, array2d
class Borrowed(dict):
    __getitem__ = list.__getitem__
    __setitem__ = list.__setitem__
    __len__ = list.__len__
def borrowed_set():
    Borrowed()[0] = 1
calls = [lambda: dict.clear([1]), lambda: dict.__len__([1]), lambda: list.__len__({}),
         lambda: str.upper(5), lambda: list.sort({}), lambda: random.Random.choices([], [1]),
         lambda: bytearray.__len__([]), lambda: bytearray.__setitem__([], 0, 1),
         lambda: dict.__new__(list), lambda: BaseException.__new__(list),
         lambda: random.Random.__new__(list), lambda: enum.Enum.__new__(list),
         lambda: io.FileIO.__new__(list, 'x', 'r'), lambda: array2d.chunked_array2d.__new__(list, 2),
         lambda: dict.__new__(5), lambda: Borrowed()[0], borrowed_set, lambda: len(Borrowed()),
         lambda: sorted([[1], [2]], key=dict.__len__)]
for call in calls:
    try:
        call()
        raise AssertionError('foreign receiver was accepted')
    except TypeError:
        pass
class Child(dict):
    pass
child = dict.__new__(Child)
child['a'] = 1
assert dict.__len__(child) == 1 and isinstance(child, Child)
assert list.__len__([1, 2]) == 2 and int.__add__(2, 1) == 3
print(len(calls), 'foreign receivers rejected')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"19 foreign receivers rejected\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}

// PocketPy sizes strings and array2d storage with C ints: a repetition count or
// dimensions whose product overflows allocated too little and wrote past it.
#[test]
fn oversized_repetitions_and_grids_raise_instead_of_overflowing() {
    let output = run(r"
import array2d
for build in [lambda: 'a' * (2**32 + 1), lambda: 'ab' * (2**31 + 1), lambda: 'xyz' * 2**62,
              lambda: 'a' * (2**31 - 1),
              lambda: array2d.array2d(65536, 65537, lambda pos: 0),
              lambda: array2d.array2d(65536, 32768), lambda: array2d.array2d(2**32 + 1, 1),
              lambda: array2d.array2d(178956971, 1), lambda: array2d.array2d(1, 89478486)]:
    try:
        build()
        raise AssertionError('oversized result was accepted')
    except ValueError as error:
        print(error)
assert '' * 2**62 == '' and 'ab' * 3 == 'ababab' and 'a' * -1 == ''
assert array2d.array2d(3, 2, 7).numel == 6
print('sizes checked')
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        b"repeated string is too long\nrepeated string is too long\nrepeated string is too long\n\
repeated string is too long\narray2d() dimensions are too large\n\
array2d() dimensions are too large\narray2d() dimensions are too large\n\
array2d() dimensions are too large\narray2d() dimensions are too large\nsizes checked\n"
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

// Integers are 64-bit: arithmetic wraps (the C core is built with -fwrapv),
// and shifts follow Python for every count instead of C's undefined ones.
#[test]
fn integer_overflow_wraps_and_shifts_are_defined() {
    let output = run(r"
big = 2**62
print(big * 4, big + big, -big - big - 1, 2**63, -(-2**63), abs(-2**63))
print((-2**63) // -1, (-2**63) % -1, divmod(-2**63, -1))
print(1 << 63, 1 << 64, 3 << 100, -1 >> 100, 5 >> 64, -8 >> 1, 1 << 0)
for expression in ['1 << -1', '1 >> -1']:
    try:
        eval(expression)
    except ValueError as error:
        print(error)
");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "0 -9223372036854775808 9223372036854775807 -9223372036854775808 \
-9223372036854775808 -9223372036854775808\n\
-9223372036854775808 0 (-9223372036854775808, 0)\n\
-9223372036854775808 0 0 -1 0 -4 1\n\
negative shift count\nnegative shift count\n"
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}
