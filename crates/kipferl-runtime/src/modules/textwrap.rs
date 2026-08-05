use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::textwrap_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, NativeSignature, return_string,
    return_string_list, type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"dedent",
        callback: dedent,
    },
    NativeFunction {
        name: c"indent",
        callback: indent,
    },
    NativeFunction {
        name: c"shorten",
        callback: shorten,
    },
];

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"wrap(text, width=70)",
        callback: wrap,
    },
    NativeSignature {
        signature: c"fill(text, width=70)",
        callback: fill,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"textwrap",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn wrap(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some((text, width)) = text_and_optional_width(argc, argv) else {
        return false;
    };
    let lines = textwrap_core::wrap(&text, width);
    return_string_list(&lines)
}

unsafe extern "C" fn fill(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some((text, width)) = text_and_optional_width(argc, argv) else {
        return false;
    };
    return_string(&textwrap_core::fill(&text, width))
}

unsafe extern "C" fn dedent(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(text) = arguments.get(0).and_then(|value| value.string()) else {
        return type_error(c"text must be a string");
    };
    return_string(&textwrap_core::dedent(&text))
}

unsafe extern "C" fn indent(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(text) = arguments.get(0).and_then(|value| value.string()) else {
        return type_error(c"text must be a string");
    };
    let Some(prefix) = arguments.get(1).and_then(|value| value.string()) else {
        return type_error(c"prefix must be a string");
    };
    return_string(&textwrap_core::indent(&text, &prefix))
}

unsafe extern "C" fn shorten(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(text) = arguments.get(0).and_then(|value| value.string()) else {
        return type_error(c"text must be a string");
    };
    let Some(width) = arguments.get(1).and_then(|value| value.integer()) else {
        return type_error(c"width must be an int");
    };
    return_string(&textwrap_core::shorten(&text, width))
}

fn text_and_optional_width(argc: c_int, argv: ffi::py_StackRef) -> Option<(String, i64)> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return None;
    }
    let Some(text) = arguments.get(0).and_then(|value| value.string()) else {
        type_error(c"text must be a string");
        return None;
    };
    let width = arguments
        .get(1)
        .and_then(|value| value.integer())
        .unwrap_or(70);
    Some((text, width))
}
