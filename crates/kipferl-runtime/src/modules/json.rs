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


def _rust_json_key(key):
    if isinstance(key, str):
        return _rust_original_json_dumps(key)
    if key is True:
        return '"true"'
    if key is False:
        return '"false"'
    if key is None:
        return '"null"'
    if isinstance(key, (int, float)):
        return '"' + _rust_original_json_dumps(key) + '"'
    raise TypeError("keys must be str, int, float, bool or None, not " + type(key).__name__)


def _rust_json_encode(obj, indent, level, item_separator, key_separator, sort_keys):
    if isinstance(obj, list) or isinstance(obj, tuple):
        if not obj:
            return "[]"
        values = []
        for value in obj:
            values.append(_rust_json_encode(value, indent, level + 1, item_separator,
                                            key_separator, sort_keys))
        if indent is None:
            return "[" + item_separator.join(values) + "]"
        inner = "\n" + indent * (level + 1)
        return "[" + inner + (item_separator + inner).join(values) + "\n" + indent * level + "]"
    if isinstance(obj, dict):
        if not obj:
            return "{}"
        keys = []
        for key in obj:
            keys.append(key)
        if sort_keys:
            keys.sort()
        values = []
        for key in keys:
            encoded_value = _rust_json_encode(obj[key], indent, level + 1, item_separator,
                                              key_separator, sort_keys)
            values.append(_rust_json_key(key) + key_separator + encoded_value)
        if indent is None:
            return "{" + item_separator.join(values) + "}"
        inner = "\n" + indent * (level + 1)
        return "{" + inner + (item_separator + inner).join(values) + "\n" + indent * level + "}"
    return _rust_original_json_dumps(obj)


def _rust_json_plain_keys(obj):
    if isinstance(obj, list) or isinstance(obj, tuple):
        for value in obj:
            if not _rust_json_plain_keys(value):
                return False
        return True
    if isinstance(obj, dict):
        for key in obj:
            if not isinstance(key, str) or not _rust_json_plain_keys(obj[key]):
                return False
    return True


def dumps(obj, indent=None, separators=None, sort_keys=False):
    plain_indent = indent is None or (isinstance(indent, int) and not isinstance(indent, bool)
                                      and indent > 0)
    if separators is None and not sort_keys and plain_indent and _rust_json_plain_keys(obj):
        if indent is None:
            return _rust_original_json_dumps(obj)
        return _rust_original_json_dumps(obj, indent=indent)
    if indent is not None and not isinstance(indent, str):
        indent = " " * indent
    if separators is None:
        item_separator = ", " if indent is None else ","
        key_separator = ": "
    else:
        item_separator = separators[0]
        key_separator = separators[1]
    return _rust_json_encode(obj, indent, 0, item_separator, key_separator, sort_keys)


def load(source):
    if hasattr(source, 'read'):
        return loads(source.read())
    stream = open(source, 'r')
    text = stream.read()
    stream.close()
    return loads(text)


def dump(obj, stream=None, indent=None, separators=None, sort_keys=False):
    text = dumps(obj, indent=indent, separators=separators, sort_keys=sort_keys)
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
