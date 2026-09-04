use std::ffi::c_int;

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use kipferl_pocketpy_sys as ffi;
use serde_json::{Map, Number, Value as JsonValue};

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

def load(source):
    if hasattr(source, 'read'):
        return loads(source.read())
    stream = open(source, 'r')
    text = stream.read()
    stream.close()
    return loads(text)

def dumps(document):
    return _dumps(_json.dumps(document))

def dump(document, stream=None):
    text = dumps(document)
    if stream is None:
        return text
    if hasattr(stream, 'write'):
        stream.write(text)
        return None
    output = open(stream, 'w')
    output.write(text)
    output.close()
    return None

def argument(value, type=None):
    return {'name': None, 'type': type, 'value': value}

def property(name, value, type=None):
    return {'name': name, 'type': type, 'value': value}

def node(name, entries=None, children=None, type=None):
    return {
        'name': name,
        'type': type,
        'entries': [] if entries is None else entries,
        'children': [] if children is None else children,
    }
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"kdl",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded KDL module");
}

unsafe extern "C" fn loads(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(text) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"KDL input must be a string");
    };
    let document: KdlDocument = match text.parse() {
        Ok(document) => document,
        Err(error) => return value_error_message(&format!("invalid KDL: {error}")),
    };
    let value = match document_to_json(&document) {
        Ok(value) => value,
        Err(error) => return value_error_message(&error),
    };
    match serde_json::to_string(&value) {
        Ok(value) => return_string(&value),
        Err(error) => value_error_message(&format!("could not decode KDL: {error}")),
    }
}

unsafe extern "C" fn dumps(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(data) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"KDL document must be JSON-compatible");
    };
    let value: JsonValue = match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(error) => return value_error_message(&format!("unsupported KDL data: {error}")),
    };
    let mut document = match document_from_json(&value) {
        Ok(value) => value,
        Err(error) => return value_error_message(&error),
    };
    document.autoformat();
    return_string(&document.to_string())
}

fn document_to_json(document: &KdlDocument) -> Result<JsonValue, String> {
    document
        .nodes()
        .iter()
        .map(node_to_json)
        .collect::<Result<Vec<_>, _>>()
        .map(JsonValue::Array)
}

fn node_to_json(node: &KdlNode) -> Result<JsonValue, String> {
    let mut value = Map::new();
    value.insert(
        "name".into(),
        JsonValue::String(node.name().value().to_owned()),
    );
    value.insert(
        "type".into(),
        node.ty().map_or(JsonValue::Null, |value| {
            JsonValue::String(value.value().to_owned())
        }),
    );
    let entries = node
        .entries()
        .iter()
        .map(entry_to_json)
        .collect::<Result<Vec<_>, _>>()?;
    value.insert("entries".into(), JsonValue::Array(entries));
    value.insert(
        "children".into(),
        match node.children() {
            Some(children) => document_to_json(children)?,
            None => JsonValue::Array(Vec::new()),
        },
    );
    Ok(JsonValue::Object(value))
}

fn entry_to_json(entry: &KdlEntry) -> Result<JsonValue, String> {
    let mut value = Map::new();
    value.insert(
        "name".into(),
        entry.name().map_or(JsonValue::Null, |value| {
            JsonValue::String(value.value().to_owned())
        }),
    );
    value.insert(
        "type".into(),
        entry.ty().map_or(JsonValue::Null, |value| {
            JsonValue::String(value.value().to_owned())
        }),
    );
    value.insert("value".into(), kdl_value_to_json(entry.value())?);
    Ok(JsonValue::Object(value))
}

fn kdl_value_to_json(value: &KdlValue) -> Result<JsonValue, String> {
    match value {
        KdlValue::String(value) => Ok(JsonValue::String(value.clone())),
        KdlValue::Integer(value) => i64::try_from(*value)
            .map(Number::from)
            .map(JsonValue::Number)
            .map_err(|_| "KDL integer is outside Kipferl's signed 64-bit range".into()),
        KdlValue::Float(value) => Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| "KDL non-finite floats are not supported".into()),
        KdlValue::Bool(value) => Ok(JsonValue::Bool(*value)),
        KdlValue::Null => Ok(JsonValue::Null),
    }
}

fn document_from_json(value: &JsonValue) -> Result<KdlDocument, String> {
    let nodes = value
        .as_array()
        .ok_or_else(|| "KDL document must be a list of nodes".to_owned())?;
    let mut document = KdlDocument::new();
    for value in nodes {
        document.nodes_mut().push(node_from_json(value)?);
    }
    Ok(document)
}

fn node_from_json(value: &JsonValue) -> Result<KdlNode, String> {
    let value = value
        .as_object()
        .ok_or_else(|| "each KDL node must be a dictionary".to_owned())?;
    let name = required_string(value, "name", "KDL node")?;
    let mut node = KdlNode::new(name);
    if let Some(value_type) = optional_string(value, "type", "KDL node")? {
        node.set_ty(value_type);
    }
    if let Some(entries) = value.get("entries") {
        for entry in entries
            .as_array()
            .ok_or_else(|| "KDL node entries must be a list".to_owned())?
        {
            node.push(entry_from_json(entry)?);
        }
    }
    if let Some(children) = value.get("children") {
        let children = document_from_json(children)?;
        if !children.nodes().is_empty() {
            node.set_children(children);
        }
    }
    Ok(node)
}

fn entry_from_json(value: &JsonValue) -> Result<KdlEntry, String> {
    let value = value
        .as_object()
        .ok_or_else(|| "each KDL entry must be a dictionary".to_owned())?;
    let entry_value = value
        .get("value")
        .ok_or_else(|| "KDL entry is missing value".to_owned())?;
    let entry_value = kdl_value_from_json(entry_value)?;
    let mut entry = match optional_string(value, "name", "KDL entry")? {
        Some(name) => KdlEntry::new_prop(name, entry_value),
        None => KdlEntry::new(entry_value),
    };
    if let Some(value_type) = optional_string(value, "type", "KDL entry")? {
        entry.set_ty(value_type);
    }
    Ok(entry)
}

fn kdl_value_from_json(value: &JsonValue) -> Result<KdlValue, String> {
    match value {
        JsonValue::Null => Ok(KdlValue::Null),
        JsonValue::Bool(value) => Ok(KdlValue::Bool(*value)),
        JsonValue::String(value) => Ok(KdlValue::String(value.clone())),
        JsonValue::Number(value) => value
            .as_i64()
            .map(i128::from)
            .or_else(|| value.as_u64().map(i128::from))
            .map(KdlValue::Integer)
            .or_else(|| value.as_f64().map(KdlValue::Float))
            .ok_or_else(|| "invalid KDL number".to_owned()),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            Err("KDL entry values must be strings, numbers, booleans, or null".into())
        }
    }
}

fn required_string<'a>(
    value: &'a Map<String, JsonValue>,
    key: &str,
    context: &str,
) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .ok_or_else(|| format!("{context} requires a string {key}"))
}

fn optional_string<'a>(
    value: &'a Map<String, JsonValue>,
    key: &str,
    context: &str,
) -> Result<Option<&'a str>, String> {
    match value.get(key) {
        None | Some(JsonValue::Null) => Ok(None),
        Some(JsonValue::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!("{context} {key} must be a string or null")),
    }
}
