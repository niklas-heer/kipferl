use std::ffi::{c_int, c_void};

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, NativeTypeAlias, RootFrame, Value,
    call, call_type, clear_exception, dict_apply, optional_attribute, return_value, type_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"copy(x)",
        callback: copy,
    },
    NativeSignature {
        signature: c"deepcopy(x, memo=None)",
        callback: deepcopy,
    },
];

const TYPE_ALIASES: &[NativeTypeAlias] = &[NativeTypeAlias {
    name: c"Error",
    value_type: ffi::py_PredefinedType_tp_RuntimeError,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"copy",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: TYPE_ALIASES,
    initializer: None,
};

unsafe extern "C" fn copy(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let object = arguments.get(0).expect("arity checked");
    if is_atomic(object) || object.is_type(ffi::py_PredefinedType_tp_tuple) {
        return return_value(object);
    }

    if object.is_type(ffi::py_PredefinedType_tp_list) {
        return shallow_list(object);
    }
    if object.is_type(ffi::py_PredefinedType_tp_dict) {
        return shallow_dict(object);
    }

    if call_type(object.value_type(), &[object]) {
        return true;
    }
    clear_exception();

    let mut roots = RootFrame::new();
    let method = match optional_attribute(&mut roots, object, c"__copy__") {
        Ok(Some(method)) => method,
        Ok(None) => return type_error(c"object does not support copy"),
        Err(()) => return false,
    };
    if !call(method, &[]) {
        return false;
    }
    let copied = roots.copy_returned();
    return_value(copied)
}

fn shallow_list(source: Value) -> bool {
    let mut roots = RootFrame::new();
    let result = roots.list();
    let length = source.list_len().expect("list checked");
    for index in 0..length {
        result.list_append(source.list_item(index).expect("valid list index"));
    }
    return_value(result)
}

struct ShallowDictContext {
    destination: Value,
}

unsafe extern "C" fn shallow_dict_item(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: `dict_apply` supplies live entries and the synchronous context.
    let context = unsafe { &mut *context.cast::<ShallowDictContext>() };
    // SAFETY: both item refs remain valid because the source dictionary is not mutated.
    let key = unsafe { Value::from_raw(key) };
    // SAFETY: `dict_apply` keeps this value entry alive for the callback.
    let value = unsafe { Value::from_raw(value) };
    context.destination.dict_set(key, value)
}

fn shallow_dict(source: Value) -> bool {
    let mut roots = RootFrame::new();
    let destination = roots.dict();
    let mut context = ShallowDictContext { destination };
    if !dict_apply(
        source,
        shallow_dict_item,
        (&mut context as *mut ShallowDictContext).cast(),
    ) {
        return false;
    }
    return_value(destination)
}

unsafe extern "C" fn deepcopy(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let object = arguments.get(0).expect("arity checked");

    let mut roots = RootFrame::new();
    let originals = roots.list();
    let copies = roots.list();
    let hook_memo = roots.dict();
    let mut memo = DeepCopyMemo {
        originals,
        copies,
        hook_memo,
    };
    let result = match deepcopy_value(object, &mut memo, &mut roots) {
        Ok(result) => result,
        Err(()) => return false,
    };
    return_value(result)
}

struct DeepCopyMemo {
    originals: Value,
    copies: Value,
    hook_memo: Value,
}

impl DeepCopyMemo {
    fn find(&self, object: Value) -> Option<Value> {
        let length = self.originals.list_len().expect("memo list");
        for index in 0..length {
            let original = self.originals.list_item(index).expect("memo index");
            if original.is_identical(object) {
                return self.copies.list_item(index);
            }
        }
        None
    }

    fn remember(&mut self, original: Value, copied: Value) {
        self.originals.list_append(original);
        self.copies.list_append(copied);
    }
}

fn deepcopy_value(
    object: Value,
    memo: &mut DeepCopyMemo,
    roots: &mut RootFrame,
) -> Result<Value, ()> {
    if is_atomic(object) {
        return Ok(roots.copy(object));
    }
    if let Some(copied) = memo.find(object) {
        return Ok(roots.copy(copied));
    }
    if object.is_type(ffi::py_PredefinedType_tp_tuple) {
        return deepcopy_tuple(object, memo, roots);
    }
    if object.is_type(ffi::py_PredefinedType_tp_list) {
        return deepcopy_list(object, memo, roots);
    }
    if object.is_type(ffi::py_PredefinedType_tp_dict) {
        return deepcopy_dict(object, memo, roots);
    }

    if let Some(method) = optional_attribute(roots, object, c"__deepcopy__")? {
        if !call(method, &[memo.hook_memo]) {
            return Err(());
        }
        let result = roots.copy_returned();
        memo.remember(object, result);
        return Ok(result);
    }
    if let Some(method) = optional_attribute(roots, object, c"__copy__")? {
        if !call(method, &[]) {
            return Err(());
        }
        let result = roots.copy_returned();
        memo.remember(object, result);
        return Ok(result);
    }

    Ok(roots.copy(object))
}

fn deepcopy_tuple(
    source: Value,
    memo: &mut DeepCopyMemo,
    roots: &mut RootFrame,
) -> Result<Value, ()> {
    let length = source.tuple_len().expect("tuple checked");
    let destination = roots.tuple(length).ok_or_else(|| {
        type_error(c"tuple is too large to copy");
    })?;
    memo.remember(source, destination);
    for index in 0..length {
        let item = source.tuple_item(index).expect("valid tuple index");
        let mut item_roots = RootFrame::new();
        let copied = deepcopy_value(item, memo, &mut item_roots)?;
        if !destination.tuple_set(index, copied) {
            return Err(());
        }
    }
    Ok(destination)
}

fn deepcopy_list(
    source: Value,
    memo: &mut DeepCopyMemo,
    roots: &mut RootFrame,
) -> Result<Value, ()> {
    let destination = roots.list();
    memo.remember(source, destination);
    let length = source.list_len().expect("list checked");
    for index in 0..length {
        let item = source.list_item(index).expect("valid list index");
        let mut item_roots = RootFrame::new();
        let copied = deepcopy_value(item, memo, &mut item_roots)?;
        destination.list_append(copied);
    }
    Ok(destination)
}

struct DeepDictContext<'a> {
    destination: Value,
    memo: &'a mut DeepCopyMemo,
}

unsafe extern "C" fn deep_dict_item(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: `dict_apply` supplies live entries and the synchronous context.
    let context = unsafe { &mut *context.cast::<DeepDictContext<'_>>() };
    // SAFETY: source entries remain valid because only the destination is mutated.
    let key = unsafe { Value::from_raw(key) };
    // SAFETY: `dict_apply` keeps this value entry alive for the callback.
    let value = unsafe { Value::from_raw(value) };
    let mut roots = RootFrame::new();
    let Ok(key) = deepcopy_value(key, context.memo, &mut roots) else {
        return false;
    };
    let Ok(value) = deepcopy_value(value, context.memo, &mut roots) else {
        return false;
    };
    context.destination.dict_set(key, value)
}

fn deepcopy_dict(
    source: Value,
    memo: &mut DeepCopyMemo,
    roots: &mut RootFrame,
) -> Result<Value, ()> {
    let destination = roots.dict();
    memo.remember(source, destination);
    let mut context = DeepDictContext { destination, memo };
    if !dict_apply(
        source,
        deep_dict_item,
        (&mut context as *mut DeepDictContext<'_>).cast(),
    ) {
        return Err(());
    }
    Ok(destination)
}

fn is_atomic(value: Value) -> bool {
    value.is_none()
        || value.is_type(ffi::py_PredefinedType_tp_bool)
        || value.is_type(ffi::py_PredefinedType_tp_int)
        || value.is_type(ffi::py_PredefinedType_tp_float)
        || value.is_type(ffi::py_PredefinedType_tp_str)
}
