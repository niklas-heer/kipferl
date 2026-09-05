use std::ffi::{CString, c_int};
use std::io::Write;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, RootFrame, Value, execute_module,
    return_value, type_error, value_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"_stream_write(error, text)",
        callback: stream_write,
    },
    NativeSignature {
        signature: c"_kipferl_compile_module(source, filename)",
        callback: compile_module,
    },
];

const SOURCE: &str = r"
version_info = (3, 11, 0)
path = []
modules = {}
stdin = None
maxsize = 9223372036854775807
executable = ''
_interned = {}

class _Implementation:
    def __init__(self):
        self.name = 'pocketpy'

class _Flags:
    pass

class _Stream:
    def __init__(self, error):
        self.error = error

    def write(self, text):
        return _stream_write(self.error, text)

    def flush(self):
        return None

implementation = _Implementation()
flags = _Flags()
stdout = _Stream(False)
stderr = _Stream(True)

def exit(code=None):
    raise SystemExit(code)

def getsizeof(value):
    if isinstance(value, str):
        return max(1, len(value))
    if isinstance(value, (list, tuple, dict)):
        return len(value) + 1
    return 1

def intern(value):
    if not isinstance(value, str):
        raise TypeError('expected string')
    if value in _interned:
        return _interned[value]
    _interned[value] = value
    return value
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"sys",
    kind: NativeModuleKind::Extend,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
#[expect(
    clippy::expect_used,
    reason = "The initializer just created sys.modules and uses the literal three-byte sys key."
)]
fn initialize(module: Value) {
    if !execute_module(module, SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded sys compatibility layer failed");
    }

    let mut roots = RootFrame::new();
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "unix"
    };
    if let Some(value) = roots.string(platform) {
        module.set_attribute(c"platform", value);
    }
    if let Some(value) = roots.string(if cfg!(target_endian = "little") {
        "little"
    } else {
        "big"
    }) {
        module.set_attribute(c"byteorder", value);
    }

    let modules = module
        .attribute(c"modules")
        .expect("sys.modules created by compatibility source");
    let key = roots.string("sys").expect("short sys.modules key");
    assert!(modules.dict_set(key, module), "insert sys into sys.modules");
}

unsafe extern "C" fn stream_write(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(error) = arguments.get(0).and_then(Value::boolean) else {
        return type_error(c"stream selector must be bool");
    };
    let Some(text) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"write() argument must be str");
    };
    let result = if error {
        std::io::stderr().write_all(text.as_bytes())
    } else {
        std::io::stdout().write_all(text.as_bytes())
    };
    if result.is_err() {
        return type_error(c"failed to write stream");
    }
    let mut roots = RootFrame::new();
    let length = roots.integer(i64::try_from(text.len()).unwrap_or(i64::MAX));
    return_value(length)
}

/// Return non-dynamic module code for generated wrappers to execute in their
/// current module via exec(code), without replacing the module's namespace.
unsafe extern "C" fn compile_module(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing argc values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(source) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"module source must be str");
    };
    let Some(filename) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"module filename must be str");
    };
    let (Ok(source), Ok(filename)) = (CString::new(source), CString::new(filename)) else {
        return value_error(c"module source or filename contains a NUL byte");
    };
    // SAFETY: the callback runs in the active VM and owns both terminated
    // strings throughout compilation. Module-mode compilation stores the code
    // in the VM's return root but never executes it. The caller may explicitly
    // exec the returned code in its own current module without globals/locals.
    unsafe {
        ffi::py_compile(
            source.as_ptr(),
            filename.as_ptr(),
            ffi::py_CompileMode_EXEC_MODE,
            false,
        )
    }
}
