use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{Arguments, RootFrame, bind_type_method, return_value, type_error};

pub(super) fn register() {
    bind_type_method(
        ffi::py_PredefinedType_tp_str as ffi::py_Type,
        c"isupper",
        isupper,
    );
}

unsafe extern "C" fn isupper(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(value) = arguments.get(0).and_then(|value| value.string()) else {
        return type_error(c"expected string");
    };
    let mut has_cased = false;
    for byte in value.bytes() {
        if byte.is_ascii_lowercase() {
            let mut roots = RootFrame::new();
            let result = roots.boolean(false);
            return return_value(result);
        }
        if byte.is_ascii_uppercase() {
            has_cased = true;
        }
    }
    let mut roots = RootFrame::new();
    let result = roots.boolean(has_cased);
    return_value(result)
}
