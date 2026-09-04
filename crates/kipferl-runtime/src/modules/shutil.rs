use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, os_error,
    return_string, return_value, type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"copy",
        callback: copy,
    },
    NativeFunction {
        name: c"move",
        callback: move_path,
    },
    NativeFunction {
        name: c"rmtree",
        callback: rmtree,
    },
    NativeFunction {
        name: c"exists",
        callback: exists,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"shutil",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn copy(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some((source, destination)) = two_paths(argc, stack) else {
        return false;
    };
    match std::fs::copy(&source, &destination) {
        Ok(_) => return_string(&destination),
        Err(_) => os_error(c"copy failed"),
    }
}

unsafe extern "C" fn move_path(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some((source, destination)) = two_paths(argc, stack) else {
        return false;
    };
    if std::fs::rename(&source, &destination).is_err()
        && (std::fs::copy(&source, &destination).is_err() || std::fs::remove_file(&source).is_err())
    {
        return os_error(c"move failed");
    }
    return_string(&destination)
}

unsafe extern "C" fn rmtree(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some(path) = one_path(argc, stack) else {
        return false;
    };
    match std::fs::remove_dir_all(path) {
        Ok(()) => {
            let mut roots = RootFrame::new();
            let none = roots.none();
            return_value(none)
        }
        Err(_) => os_error(c"rmtree failed"),
    }
}

unsafe extern "C" fn exists(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Some(path) = one_path(argc, stack) else {
        return false;
    };
    let mut roots = RootFrame::new();
    let value = roots.boolean(std::path::Path::new(&path).exists());
    return_value(value)
}

fn one_path(argc: c_int, stack: ffi::py_StackRef) -> Option<String> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        type_error(c"path must be a string");
        return None;
    };
    Some(path)
}

fn two_paths(argc: c_int, stack: ffi::py_StackRef) -> Option<(String, String)> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return None;
    }
    let Some(source) = arguments.get(0).and_then(Value::string) else {
        type_error(c"src must be a string");
        return None;
    };
    let Some(destination) = arguments.get(1).and_then(Value::string) else {
        type_error(c"dst must be a string");
        return None;
    };
    Some((source, destination))
}
