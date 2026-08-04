use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
_ITEM_SIZES = {
    "b": 1, "B": 1,
    "h": 2, "H": 2,
    "i": 4, "I": 4,
    "l": 8, "L": 8,
    "q": 8, "Q": 8,
    "f": 4, "d": 8,
}


class array:
    def __init__(self, typecode, initializer=None):
        if typecode not in _ITEM_SIZES:
            raise ValueError("bad typecode")
        self.typecode = typecode
        self.itemsize = _ITEM_SIZES[typecode]
        self._values = []
        if initializer is not None:
            if isinstance(initializer, bytes) and typecode in ("b", "B"):
                for value in initializer:
                    self.append(value)
            else:
                self.extend(initializer)

    def __len__(self):
        return len(self._values)

    def __iter__(self):
        return iter(self._values)

    def __getitem__(self, key):
        if isinstance(key, slice):
            return array(self.typecode, self._values[key])
        return self._values[key]

    def __setitem__(self, key, value):
        self._values[key] = self._coerce(value)

    def __contains__(self, value):
        return value in self._values

    def __add__(self, other):
        self._require_same_type(other)
        return array(self.typecode, self._values + other._values)

    def __iadd__(self, other):
        self.extend(other)
        return self

    def __mul__(self, count):
        return array(self.typecode, self._values * count)

    def __rmul__(self, count):
        return self * count

    def __eq__(self, other):
        return isinstance(other, array) and self.typecode == other.typecode and self._values == other._values

    def __ne__(self, other):
        return not self == other

    def _require_same_type(self, other):
        if not isinstance(other, array) or self.typecode != other.typecode:
            raise TypeError("bad argument type for built-in operation")

    def _coerce(self, value):
        if self.typecode in ("f", "d"):
            if not isinstance(value, (int, float)):
                raise TypeError("must be real number")
            return float(value)
        if not isinstance(value, int):
            raise TypeError("an integer is required")
        return value

    def append(self, value):
        self._values.append(self._coerce(value))

    def extend(self, values):
        if isinstance(values, array):
            self._require_same_type(values)
        for value in values:
            self.append(value)

    def pop(self, index=-1):
        return self._values.pop(index)

    def insert(self, index, value):
        self._values.insert(index, self._coerce(value))

    def remove(self, value):
        self._values.remove(value)

    def index(self, value):
        return self._values.index(value)

    def count(self, value):
        return self._values.count(value)

    def reverse(self):
        self._values.reverse()

    def tolist(self):
        return self._values.copy()

    def tobytes(self):
        import struct
        if len(self._values) == 0:
            return b""
        return struct.pack("<" + str(len(self._values)) + self.typecode, *self._values)


typecodes = "bBuhHiIlLqQfd"
ArrayType = array
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"array",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: module initialization failed with an active PocketPy
        // exception; print it before aborting startup so embedded-source
        // regressions remain diagnosable.
        unsafe { ucharm_pocketpy_sys::py_printexc() };
        panic!("embedded array compatibility layer failed");
    }
}
