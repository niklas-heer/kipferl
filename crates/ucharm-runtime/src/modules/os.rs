use std::ffi::c_int;
use std::os::unix::fs::MetadataExt;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value,
    execute_module, os_error, return_string_list, return_value, type_error,
};

const COMPATIBILITY_SOURCE: &str = r#"
sep = "/"
name = "posix"
linesep = "\n"


def getenv(key, default=None):
    if not isinstance(key, str):
        raise TypeError("key must be a string")
    if key in environ:
        return environ[key]
    return default
"#;

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"remove",
        callback: remove,
    },
    NativeFunction {
        name: c"unlink",
        callback: remove,
    },
    NativeFunction {
        name: c"rmdir",
        callback: rmdir,
    },
    NativeFunction {
        name: c"stat",
        callback: stat,
    },
];

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"listdir(path='.')",
        callback: listdir,
    },
    NativeSignature {
        signature: c"mkdir(path, mode=511)",
        callback: mkdir,
    },
    NativeSignature {
        signature: c"makedirs(name, mode=511, exist_ok=False)",
        callback: makedirs,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"os",
    kind: NativeModuleKind::ImportAndExtend,
    functions: FUNCTIONS,
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded os compatibility layer failed");
    }
    let mut roots = RootFrame::new();
    let environment = roots.dict();
    for (key, value) in std::env::vars() {
        let Some(key) = roots.string(&key) else {
            continue;
        };
        let Some(value) = roots.string(&value) else {
            continue;
        };
        if !environment.dict_set(key, value) {
            break;
        }
    }
    module.set_attribute(c"environ", environment);
}

unsafe extern "C" fn listdir(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"path must be a string");
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        return os_error(c"listdir failed");
    };
    let mut names = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            return os_error(c"listdir failed");
        };
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_owned());
        }
    }
    return_string_list(&names)
}

unsafe extern "C" fn mkdir(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"path must be a string");
    };
    match std::fs::create_dir(path) {
        Ok(()) => return_none(),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return_none(),
        Err(_) => os_error(c"mkdir failed"),
    }
}

unsafe extern "C" fn makedirs(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"name must be a string");
    };
    let exist_ok = arguments.get(2).and_then(Value::boolean).unwrap_or(false);
    if std::path::Path::new(&path).exists() && !exist_ok {
        return os_error(c"makedirs failed");
    }
    match std::fs::create_dir_all(&path) {
        Ok(()) => return_none(),
        Err(_) => os_error(c"makedirs failed"),
    }
}

unsafe extern "C" fn remove(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(path) = one_path(argc, argv) else {
        return false;
    };
    match std::fs::remove_file(path) {
        Ok(()) => return_none(),
        Err(_) => os_error(c"remove failed"),
    }
}

unsafe extern "C" fn rmdir(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(path) = one_path(argc, argv) else {
        return false;
    };
    match std::fs::remove_dir(path) {
        Ok(()) => return_none(),
        Err(_) => os_error(c"rmdir failed"),
    }
}

unsafe extern "C" fn stat(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(path) = one_path(argc, argv) else {
        return false;
    };
    let Ok(metadata) = std::fs::metadata(path) else {
        return os_error(c"stat failed");
    };
    let mut roots = RootFrame::new();
    let Some(result) = roots.tuple(7) else {
        return os_error(c"stat failed");
    };
    let values = [
        i64::from(metadata.mode()),
        0,
        0,
        0,
        0,
        0,
        i64::try_from(metadata.len()).unwrap_or(i64::MAX),
    ];
    for (index, value) in values.into_iter().enumerate() {
        let value = roots.integer(value);
        assert!(
            result.tuple_set(index, value),
            "new stat tuple index is valid"
        );
    }
    return_value(result)
}

fn one_path(argc: c_int, argv: ffi::py_StackRef) -> Option<String> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        type_error(c"path must be a string");
        return None;
    };
    Some(path)
}

fn return_none() -> bool {
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}
