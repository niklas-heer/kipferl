use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    return_string, type_error,
};

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"_text(value)",
    callback: text,
}];

const SOURCE: &str = r#"
def _strip_comment(line):
    quote = None
    escaped = False
    result = ''
    for char in line:
        if escaped:
            result += char
            escaped = False
        elif char == '\\' and quote == '"':
            result += char
            escaped = True
        elif char == "'" or char == '"':
            if quote is None:
                quote = char
            elif quote == char:
                quote = None
            result += char
        elif char == '#' and quote is None:
            return result
        else:
            result += char
    return result

def _value(raw):
    raw = raw.strip()
    if len(raw) >= 2 and raw[0] in "'\"" and raw[-1] == raw[0]:
        return raw[1:-1]
    if raw == 'true':
        return True
    if raw == 'false':
        return False
    if raw.startswith('[') and raw.endswith(']'):
        inside = raw[1:-1].strip()
        if not inside:
            return []
        return [_value(item) for item in inside.split(',')]
    if '.' in raw:
        return float(raw)
    return int(raw)

def loads(data):
    data = _text(data)
    root = {}
    current = root
    for raw_line in data.split('\n'):
        line = _strip_comment(raw_line).strip()
        if not line:
            continue
        if line.startswith('[') and line.endswith(']'):
            current = root
            for part in line[1:-1].split('.'):
                key = part.strip()
                if key not in current:
                    current[key] = {}
                current = current[key]
            continue
        index = line.index('=')
        key = line[:index].strip()
        raw = line[index + 1:]
        target = current
        parts = key.split('.')
        for part in parts[:-1]:
            part = part.strip()
            if part not in target:
                target[part] = {}
            target = target[part]
        target[parts[-1].strip()] = _value(raw)
    return root

def load(file):
    if hasattr(file, 'read'):
        return loads(file.read())
    stream = open(file, 'rb')
    data = stream.read()
    stream.close()
    return loads(data)
"#;

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
    if !execute_module(module, SOURCE) {
        // SAFETY: module initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded tomllib module");
    }
}

unsafe extern "C" fn text(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(value) = arguments.get(0) else {
        return type_error(c"expected str or bytes");
    };
    if let Some(value) = value.string() {
        return return_string(&value);
    }
    if let Some(value) = value.bytes() {
        let Ok(value) = String::from_utf8(value) else {
            return type_error(c"TOML must be UTF-8");
        };
        return return_string(&value);
    }
    type_error(c"expected str or bytes")
}
