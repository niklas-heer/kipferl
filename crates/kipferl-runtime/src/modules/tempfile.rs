use std::ffi::c_int;
use std::fs::OpenOptions;

use kipferl_pocketpy_sys as ffi;

use super::filesystem_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, NativeSignature, os_error,
    return_string, type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"gettempdir",
        callback: gettempdir,
    },
    NativeFunction {
        name: c"mktemp",
        callback: mktemp,
    },
    NativeFunction {
        name: c"mkstemp",
        callback: mkstemp,
    },
];

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"mkdtemp(prefix=None)",
    callback: mkdtemp,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"tempfile",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn gettempdir(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    return_path(&filesystem_core::temporary_directory())
}

unsafe extern "C" fn mktemp(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let prefix = filesystem_core::temporary_directory().join("tmp");
    for _ in 0..1000 {
        let candidate = filesystem_core::unique_path(&prefix.to_string_lossy());
        if !candidate.exists() {
            return return_path(&candidate);
        }
    }
    os_error(c"mktemp failed")
}

unsafe extern "C" fn mkstemp(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let prefix = filesystem_core::temporary_directory().join("tmp");
    for _ in 0..1000 {
        let candidate = filesystem_core::unique_path(&prefix.to_string_lossy());
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                drop(file);
                return return_path(&candidate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return os_error(c"mkstemp failed"),
        }
    }
    os_error(c"mkstemp failed")
}

unsafe extern "C" fn mkdtemp(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let prefix = match arguments.get(0) {
        None => filesystem_core::temporary_directory().join("tmp"),
        Some(value) if value.is_none() => filesystem_core::temporary_directory().join("tmp"),
        Some(value) => {
            let Some(prefix) = value.string() else {
                return type_error(c"prefix must be a string");
            };
            std::path::PathBuf::from(prefix)
        }
    };
    for _ in 0..1000 {
        let candidate = filesystem_core::unique_path(&prefix.to_string_lossy());
        match std::fs::create_dir(&candidate) {
            Ok(()) => return return_path(&candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return os_error(c"mkdtemp failed"),
        }
    }
    os_error(c"mkdtemp failed")
}

fn return_path(path: &std::path::Path) -> bool {
    let Some(path) = filesystem_core::path_string(path) else {
        return os_error(c"temporary path is not valid UTF-8");
    };
    return_string(&path)
}
