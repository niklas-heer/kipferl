use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{Arguments, Value, bind_type_signature, return_bytes, type_error, value_error};

const MAX_BYTES_SIZE: usize = 64 * 1024 * 1024;

pub(super) fn register() {
    let bytes_type = crate::native::predefined_type(ffi::py_PredefinedType_tp_bytes);
    bind_type_signature(bytes_type, c"__mul__(self, count)", multiply);
    bind_type_signature(bytes_type, c"__rmul__(self, count)", multiply);
}

unsafe extern "C" fn multiply(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(bytes) = arguments.get(0).and_then(Value::bytes) else {
        return type_error(c"expected bytes");
    };
    let Some(count) = arguments.get(1).and_then(Value::integer) else {
        return type_error(c"can't multiply sequence by non-int");
    };
    if count <= 0 || bytes.is_empty() {
        return return_bytes(&[]);
    }
    let Ok(count) = usize::try_from(count) else {
        return value_error(c"repeated bytes are too large");
    };
    let Some(length) = bytes.len().checked_mul(count) else {
        return value_error(c"repeated bytes are too large");
    };
    if length > MAX_BYTES_SIZE {
        return value_error(c"repeated bytes are too large");
    }
    return_bytes(&bytes.repeat(count))
}
