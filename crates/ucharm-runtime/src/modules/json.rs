use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
JSONDecodeError = ValueError
_rust_original_json_loads = loads
_rust_original_json_dumps = dumps


def loads(source):
    compact = source.replace(" ", "").replace("\n", "").replace("\r", "").replace("\t", "")
    if ",]" in compact or ",}" in compact:
        raise JSONDecodeError("Invalid JSON")
    try:
        return _rust_original_json_loads(source)
    except Exception:
        raise JSONDecodeError("Invalid JSON")


def _rust_json_compact(obj, item_separator, key_separator, sort_keys):
    if isinstance(obj, list) or isinstance(obj, tuple):
        values = []
        for value in obj:
            values.append(_rust_json_compact(value, item_separator, key_separator, sort_keys))
        return "[" + item_separator.join(values) + "]"
    if isinstance(obj, dict):
        keys = []
        for key in obj:
            keys.append(key)
        if sort_keys:
            keys.sort()
        values = []
        for key in keys:
            encoded_key = _rust_original_json_dumps(key)
            encoded_value = _rust_json_compact(obj[key], item_separator, key_separator, sort_keys)
            values.append(encoded_key + key_separator + encoded_value)
        return "{" + item_separator.join(values) + "}"
    return _rust_original_json_dumps(obj)


def dumps(obj, indent=None, separators=None, sort_keys=False):
    if separators is None and not sort_keys:
        if indent is None:
            return _rust_original_json_dumps(obj)
        return _rust_original_json_dumps(obj, indent=indent)
    if separators is None:
        item_separator = ", "
        key_separator = ": "
    else:
        item_separator = separators[0]
        key_separator = separators[1]
    return _rust_json_compact(obj, item_separator, key_separator, sort_keys)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"json",
    kind: NativeModuleKind::ImportAndExtend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded json compatibility layer failed"
    );
}
