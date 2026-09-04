use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeIntConstant, NativeModule, NativeModuleKind, NativeSignature, Value,
    execute_module, return_value, type_error,
};

const CONSTANTS: &[NativeIntConstant] = &[
    NativeIntConstant {
        name: c"NOTSET",
        value: 0,
    },
    NativeIntConstant {
        name: c"DEBUG",
        value: 10,
    },
    NativeIntConstant {
        name: c"INFO",
        value: 20,
    },
    NativeIntConstant {
        name: c"WARNING",
        value: 30,
    },
    NativeIntConstant {
        name: c"WARN",
        value: 30,
    },
    NativeIntConstant {
        name: c"ERROR",
        value: 40,
    },
    NativeIntConstant {
        name: c"CRITICAL",
        value: 50,
    },
    NativeIntConstant {
        name: c"FATAL",
        value: 50,
    },
];

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"_emit(level, message)",
    callback: emit,
}];

const COMPATIBILITY_SOURCE: &str = r#"
class Handler:
    def __init__(self):
        self.level = 0
        self.formatter = None

    def setLevel(self, level):
        self.level = level

    def setFormatter(self, formatter):
        self.formatter = formatter


class StreamHandler(Handler):
    pass


class FileHandler(Handler):
    def __init__(self, filename=None):
        Handler.__init__(self)
        self.filename = filename


class Formatter:
    def __init__(self, fmt=None):
        self.fmt = fmt


class Logger:
    def __init__(self, name, parent=None):
        self.name = name
        self.level = 0
        self.handlers = []
        self.parent = parent

    def setLevel(self, level):
        self.level = level

    def getEffectiveLevel(self):
        return self.level

    def addHandler(self, handler):
        self.handlers.append(handler)

    def debug(self, message, *args, **kwargs):
        if DEBUG >= self.getEffectiveLevel():
            _emit(DEBUG, message)

    def info(self, message, *args, **kwargs):
        if INFO >= self.getEffectiveLevel():
            _emit(INFO, message)

    def warning(self, message, *args, **kwargs):
        if WARNING >= self.getEffectiveLevel():
            _emit(WARNING, message)

    def error(self, message, *args, **kwargs):
        if ERROR >= self.getEffectiveLevel():
            _emit(ERROR, message)

    def critical(self, message, *args, **kwargs):
        if CRITICAL >= self.getEffectiveLevel():
            _emit(CRITICAL, message)


_root = Logger("")
_loggers = {"": _root}


def getLogger(name=None):
    if name is None or name == "":
        return _root
    if name in _loggers:
        return _loggers[name]
    parent = _root
    if "." in name:
        parent = getLogger(name.rsplit(".", 1)[0])
    logger = Logger(name, parent)
    _loggers[name] = logger
    return logger


def basicConfig(**kwargs):
    return None


def debug(message, *args, **kwargs):
    return _emit(DEBUG, message)


def info(message, *args, **kwargs):
    return _emit(INFO, message)


def warning(message, *args, **kwargs):
    return _emit(WARNING, message)


def error(message, *args, **kwargs):
    return _emit(ERROR, message)


def critical(message, *args, **kwargs):
    return _emit(CRITICAL, message)


def log(level, message, *args, **kwargs):
    return _emit(level, message)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"logging",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: CONSTANTS,
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded logging compatibility layer failed");
    }
}

unsafe extern "C" fn emit(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(level) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"level must be an integer");
    };
    let Some(message) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"message must be a string");
    };
    let prefix = match level {
        10 => "DEBUG",
        20 => "INFO",
        30 => "WARNING",
        40 => "ERROR",
        50 => "CRITICAL",
        _ => "LOG",
    };
    eprintln!("{prefix}: {message}");
    let mut roots = crate::native::RootFrame::new();
    let none = roots.none();
    return_value(none)
}
