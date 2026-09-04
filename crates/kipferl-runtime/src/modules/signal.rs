use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeIntConstant, NativeModule, NativeModuleKind, NativeSignature, RootFrame,
    Value, execute_module, return_value, type_error,
};

const CONSTANTS: &[NativeIntConstant] = &[
    NativeIntConstant {
        name: c"SIG_DFL",
        value: 0,
    },
    NativeIntConstant {
        name: c"SIG_IGN",
        value: 1,
    },
    NativeIntConstant {
        name: c"SIGHUP",
        value: 1,
    },
    NativeIntConstant {
        name: c"SIGINT",
        value: 2,
    },
    NativeIntConstant {
        name: c"SIGQUIT",
        value: 3,
    },
    NativeIntConstant {
        name: c"SIGILL",
        value: 4,
    },
    NativeIntConstant {
        name: c"SIGTRAP",
        value: 5,
    },
    NativeIntConstant {
        name: c"SIGABRT",
        value: 6,
    },
    NativeIntConstant {
        name: c"SIGBUS",
        value: 7,
    },
    NativeIntConstant {
        name: c"SIGFPE",
        value: 8,
    },
    NativeIntConstant {
        name: c"SIGKILL",
        value: 9,
    },
    NativeIntConstant {
        name: c"SIGUSR1",
        value: 10,
    },
    NativeIntConstant {
        name: c"SIGSEGV",
        value: 11,
    },
    NativeIntConstant {
        name: c"SIGUSR2",
        value: 12,
    },
    NativeIntConstant {
        name: c"SIGPIPE",
        value: 13,
    },
    NativeIntConstant {
        name: c"SIGALRM",
        value: 14,
    },
    NativeIntConstant {
        name: c"SIGTERM",
        value: 15,
    },
    NativeIntConstant {
        name: c"SIGCHLD",
        value: 17,
    },
    NativeIntConstant {
        name: c"SIGCONT",
        value: 18,
    },
    NativeIntConstant {
        name: c"SIGSTOP",
        value: 19,
    },
    NativeIntConstant {
        name: c"SIGTSTP",
        value: 20,
    },
];

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"raise_signal(signum)",
    callback: raise_signal,
}];

const COMPATIBILITY_SOURCE: &str = r"
_handlers = {}


def getsignal(signum):
    if signum in _handlers:
        return _handlers[signum]
    return SIG_DFL


def signal(signum, handler):
    previous = getsignal(signum)
    _handlers[signum] = handler
    return previous


def alarm(seconds):
    return 0


def pause():
    return None
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"signal",
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
        panic!("embedded signal compatibility layer failed");
    }
}

unsafe extern "C" fn raise_signal(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(signum) = arguments.get(0).and_then(Value::integer) else {
        return type_error(c"signum must be an integer");
    };
    let Ok(signum) = c_int::try_from(signum) else {
        return type_error(c"signum is out of range");
    };
    // SAFETY: libc validates the numeric signal. This intentionally preserves
    // the legacy module's process-level raise semantics.
    unsafe { libc::raise(signum) };
    let mut roots = RootFrame::new();
    let none = roots.none();
    return_value(none)
}
