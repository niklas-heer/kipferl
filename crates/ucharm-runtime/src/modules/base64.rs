use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use super::encoding_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, Value, return_bytes, type_error,
    value_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"urlsafe_b64encode",
        callback: urlsafe_b64encode,
    },
    NativeFunction {
        name: c"urlsafe_b64decode",
        callback: urlsafe_b64decode,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"base64",
    kind: NativeModuleKind::Extend,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn urlsafe_b64encode(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(input) = arguments.get(0).and_then(Value::bytes) else {
        return type_error(c"expected bytes");
    };
    let output = encoding_core::base64_encode(&input, true);
    if output.len() > 4096 {
        return value_error(c"data too large");
    }
    return_bytes(&output)
}

unsafe extern "C" fn urlsafe_b64decode(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let value = arguments.get(0).expect("arity checked");
    let input = if let Some(bytes) = value.bytes() {
        bytes
    } else if let Some(string) = value.string() {
        string.into_bytes()
    } else {
        return type_error(c"expected bytes or string");
    };
    if input.len() > 4096 {
        return value_error(c"data too large");
    }
    let Ok(output) = encoding_core::base64_decode(&input, true) else {
        return value_error(c"invalid base64");
    };
    if output.len() > 4096 {
        return value_error(c"data too large");
    }
    return_bytes(&output)
}
