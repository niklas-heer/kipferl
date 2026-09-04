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

unsafe extern "C" fn copy(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(object) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };
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

#[expect(
    clippy::expect_used,
    reason = "The exact list type was checked and the loop uses its length; shallow copying only appends to a separate list and invokes no Python hooks."
)]
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
    if !dict_apply(source, shallow_dict_item, (&raw mut context).cast()) {
        return false;
    }
    return_value(destination)
}

unsafe extern "C" fn deepcopy(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let Some(object) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };

    let mut roots = RootFrame::new();
    let originals = roots.list();
    let copies = roots.list();
    let hook_memo = roots.dict();
    let mut memo = DeepCopyMemo {
        originals,
        copies,
        hook_memo,
    };
    let Ok(result) = deepcopy_value(object, &mut memo, &mut roots) else {
        return false;
    };
    return_value(result)
}

struct DeepCopyMemo {
    originals: Value,
    copies: Value,
    hook_memo: Value,
}

impl DeepCopyMemo {
    #[expect(
        clippy::expect_used,
        reason = "The private memo owns parallel VM lists; identity lookup cannot run user code or mutate either list."
    )]
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

    fn remember(&self, original: Value, copied: Value) {
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

#[expect(
    clippy::expect_used,
    reason = "The exact tuple type was checked; Python tuples cannot change size while recursive copy hooks run."
)]
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
    let snapshot = roots.list();
    let length = source.list_len().ok_or_else(|| {
        type_error(c"deepcopy requires a list");
    })?;
    for index in 0..length {
        let item = source.list_item(index).ok_or_else(|| {
            type_error(c"list changed during deepcopy");
        })?;
        snapshot.list_append(item);
    }
    for index in 0..length {
        let item = snapshot.list_item(index).ok_or_else(|| {
            type_error(c"invalid deepcopy snapshot");
        })?;
        let mut item_roots = RootFrame::new();
        let item = item_roots.copy(item);
        let copied = deepcopy_value(item, memo, &mut item_roots)?;
        destination.list_append(copied);
    }
    Ok(destination)
}

unsafe extern "C" fn snapshot_dict_item(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: dict_apply invokes this synchronously with live entries and a
    // pointer to the rooted snapshot list. Appending cannot invoke Python.
    let snapshot = unsafe { &*context.cast::<Value>() };
    // SAFETY: the synchronous traversal keeps the key initialized.
    let key = unsafe { Value::from_raw(key) };
    // SAFETY: the synchronous traversal keeps the value initialized.
    let value = unsafe { Value::from_raw(value) };
    snapshot.list_append(key);
    snapshot.list_append(value);
    true
}

fn deepcopy_dict(
    source: Value,
    memo: &mut DeepCopyMemo,
    roots: &mut RootFrame,
) -> Result<Value, ()> {
    let destination = roots.dict();
    memo.remember(source, destination);
    let mut snapshot = roots.list();
    if !dict_apply(source, snapshot_dict_item, (&raw mut snapshot).cast()) {
        return Err(());
    }
    let length = snapshot.list_len().ok_or_else(|| {
        type_error(c"invalid deepcopy snapshot");
    })?;
    for index in (0..length).step_by(2) {
        let mut iteration = RootFrame::new();
        let key = snapshot.list_item(index).ok_or_else(|| {
            type_error(c"invalid deepcopy key");
        })?;
        let value = index
            .checked_add(1)
            .and_then(|index| snapshot.list_item(index))
            .ok_or_else(|| {
                type_error(c"invalid deepcopy value");
            })?;
        let key = iteration.copy(key);
        let value = iteration.copy(value);
        let key = deepcopy_value(key, memo, &mut iteration)?;
        let value = deepcopy_value(value, memo, &mut iteration)?;
        if !destination.dict_set(key, value) {
            return Err(());
        }
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
