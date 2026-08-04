use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, Value, execute_module, return_bytes,
    runtime_error, type_error, value_error,
};

const MAX_ARCHIVE_SIZE: usize = 64 * 1024 * 1024;

const COMPATIBILITY_SOURCE: &str = r#"
import io


ReadError = ValueError


def _text_field(data):
    end = 0
    while end < len(data) and data[end] != 0:
        end += 1
    return data[:end].decode()


def _octal_field(data):
    text = _text_field(data).strip()
    if text == "":
        return 0
    return int(text, 8)


def is_tarfile(name):
    try:
        data = _read_file(name)
        return len(data) >= 512 and data[257:262] == b"ustar"
    except Exception:
        return False


class TarFile:
    def __init__(self, name, mode="r"):
        if mode != "r":
            raise ValueError("only read mode is supported")
        self._data = _read_file(name)
        self._entries = []
        self.closed = False
        offset = 0
        while offset + 512 <= len(self._data):
            header = self._data[offset:offset + 512]
            if header[0] == 0:
                break
            name = _text_field(header[:100])
            size = _octal_field(header[124:136])
            typeflag = header[156]
            data_offset = offset + 512
            if typeflag == 0 or typeflag == 48:
                self._entries.append((name, data_offset, size))
            offset = data_offset + ((size + 511) // 512) * 512
        if len(self._entries) == 0 and not is_tarfile(name):
            raise ReadError("not a tar archive")

    def getnames(self):
        names = []
        for entry in self._entries:
            names.append(entry[0])
        return names

    def extractfile(self, member):
        for entry in self._entries:
            if entry[0] == member:
                return io.BytesIO(self._data[entry[1]:entry[1] + entry[2]])
        return None

    def close(self):
        self.closed = True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


def open(name, mode="r"):
    return TarFile(name, mode)
"#;

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_read_file",
    callback: read_file,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"tarfile",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ucharm_pocketpy_sys::py_printexc() };
        panic!("embedded tarfile compatibility layer failed");
    }
}

unsafe extern "C" fn read_file(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"filename must be a string");
    };
    match std::fs::read(path) {
        Ok(data) if data.len() <= MAX_ARCHIVE_SIZE => return_bytes(&data),
        Ok(_) => value_error(c"archive is too large"),
        Err(_) => runtime_error(c"unable to read archive"),
    }
}
