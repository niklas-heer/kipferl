use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use super::hash_core::{self, Algorithm};
use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, execute_module,
    return_bytes, return_value, type_error, value_error,
};

const COMPATIBILITY_SOURCE: &str = r#"
import binascii


class HMAC:
    def __init__(self, key, msg=None, digestmod="sha256"):
        if msg is None:
            msg = b""
        self._key = key
        self._message = msg
        if isinstance(digestmod, str):
            self._algorithm = digestmod.lower().replace("-", "")
        else:
            self._algorithm = digestmod.__name__
        if self._algorithm not in ("md5", "sha1", "sha256", "sha512"):
            raise ValueError("unsupported hash type")
        self.digest_size = len(self.digest())
        self.block_size = 128 if self._algorithm == "sha512" else 64

    def update(self, msg):
        if not isinstance(msg, bytes):
            raise TypeError("a bytes-like object is required")
        self._message += msg

    def digest(self):
        return _hmac_digest(self._algorithm, self._key, self._message)

    def hexdigest(self):
        return binascii.hexlify(self.digest()).decode()

    def copy(self):
        return HMAC(self._key, self._message, self._algorithm)


def new(key, msg=None, digestmod="sha256"):
    return HMAC(key, msg, digestmod)
"#;

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"_hmac_digest",
        callback: hmac_digest,
    },
    NativeFunction {
        name: c"compare_digest",
        callback: compare_digest,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"hmac",
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
        unsafe { ucharm_pocketpy_sys::py_printexc() };
        panic!("embedded hmac compatibility layer failed");
    }
}

unsafe extern "C" fn hmac_digest(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(3, 3) {
        return false;
    }
    let Some(name) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"hash name must be a string");
    };
    let Some(key) = arguments.get(1).and_then(Value::bytes) else {
        return type_error(c"key must be bytes");
    };
    let Some(message) = arguments.get(2).and_then(Value::bytes) else {
        return type_error(c"msg must be bytes");
    };
    let Some(algorithm) = Algorithm::parse(&name) else {
        return value_error(c"unsupported hash type");
    };
    return_bytes(&hash_core::hmac(algorithm, &key, &message))
}

unsafe extern "C" fn compare_digest(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let left = arguments.get(0).expect("arity checked");
    let right = arguments.get(1).expect("arity checked");
    let (left, right) = if let (Some(left), Some(right)) = (left.bytes(), right.bytes()) {
        (left, right)
    } else if let (Some(left), Some(right)) = (left.string(), right.string()) {
        (left.into_bytes(), right.into_bytes())
    } else {
        return type_error(c"unsupported operand types for compare_digest");
    };
    let mut difference = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    let mut roots = RootFrame::new();
    let result = roots.boolean(difference == 0);
    return_value(result)
}
