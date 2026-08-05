use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, RootFrame, Value, bind_type_method, bind_type_signature, return_string_list,
    return_value, type_error, value_error,
};

pub(super) fn register() {
    for (name, callback) in [
        (c"isdigit", isdigit as crate::native::Callback),
        (c"isalpha", isalpha),
        (c"isalnum", isalnum),
        (c"isspace", isspace),
        (c"islower", islower),
        (c"istitle", istitle),
        (c"isdecimal", isdigit),
        (c"isnumeric", isdigit),
        (c"isidentifier", isidentifier),
        (c"isprintable", isprintable),
        (c"isascii", isascii),
    ] {
        bind_type_method(
            ffi::py_PredefinedType_tp_str as ffi::py_Type,
            name,
            callback,
        );
    }
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

fn return_bool(value: bool) -> bool {
    let mut roots = RootFrame::new();
    let result = roots.boolean(value);
    return_value(result)
}

fn string_argument(argc: c_int, argv: ffi::py_StackRef) -> Result<String, ()> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return Err(());
    }
    arguments.get(0).and_then(Value::string).ok_or_else(|| {
        let _ = type_error(c"expected string");
    })
}

unsafe extern "C" fn isdigit(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

unsafe extern "C" fn isalpha(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphabetic()))
}

unsafe extern "C" fn isalnum(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
}

unsafe extern "C" fn isspace(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)),
    )
}

unsafe extern "C" fn islower(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    let has_cased = value.bytes().any(|byte| byte.is_ascii_lowercase());
    return_bool(has_cased && !value.bytes().any(|byte| byte.is_ascii_uppercase()))
}

unsafe extern "C" fn istitle(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    let mut previous_cased = false;
    let mut has_cased = false;
    for byte in value.bytes() {
        if byte.is_ascii_uppercase() {
            if previous_cased {
                return return_bool(false);
            }
            previous_cased = true;
            has_cased = true;
        } else if byte.is_ascii_lowercase() {
            if !previous_cased {
                return return_bool(false);
            }
            previous_cased = true;
            has_cased = true;
        } else {
            previous_cased = false;
        }
    }
    return_bool(has_cased)
}

unsafe extern "C" fn isidentifier(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return return_bool(false);
    };
    return_bool(
        (first.is_ascii_alphabetic() || first == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
    )
}

unsafe extern "C" fn isprintable(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)))
}

unsafe extern "C" fn isascii(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, argv) else {
        return false;
    };
    return_bool(value.is_ascii())
}

unsafe extern "C" fn isupper(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
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
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
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
