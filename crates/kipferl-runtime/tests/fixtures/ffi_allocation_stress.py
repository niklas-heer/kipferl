import ansi, args, array, base64, binascii, tui, collections, copy, csv, dataclasses, datetime, errno, fnmatch, functools, heapq, input, io, itertools, json, operator, random, secrets, statistics, struct, term, textwrap, typing, uuid
spec = {
    '--name': str,
    '--count': (int, 0),
    '--verbose': bool,
    '-n': '--name',
}
retained = []
matched_retained = []
encoded_retained = []
statistic_retained = []
heap_retained = []
iterator_retained = []
operator_retained = []
cache_calls = 0
@functools.lru_cache(maxsize=32)
def cached_pair(value):
    global cache_calls
    cache_calls += 1
    return [value, value + 1]
@dataclasses.dataclass
class StressPoint:
    x: int
    y: int = 2
for i in range(4000):
    parsed = args.parse(spec)
    assert parsed['name'] == 'alice'
    assert parsed['count'] == 42
    assert parsed['verbose'] is True
    assert parsed['_'] == ['tail']
    retained.append(parsed)
    if len(retained) == 32:
        for value in retained:
            assert value['name'] == 'alice'
            assert value['count'] == 42
        retained = []
    matched = fnmatch.filter(['alpha', 'beta', 'alpine'], 'al*')
    assert matched == ['alpha', 'alpine']
    matched_retained.append(matched)
    if len(matched_retained) == 32:
        for value in matched_retained:
            assert value == ['alpha', 'alpine']
        matched_retained = []
    encoded = base64.urlsafe_b64encode(bytes([i % 256, (i + 1) % 256]))
    assert base64.urlsafe_b64decode(encoded) == bytes([i % 256, (i + 1) % 256])
    assert binascii.unhexlify(binascii.hexlify(encoded)) == encoded
    encoded_retained.append(encoded)
    if len(encoded_retained) == 32:
        for value in encoded_retained:
            assert len(value) == 4
        encoded_retained = []
    average = statistics.mean([i, i + 1, i + 2])
    assert average == i + 1
    assert statistics.median([i + 2, i, i + 1]) == i + 1
    assert statistics.mode([encoded, encoded, b'x']) is encoded
    statistic_retained.append(average)
    if len(statistic_retained) == 32:
        for value in statistic_retained:
            assert type(value) is float
        statistic_retained = []
    heap = [i + 2, i, i + 1]
    heapq.heapify(heap)
    heapq.heappush(heap, i - 1)
    assert heapq.heappop(heap) == i - 1
    ranked = heapq.nlargest(2, [i, i + 2, i + 1])
    assert ranked == [i + 2, i + 1]
    heap_retained.append(ranked)
    if len(heap_retained) == 32:
        for value in heap_retained:
            assert len(value) == 2
        heap_retained = []
    counter = itertools.count(i, 2)
    assert itertools.islice(counter, 1, 5, 2) == [i + 2, i + 6]
    assert next(counter) == i + 10
    repeated = itertools.repeat(encoded, 2)
    assert next(repeated) is encoded
    assert next(repeated) is encoded
    cycled = itertools.cycle([encoded, ranked])
    assert next(cycled) is encoded
    assert next(cycled) is ranked
    iterator_retained.append(itertools.cycle([encoded]))
    if len(iterator_retained) == 32:
        for value in iterator_retained:
            assert len(next(value)) == 4
        iterator_retained = []
    assert itertools.chain([i], (i + 1,), 'x') == [i, i + 1, 'x']
    assert itertools.takewhile(lambda x: x < 3, [1, 2, 4]) == [1, 2]
    assert itertools.dropwhile(lambda x: x < 3, [1, 2, 4]) == [4]
    assert errno.errorcode[errno.ENOENT] == 'ENOENT'
    assert 'EAGAIN'.isupper() is True
    os_error = OSError(i, 'ignored')
    assert os_error.args == (i,)
    nested = [[i], encoded]
    nested_copy = copy.deepcopy(nested)
    assert nested_copy == nested and nested_copy is not nested
    assert nested_copy[0] is not nested[0]
    cycle = []
    cycle.append(cycle)
    cycle_copy = copy.deepcopy(cycle)
    assert cycle_copy is not cycle and cycle_copy[0] is cycle_copy
    mutable = bytearray(encoded)
    mutable_copy = copy.copy(mutable)
    assert mutable_copy == mutable and mutable_copy is not mutable
    cached = cached_pair(i % 16)
    assert cached_pair(i % 16) is cached
    assert cached == [i % 16, i % 16 + 1]
    assert functools.reduce(lambda x, y: x + y, [i, 1, 2]) == i + 3
    counter_value = collections.Counter([i, i, i + 1])
    assert counter_value[i] == 2 and counter_value[i + 1] == 1
    csv_output = io.StringIO()
    csv.writer(csv_output).writerow([str(i), 'a,b'])
    assert list(csv.reader(csv_output.getvalue().split('\r\n')[:-1])) == [[str(i), 'a,b']]
    point_value = StressPoint(i)
    assert dataclasses.is_dataclass(point_value) and point_value.y == 2
    assert datetime.date(2024, 1, 15).weekday() == 0
    json_value = json.dumps({'b': i + 1, 'a': i}, separators=(',', ':'), sort_keys=True)
    assert json.loads(json_value) == {'a': i, 'b': i + 1}
    assert 0 <= random.getrandbits(8) < 256
    binary_values = array.array('I', [i, i + 1])
    binary_data = binary_values.tobytes()
    assert struct.unpack('<2I', binary_data) == (i, i + 1)
    binary_buffer = bytearray(12)
    struct.pack_into('>I', binary_buffer, 4, i)
    assert struct.unpack_from('>I', binary_buffer, 4) == (i,)
    assert len(secrets.token_bytes(8)) == 8
    assert 0 <= secrets.randbelow(17) < 17
    uuid_value = uuid.UUID('12345678-1234-4678-9234-567812345678')
    assert uuid_value.version == 4 and len(uuid_value.bytes) == 16
    picked = operator.itemgetter(0, 1)(([i], [i + 1]))
    assert picked == ([i], [i + 1])
    operator_retained.append(picked)
    if len(operator_retained) == 32:
        for value in operator_retained:
            assert len(value) == 2 and len(value[0]) == 1
        operator_retained = []
    hint = typing.TypeVar('T')
    assert repr(hint) == '~T'
    assert hint.__name__ == 'T'
    assert typing.cast(bytes, encoded) is encoded
    assert typing.final(ranked) is ranked
    assert typing.get_args(hint) == ()
    assert typing.get_origin(hint) is None
    wrapped = textwrap.wrap('alpha beta gamma', 10)
    assert wrapped == ['alpha beta']
    assert textwrap.fill('alpha beta gamma', 10) == 'alpha beta\ngamma'
    assert textwrap.dedent('  alpha\n  beta') == 'alpha\nbeta'
    assert textwrap.indent('alpha\nbeta', '> ') == '> alpha\n> beta'
    assert textwrap.shorten('alpha beta gamma', 10) == 'alpha...'
    styled = tui.style('界', fg='#abc', bold=True)
    assert styled == '\x1b[1;38;2;170;187;204m界\x1b[0m'
    assert tui.visible_len(styled) == 2
    expected = '\x1b[38;2;' + str(i % 256) + ';' + str((i + 1) % 256) + ';' + str((i + 2) % 256) + 'm'
    assert ansi.rgb(i, i + 1, i + 2) == expected
    assert input.select('', []) is None
    assert input.multiselect('', []) == []
    assert len(term.size()) == 2
    caught = False
    try:
        args.get('bad index')
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        typing.cast(int)
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        fnmatch.translate(1)
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        binascii.a2b_base64(b'bad')
    except ValueError:
        caught = True
    assert caught
    caught = False
    try:
        statistics.mean([True])
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        textwrap.shorten('alpha', 'bad')
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        heapq.heappop([])
    except IndexError:
        caught = True
    assert caught
    caught = False
    try:
        itertools.islice([1], 0, 1, 0)
    except ValueError:
        caught = True
    assert caught
    caught = False
    try:
        OSError(1, 2, 3)
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        copy.copy()
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        functools.reduce(lambda x, y: x + y, [])
    except TypeError:
        caught = True
    assert caught
    caught = False
    try:
        operator.indexOf([1], 2)
    except ValueError:
        caught = True
    assert caught
    caught = False
    try:
        json.loads('[1,]')
    except json.JSONDecodeError:
        caught = True
    assert caught
    caught = False
    try:
        struct.unpack('I', b'\x00')
    except struct.error:
        caught = True
    assert caught
assert len(retained) == 0
assert len(matched_retained) == 0
assert len(encoded_retained) == 0
assert len(statistic_retained) == 0
assert len(heap_retained) == 0
assert len(iterator_retained) == 0
assert len(operator_retained) == 0
assert cache_calls == 16
