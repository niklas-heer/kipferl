use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
class _Buffer:
    def _initialize(self, initial_value, binary):
        if binary:
            if not isinstance(initial_value, bytes):
                raise TypeError("a bytes-like object is required")
            self._empty = b""
            self._zero = b"\x00"
            self._newline = 10
        else:
            if not isinstance(initial_value, str):
                raise TypeError("initial_value must be str")
            self._empty = ""
            self._zero = "\x00"
            self._newline = "\n"
        self._binary = binary
        self._value = initial_value
        self._position = 0
        self.closed = False

    def _check_open(self):
        if self.closed:
            raise ValueError("I/O operation on closed file")

    def getvalue(self):
        self._check_open()
        return self._value

    def tell(self):
        self._check_open()
        return self._position

    def seek(self, offset, whence=0):
        self._check_open()
        if whence == 0:
            position = offset
        elif whence == 1:
            position = self._position + offset
        elif whence == 2:
            position = len(self._value) + offset
        else:
            raise ValueError("invalid whence")
        if position < 0:
            raise ValueError("negative seek position")
        self._position = position
        return position

    def read(self, size=-1):
        self._check_open()
        if size is None or size < 0:
            end = len(self._value)
        else:
            end = self._position + size
            if end > len(self._value):
                end = len(self._value)
        if self._position >= len(self._value):
            return self._empty
        output = self._value[self._position:end]
        self._position = end
        return output

    def write(self, value):
        self._check_open()
        if self._binary:
            if not isinstance(value, bytes):
                raise TypeError("a bytes-like object is required")
        elif not isinstance(value, str):
            raise TypeError("string argument expected")
        while self._position > len(self._value):
            self._value += self._zero
        end = self._position + len(value)
        suffix = self._empty
        if end < len(self._value):
            suffix = self._value[end:]
        self._value = self._value[:self._position] + value + suffix
        self._position = end
        return len(value)

    def readline(self, size=-1):
        self._check_open()
        if self._position >= len(self._value) or size == 0:
            return self._empty
        start = self._position
        end = start
        while end < len(self._value) and (size is None or size < 0 or end - start < size):
            current = self._value[end]
            end += 1
            if current == self._newline:
                break
        self._position = end
        return self._value[start:end]

    def readlines(self, hint=-1):
        self._check_open()
        output = []
        total = 0
        while True:
            line = self.readline()
            if line == self._empty:
                break
            output.append(line)
            total += len(line)
            if hint is not None and hint > 0 and total >= hint:
                break
        return output

    def writelines(self, lines):
        self._check_open()
        for line in lines:
            self.write(line)

    def truncate(self, size=None):
        self._check_open()
        if size is None:
            size = self._position
        if size < 0:
            raise ValueError("negative size value")
        if size < len(self._value):
            self._value = self._value[:size]
        return size

    def close(self):
        self.closed = True

    def flush(self):
        self._check_open()

    def readable(self):
        return not self.closed

    def writable(self):
        return not self.closed

    def seekable(self):
        return not self.closed

    def __enter__(self):
        self._check_open()
        return self

    def __exit__(self, *args):
        self.close()

    def __iter__(self):
        return self

    def __next__(self):
        line = self.readline()
        if line == self._empty:
            raise StopIteration
        return line


class BytesIO(_Buffer):
    def __init__(self, initial_bytes=None):
        if initial_bytes is None:
            initial_bytes = b""
        self._initialize(initial_bytes, True)


class StringIO(_Buffer):
    def __init__(self, initial_value="", newline=None):
        self._initialize(initial_value, False)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"io",
    kind: NativeModuleKind::ImportAndExtend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ucharm_pocketpy_sys::py_printexc() };
        panic!("embedded io compatibility layer failed");
    }
}
