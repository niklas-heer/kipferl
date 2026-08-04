use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use super::{encoding_core, random};
use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value, execute_module,
    return_bytes, return_string_bytes, return_value, runtime_error, type_error, value_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"token_bytes(nbytes=32)",
        callback: token_bytes,
    },
    NativeSignature {
        signature: c"token_hex(nbytes=32)",
        callback: token_hex,
    },
    NativeSignature {
        signature: c"token_urlsafe(nbytes=32)",
        callback: token_urlsafe,
    },
    NativeSignature {
        signature: c"randbelow(exclusive_upper_bound)",
        callback: randbelow,
    },
];

const COMPATIBILITY_SOURCE: &str = r#"
def choice(sequence):
    if len(sequence) == 0:
        raise IndexError("cannot choose from an empty sequence")
    return sequence[randbelow(len(sequence))]
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"secrets",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded secrets compatibility layer failed"
    );
}

unsafe extern "C" fn token_bytes(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(length) = byte_length(argc, argv) else {
        return false;
    };
    let Some(bytes) = random::secure_bytes(length) else {
        return runtime_error(c"OS random source failed");
    };
    return_bytes(&bytes)
}

unsafe extern "C" fn token_hex(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(length) = byte_length(argc, argv) else {
        return false;
    };
    let Some(bytes) = random::secure_bytes(length) else {
        return runtime_error(c"OS random source failed");
    };
    return_string_bytes(&encoding_core::hex_encode(&bytes))
}

unsafe extern "C" fn token_urlsafe(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(length) = byte_length(argc, argv) else {
        return false;
    };
    let Some(bytes) = random::secure_bytes(length) else {
        return runtime_error(c"OS random source failed");
    };
    let mut encoded = encoding_core::base64_encode(&bytes, true);
    while encoded.last() == Some(&b'=') {
        encoded.pop();
    }
    return_string_bytes(&encoded)
}

unsafe extern "C" fn randbelow(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from PocketPy with its active callback stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(bound) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"exclusive_upper_bound must be an integer");
    };
    if bound <= 0 {
        return value_error(c"Upper bound must be positive");
    }
    let bound = u64::try_from(bound).expect("positive bound fits u64");
    let range = 1_u64 << 63;
    let limit = range - range % bound;
    let word = loop {
        let Some(bytes) = random::secure_bytes(8) else {
            return runtime_error(c"OS random source failed");
        };
        let candidate =
            u64::from_le_bytes(bytes.try_into().expect("requested eight bytes")) & (range - 1);
        if candidate < limit {
            break candidate;
        }
    };
    let value = i64::try_from(word % bound).expect("result is below i64 bound");
    let mut roots = RootFrame::new();
    let value = roots.integer(value);
    return_value(value)
}

fn byte_length(argc: c_int, argv: ffi::py_StackRef) -> Option<usize> {
    // The signature binder supplies the default argument.
    // SAFETY: called only from PocketPy callbacks with active argument stacks.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let Some(length) = arguments.get(0).and_then(Value::integer) else {
        type_error(c"nbytes must be an integer");
        return None;
    };
    if !(0..=4096).contains(&length) {
        value_error(c"nbytes must be between 0 and 4096");
        return None;
    }
    usize::try_from(length).ok()
}
