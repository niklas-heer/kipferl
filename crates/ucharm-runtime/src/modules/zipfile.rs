use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, Value, execute_module, return_bytes,
    runtime_error, type_error, value_error,
};

const MAX_ARCHIVE_SIZE: usize = 64 * 1024 * 1024;

const COMPATIBILITY_SOURCE: &str = r#"
from gzip import _inflate_raw


ZIP_STORED = 0
ZIP_DEFLATED = 8
BadZipFile = ValueError


def _u16(data, offset):
    return data[offset] | (data[offset + 1] << 8)


def _u32(data, offset):
    return _u16(data, offset) | (_u16(data, offset + 2) << 16)


def is_zipfile(filename):
    try:
        data = _read_file(filename)
        return len(data) >= 4 and data[:4] == b"PK\x03\x04"
    except Exception:
        return False


class ZipFile:
    def __init__(self, file, mode="r", compression=0):
        if mode != "r":
            raise ValueError("only read mode is supported")
        self._data = _read_file(file)
        self._entries = []
        self._closed = False
        offset = 0
        while offset + 46 <= len(self._data):
            if self._data[offset:offset + 4] == b"PK\x01\x02":
                method = _u16(self._data, offset + 10)
                compressed_size = _u32(self._data, offset + 20)
                size = _u32(self._data, offset + 24)
                name_length = _u16(self._data, offset + 28)
                extra_length = _u16(self._data, offset + 30)
                comment_length = _u16(self._data, offset + 32)
                local_offset = _u32(self._data, offset + 42)
                name_start = offset + 46
                name = self._data[name_start:name_start + name_length].decode()
                self._entries.append((name, method, compressed_size, size, local_offset))
                offset = name_start + name_length + extra_length + comment_length
            else:
                offset += 1
        if len(self._entries) == 0:
            raise BadZipFile("File is not a zip file")

    def namelist(self):
        names = []
        for entry in self._entries:
            names.append(entry[0])
        return names

    def read(self, name):
        for entry in self._entries:
            if entry[0] == name:
                local_offset = entry[4]
                if self._data[local_offset:local_offset + 4] != b"PK\x03\x04":
                    raise BadZipFile("Bad local file header")
                name_length = _u16(self._data, local_offset + 26)
                extra_length = _u16(self._data, local_offset + 28)
                start = local_offset + 30 + name_length + extra_length
                compressed = self._data[start:start + entry[2]]
                if entry[1] == ZIP_STORED:
                    return compressed
                if entry[1] == ZIP_DEFLATED:
                    return _inflate_raw(compressed)
                raise NotImplementedError("compression method is not supported")
        raise KeyError(name)

    def close(self):
        self._closed = True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
"#;

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_read_file",
    callback: read_file,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"zipfile",
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
        panic!("embedded zipfile compatibility layer failed");
    }
}

unsafe extern "C" fn read_file(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
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
