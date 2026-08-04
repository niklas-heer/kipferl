use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, RootFrame, Value, bind_type_method, bind_type_signature, return_string_list,
    return_value, type_error, value_error,
};

pub(super) fn register() {
    bind_type_method(
        ffi::py_PredefinedType_tp_str as ffi::py_Type,
        c"isupper",
        isupper,
    );
    bind_type_signature(
        ffi::py_PredefinedType_tp_str as ffi::py_Type,
        c"rsplit(self, sep=None, maxsplit=-1)",
        rsplit,
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

unsafe extern "C" fn rsplit(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(value) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"expected string");
    };
    let Some(separator) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"separator must be a string");
    };
    if separator.is_empty() {
        return value_error(c"empty separator");
    }
    let maxsplit = arguments.get(2).and_then(Value::integer).unwrap_or(-1);
    let mut values = if maxsplit < 0 {
        value
            .split(&separator)
            .map(str::to_owned)
            .collect::<Vec<_>>()
    } else {
        let count = usize::try_from(maxsplit).unwrap_or(usize::MAX);
        let mut values = value
            .rsplitn(count.saturating_add(1), &separator)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        values.reverse();
        values
    };
    if values.is_empty() {
        values.push(value);
    }
    return_string_list(&values)
}
