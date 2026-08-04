use std::io::{self, IsTerminal, Read, Write};
use std::mem::MaybeUninit;
use std::sync::{Mutex, MutexGuard};

use ucharm_pocketpy_sys as ffi;

use super::term_core::{self, DecodedKey};
use crate::native::{
    Arguments, NativeFunction, NativeModule, RootFrame, return_string, return_string_bytes,
    return_value, runtime_error, type_error,
};

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"size",
        callback: size,
    },
    NativeFunction {
        name: c"raw_mode",
        callback: raw_mode,
    },
    NativeFunction {
        name: c"read_key",
        callback: read_key,
    },
    NativeFunction {
        name: c"cursor_pos",
        callback: cursor_pos,
    },
    NativeFunction {
        name: c"cursor_up",
        callback: cursor_up,
    },
    NativeFunction {
        name: c"cursor_down",
        callback: cursor_down,
    },
    NativeFunction {
        name: c"cursor_left",
        callback: cursor_left,
    },
    NativeFunction {
        name: c"cursor_right",
        callback: cursor_right,
    },
    NativeFunction {
        name: c"clear",
        callback: clear,
    },
    NativeFunction {
        name: c"clear_line",
        callback: clear_line,
    },
    NativeFunction {
        name: c"hide_cursor",
        callback: hide_cursor,
    },
    NativeFunction {
        name: c"show_cursor",
        callback: show_cursor,
    },
    NativeFunction {
        name: c"is_tty",
        callback: is_tty,
    },
    NativeFunction {
        name: c"write",
        callback: write,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"term",
    functions: FUNCTIONS,
    signatures: &[],
};

#[derive(Default)]
struct RawModeState {
    original: Option<libc::termios>,
}

static RAW_MODE: Mutex<RawModeState> = Mutex::new(RawModeState { original: None });

enum RawModeError {
    Read,
    Enable,
}

fn raw_mode_state() -> MutexGuard<'static, RawModeState> {
    RAW_MODE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn enable_raw_mode() -> Result<(), RawModeError> {
    let mut state = raw_mode_state();
    if state.original.is_some() {
        return Ok(());
    }

    let mut original = MaybeUninit::<libc::termios>::uninit();
    // SAFETY: `original` points to writable storage and stdin is a valid file
    // descriptor. A zero return initializes the entire `termios` value.
    if unsafe { libc::tcgetattr(libc::STDIN_FILENO, original.as_mut_ptr()) } != 0 {
        return Err(RawModeError::Read);
    }
    // SAFETY: the successful `tcgetattr` call initialized `original`.
    let original = unsafe { original.assume_init() };
    let mut raw = original;
    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
    raw.c_iflag &= !(libc::IXON | libc::ICRNL | libc::BRKINT | libc::INPCK | libc::ISTRIP);
    raw.c_oflag &= !libc::OPOST;
    raw.c_cflag |= libc::CS8;
    raw.c_cc[libc::VMIN] = 0;
    raw.c_cc[libc::VTIME] = 1;

    // SAFETY: both the file descriptor and initialized settings are valid for
    // the duration of this call.
    if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) } != 0 {
        return Err(RawModeError::Enable);
    }
    state.original = Some(original);
    Ok(())
}

fn restore_raw_mode() {
    let mut state = raw_mode_state();
    let Some(original) = state.original else {
        return;
    };
    // SAFETY: stdin and the saved settings were valid when raw mode was
    // enabled. Retain the saved value if restoration fails so shutdown gets
    // another opportunity to restore it.
    if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &original) } == 0 {
        state.original = None;
    }
}

pub(super) fn shutdown() {
    restore_raw_mode();
}

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

fn return_bool(value: bool) -> bool {
    let mut roots = RootFrame::new();
    let value = roots.boolean(value);
    return_value(value)
}

unsafe extern "C" fn size(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }

    let mut window = MaybeUninit::<libc::winsize>::uninit();
    // SAFETY: `window` points to writable storage and stdout is a valid file
    // descriptor. A successful ioctl initializes the structure.
    let status = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, window.as_mut_ptr()) };
    let (columns, rows) = if status == -1 {
        (80_i64, 24_i64)
    } else {
        // SAFETY: the successful ioctl initialized `window`.
        let window = unsafe { window.assume_init() };
        if window.ws_col == 0 {
            (80, 24)
        } else {
            (i64::from(window.ws_col), i64::from(window.ws_row))
        }
    };

    let mut roots = RootFrame::new();
    let Some(result) = roots.tuple(2) else {
        return runtime_error(c"failed to create terminal size");
    };
    let columns = roots.integer(columns);
    let rows = roots.integer(rows);
    if !result.tuple_set(0, columns) || !result.tuple_set(1, rows) {
        return runtime_error(c"failed to create terminal size");
    }
    return_value(result)
}

unsafe extern "C" fn raw_mode(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(enable) = arguments.get(0).and_then(|value| value.boolean()) else {
        return type_error(c"expected bool");
    };
    if enable {
        match enable_raw_mode() {
            Ok(()) => {}
            Err(RawModeError::Read) => {
                return runtime_error(c"failed to read terminal settings");
            }
            Err(RawModeError::Enable) => return runtime_error(c"failed to enable raw mode"),
        }
    } else {
        restore_raw_mode();
    }
    return_none()
}

unsafe extern "C" fn read_key(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    let mut buffer = [0_u8; 8];
    let count = io::stdin().read(&mut buffer).unwrap_or(0);
    match term_core::decode_key(&buffer[..count]) {
        DecodedKey::None => return_none(),
        DecodedKey::Named(name) => return_string(name),
        DecodedKey::Text(bytes) => return_string_bytes(bytes),
    }
}

unsafe extern "C" fn cursor_pos(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(x) = arguments.get(0).and_then(|value| value.integer()) else {
        return type_error(c"x must be int");
    };
    let Some(y) = arguments.get(1).and_then(|value| value.integer()) else {
        return type_error(c"y must be int");
    };
    let Some(sequence) = term_core::cursor_position(x, y) else {
        return runtime_error(c"failed to format cursor position");
    };
    write_output(sequence.as_bytes());
    return_none()
}

fn move_cursor(argc: libc::c_int, argv: ffi::py_StackRef, direction: char) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 1) {
        return false;
    }
    let count = arguments
        .get(0)
        .and_then(|value| value.integer())
        .unwrap_or(1);
    let Some(sequence) = term_core::cursor_move(count, direction) else {
        return runtime_error(c"failed to format cursor move");
    };
    write_output(sequence.as_bytes());
    return_none()
}

unsafe extern "C" fn cursor_up(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    move_cursor(argc, argv, 'A')
}

unsafe extern "C" fn cursor_down(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    move_cursor(argc, argv, 'B')
}

unsafe extern "C" fn cursor_left(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    move_cursor(argc, argv, 'D')
}

unsafe extern "C" fn cursor_right(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    move_cursor(argc, argv, 'C')
}

fn fixed_output(argc: libc::c_int, argv: ffi::py_StackRef, bytes: &'static [u8]) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    write_output(bytes);
    return_none()
}

unsafe extern "C" fn clear(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    fixed_output(argc, argv, b"\x1b[2J\x1b[H")
}

unsafe extern "C" fn clear_line(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    fixed_output(argc, argv, b"\x1b[2K\r")
}

unsafe extern "C" fn hide_cursor(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    fixed_output(argc, argv, b"\x1b[?25l")
}

unsafe extern "C" fn show_cursor(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    fixed_output(argc, argv, b"\x1b[?25h")
}

unsafe extern "C" fn is_tty(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }
    return_bool(io::stdout().is_terminal())
}

unsafe extern "C" fn write(argc: libc::c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(text) = arguments.get(0).and_then(|value| value.string()) else {
        return type_error(c"text must be a string");
    };
    write_output(text.as_bytes());
    return_none()
}
