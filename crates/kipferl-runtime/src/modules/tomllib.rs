use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::toml_core;
use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    return_string, type_error, value_error_message,
};

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"_loads(data)",
    callback: loads,
}];

const SOURCE: &str = r"
import json as _json

def loads(data):
    return _json.loads(_loads(data))

def load(source):
    if hasattr(source, 'read'):
        return loads(source.read())
    stream = open(source, 'rb')
    data = stream.read()
    stream.close()
    return loads(data)
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"tomllib",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded tomllib module");
}

unsafe extern "C" fn loads(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
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
