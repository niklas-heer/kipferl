use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
class OrderedDict(dict):
    def __init__(self, items=None):
        super().__init__()
        self._ordered_keys = []
        if items is not None:
            for key, value in items:
                self[key] = value

    def __setitem__(self, key, value):
        if key not in self:
            self._ordered_keys.append(key)
        super().__setitem__(key, value)

    def __delitem__(self, key):
        super().__delitem__(key)
        self._ordered_keys.remove(key)

    def keys(self):
        return self._ordered_keys.copy()

    def values(self):
        values = []
        for key in self._ordered_keys:
            values.append(self[key])
        return values

    def items(self):
        values = []
        for key in self._ordered_keys:
            values.append((key, self[key]))
        return values

    def move_to_end(self, key, last=True):
        if key not in self:
            raise KeyError(key)
        self._ordered_keys.remove(key)
        if last:
            self._ordered_keys.append(key)
        else:
            self._ordered_keys.insert(0, key)

    def popitem(self, last=True):
        if len(self._ordered_keys) == 0:
            raise KeyError("dictionary is empty")
        index = -1 if last else 0
        key = self._ordered_keys[index]
        value = self[key]
        del self[key]
        return (key, value)


class _RustNamedTuple:
    def __init__(self, typename, fields, values):
        self._typename = typename
        self._fields = fields
        self._values = values
        index = 0
        for field in fields:
            setattr(self, field, values[index])
            index += 1

    def __getitem__(self, index):
        return self._values[index]

    def __iter__(self):
        return iter(self._values)

    def __len__(self):
        return len(self._values)

    def __repr__(self):
        parts = []
        index = 0
        for field in self._fields:
            parts.append(field + "=" + repr(self._values[index]))
            index += 1
        return self._typename + "(" + ", ".join(parts) + ")"

    def _asdict(self):
        result = {}
        index = 0
        for field in self._fields:
            result[field] = self._values[index]
            index += 1
        return result


class _RustNamedTupleFactory:
    def __init__(self, typename, fields):
        self.__name__ = typename
        self._typename = typename
        self._fields = fields

    def __call__(self, *values):
        if len(values) != len(self._fields):
            raise TypeError("wrong number of arguments")
        return _RustNamedTuple(self._typename, self._fields, values)


def namedtuple(typename, field_names):
    if isinstance(field_names, str):
        field_names = field_names.replace(",", " ").split()
    return _RustNamedTupleFactory(typename, tuple(field_names))


class _RustCounter(dict):
    def __init__(self, iterable=None, keyword_values=None):
        super().__init__()
        if iterable is not None:
            self.update(iterable)
        if keyword_values is not None:
            self.update(keyword_values)

    def __missing__(self, key):
        return 0

    def update(self, iterable=None, **kwargs):
        if iterable is not None:
            if isinstance(iterable, dict):
                for key in iterable:
                    self[key] = self[key] + iterable[key]
            else:
                for key in iterable:
                    self[key] = self[key] + 1
        for key in kwargs:
            self[key] = self[key] + kwargs[key]

    def most_common(self, count=None):
        pairs = []
        for key in self:
            pairs.append((key, self[key]))
        pairs.sort(key=lambda pair: pair[1], reverse=True)
        if count is None:
            return pairs
        return pairs[:count]

    def elements(self):
        values = []
        for key in self:
            for _ in range(max(0, self[key])):
                values.append(key)
        return values

    def subtract(self, iterable=None, **kwargs):
        if iterable is not None:
            if isinstance(iterable, dict):
                for key in iterable:
                    self[key] = self[key] - iterable[key]
            else:
                for key in iterable:
                    self[key] = self[key] - 1
        for key in kwargs:
            self[key] = self[key] - kwargs[key]


def Counter(iterable=None, **kwargs):
    return _RustCounter(iterable, kwargs)


class defaultdict(dict):
    def __init__(self, *args):
        if len(args) == 0:
            self.default_factory = None
            super().__init__()
        else:
            self.default_factory = args[0]
            super().__init__(*args[1:])

    def __missing__(self, key):
        if self.default_factory is None:
            raise KeyError(key)
        value = self.default_factory()
        self[key] = value
        return value

    def copy(self):
        return defaultdict(self.default_factory, self)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"collections",
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
        "embedded collections compatibility layer failed"
    );
}
