use std::ffi::c_int;
use std::fmt::Write as _;
use std::io::{self, Write};

use ucharm_pocketpy_sys as ffi;

use super::charm_core::{self, BorderStyle};
use crate::native::{
    Arguments, NativeFunction, NativeIntConstant, NativeModule, NativeSignature, RootFrame, Value,
    return_string, return_value, runtime_error, type_error, value_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"visible_len",
        callback: visible_len,
    },
    NativeFunction {
        name: c"success",
        callback: success,
    },
    NativeFunction {
        name: c"error",
        callback: error,
    },
    NativeFunction {
        name: c"warning",
        callback: warning,
    },
    NativeFunction {
        name: c"info",
        callback: info,
    },
    NativeFunction {
        name: c"progress_done",
        callback: progress_done,
    },
    NativeFunction {
        name: c"spinner_frame",
        callback: spinner_frame,
    },
];

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"style(text, fg=None, bg=None, bold=False, dim=False, italic=False, underline=False, strikethrough=False)",
        callback: style,
    },
    // These declarations intentionally preserve the production Zig binding,
    // including its historical parameter-name mismatches with the stubs.
    NativeSignature {
        signature: c"box(content, title=None, border_color=None, padding=0, border_style=None)",
        callback: box_output,
    },
    NativeSignature {
        signature: c"rule(title=None, color=None, align=0, width=0)",
        callback: rule,
    },
    NativeSignature {
        signature: c"progress(current, total, label=None, width=40, color=None, elapsed=None)",
        callback: progress,
    },
    NativeSignature {
        signature: c"spinner(frame, message=None, color=None)",
        callback: spinner,
    },
    NativeSignature {
        signature: c"table(rows, headers=False, border='square', border_color=None)",
        callback: table,
    },
];

const INT_CONSTANTS: &[NativeIntConstant] = &[
    NativeIntConstant {
        name: c"BORDER_ROUNDED",
        value: charm_core::BORDER_ROUNDED,
    },
    NativeIntConstant {
        name: c"BORDER_SQUARE",
        value: charm_core::BORDER_SQUARE,
    },
    NativeIntConstant {
        name: c"BORDER_DOUBLE",
        value: charm_core::BORDER_DOUBLE,
    },
    NativeIntConstant {
        name: c"BORDER_HEAVY",
        value: charm_core::BORDER_HEAVY,
    },
    NativeIntConstant {
        name: c"BORDER_NONE",
        value: charm_core::BORDER_NONE,
    },
    NativeIntConstant {
        name: c"ALIGN_LEFT",
        value: 0,
    },
    NativeIntConstant {
        name: c"ALIGN_RIGHT",
        value: 1,
    },
    NativeIntConstant {
        name: c"ALIGN_CENTER",
        value: 2,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"charm",
    functions: FUNCTIONS,
    signatures: SIGNATURES,
    int_constants: INT_CONSTANTS,
};

fn write_output(bytes: &[u8]) {
    let mut output = io::stdout().lock();
    let _ = output.write_all(bytes);
    let _ = output.flush();
}

fn return_none() -> bool {
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

fn legacy_c_string(mut value: String) -> String {
    if let Some(nul) = value.find('\0') {
        value.truncate(nul);
    }
    value
}

fn optional_string(arguments: &Arguments, index: usize, c_string: bool) -> Option<String> {
    let value = arguments.get(index)?;
    if value.is_none() {
        return None;
    }
    let value = value.string()?;
    Some(if c_string {
        legacy_c_string(value)
    } else {
        value
    })
}

fn style_prefix(color: Option<&str>) -> (String, &'static str) {
    let start = charm_core::style_code(color, None, false, false, false, false, false);
    if start.is_empty() {
        (start, "")
    } else {
        (start, "\x1b[0m")
    }
}

unsafe extern "C" fn visible_len(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(text) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    if text.len() >= 4096 {
        return value_error(c"text too long");
    }
    let mut roots = RootFrame::new();
    let length = roots.integer(charm_core::visible_len(&text) as i64);
    return_value(length)
}

unsafe extern "C" fn style(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(text) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"text must be a string");
    };
    let foreground = optional_string(&arguments, 1, true);
    let background = optional_string(&arguments, 2, true);
    let prefix = charm_core::style_code(
        foreground.as_deref(),
        background.as_deref(),
        arguments.get(3).and_then(Value::boolean).unwrap_or(false),
        arguments.get(4).and_then(Value::boolean).unwrap_or(false),
        arguments.get(5).and_then(Value::boolean).unwrap_or(false),
        arguments.get(6).and_then(Value::boolean).unwrap_or(false),
        arguments.get(7).and_then(Value::boolean).unwrap_or(false),
    );
    if prefix.is_empty() {
        return return_string(&text);
    }
    return_string(&format!("{prefix}{text}\x1b[0m"))
}

unsafe extern "C" fn box_output(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(content) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"content must be a string");
    };
    let title = optional_string(&arguments, 1, true);
    let border = optional_string(&arguments, 2, true).unwrap_or_else(|| "rounded".to_owned());
    let border_color = optional_string(&arguments, 3, true);
    let padding = arguments.get(4).and_then(Value::integer).unwrap_or(1) as u32 as usize;
    if padding > 1_000_000 {
        return runtime_error(c"box is too large");
    }

    let chars = charm_core::box_chars(BorderStyle::for_box(&border));
    let maximum_width = content
        .split('\n')
        .map(charm_core::visible_len)
        .max()
        .unwrap_or(0);
    let title_length = title.as_ref().map_or(0, String::len);
    let title_width = if title.is_some() {
        title_length.saturating_add(4)
    } else {
        0
    };
    let content_width = maximum_width.max(title_width.saturating_sub(2));
    let Some(inner_width) = content_width.checked_add(padding.saturating_mul(2)) else {
        return runtime_error(c"box is too large");
    };
    if inner_width > 1_000_000 {
        return runtime_error(c"box is too large");
    }

    let (color_start, color_end) = style_prefix(border_color.as_deref());
    let mut output = String::new();
    if let Some(title) = title.as_deref() {
        output.push_str(&color_start);
        output.push_str(chars.tl);
        output.push_str(chars.h);
        output.push_str(color_end);
        output.push_str("\x1b[1m ");
        output.push_str(title);
        output.push_str(" \x1b[0m");
        output.push_str(&color_start);
        output.push_str(&charm_core::repeat(
            chars.h,
            inner_width.saturating_sub(title_length.saturating_add(3)),
        ));
        output.push_str(chars.tr);
        output.push_str(color_end);
        output.push('\n');
    } else {
        output.push_str(&color_start);
        output.push_str(chars.tl);
        output.push_str(&charm_core::repeat(chars.h, inner_width));
        output.push_str(chars.tr);
        output.push_str(color_end);
        output.push('\n');
    }

    let side_padding = " ".repeat(padding);
    for line in content.split('\n') {
        let line = legacy_c_string(line.to_owned());
        output.push_str(&color_start);
        output.push_str(chars.v);
        output.push_str(color_end);
        output.push_str(&side_padding);
        output.push_str(&charm_core::pad_left(&line, content_width));
        output.push_str(&side_padding);
        output.push_str(&color_start);
        output.push_str(chars.v);
        output.push_str(color_end);
        output.push('\n');
    }

    output.push_str(&color_start);
    output.push_str(chars.bl);
    output.push_str(&charm_core::repeat(chars.h, inner_width));
    output.push_str(chars.br);
    output.push_str(color_end);
    output.push('\n');
    write_output(output.as_bytes());
    return_none()
}

unsafe extern "C" fn rule(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let title = optional_string(&arguments, 0, true);
    let character = optional_string(&arguments, 1, true).unwrap_or_else(|| "─".to_owned());
    let color = optional_string(&arguments, 2, true);
    let width = arguments
        .get(3)
        .and_then(Value::integer)
        .unwrap_or(80)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    let (color_start, color_end) = style_prefix(color.as_deref());
    let mut output = String::new();
    if let Some(title) = title.as_deref() {
        let title_length = i32::try_from(title.len()).unwrap_or(i32::MAX);
        let side = ((width.saturating_sub(title_length).saturating_sub(2)) / 2).max(0);
        output.push_str(&color_start);
        output.push_str(&charm_core::repeat(&character, side as usize));
        output.push_str(color_end);
        output.push(' ');
        output.push_str(title);
        output.push(' ');
        let remaining = width
            .saturating_sub(side)
            .saturating_sub(title_length)
            .saturating_sub(2)
            .max(0);
        output.push_str(&color_start);
        output.push_str(&charm_core::repeat(&character, remaining as usize));
        output.push_str(color_end);
        output.push('\n');
    } else {
        output.push_str(&color_start);
        output.push_str(&charm_core::repeat(&character, width.max(0) as usize));
        output.push_str(color_end);
        output.push('\n');
    }
    write_output(output.as_bytes());
    return_none()
}

fn status_message(
    argc: c_int,
    argv: ffi::py_StackRef,
    color: &'static str,
    symbol: &'static str,
) -> bool {
    // SAFETY: called only from PocketPy callbacks with an active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(message) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"message must be a string");
    };
    write_output(format!("\x1b[1;{color}m{symbol} \x1b[0m{message}\n").as_bytes());
    return_none()
}

unsafe extern "C" fn success(argc: c_int, argv: ffi::py_StackRef) -> bool {
    status_message(argc, argv, "32", charm_core::SYMBOL_SUCCESS)
}

unsafe extern "C" fn error(argc: c_int, argv: ffi::py_StackRef) -> bool {
    status_message(argc, argv, "31", charm_core::SYMBOL_ERROR)
}

unsafe extern "C" fn warning(argc: c_int, argv: ffi::py_StackRef) -> bool {
    status_message(argc, argv, "33", charm_core::SYMBOL_WARNING)
}

unsafe extern "C" fn info(argc: c_int, argv: ffi::py_StackRef) -> bool {
    status_message(argc, argv, "34", charm_core::SYMBOL_INFO)
}

unsafe extern "C" fn progress(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(current) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"current must be int");
    };
    let Some(total) = arguments.get(1).and_then(Value::integer) else {
        return type_error(c"total must be int");
    };
    let label = optional_string(&arguments, 2, true);
    let width = arguments.get(3).and_then(Value::integer).unwrap_or(40) as u32;
    if width > 1_000_000 {
        return runtime_error(c"progress bar is too large");
    }
    let color = optional_string(&arguments, 4, true);
    let elapsed = arguments.get(5).and_then(|value| {
        if value.is_none() {
            None
        } else {
            value.number()
        }
    });
    let (color_start, color_end) = style_prefix(color.as_deref());
    let mut output = String::from("\r");
    if let Some(label) = label {
        output.push_str(&label);
        output.push(' ');
    }
    output.push_str(&color_start);
    output.push_str(&charm_core::progress_bar(
        current as u32,
        total as u32,
        width,
    ));
    output.push_str(color_end);
    output.push(' ');
    output.push_str(&charm_core::percent_string(current as u32, total as u32));
    if let Some(elapsed) = elapsed {
        let _ = write!(output, "  {}s", charm_core::elapsed_string(elapsed));
    }
    output.push_str("\x1b[K");
    write_output(output.as_bytes());
    return_none()
}

unsafe extern "C" fn progress_done(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    write_output(b"\n");
    return_none()
}

unsafe extern "C" fn spinner_frame(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(index) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"index must be int");
    };
    return_string(charm_core::spinner_frame(index as u32))
}

unsafe extern "C" fn spinner(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(index) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"index must be int");
    };
    let message = optional_string(&arguments, 1, false);
    let color = optional_string(&arguments, 2, true);
    let (color_start, color_end) = style_prefix(color.as_deref());
    let mut output = format!(
        "\r{color_start}{}{color_end}",
        charm_core::spinner_frame(index as u32)
    );
    if let Some(message) = message {
        output.push(' ');
        output.push_str(&message);
    }
    output.push_str("\x1b[K");
    write_output(output.as_bytes());
    return_none()
}

fn horizontal_line(
    output: &mut String,
    edges: (&str, &str, &str),
    horizontal: &str,
    widths: &[usize],
    color_start: &str,
    color_end: &str,
) {
    let (left, middle, right) = edges;
    output.push_str(color_start);
    output.push_str(left);
    for (index, width) in widths.iter().enumerate() {
        output.push_str(&charm_core::repeat(horizontal, width + 2));
        if index + 1 < widths.len() {
            output.push_str(middle);
        }
    }
    output.push_str(right);
    output.push_str(color_end);
    output.push('\n');
}

unsafe extern "C" fn table(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(rows) = arguments.get(0) else {
        return type_error(c"rows must be a list");
    };
    let Some(row_count) = rows.list_len() else {
        return type_error(c"rows must be a list");
    };
    if row_count == 0 {
        return return_none();
    }
    let Some(first_row) = rows.list_item(0) else {
        return type_error(c"rows must not be empty");
    };
    let Some(column_count) = first_row.list_len() else {
        return type_error(c"each row must be a list");
    };
    if column_count == 0 {
        return return_none();
    }

    let has_headers = arguments.get(1).and_then(Value::boolean).unwrap_or(false);
    let border = optional_string(&arguments, 2, false).unwrap_or_else(|| "square".to_owned());
    let border_color = optional_string(&arguments, 3, true);
    let chars = charm_core::table_chars(BorderStyle::for_table(&border));
    let actual_columns = column_count.min(32);
    let mut widths = vec![0_usize; actual_columns];
    for row_index in 0..row_count {
        let Some(row) = rows.list_item(row_index) else {
            continue;
        };
        let Some(row_length) = row.list_len() else {
            continue;
        };
        for (column_index, width) in widths
            .iter_mut()
            .enumerate()
            .take(row_length.min(actual_columns))
        {
            let cell = row
                .list_item(column_index)
                .and_then(Value::string)
                .unwrap_or_default();
            *width = (*width).max(charm_core::visible_len(&cell));
        }
    }

    let (color_start, color_end) = style_prefix(border_color.as_deref());
    let mut output = String::new();
    horizontal_line(
        &mut output,
        (chars.tl, chars.th, chars.tr),
        chars.h,
        &widths,
        &color_start,
        color_end,
    );
    for row_index in 0..row_count {
        let Some(row) = rows.list_item(row_index) else {
            continue;
        };
        let Some(row_length) = row.list_len() else {
            continue;
        };
        output.push_str(&color_start);
        output.push_str(chars.v);
        output.push_str(color_end);
        for (column_index, width) in widths.iter().enumerate() {
            output.push(' ');
            let cell = if column_index < row_length {
                row.list_item(column_index)
                    .and_then(Value::string)
                    .unwrap_or_default()
            } else {
                String::new()
            };
            if has_headers && row_index == 0 {
                output.push_str("\x1b[1m");
            }
            output.push_str(&cell);
            if has_headers && row_index == 0 {
                output.push_str("\x1b[0m");
            }
            output.push_str(&" ".repeat(width.saturating_sub(charm_core::visible_len(&cell))));
            output.push(' ');
            output.push_str(&color_start);
            output.push_str(chars.v);
            output.push_str(color_end);
        }
        output.push('\n');
        if has_headers && row_index == 0 && row_count > 1 {
            horizontal_line(
                &mut output,
                (chars.lv, chars.cross, chars.rv),
                chars.h,
                &widths,
                &color_start,
                color_end,
            );
        }
    }
    horizontal_line(
        &mut output,
        (chars.bl, chars.bh, chars.br),
        chars.h,
        &widths,
        &color_start,
        color_end,
    );
    write_output(output.as_bytes());
    return_none()
}
