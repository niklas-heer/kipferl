use std::ffi::c_int;
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::sync::{Mutex, MutexGuard};

use ucharm_pocketpy_sys as ffi;

use crate::input_core::{clamp, wrap_index};
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeSignature, RootFrame, Value, return_string,
    return_value, type_error,
};

const SELECT: &[u8] = "❯ ".as_bytes();
const CHECKBOX_ON: &[u8] = "◉ ".as_bytes();
const CHECKBOX_OFF: &[u8] = "○ ".as_bytes();
const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
const CLEAR_LINE: &[u8] = b"\x1b[2K\r";
const CYAN: &[u8] = b"\x1b[36m";
const BOLD: &[u8] = b"\x1b[1m";
const RESET: &[u8] = b"\x1b[0m";
const DIM: &[u8] = b"\x1b[2m";
const TEST_FD: c_int = 3;

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"password",
    callback: password,
}];

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"select(prompt, choices, default=0)",
        callback: select,
    },
    NativeSignature {
        signature: c"multiselect(prompt, choices, defaults=None)",
        callback: multiselect,
    },
    NativeSignature {
        signature: c"confirm(prompt, default=True)",
        callback: confirm,
    },
    NativeSignature {
        signature: c"prompt(message, default=None)",
        callback: prompt,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"input",
    functions: FUNCTIONS,
    signatures: SIGNATURES,
};

struct InputState {
    tty_fd: c_int,
    original: Option<libc::termios>,
    raw_fd: Option<c_int>,
    test_keys: Option<Vec<u8>>,
    test_position: usize,
    test_initialized: bool,
}

static INPUT_STATE: Mutex<InputState> = Mutex::new(InputState {
    tty_fd: -1,
    original: None,
    raw_fd: None,
    test_keys: None,
    test_position: 0,
    test_initialized: false,
});

fn input_state() -> MutexGuard<'static, InputState> {
    INPUT_STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_output(bytes: &[u8]) {
    let mut output = io::stdout().lock();
    let _ = output.write_all(bytes);
    let _ = output.flush();
}

fn write_text(text: &str) {
    write_output(text.as_bytes());
}

fn write_newline() {
    write_output(b"\n");
}

fn ensure_tty_fd(state: &mut InputState) -> c_int {
    if state.tty_fd < 0 {
        // SAFETY: the path is a static NUL-terminated string and the flags do
        // not require a variadic mode argument.
        let opened = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
        state.tty_fd = if opened >= 0 {
            opened
        } else {
            libc::STDIN_FILENO
        };
    }
    state.tty_fd
}

fn initialize_test_mode(state: &mut InputState) {
    if state.test_initialized {
        return;
    }
    state.test_initialized = true;

    if let Some(keys) = std::env::var_os("MCHARM_TEST_KEYS") {
        let bytes = keys.as_os_str().as_bytes();
        if !bytes.is_empty() {
            state.test_keys = Some(bytes.to_vec());
            return;
        }
    }

    // Preserve the legacy fd 3 key-injection hook used by external E2E
    // harnesses. The descriptor is probed non-blockingly and left untouched
    // when it is absent.
    // SAFETY: fcntl and read receive a fixed descriptor and valid buffer.
    let flags = unsafe { libc::fcntl(TEST_FD, libc::F_GETFL) };
    if flags == -1 {
        return;
    }
    // SAFETY: `flags` came from F_GETFL for this descriptor.
    unsafe { libc::fcntl(TEST_FD, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    let mut buffer = [0_u8; 4095];
    // SAFETY: the buffer is writable for its full length.
    let count = unsafe { libc::read(TEST_FD, buffer.as_mut_ptr().cast(), buffer.len()) };
    // SAFETY: restore the descriptor's exact original status flags.
    unsafe { libc::fcntl(TEST_FD, libc::F_SETFL, flags) };
    let Ok(count) = usize::try_from(count) else {
        return;
    };
    if count == 0 {
        return;
    }

    let mut keys = buffer[..count].to_vec();
    for byte in &mut keys {
        if *byte == b'\n' {
            *byte = b',';
        }
    }
    if keys.last() == Some(&b',') {
        keys.pop();
    }
    state.test_keys = Some(keys);
}

fn map_key_name(name: &[u8]) -> u8 {
    match name {
        b"up" | b"k" => b'u',
        b"down" | b"j" => b'd',
        b"enter" => b'e',
        b"space" => b's',
        b"escape" => b'q',
        b"backspace" => b'b',
        [byte] => *byte,
        _ => 0,
    }
}

fn map_raw_key_name(name: &[u8]) -> u8 {
    match name {
        b"enter" => b'\r',
        b"space" => b' ',
        b"escape" => 0x1b,
        b"backspace" => 0x7f,
        [byte] => *byte,
        _ => 0,
    }
}

fn read_test_token(state: &mut InputState, raw: bool) -> u8 {
    let Some(buffer) = state.test_keys.as_ref() else {
        return 0;
    };
    let mut start = state.test_position;
    while start < buffer.len() && matches!(buffer[start], b',' | b' ' | b'\t') {
        start += 1;
    }
    if start >= buffer.len() {
        state.test_position = start;
        return 0;
    }
    let mut end = start;
    while end < buffer.len() && buffer[end] != b',' {
        end += 1;
    }
    let mut trimmed_end = end;
    while trimmed_end > start && matches!(buffer[trimmed_end - 1], b' ' | b'\t') {
        trimmed_end -= 1;
    }
    let key = if raw {
        map_raw_key_name(&buffer[start..trimmed_end])
    } else {
        map_key_name(&buffer[start..trimmed_end])
    };
    state.test_position = if end < buffer.len() { end + 1 } else { end };
    key
}

fn read_raw_character() -> u8 {
    let mut state = input_state();
    initialize_test_mode(&mut state);
    if state.test_keys.is_some() {
        return read_test_token(&mut state, true);
    }
    let fd = ensure_tty_fd(&mut state);
    let mut byte = 0_u8;
    // SAFETY: `byte` is writable for one byte and `fd` is either /dev/tty or
    // stdin. Raw mode bounds the wait when the descriptor is a terminal.
    let count = unsafe { libc::read(fd, (&mut byte as *mut u8).cast(), 1) };
    if count == 1 { byte } else { 0 }
}

fn read_key() -> u8 {
    let mut state = input_state();
    initialize_test_mode(&mut state);
    if state.test_keys.is_some() {
        return read_test_token(&mut state, false);
    }

    let fd = ensure_tty_fd(&mut state);
    let mut buffer = [0_u8; 8];
    // SAFETY: the buffer is writable for seven bytes and the descriptor is
    // either /dev/tty or stdin.
    let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len() - 1) };
    let Ok(count) = usize::try_from(count) else {
        return 0;
    };
    if count == 0 {
        return 0;
    }

    if buffer[0] == 0x1b {
        let mut total = count;
        if count == 1 {
            let end = buffer.len() - 1;
            let remaining = end - 1;
            // SAFETY: the remaining buffer is writable and `fd` is valid.
            let extra = unsafe { libc::read(fd, buffer[1..end].as_mut_ptr().cast(), remaining) };
            if let Ok(extra) = usize::try_from(extra) {
                total += extra;
            }
            if total == 1 {
                return b'q';
            }
        }
        if total >= 3 && buffer[1] == b'[' {
            return match buffer[2] {
                b'A' => b'u',
                b'B' => b'd',
                _ => 0,
            };
        }
        return 0;
    }

    if count == 1 {
        return match buffer[0] {
            b'\r' | b'\n' => b'e',
            b' ' => b's',
            b'j' => b'd',
            b'k' => b'u',
            b'q' | 0x03 => b'q',
            b'y' | b'Y' | b'n' | b'N' => buffer[0],
            0x7f | 0x08 => b'b',
            byte => byte,
        };
    }
    0
}

fn enable_raw_mode() {
    let mut state = input_state();
    if state.original.is_some() {
        return;
    }
    let stdin = libc::STDIN_FILENO;
    // SAFETY: stdin is a valid process descriptor.
    let fd = if unsafe { libc::isatty(stdin) } == 1 {
        stdin
    } else {
        let tty = ensure_tty_fd(&mut state);
        // SAFETY: process-group queries use the current process and selected
        // terminal descriptor. Failures are deliberately non-fatal for parity.
        unsafe {
            let process_group = libc::getpgrp();
            if libc::tcgetpgrp(tty) != process_group {
                libc::tcsetpgrp(tty, process_group);
            }
        }
        tty
    };

    let mut original = MaybeUninit::<libc::termios>::uninit();
    // SAFETY: `original` points to writable termios storage.
    if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
        return;
    }
    // SAFETY: successful tcgetattr initialized the entire value.
    let original = unsafe { original.assume_init() };
    let mut raw = original;
    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
    raw.c_iflag &= !(libc::IXON | libc::ICRNL | libc::BRKINT | libc::INPCK | libc::ISTRIP);
    raw.c_oflag &= !libc::OPOST;
    raw.c_cflag |= libc::CS8;
    raw.c_cc[libc::VMIN] = 0;
    raw.c_cc[libc::VTIME] = 1;
    // SAFETY: both the terminal descriptor and settings are valid.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } == 0 {
        state.original = Some(original);
        state.raw_fd = Some(fd);
    }
}

fn disable_raw_mode() {
    let mut state = input_state();
    let (Some(original), Some(fd)) = (state.original, state.raw_fd) else {
        return;
    };
    // SAFETY: `fd` and `original` were captured by a successful raw-mode
    // transition. Keep them on failure so VM shutdown can retry restoration.
    if unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &original) } == 0 {
        state.original = None;
        state.raw_fd = None;
    }
}

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> Self {
        enable_raw_mode();
        Self
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_raw_mode();
    }
}

struct CursorGuard;

impl CursorGuard {
    fn hide() -> Self {
        write_output(HIDE_CURSOR);
        Self
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        write_output(SHOW_CURSOR);
    }
}

pub(super) fn shutdown() {
    disable_raw_mode();
    let mut state = input_state();
    if state.tty_fd > libc::STDERR_FILENO {
        // SAFETY: μcharm uniquely owns descriptors it opened for /dev/tty.
        unsafe { libc::close(state.tty_fd) };
    }
    state.tty_fd = -1;
    state.test_keys = None;
    state.test_position = 0;
    state.test_initialized = false;
}

fn sequence_len(value: Value) -> Option<usize> {
    value.list_len().or_else(|| value.tuple_len())
}

fn sequence_item(value: Value, index: usize) -> Option<Value> {
    value.list_item(index).or_else(|| value.tuple_item(index))
}

fn display_text(value: Value) -> Option<String> {
    let mut text = value.string()?;
    if let Some(nul) = text.find('\0') {
        text.truncate(nul);
    }
    Some(text)
}

fn return_none() -> bool {
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}

fn render_select(choices: Value, length: usize, selected: i32, clear: bool) {
    for index in 0..length {
        if clear {
            write_output(CLEAR_LINE);
        }
        let Some(choice) = sequence_item(choices, index).and_then(display_text) else {
            continue;
        };
        if i32::try_from(index) == Ok(selected) {
            write_output(CYAN);
            write_output(b"  ");
            write_output(SELECT);
            write_text(&choice);
            write_output(RESET);
        } else {
            write_output(b"    ");
            write_text(&choice);
        }
        write_newline();
    }
}

fn cursor_up(count: usize) {
    write_text(&format!("\x1b[{count}A"));
}

unsafe extern "C" fn select(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(prompt) = arguments.get(0).and_then(display_text) else {
        return type_error(c"prompt must be a string");
    };
    let Some(choices) = arguments.get(1) else {
        return type_error(c"choices required");
    };
    let Some(length) = sequence_len(choices) else {
        return return_none();
    };
    if length == 0 {
        return return_none();
    }
    let maximum = i32::try_from(length - 1).unwrap_or(i32::MAX);
    let requested = arguments
        .get(2)
        .and_then(Value::integer)
        .unwrap_or(0)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    let mut selected = clamp(requested, 0, maximum);

    write_output(CYAN);
    write_output(BOLD);
    write_output(b"? ");
    write_output(RESET);
    write_text(&prompt);
    write_newline();

    let cursor = CursorGuard::hide();
    render_select(choices, length, selected, false);
    let raw = RawModeGuard::enable();
    let result = loop {
        match read_key() {
            b'd' => selected = wrap_index(selected + 1, maximum.saturating_add(1)),
            b'u' => selected = wrap_index(selected - 1, maximum.saturating_add(1)),
            b'e' | b's' => break usize::try_from(selected).ok(),
            b'q' => break None,
            _ => continue,
        }
        cursor_up(length);
        render_select(choices, length, selected, true);
    };
    drop(raw);
    drop(cursor);

    if let Some(result) = result.and_then(|index| sequence_item(choices, index)) {
        return_value(result)
    } else {
        return_none()
    }
}

fn render_multiselect(
    choices: Value,
    visible_length: usize,
    cursor: i32,
    selected: &[bool; 256],
    clear: bool,
) {
    for (index, is_selected) in selected.iter().enumerate().take(visible_length) {
        if clear {
            write_output(CLEAR_LINE);
        }
        let Some(choice) = sequence_item(choices, index).and_then(display_text) else {
            continue;
        };
        if i32::try_from(index) == Ok(cursor) {
            write_output(CYAN);
            write_output(b"  ");
        } else {
            write_output(b"  ");
        }
        write_output(if *is_selected {
            CHECKBOX_ON
        } else {
            CHECKBOX_OFF
        });
        write_text(&choice);
        if i32::try_from(index) == Ok(cursor) {
            write_output(RESET);
        }
        write_newline();
    }
}

unsafe extern "C" fn multiselect(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(prompt) = arguments.get(0).and_then(display_text) else {
        return type_error(c"prompt must be a string");
    };
    let Some(choices) = arguments.get(1) else {
        return type_error(c"choices required");
    };
    let length = sequence_len(choices).unwrap_or(0);
    if length == 0 {
        let mut roots = RootFrame::new();
        let result = roots.list();
        return return_value(result);
    }

    let visible_length = length.min(256);
    let mut selected = [false; 256];
    if let Some(defaults) = arguments.get(2)
        && let Some(default_length) = defaults.list_len()
    {
        for default_index in 0..default_length {
            let Some(default) = defaults.list_item(default_index).and_then(display_text) else {
                continue;
            };
            for (index, slot) in selected.iter_mut().enumerate().take(visible_length) {
                if sequence_item(choices, index)
                    .and_then(display_text)
                    .is_some_and(|choice| choice == default)
                {
                    *slot = true;
                    break;
                }
            }
        }
    }

    write_output(CYAN);
    write_output(BOLD);
    write_output(b"? ");
    write_output(RESET);
    write_text(&prompt);
    write_output(DIM);
    write_output(b" (space to toggle, enter to confirm)");
    write_output(RESET);
    write_newline();

    let cursor_guard = CursorGuard::hide();
    let mut cursor = 0_i32;
    render_multiselect(choices, visible_length, cursor, &selected, false);
    let raw = RawModeGuard::enable();
    let confirmed = loop {
        match read_key() {
            b'd' => {
                cursor = wrap_index(cursor + 1, i32::try_from(length).unwrap_or(i32::MAX));
            }
            b'u' => {
                cursor = wrap_index(cursor - 1, i32::try_from(length).unwrap_or(i32::MAX));
            }
            b's' => {
                if let Ok(index) = usize::try_from(cursor)
                    && index < selected.len()
                {
                    selected[index] = !selected[index];
                }
            }
            b'e' => break true,
            b'q' => break false,
            _ => continue,
        }
        cursor_up(visible_length);
        render_multiselect(choices, visible_length, cursor, &selected, true);
    };
    drop(raw);
    drop(cursor_guard);

    let mut roots = RootFrame::new();
    let result = roots.list();
    if confirmed {
        for (index, is_selected) in selected.iter().enumerate().take(visible_length) {
            if *is_selected && let Some(choice) = sequence_item(choices, index) {
                result.list_append(choice);
            }
        }
    }
    return_value(result)
}

unsafe extern "C" fn confirm(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(prompt) = arguments.get(0).and_then(display_text) else {
        return type_error(c"prompt must be a string");
    };
    let default = arguments.get(1).and_then(Value::boolean).unwrap_or(true);

    write_output(CYAN);
    write_output(BOLD);
    write_output(b"? ");
    write_output(RESET);
    write_text(&prompt);
    write_output(b" ");
    write_output(DIM);
    write_output(if default { b"(Y/n)" } else { b"(y/N)" });
    write_output(RESET);
    write_output(b" ");

    let raw = RawModeGuard::enable();
    let result = loop {
        match read_key() {
            b'y' | b'Y' => break true,
            b'n' | b'N' | b'q' => break false,
            b'e' => break default,
            _ => continue,
        }
    };
    drop(raw);

    write_output(CYAN);
    write_output(if result { b"Yes" } else { b"No" });
    write_output(RESET);
    write_newline();

    let mut roots = RootFrame::new();
    let result = roots.boolean(result);
    return_value(result)
}

unsafe extern "C" fn prompt(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(message) = arguments.get(0).and_then(display_text) else {
        return type_error(c"message must be a string");
    };
    let default = arguments.get(1).and_then(|value| {
        if value.is_none() {
            None
        } else {
            display_text(value)
        }
    });

    write_output(CYAN);
    write_output(BOLD);
    write_output(b"? ");
    write_output(RESET);
    write_text(&message);
    if let Some(default) = default.as_deref() {
        write_output(DIM);
        write_output(b" (");
        write_text(default);
        write_output(b")");
        write_output(RESET);
    }
    write_output(b" ");

    let raw = RawModeGuard::enable();
    let mut input = Vec::with_capacity(1023);
    while input.len() < 1023 {
        match read_raw_character() {
            0 => continue,
            b'\r' | b'\n' => break,
            0x1b | 0x03 => {
                drop(raw);
                write_newline();
                return return_string(default.as_deref().unwrap_or(""));
            }
            0x7f | 0x08 if input.pop().is_some() => write_output(b"\x08 \x08"),
            0x7f | 0x08 => {}
            byte @ 32..=126 => {
                input.push(byte);
                write_output(&[byte]);
            }
            _ => {}
        }
    }
    drop(raw);
    write_newline();

    if input.is_empty()
        && let Some(default) = default
    {
        return return_string(&default);
    }
    return_string(std::str::from_utf8(&input).unwrap_or(""))
}

unsafe extern "C" fn password(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active argument stack to this callback.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(message) = arguments.get(0).and_then(display_text) else {
        return type_error(c"message must be a string");
    };

    write_output(CYAN);
    write_output(BOLD);
    write_output(b"? ");
    write_output(RESET);
    write_text(&message);
    write_output(b" ");

    let raw = RawModeGuard::enable();
    let mut input = Vec::with_capacity(1023);
    while input.len() < 1023 {
        match read_raw_character() {
            0 => continue,
            b'\r' | b'\n' => break,
            0x1b | 0x03 => {
                drop(raw);
                write_newline();
                return return_string("");
            }
            0x7f | 0x08 => {
                input.pop();
            }
            byte @ 32..=126 => input.push(byte),
            _ => {}
        }
    }
    drop(raw);
    write_newline();
    return_string(std::str::from_utf8(&input).unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::{map_key_name, map_raw_key_name};

    #[test]
    fn maps_navigation_test_tokens() {
        assert_eq!(map_key_name(b"up"), b'u');
        assert_eq!(map_key_name(b"down"), b'd');
        assert_eq!(map_key_name(b"enter"), b'e');
        assert_eq!(map_key_name(b"space"), b's');
        assert_eq!(map_key_name(b"escape"), b'q');
        assert_eq!(map_key_name(b"backspace"), b'b');
        assert_eq!(map_key_name(b"j"), b'd');
        assert_eq!(map_key_name(b"k"), b'u');
        assert_eq!(map_key_name(b"yes"), 0);
    }

    #[test]
    fn maps_raw_test_tokens() {
        assert_eq!(map_raw_key_name(b"enter"), b'\r');
        assert_eq!(map_raw_key_name(b"space"), b' ');
        assert_eq!(map_raw_key_name(b"escape"), 0x1b);
        assert_eq!(map_raw_key_name(b"backspace"), 0x7f);
        assert_eq!(map_raw_key_name(b"x"), b'x');
        assert_eq!(map_raw_key_name(b"text"), 0);
    }
}
