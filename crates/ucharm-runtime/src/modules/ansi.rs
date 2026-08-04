use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use super::ansi_core;
use crate::native::{Arguments, NativeFunction, NativeModule, return_string, type_error};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"reset",
        callback: reset,
    },
    NativeFunction {
        name: c"fg",
        callback: foreground,
    },
    NativeFunction {
        name: c"bg",
        callback: background,
    },
    NativeFunction {
        name: c"rgb",
        callback: rgb,
    },
    NativeFunction {
        name: c"bold",
        callback: bold,
    },
    NativeFunction {
        name: c"dim",
        callback: dim,
    },
    NativeFunction {
        name: c"italic",
        callback: italic,
    },
    NativeFunction {
        name: c"underline",
        callback: underline,
    },
    NativeFunction {
        name: c"blink",
        callback: blink,
    },
    NativeFunction {
        name: c"reverse",
        callback: reverse,
    },
    NativeFunction {
        name: c"hidden",
        callback: hidden,
    },
    NativeFunction {
        name: c"strikethrough",
        callback: strikethrough,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"ansi",
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
};

unsafe extern "C" fn reset(argc: c_int, argv: ffi::py_StackRef) -> bool {
    no_argument_style(argc, argv, "\x1b[0m")
}

unsafe extern "C" fn foreground(argc: c_int, argv: ffi::py_StackRef) -> bool {
    color(argc, argv, false)
}

unsafe extern "C" fn background(argc: c_int, argv: ffi::py_StackRef) -> bool {
    color(argc, argv, true)
}

fn color(argc: c_int, argv: ffi::py_StackRef, background: bool) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let value = arguments.get(0).expect("arity checked");
    if let Some(index) = value.integer() {
        let code = if background {
            ansi_core::background(index)
        } else {
            ansi_core::foreground(index)
        };
        return return_string(&code);
    }
    if let Some(name) = value.string() {
        if name.len() >= 64 {
            return return_string("");
        }
        let code = if name.starts_with('#') {
            if background {
                ansi_core::hex_background(&name)
            } else {
                ansi_core::hex_foreground(&name)
            }
        } else if background {
            ansi_core::named_background(&name)
        } else {
            ansi_core::named_foreground(&name)
        };
        return return_string(&code);
    }
    type_error(c"color must be a string or int")
}

unsafe extern "C" fn rgb(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(3, 4) {
        return false;
    }
    let Some(red) = arguments.get(0).and_then(|value| value.integer()) else {
        return type_error(c"r must be int");
    };
    let Some(green) = arguments.get(1).and_then(|value| value.integer()) else {
        return type_error(c"g must be int");
    };
    let Some(blue) = arguments.get(2).and_then(|value| value.integer()) else {
        return type_error(c"b must be int");
    };
    let background = arguments.get(3).is_some_and(|value| value.truthy());
    return_string(&ansi_core::rgb(
        red as u8,
        green as u8,
        blue as u8,
        background,
    ))
}

fn no_argument_style(argc: c_int, argv: ffi::py_StackRef, code: &'static str) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    arguments.require_arity(0, 0) && return_string(code)
}

macro_rules! style_callback {
    ($name:ident, $code:literal) => {
        unsafe extern "C" fn $name(argc: c_int, argv: ffi::py_StackRef) -> bool {
            no_argument_style(argc, argv, $code)
        }
    };
}

style_callback!(bold, "\x1b[1m");
style_callback!(dim, "\x1b[2m");
style_callback!(italic, "\x1b[3m");
style_callback!(underline, "\x1b[4m");
style_callback!(blink, "\x1b[5m");
style_callback!(reverse, "\x1b[7m");
style_callback!(hidden, "\x1b[8m");
style_callback!(strikethrough, "\x1b[9m");
