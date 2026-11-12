use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, RootFrame, Value, bind_type_method, bind_type_signature, return_string,
    return_string_list, return_value, type_error, value_error,
};

pub(super) fn register() {
    let methods: &[(&std::ffi::CStr, crate::native::Callback)] = &[
        (c"isdigit", isdigit),
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
    ];
    for &(name, callback) in methods {
        bind_type_method(
            crate::native::predefined_type(ffi::py_PredefinedType_tp_str),
            name,
            callback,
        );
    }
    bind_type_method(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_str),
        c"isupper",
        isupper,
    );
    bind_type_signature(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_str),
        c"rsplit(self, sep=None, maxsplit=-1)",
        rsplit,
    );
    bind_type_signature(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_str),
        c"split(self, sep=None, maxsplit=-1)",
        split,
    );
    bind_type_signature(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_str),
        c"replace(self, old, new, count=-1)",
        replace,
    );
}

fn return_bool(value: bool) -> bool {
    let mut roots = RootFrame::new();
    let result = roots.boolean(value);
    return_value(result)
}

fn string_argument(argc: c_int, stack: ffi::py_StackRef) -> Result<String, ()> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return Err(());
    }
    arguments.get(0).and_then(Value::string).ok_or_else(|| {
        let _ = type_error(c"expected string");
    })
}

unsafe extern "C" fn isdigit(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

unsafe extern "C" fn isalpha(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphabetic()))
}

unsafe extern "C" fn isalnum(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric()))
}

unsafe extern "C" fn isspace(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)),
    )
}

unsafe extern "C" fn islower(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    let has_cased = value.bytes().any(|byte| byte.is_ascii_lowercase());
    return_bool(has_cased && !value.bytes().any(|byte| byte.is_ascii_uppercase()))
}

unsafe extern "C" fn istitle(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
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

unsafe extern "C" fn isidentifier(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
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

unsafe extern "C" fn isprintable(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(value.bytes().all(|byte| (0x20..=0x7e).contains(&byte)))
}

unsafe extern "C" fn isascii(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok(value) = string_argument(argc, stack) else {
        return false;
    };
    return_bool(value.is_ascii())
}

unsafe extern "C" fn isupper(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(value) = arguments
        .get(0)
        .and_then(super::super::native::Value::string)
    else {
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

/// The receiver, the separator (`None` splits on whitespace runs) and
/// `maxsplit`, where a negative count means no limit.
fn split_arguments(
    argc: c_int,
    stack: ffi::py_StackRef,
) -> Result<(String, Option<String>, Option<usize>), ()> {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(value) = arguments.get(0).and_then(Value::string) else {
        let _ = type_error(c"expected string");
        return Err(());
    };
    let separator = match arguments.get(1) {
        Some(separator) if !separator.is_none() => {
            let Some(separator) = separator.string() else {
                let _ = type_error(c"must be str or None");
                return Err(());
            };
            if separator.is_empty() {
                let _ = value_error(c"empty separator");
                return Err(());
            }
            Some(separator)
        }
        _ => None,
    };
    let Some(maxsplit) = arguments.get(2).map_or(Some(-1), Value::integer) else {
        let _ = type_error(c"maxsplit must be an integer");
        return Err(());
    };
    Ok((value, separator, usize::try_from(maxsplit).ok()))
}

/// Python's whitespace split: runs of whitespace separate fields, and the
/// remainder after `limit` splits keeps its trailing whitespace.
fn split_whitespace(value: &str, limit: Option<usize>) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = value.trim_start();
    while !rest.is_empty() {
        if limit == Some(parts.len()) {
            parts.push(rest.to_owned());
            break;
        }
        let Some(end) = rest.find(char::is_whitespace) else {
            parts.push(rest.to_owned());
            break;
        };
        let (field, tail) = rest.split_at(end);
        parts.push(field.to_owned());
        rest = tail.trim_start();
    }
    parts
}

fn rsplit_whitespace(value: &str, limit: Option<usize>) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = value.trim_end();
    while !rest.is_empty() {
        if limit == Some(parts.len()) {
            parts.push(rest.to_owned());
            break;
        }
        let Some(start) = rest.rfind(char::is_whitespace) else {
            parts.push(rest.to_owned());
            break;
        };
        let (head, field) = rest.split_at(start);
        parts.push(field.trim_start().to_owned());
        rest = head.trim_end();
    }
    parts.reverse();
    parts
}

unsafe extern "C" fn split(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok((value, separator, limit)) = split_arguments(argc, stack) else {
        return false;
    };
    let values = match (separator, limit) {
        (None, limit) => split_whitespace(&value, limit),
        (Some(separator), None) => value.split(&separator).map(str::to_owned).collect(),
        (Some(separator), Some(limit)) => value
            .splitn(limit.saturating_add(1), &separator)
            .map(str::to_owned)
            .collect(),
    };
    return_string_list(&values)
}

unsafe extern "C" fn rsplit(argc: c_int, stack: ffi::py_StackRef) -> bool {
    let Ok((value, separator, limit)) = split_arguments(argc, stack) else {
        return false;
    };
    let values = match (separator, limit) {
        (None, limit) => rsplit_whitespace(&value, limit),
        (Some(separator), None) => value.split(&separator).map(str::to_owned).collect(),
        (Some(separator), Some(limit)) => {
            let mut values = value
                .rsplitn(limit.saturating_add(1), &separator)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            values.reverse();
            values
        }
    };
    return_string_list(&values)
}

unsafe extern "C" fn replace(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let (Some(value), Some(old), Some(new)) = (
        arguments.get(0).and_then(Value::string),
        arguments.get(1).and_then(Value::string),
        arguments.get(2).and_then(Value::string),
    ) else {
        return type_error(c"replace() arguments must be str");
    };
    let Some(count) = arguments.get(3).map_or(Some(-1), Value::integer) else {
        return type_error(c"count must be an integer");
    };
    // a negative count replaces every occurrence
    let result = usize::try_from(count).map_or_else(
        |_| value.replace(&old, &new),
        |count| value.replacen(&old, &new, count),
    );
    return_string(&result)
}
