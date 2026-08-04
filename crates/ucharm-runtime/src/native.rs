use std::ffi::{CStr, c_int, c_void};
use std::ptr;
use std::slice;

use ucharm_pocketpy_sys as ffi;

pub(crate) type Callback = unsafe extern "C" fn(c_int, ffi::py_StackRef) -> bool;

pub(crate) struct NativeFunction {
    pub name: &'static CStr,
    pub callback: Callback,
}

pub(crate) struct NativeSignature {
    pub signature: &'static CStr,
    pub callback: Callback,
}

pub(crate) struct NativeIntConstant {
    pub name: &'static CStr,
    pub value: i64,
}

pub(crate) struct NativeModule {
    pub name: &'static CStr,
    pub functions: &'static [NativeFunction],
    pub signatures: &'static [NativeSignature],
    pub int_constants: &'static [NativeIntConstant],
}

pub(crate) fn register_modules(modules: &[NativeModule]) {
    for module in modules {
        // SAFETY: registration runs while the uniquely owned VM is active.
        // Module/function names have static storage and every callback uses
        // PocketPy's required C ABI.
        unsafe {
            let object = ffi::py_newmodule(module.name.as_ptr());
            for function in module.functions {
                ffi::py_bindfunc(object, function.name.as_ptr(), Some(function.callback));
            }
            for function in module.signatures {
                ffi::py_bind(object, function.signature.as_ptr(), Some(function.callback));
            }
            for constant in module.int_constants {
                let value = ffi::py_pushtmp();
                ffi::py_newint(value, constant.value);
                ffi::py_setdict(object, ffi::py_name(constant.name.as_ptr()), value);
                ffi::py_pop();
            }
        }
    }
}

pub(crate) struct Arguments {
    count: usize,
    values: ffi::py_StackRef,
}

impl Arguments {
    /// # Safety
    ///
    /// `values` must point to the active callback's PocketPy argument stack,
    /// which contains at least `count` initialized values.
    pub unsafe fn from_raw(count: c_int, values: ffi::py_StackRef) -> Self {
        Self {
            count: usize::try_from(count).unwrap_or(0),
            values,
        }
    }

    pub fn require_arity(&self, minimum: usize, maximum: usize) -> bool {
        if self.count < minimum {
            return type_error(c"too few arguments");
        }
        if self.count > maximum {
            return type_error(c"too many arguments");
        }
        true
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.count {
            return None;
        }
        // SAFETY: `from_raw` establishes a stack containing `count` values,
        // and the bounds check above keeps this pointer within that stack.
        Some(Value {
            raw: unsafe { self.values.add(index) },
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Value {
    raw: ffi::py_Ref,
}

impl Value {
    /// # Safety
    ///
    /// `raw` must point to an initialized PocketPy value that remains alive
    /// for every use of the returned wrapper.
    pub unsafe fn from_raw(raw: ffi::py_Ref) -> Self {
        Self { raw }
    }

    pub fn integer(self) -> Option<i64> {
        if !self.is_type(ffi::py_PredefinedType_tp_int) {
            return None;
        }
        // SAFETY: the exact type check above establishes an integer value.
        Some(unsafe { ffi::py_toint(self.raw) })
    }

    pub fn boolean(self) -> Option<bool> {
        if !self.is_type(ffi::py_PredefinedType_tp_bool) {
            return None;
        }
        // SAFETY: the exact type check above establishes a boolean value.
        Some(unsafe { ffi::py_tobool(self.raw) })
    }

    pub fn number(self) -> Option<f64> {
        if self.is_type(ffi::py_PredefinedType_tp_int) {
            // SAFETY: the exact type check above establishes an integer value.
            return Some(unsafe { ffi::py_toint(self.raw) } as f64);
        }
        if self.is_type(ffi::py_PredefinedType_tp_float) {
            // SAFETY: the exact type check above establishes a float value.
            return Some(unsafe { ffi::py_tofloat(self.raw) });
        }
        None
    }

    pub fn is_none(self) -> bool {
        self.is_type(ffi::py_PredefinedType_tp_NoneType)
    }

    pub fn string(self) -> Option<String> {
        if !self.is_type(ffi::py_PredefinedType_tp_str) {
            return None;
        }
        let mut length = 0;
        // SAFETY: the exact type check establishes a string. PocketPy owns the
        // returned UTF-8 buffer for at least the duration of this callback.
        let data = unsafe { ffi::py_tostrn(self.raw, &mut length) };
        let length = usize::try_from(length).ok()?;
        if length == 0 {
            return Some(String::new());
        }
        // SAFETY: PocketPy returned `length` initialized bytes. Python strings
        // are represented as valid UTF-8 by PocketPy.
        let bytes = unsafe { slice::from_raw_parts(data.cast::<u8>(), length) };
        String::from_utf8(bytes.to_vec()).ok()
    }

    pub fn truthy(self) -> bool {
        // SAFETY: every initialized PocketPy value supports truth conversion.
        unsafe { ffi::py_tobool(self.raw) }
    }

    pub fn is_type(self, value_type: ffi::py_PredefinedType) -> bool {
        // SAFETY: `self.raw` points to an initialized callback argument, and
        // predefined type identifiers fit PocketPy's `py_Type` representation.
        unsafe { ffi::py_istype(self.raw, value_type as ffi::py_Type) }
    }

    pub fn is_type_object(self, value_type: ffi::py_PredefinedType) -> bool {
        // SAFETY: `self.raw` is initialized and `py_tpobject` returns a
        // process-global type object for a predefined type.
        unsafe { ffi::py_isidentical(self.raw, ffi::py_tpobject(value_type as ffi::py_Type)) }
    }

    pub fn list_len(self) -> Option<usize> {
        if !self.is_type(ffi::py_PredefinedType_tp_list) {
            return None;
        }
        // SAFETY: the exact type check above establishes a list.
        usize::try_from(unsafe { ffi::py_list_len(self.raw) }).ok()
    }

    pub fn list_item(self, index: usize) -> Option<Self> {
        let length = self.list_len()?;
        if index >= length {
            return None;
        }
        let index = c_int::try_from(index).ok()?;
        // SAFETY: the exact type and bounds checks establish a valid item.
        Some(Self {
            raw: unsafe { ffi::py_list_getitem(self.raw, index) },
        })
    }

    pub fn list_append(self, value: Self) {
        debug_assert!(self.is_type(ffi::py_PredefinedType_tp_list));
        // SAFETY: `self` is a list and both values remain VM-rooted for the
        // duration of the operation.
        unsafe { ffi::py_list_append(self.raw, value.raw) };
    }

    pub fn tuple_len(self) -> Option<usize> {
        if !self.is_type(ffi::py_PredefinedType_tp_tuple) {
            return None;
        }
        // SAFETY: the exact type check above establishes a tuple.
        usize::try_from(unsafe { ffi::py_tuple_len(self.raw) }).ok()
    }

    pub fn tuple_item(self, index: usize) -> Option<Self> {
        let length = self.tuple_len()?;
        if index >= length {
            return None;
        }
        let index = c_int::try_from(index).ok()?;
        // SAFETY: the exact type and bounds checks establish a valid item.
        Some(Self {
            raw: unsafe { ffi::py_tuple_getitem(self.raw, index) },
        })
    }

    pub fn tuple_set(self, index: usize, value: Self) -> bool {
        let Some(length) = self.tuple_len() else {
            return false;
        };
        if index >= length {
            return false;
        }
        let Ok(index) = c_int::try_from(index) else {
            return false;
        };
        // SAFETY: the exact type and bounds checks establish a valid tuple
        // slot, and both values remain rooted during the assignment.
        unsafe { ffi::py_tuple_setitem(self.raw, index, value.raw) };
        true
    }

    pub fn dict_set(self, key: Self, value: Self) -> bool {
        debug_assert!(self.is_type(ffi::py_PredefinedType_tp_dict));
        // SAFETY: `self` is a dictionary and all values remain VM-rooted for
        // the duration of the operation.
        unsafe { ffi::py_dict_setitem(self.raw, key.raw, value.raw) }
    }
}

/// A LIFO frame for PocketPy temporary stack roots.
///
/// Values created by native callbacks must live on PocketPy's stack—not only
/// on Rust's stack—while later VM calls may allocate. This frame owns those
/// roots and releases them in one place when the callback finishes.
pub(crate) struct RootFrame {
    count: usize,
}

impl RootFrame {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    fn push(&mut self) -> Value {
        // SAFETY: native callbacks run with the VM active. Each successful
        // push is paired with one `py_pop` in this frame's `Drop`.
        let raw = unsafe { ffi::py_pushtmp() };
        self.count += 1;
        Value { raw }
    }

    pub fn copy_returned(&mut self) -> Value {
        // Copy the return register before any other PocketPy call can replace
        // it. The copied value is then installed in a VM-visible root.
        let returned = unsafe { *ffi::py_retval() };
        let root = self.push();
        // SAFETY: `root.raw` points to writable storage for one value.
        unsafe { ptr::write(root.raw, returned) };
        root
    }

    pub fn string(&mut self, value: &str) -> Option<Value> {
        let length = c_int::try_from(value.len()).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage and PocketPy reserves
        // exactly `length` bytes plus its trailing NUL.
        let destination = unsafe { ffi::py_newstrn(root.raw, length) };
        if !value.is_empty() {
            // SAFETY: both buffers are valid for `value.len()` bytes and do
            // not overlap.
            unsafe {
                ptr::copy_nonoverlapping(value.as_ptr(), destination.cast::<u8>(), value.len())
            };
        }
        Some(root)
    }

    pub fn integer(&mut self, value: i64) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newint(root.raw, value) };
        root
    }

    pub fn boolean(&mut self, value: bool) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newbool(root.raw, value) };
        root
    }

    pub fn none(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newnone(root.raw) };
        root
    }

    pub fn list(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newlist(root.raw) };
        root
    }

    pub fn tuple(&mut self, length: usize) -> Option<Value> {
        let length = c_int::try_from(length).ok()?;
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage and every tuple slot is
        // initialized by the caller before the tuple becomes observable.
        unsafe { ffi::py_newtuple(root.raw, length) };
        Some(root)
    }

    pub fn dict(&mut self) -> Value {
        let root = self.push();
        // SAFETY: `root` is writable VM stack storage.
        unsafe { ffi::py_newdict(root.raw) };
        root
    }

    pub fn dict_get(&mut self, dict: Value, key: Value) -> Result<Option<Value>, ()> {
        debug_assert!(dict.is_type(ffi::py_PredefinedType_tp_dict));
        // SAFETY: `dict` and `key` remain rooted throughout the operation.
        match unsafe { ffi::py_dict_getitem(dict.raw, key.raw) } {
            -1 => Err(()),
            0 => Ok(None),
            1 => Ok(Some(self.copy_returned())),
            _ => Err(()),
        }
    }
}

impl Drop for RootFrame {
    fn drop(&mut self) {
        for _ in 0..self.count {
            // SAFETY: `count` exactly tracks roots pushed by this frame and
            // frames are dropped in lexical LIFO order.
            unsafe { ffi::py_pop() };
        }
    }
}

pub(crate) type DictCallback = unsafe extern "C" fn(ffi::py_Ref, ffi::py_Ref, *mut c_void) -> bool;

pub(crate) fn dict_apply(dict: Value, callback: DictCallback, context: *mut c_void) -> bool {
    debug_assert!(dict.is_type(ffi::py_PredefinedType_tp_dict));
    // SAFETY: `dict` remains rooted for the call and `context` is supplied by
    // the caller with a lifetime covering the synchronous traversal.
    unsafe { ffi::py_dict_apply(dict.raw, Some(callback), context) }
}

pub(crate) fn return_value(value: Value) -> bool {
    // SAFETY: the value and return register are initialized storage for one
    // `py_TValue`; `ptr::copy` permits overlap.
    unsafe { ptr::copy(value.raw, ffi::py_retval(), 1) };
    true
}

pub(crate) fn return_string(value: &str) -> bool {
    return_string_bytes(value.as_bytes())
}

pub(crate) fn return_string_bytes(value: &[u8]) -> bool {
    let Ok(length) = c_int::try_from(value.len()) else {
        return type_error(c"return string is too large");
    };
    // SAFETY: the VM is active during a callback. `py_newstrn` reserves exactly
    // `length` writable bytes in the return register.
    let destination = unsafe { ffi::py_newstrn(ffi::py_retval(), length) };
    if !value.is_empty() {
        // SAFETY: both regions are valid for `value.len()` bytes and do not
        // overlap; PocketPy owns the destination after this copy.
        unsafe { ptr::copy_nonoverlapping(value.as_ptr(), destination.cast::<u8>(), value.len()) };
    }
    true
}

pub(crate) fn type_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_TypeError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

pub(crate) fn runtime_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_RuntimeError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}

pub(crate) fn value_error(message: &'static CStr) -> bool {
    // SAFETY: the VM is active during a callback. The format string and message
    // have static storage and match PocketPy's `%s` vararg contract.
    unsafe {
        ffi::py_exception(
            ffi::py_PredefinedType_tp_ValueError as ffi::py_Type,
            c"%s".as_ptr(),
            message.as_ptr(),
        )
    }
}
