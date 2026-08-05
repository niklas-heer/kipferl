use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::encoding_core::{self, HexDecodeError};
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, NativeSignature, NativeTypeAlias,
    RootFrame, Value, return_bytes, return_value, type_error, value_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"hexlify",
        callback: hexlify,
    },
    NativeFunction {
        name: c"unhexlify",
        callback: unhexlify,
    },
    NativeFunction {
        name: c"b2a_hex",
        callback: hexlify,
    },
    NativeFunction {
        name: c"a2b_hex",
        callback: unhexlify,
    },
    NativeFunction {
        name: c"b2a_base64",
        callback: b2a_base64,
    },
    NativeFunction {
        name: c"a2b_base64",
        callback: a2b_base64,
    },
];

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"crc32(data, value=0)",
    callback: crc32,
}];

const TYPE_ALIASES: &[NativeTypeAlias] = &[
    NativeTypeAlias {
        name: c"Error",
        value_type: ffi::py_PredefinedType_tp_ValueError,
    },
    NativeTypeAlias {
        name: c"Incomplete",
        value_type: ffi::py_PredefinedType_tp_ValueError,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"binascii",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: TYPE_ALIASES,
    initializer: None,
};

unsafe extern "C" fn hexlify(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = one_bytes_argument(argc, argv) else {
        return false;
    };
    return_bytes(&encoding_core::hex_encode(&input))
}

unsafe extern "C" fn unhexlify(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = one_bytes_or_string_argument(argc, argv) else {
        return false;
    };
    match encoding_core::hex_decode(&input) {
        Ok(output) => return_bytes(&output),
        Err(HexDecodeError::OddLength) => value_error(c"Odd-length string"),
        Err(HexDecodeError::InvalidDigit) => value_error(c"Non-hexadecimal digit found"),
    }
}

unsafe extern "C" fn b2a_base64(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = one_bytes_argument(argc, argv) else {
        return false;
    };
    let mut output = encoding_core::base64_encode(&input, false);
    if output.len() + 1 > 8192 {
        return value_error(c"data too large");
    }
    output.push(b'\n');
    return_bytes(&output)
}

unsafe extern "C" fn a2b_base64(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(mut input) = one_bytes_or_string_argument(argc, argv) else {
        return false;
    };
    while input
        .last()
        .is_some_and(|byte| matches!(byte, b'\n' | b'\r' | b' ' | b'\t'))
    {
        input.pop();
    }
    let Ok(output) = encoding_core::base64_decode(&input, false) else {
        return value_error(c"Invalid base64-encoded string");
    };
    if output.len() > 8192 {
        return value_error(c"data too large");
    }
    return_bytes(&output)
}

unsafe extern "C" fn crc32(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // The declaration binder supplies the default second argument. The Zig
    // implementation intentionally ignores it, so only validate `data`.
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(input) = arguments.get(0).and_then(Value::bytes) else {
        return type_error(c"a bytes-like object is required");
    };
    let mut roots = RootFrame::new();
    let checksum = roots.integer(i64::from(encoding_core::crc32(&input)));
    return_value(checksum)
}

fn one_bytes_argument(argc: c_int, argv: ffi::py_StackRef) -> Option<Vec<u8>> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let Some(input) = arguments.get(0).and_then(Value::bytes) else {
        type_error(c"a bytes-like object is required");
        return None;
    };
    Some(input)
}

fn one_bytes_or_string_argument(argc: c_int, argv: ffi::py_StackRef) -> Option<Vec<u8>> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let value = arguments.get(0).expect("arity checked");
    if let Some(bytes) = value.bytes() {
        return Some(bytes);
    }
    if let Some(string) = value.string() {
        return Some(string.into_bytes());
    }
    type_error(c"a bytes-like object is required");
    None
}
