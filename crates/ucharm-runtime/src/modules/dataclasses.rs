use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
_rust_original_dataclass = dataclass


def dataclass(cls):
    cls = _rust_original_dataclass(cls)
    fields = {}
    for name in _get_annotations(cls):
        fields[name] = name
    cls.__dataclass_fields__ = fields
    return cls


def is_dataclass(obj):
    if type(obj) is type:
        cls = obj
    else:
        cls = type(obj)
    return hasattr(cls, "__dataclass_fields__")
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"dataclasses",
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
        "embedded dataclasses compatibility layer failed"
    );
}
