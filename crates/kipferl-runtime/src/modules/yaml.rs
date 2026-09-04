use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

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

const SOURCE: &str = r"
import json as _json

def loads(text):
    return _json.loads(_loads(text))

def safe_load(text):
    return loads(text)

def load(source):
    if hasattr(source, 'read'):
        return loads(source.read())
    stream = open(source, 'r')
    text = stream.read()
    stream.close()
    return loads(text)

def dumps(data):
    return _dumps(_json.dumps(data))

def safe_dump(data, stream=None):
    return dump(data, stream)

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
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"yaml",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded YAML module");
}

unsafe extern "C" fn loads(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(text) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"YAML input must be a string");
    };
    let value: serde_json::Value = match yaml_serde::from_str(&text) {
        Ok(value) => value,
        Err(error) => return value_error_message(&format!("invalid YAML: {error}")),
    };
    match serde_json::to_string(&value) {
        Ok(value) => return_string(&value),
        Err(error) => value_error_message(&format!("unsupported YAML value: {error}")),
    }
}

unsafe extern "C" fn dumps(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(data) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"YAML data must be JSON-compatible");
    };
    let value: serde_json::Value = match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(error) => return value_error_message(&format!("unsupported YAML data: {error}")),
    };
    match yaml_serde::to_string(&value) {
        Ok(value) => return_string(&value),
        Err(error) => value_error_message(&format!("could not encode YAML: {error}")),
    }
}
