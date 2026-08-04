use std::ffi::{c_int, c_void};

use ucharm_pocketpy_sys as ffi;

use crate::args_core;
use crate::native::{
    Arguments, NativeFunction, NativeModule, RootFrame, Value, dict_apply, return_value,
    runtime_error, type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"raw",
        callback: raw,
    },
    NativeFunction {
        name: c"get",
        callback: get,
    },
    NativeFunction {
        name: c"count",
        callback: count,
    },
    NativeFunction {
        name: c"has",
        callback: has,
    },
    NativeFunction {
        name: c"value",
        callback: value,
    },
    NativeFunction {
        name: c"int_value",
        callback: int_value,
    },
    NativeFunction {
        name: c"positional",
        callback: positional,
    },
    NativeFunction {
        name: c"parse",
        callback: parse,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"args",
    functions: FUNCTIONS,
};

fn sys_argv() -> Option<Value> {
    // SAFETY: this helper is called only while a native callback has an active
    // VM. Both returned references are owned by process-global Python objects.
    unsafe {
        let sys = ffi::py_getmodule(c"sys".as_ptr());
        if sys.is_null() {
            return None;
        }
        let argv = ffi::py_getdict(sys, ffi::py_name(c"argv".as_ptr()));
        if argv.is_null() {
            return None;
        }
        Some(Value::from_raw(argv))
    }
}

unsafe extern "C" fn raw(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    return_value(argv)
}

unsafe extern "C" fn get(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let Some(mut index) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"index must be int");
    };
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    let length = argv.list_len().unwrap_or(0);
    if index < 0 {
        index += i64::try_from(length).unwrap_or(i64::MAX);
    }
    if let Ok(index) = usize::try_from(index)
        && let Some(item) = argv.list_item(index)
    {
        return return_value(item);
    }
    if let Some(default) = arguments.get(1) {
        return return_value(default);
    }
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

unsafe extern "C" fn count(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    let mut roots = RootFrame::new();
    let count = roots.integer(argv.list_len().unwrap_or(0) as i64);
    return_value(count)
}

unsafe extern "C" fn has(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(flag) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"flag must be a string");
    };
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    let found = (0..argv.list_len().unwrap_or(0)).any(|index| {
        argv.list_item(index)
            .and_then(Value::string)
            .is_some_and(|argument| argument == flag)
    });
    let mut roots = RootFrame::new();
    let found = roots.boolean(found);
    return_value(found)
}

enum FoundValue {
    Existing(Value),
    Inline(String),
}

impl FoundValue {
    fn string(&self) -> Option<String> {
        match self {
            Self::Existing(value) => value.string(),
            Self::Inline(value) => Some(value.clone()),
        }
    }
}

fn find_value(argv: Value, flag: &str) -> Option<FoundValue> {
    for index in 0..argv.list_len().unwrap_or(0) {
        let item = argv.list_item(index)?;
        let Some(argument) = item.string() else {
            continue;
        };
        if argument == flag {
            return argv.list_item(index + 1).map(FoundValue::Existing);
        }
        if let Some(value) = argument
            .strip_prefix(flag)
            .and_then(|remainder| remainder.strip_prefix('='))
        {
            return Some(FoundValue::Inline(value.to_owned()));
        }
    }
    None
}

unsafe extern "C" fn value(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let Some(flag) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"flag must be a string");
    };
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    match find_value(argv, &flag) {
        Some(FoundValue::Existing(value)) => return return_value(value),
        Some(FoundValue::Inline(value)) => {
            let mut roots = RootFrame::new();
            let Some(value) = roots.string(&value) else {
                return type_error(c"argument value is too large");
            };
            return return_value(value);
        }
        None => {}
    }
    if let Some(default) = arguments.get(1) {
        return return_value(default);
    }
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

unsafe extern "C" fn int_value(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let Some(flag) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"flag must be a string");
    };
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    if let Some(value) = find_value(argv, &flag).and_then(|value| value.string())
        && args_core::is_valid_integer(&value)
    {
        let mut roots = RootFrame::new();
        let value = roots.integer(args_core::parse_integer(&value));
        return return_value(value);
    }
    if let Some(default) = arguments.get(1) {
        return return_value(default);
    }
    let mut roots = RootFrame::new();
    let zero = roots.integer(0);
    return_value(zero)
}

fn is_flag(value: &str) -> bool {
    args_core::is_long_flag(value)
        || (args_core::is_short_flag(value) && !args_core::is_negative_number(value))
}

unsafe extern "C" fn positional(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };
    let mut roots = RootFrame::new();
    let output = roots.list();
    let length = argv.list_len().unwrap_or(0);
    let mut after_double_dash = false;
    let mut skip_next = false;

    for index in 1..length {
        if skip_next {
            skip_next = false;
            continue;
        }
        let Some(item) = argv.list_item(index) else {
            continue;
        };
        let Some(argument) = item.string() else {
            continue;
        };
        if after_double_dash {
            output.list_append(item);
            continue;
        }
        if args_core::is_double_dash(&argument) {
            after_double_dash = true;
            continue;
        }
        if args_core::is_long_flag(&argument) {
            if !argument.contains('=')
                && let Some(next) = argv.list_item(index + 1).and_then(Value::string)
                && !args_core::is_long_flag(&next)
                && !args_core::is_short_flag(&next)
            {
                skip_next = true;
            }
            continue;
        }
        if args_core::is_short_flag(&argument) && !args_core::is_negative_number(&argument) {
            if let Some(next) = argv.list_item(index + 1).and_then(Value::string)
                && !args_core::is_long_flag(&next)
                && !args_core::is_short_flag(&next)
            {
                skip_next = true;
            }
            continue;
        }
        output.list_append(item);
    }
    return_value(output)
}

struct AliasContext {
    aliases: Value,
}

unsafe extern "C" fn collect_alias(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: PocketPy supplies initialized dictionary item references and
    // `parse` supplies an `AliasContext` for the synchronous traversal.
    let (key, value, context) = unsafe {
        (
            Value::from_raw(key),
            Value::from_raw(value),
            &mut *context.cast::<AliasContext>(),
        )
    };
    if !value.is_type(ffi::py_PredefinedType_tp_str) {
        return true;
    }
    context.aliases.dict_set(key, value)
}

struct DefaultsContext {
    result: Value,
}

unsafe extern "C" fn collect_default(
    key: ffi::py_Ref,
    value: ffi::py_Ref,
    context: *mut c_void,
) -> bool {
    // SAFETY: PocketPy supplies initialized dictionary item references and
    // `parse` supplies a `DefaultsContext` for the synchronous traversal.
    let (key, value, context) = unsafe {
        (
            Value::from_raw(key),
            Value::from_raw(value),
            &mut *context.cast::<DefaultsContext>(),
        )
    };
    if value.is_type(ffi::py_PredefinedType_tp_str) {
        return true;
    }
    let Some(key) = key.string() else {
        return true;
    };
    if !args_core::is_long_flag(&key) && !args_core::is_short_flag(&key) {
        return true;
    }

    let mut roots = RootFrame::new();
    let Some(name) = roots.string(args_core::flag_name(&key)) else {
        return type_error(c"argument name is too large");
    };
    match roots.dict_get(context.result, name) {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(()) => return false,
    }

    if let Some(length) = value.tuple_len() {
        if length >= 2 {
            let Some(default) = value.tuple_item(1) else {
                return runtime_error(c"argument specification changed during parsing");
            };
            return context.result.dict_set(name, default);
        }
        if length == 1
            && value
                .tuple_item(0)
                .is_some_and(|item| item.is_type_object(ffi::py_PredefinedType_tp_bool))
        {
            let default = roots.boolean(false);
            return context.result.dict_set(name, default);
        }
        return true;
    }

    if value.is_type_object(ffi::py_PredefinedType_tp_bool) {
        let default = roots.boolean(false);
        return context.result.dict_set(name, default);
    }
    true
}

unsafe extern "C" fn parse(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(spec) = arguments.get(0) else {
        return type_error(c"expected 1 argument");
    };
    if !spec.is_type(ffi::py_PredefinedType_tp_dict) {
        return type_error(c"spec must be a dict");
    }
    let Some(argv) = sys_argv() else {
        return runtime_error(c"sys.argv not available");
    };

    let mut roots = RootFrame::new();
    let result = roots.dict();
    let positional = roots.list();
    let Some(positional_key) = roots.string("_") else {
        return type_error(c"argument name is too large");
    };
    if !result.dict_set(positional_key, positional) {
        return false;
    }
    let aliases = roots.dict();
    let mut alias_context = AliasContext { aliases };
    if !dict_apply(
        spec,
        collect_alias,
        (&mut alias_context as *mut AliasContext).cast(),
    ) {
        return false;
    }

    let length = argv.list_len().unwrap_or(0);
    let mut after_double_dash = false;
    let mut index = 1;
    while index < length {
        let Some(argument_value) = argv.list_item(index) else {
            index += 1;
            continue;
        };
        let Some(argument) = argument_value.string() else {
            index += 1;
            continue;
        };

        if after_double_dash {
            positional.list_append(argument_value);
            index += 1;
            continue;
        }
        if args_core::is_double_dash(&argument) {
            after_double_dash = true;
            index += 1;
            continue;
        }
        if !is_flag(&argument) {
            positional.list_append(argument_value);
            index += 1;
            continue;
        }

        let mut iteration = RootFrame::new();
        let (mut flag_key, inline_value) = match argument.find('=') {
            Some(position) if position < 127 => {
                let Some(key) = iteration.string(&argument[..position]) else {
                    return type_error(c"argument name is too large");
                };
                (key, Some(&argument[position + 1..]))
            }
            _ => (argument_value, None),
        };

        match iteration.dict_get(aliases, flag_key) {
            Ok(Some(alias)) => flag_key = alias,
            Ok(None) => {}
            Err(()) => return false,
        }

        let specification = match iteration.dict_get(spec, flag_key) {
            Ok(Some(specification)) => Some(specification),
            Ok(None) => None,
            Err(()) => return false,
        };
        let Some(mut specification) = specification else {
            let Some(flag) = flag_key.string() else {
                index += 1;
                continue;
            };
            let name = args_core::flag_name(&flag);
            if args_core::is_negated_flag(name) {
                let base = args_core::negated_base(name);
                let full_flag = format!("--{base}");
                if full_flag.len() < 128 {
                    let Some(full_flag) = iteration.string(&full_flag) else {
                        return type_error(c"argument name is too large");
                    };
                    match iteration.dict_get(spec, full_flag) {
                        Ok(Some(_)) => {
                            let Some(name) = iteration.string(base) else {
                                return type_error(c"argument name is too large");
                            };
                            let value = iteration.boolean(false);
                            if !result.dict_set(name, value) {
                                return false;
                            }
                            index += 1;
                            continue;
                        }
                        Ok(None) => {}
                        Err(()) => return false,
                    }
                }
            }
            index += 1;
            continue;
        };

        let Some(flag) = flag_key.string() else {
            index += 1;
            continue;
        };
        let Some(name) = iteration.string(args_core::flag_name(&flag)) else {
            return type_error(c"argument name is too large");
        };
        if specification.tuple_len().is_some_and(|length| length >= 1) {
            let Some(first) = specification.tuple_item(0) else {
                return runtime_error(c"argument specification changed during parsing");
            };
            specification = first;
        }

        if specification.is_type_object(ffi::py_PredefinedType_tp_bool) {
            let value = iteration.boolean(true);
            if !result.dict_set(name, value) {
                return false;
            }
        } else {
            let value = if let Some(inline_value) = inline_value {
                let Some(value) = iteration.string(inline_value) else {
                    return type_error(c"argument value is too large");
                };
                value
            } else if index + 1 < length {
                index += 1;
                let Some(value) = argv.list_item(index) else {
                    return runtime_error(c"sys.argv changed during parsing");
                };
                value
            } else {
                index += 1;
                continue;
            };

            if specification.is_type_object(ffi::py_PredefinedType_tp_int) {
                if let Some(value) = value.string()
                    && args_core::is_valid_integer(&value)
                {
                    let value = iteration.integer(args_core::parse_integer(&value));
                    if !result.dict_set(name, value) {
                        return false;
                    }
                }
            } else if !result.dict_set(name, value) {
                return false;
            }
        }
        index += 1;
    }

    let mut defaults_context = DefaultsContext { result };
    if !dict_apply(
        spec,
        collect_default,
        (&mut defaults_context as *mut DefaultsContext).cast(),
    ) {
        return false;
    }
    return_value(result)
}
