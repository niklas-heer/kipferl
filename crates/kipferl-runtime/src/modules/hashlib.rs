use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use super::hash_core::{self, Algorithm};
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, Value, execute_module, return_bytes,
    type_error, value_error,
};

const COMPATIBILITY_SOURCE: &str = r#"
import binascii


class _Hash:
    def __init__(self, algorithm, data=None):
        if data is None:
            data = b""
        self._algorithm = algorithm
        self._data = data
        if algorithm == "md5":
            self.digest_size = 16
            self.block_size = 64
        elif algorithm == "sha1":
            self.digest_size = 20
            self.block_size = 64
        elif algorithm == "sha256":
            self.digest_size = 32
            self.block_size = 64
        else:
            self.digest_size = 64
            self.block_size = 128

    def update(self, data):
        if not isinstance(data, bytes):
            raise TypeError("a bytes-like object is required")
        self._data += data

    def digest(self):
        return _digest(self._algorithm, self._data)

    def hexdigest(self):
        return binascii.hexlify(self.digest()).decode()

    def copy(self):
        return _Hash(self._algorithm, self._data)


def md5(data=None):
    return _Hash("md5", data)


def sha1(data=None):
    return _Hash("sha1", data)


def sha256(data=None):
    return _Hash("sha256", data)


def sha512(data=None):
    return _Hash("sha512", data)


def new(name, data=None):
    normalized = name.lower().replace("-", "")
    if normalized not in ("md5", "sha1", "sha256", "sha512"):
        raise ValueError("unsupported hash type")
    return _Hash(normalized, data)


algorithms_guaranteed = {"md5", "sha1", "sha256", "sha512"}
algorithms_available = algorithms_guaranteed
"#;

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_digest",
    callback: digest,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"hashlib",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { kipferl_pocketpy_sys::py_printexc() };
        panic!("embedded hashlib compatibility layer failed");
    }
}

unsafe extern "C" fn digest(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(name) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"hash name must be a string");
    };
    let Some(input) = arguments.get(1).and_then(Value::bytes) else {
        return type_error(c"a bytes-like object is required");
    };
    let Some(algorithm) = Algorithm::parse(&name) else {
        return value_error(c"unsupported hash type");
    };
    return_bytes(&hash_core::digest(algorithm, &input))
}
