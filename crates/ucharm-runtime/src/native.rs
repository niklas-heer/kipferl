use std::ffi::{CStr, c_int};
use std::ptr;
use std::slice;

use ucharm_pocketpy_sys as ffi;

pub(crate) type Callback = unsafe extern "C" fn(c_int, ffi::py_StackRef) -> bool;

pub(crate) struct NativeFunction {
    pub name: &'static CStr,
    pub callback: Callback,
}

pub(crate) struct NativeModule {
    pub name: &'static CStr,
    pub functions: &'static [NativeFunction],
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
    pub fn integer(self) -> Option<i64> {
        if !self.is_type(ffi::py_PredefinedType_tp_int) {
            return None;
        }
        // SAFETY: the exact type check above establishes an integer value.
        Some(unsafe { ffi::py_toint(self.raw) })
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

    fn is_type(self, value_type: ffi::py_PredefinedType) -> bool {
        // SAFETY: `self.raw` points to an initialized callback argument, and
        // predefined type identifiers fit PocketPy's `py_Type` representation.
        unsafe { ffi::py_istype(self.raw, value_type as ffi::py_Type) }
    }
}

pub(crate) fn return_string(value: &str) -> bool {
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
