use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::toml_core;
use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    return_string, type_error, value_error_message,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"_loads(text)",
        callback: loads,
    },
    NativeSignature {
        signature: c"_dumps(data)",
        callback: dumps,
    },
];

const SOURCE: &str = r#"
import json as _json

def loads(text):
    return _json.loads(_loads(text))

def load(source):
    if hasattr(source, 'read'):
        return loads(source.read())
    stream = open(source, 'rb')
    text = stream.read()
    stream.close()
    return loads(text)

def dumps(data):
    return _dumps(_json.dumps(data))

def dump(data, stream=None):
    text = dumps(data)
    if stream is None:
        return text
    if hasattr(stream, 'write'):
        stream.write(text)
        return None
    output = open(stream, 'w')
    output.write(text)
    output.close()
    return None
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"toml",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded TOML module");
}

unsafe extern "C" fn loads(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(value) = arguments.get(0) else {
        return type_error(c"TOML input must be a string or bytes");
    };
    let text = if let Some(text) = value.string() {
        text
    } else if let Some(bytes) = value.bytes() {
        match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return type_error(c"TOML must be UTF-8"),
        }
    } else {
        return type_error(c"TOML input must be a string or bytes");
    };
    match toml_core::loads(&text) {
        Ok(value) => return_string(&value),
        Err(error) => value_error_message(&error),
    }
}

unsafe extern "C" fn dumps(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(data) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"TOML data must be JSON-compatible");
    };
    match toml_core::dumps(&data) {
        Ok(value) => return_string(&value),
        Err(error) => value_error_message(&error),
    }
}
