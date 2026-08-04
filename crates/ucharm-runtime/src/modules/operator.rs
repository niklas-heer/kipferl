use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
def pos(value):
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return value * 1.0
    raise TypeError("bad operand type for unary +")


def abs(value):
    if value < 0:
        return -value
    return value


def index(value):
    if isinstance(value, int):
        return value
    raise TypeError("'index' requires an integer")


inv = invert


def is_none(value):
    return value is None


def is_not_none(value):
    return value is not None


def concat(left, right):
    if isinstance(left, list) and isinstance(right, list):
        return left + right
    if isinstance(left, str) and isinstance(right, str):
        return left + right
    raise TypeError("can only concatenate sequences")


def countOf(sequence, value):
    if not isinstance(sequence, list):
        raise TypeError("expected list")
    count = 0
    for item in sequence:
        if item == value:
            count += 1
    return count


def indexOf(sequence, value):
    if not isinstance(sequence, list):
        raise TypeError("expected list")
    index = 0
    for item in sequence:
        if item == value:
            return index
        index += 1
    raise ValueError("sequence.index(x): x not in sequence")


def ipow(left, right):
    return left ** right


def iconcat(left, right):
    if not isinstance(left, list) or not isinstance(right, list):
        raise TypeError("can only concatenate sequences")
    for item in tuple(right):
        left.append(item)
    return left


class itemgetter:
    def __init__(self, *items):
        if len(items) == 0:
            raise TypeError("itemgetter requires at least one argument")
        self._items = items

    def __call__(self, obj):
        if len(self._items) > 16:
            raise ValueError("too many keys")
        if len(self._items) == 1:
            return obj[self._items[0]]
        values = []
        for item in self._items:
            values.append(obj[item])
        return tuple(values)


def _rust_nested_attr(obj, path):
    if not isinstance(path, str):
        raise TypeError("attribute name must be string")
    for segment in path.split("."):
        if len(segment) != 0:
            if len(segment.encode()) >= 128:
                raise ValueError("attribute name too long")
            obj = getattr(obj, segment)
    return obj


class attrgetter:
    def __init__(self, *attrs):
        if len(attrs) == 0:
            raise TypeError("attrgetter requires at least one argument")
        self._attrs = attrs

    def __call__(self, obj):
        if len(self._attrs) > 16:
            raise ValueError("too many attributes")
        if len(self._attrs) == 1:
            return _rust_nested_attr(obj, self._attrs[0])
        values = []
        for attr in self._attrs:
            values.append(_rust_nested_attr(obj, attr))
        return tuple(values)


class methodcaller:
    def __init__(self, name, *args, **kwargs):
        self._name = name
        self._args = args
        self._kwargs = kwargs

    def __call__(self, obj):
        if not isinstance(self._name, str):
            raise TypeError("method name must be string")
        if len(self._args) > 16:
            raise ValueError("too many arguments")
        method = getattr(obj, self._name)
        return method(*self._args, **self._kwargs)


def length_hint(obj, default=0):
    if isinstance(obj, str):
        return len(obj.encode())
    if isinstance(obj, list) or isinstance(obj, tuple):
        return len(obj)
    if not isinstance(default, int):
        return 0
    return default


def call(function, *args, **kwargs):
    if len(args) > 16:
        raise ValueError("too many arguments")
    return function(*args, **kwargs)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"operator",
    kind: NativeModuleKind::ImportAndExtend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded operator compatibility layer failed"
    );
}
