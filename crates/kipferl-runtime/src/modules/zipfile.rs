use std::ffi::c_int;
use std::fs::File;
use std::io::Read;

use kipferl_pocketpy_sys as ffi;
use zip::ZipArchive;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, execute_module,
    return_bytes, return_value, runtime_error, type_error, value_error,
};

const MAX_ARCHIVE_SIZE: u32 = 64 * 1024 * 1024;

const COMPATIBILITY_SOURCE: &str = r#"
ZIP_STORED = 0
ZIP_DEFLATED = 8
BadZipFile = ValueError


def is_zipfile(filename):
    try:
        _names(filename)
        return True
    except Exception:
        return False


class ZipFile:
    def __init__(self, file, mode="r", compression=0):
        if mode != "r":
            raise ValueError("only read mode is supported")
        self._file = file
        self._entries = _names(file)
        self._closed = False

    def namelist(self):
        return list(self._entries)

    def read(self, name):
        if name not in self._entries:
            raise KeyError(name)
        return _read_member(self._file, name)

    def close(self):
        self._closed = True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
"#;

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"_names",
        callback: names,
    },
    NativeFunction {
        name: c"_read_member",
        callback: read_member,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"zipfile",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { kipferl_pocketpy_sys::py_printexc() };
        panic!("embedded zipfile compatibility layer failed");
    }
}

fn open_archive(path: &str) -> Result<ZipArchive<File>, ()> {
    let file = File::open(path).map_err(|_| ())?;
    if file.metadata().map_err(|_| ())?.len() > u64::from(MAX_ARCHIVE_SIZE) {
        return Err(());
    }
    ZipArchive::new(file).map_err(|_| ())
}

unsafe extern "C" fn names(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"filename must be a string");
    };
    let Ok(archive) = open_archive(&path) else {
        return value_error(c"File is not a zip file");
    };
    let archive_names: Vec<_> = archive.file_names().map(str::to_owned).collect();
    let mut roots = RootFrame::new();
    let output = roots.list();
    for name in archive_names {
        let Some(name) = roots.string(&name) else {
            return value_error(c"zip member name is too large");
        };
        output.list_append(name);
    }
    return_value(output)
}

unsafe extern "C" fn read_member(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"filename must be a string");
    };
    let Some(name) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"member name must be a string");
    };
    let Ok(mut archive) = open_archive(&path) else {
        return value_error(c"File is not a zip file");
    };
    let Ok(member) = archive.by_name(&name) else {
        return runtime_error(c"unable to read zip member");
    };
    if member.size() > u64::from(MAX_ARCHIVE_SIZE) {
        return value_error(c"zip member is too large");
    }
    let mut bytes = Vec::new();
    if member
        .take(u64::from(MAX_ARCHIVE_SIZE.saturating_add(1)))
        .read_to_end(&mut bytes)
        .is_err()
    {
        return runtime_error(c"unable to read zip member");
    }
    if bytes.len() > usize::try_from(MAX_ARCHIVE_SIZE).unwrap_or(usize::MAX) {
        return value_error(c"zip member is too large");
    }
    return_bytes(&bytes)
}
