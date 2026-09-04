use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::fnmatch_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, RootFrame, Value, return_string_bytes, return_value,
    type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"fnmatch",
        callback: fnmatch,
    },
    NativeFunction {
        name: c"fnmatchcase",
        callback: fnmatchcase,
    },
    NativeFunction {
        name: c"filter",
        callback: filter,
    },
    NativeFunction {
        name: c"translate",
        callback: translate,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"fnmatch",
    kind: crate::native::NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn fnmatch(argc: c_int, stack: ffi::py_StackRef) -> bool {
    match_names(argc, stack)
}

unsafe extern "C" fn fnmatchcase(argc: c_int, stack: ffi::py_StackRef) -> bool {
    match_names(argc, stack)
}

fn match_names(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(name) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"name must be a string");
    };
    let Some(pattern) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };

    let mut roots = RootFrame::new();
    let matched = roots.boolean(fnmatch_core::matches(&pattern, &name));
    return_value(matched)
}

unsafe extern "C" fn filter(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(names) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };
    if names.list_len().is_none() {
        return type_error(c"names must be a list");
    }
    let Some(pattern) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };

    let mut roots = RootFrame::new();
    let output = roots.list();
    for index in 0..names.list_len().unwrap_or(0) {
        let Some(item) = names.list_item(index) else {
            continue;
        };
        if item
            .string()
            .is_some_and(|name| fnmatch_core::matches(&pattern, &name))
        {
            output.list_append(item);
        }
    }
    return_value(output)
}

unsafe extern "C" fn translate(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    return_string_bytes(&fnmatch_core::translate(&pattern))
}
