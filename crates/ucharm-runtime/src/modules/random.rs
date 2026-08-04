use std::ffi::{c_int, c_void};

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, execute_module,
    return_value, runtime_error,
};

unsafe extern "C" {
    fn getentropy(buffer: *mut c_void, length: usize) -> c_int;
}

const FUNCTIONS: &[NativeFunction] = &[NativeFunction {
    name: c"_ucharm_secure_word",
    callback: secure_word,
}];

const COMPATIBILITY_SOURCE: &str = r#"
_rust_original_random_choice = choice


def randrange(start, stop=None, step=1):
    if stop is None:
        stop = start
        start = 0
    if step == 0:
        raise ValueError("zero step for randrange()")
    if step > 0:
        if start >= stop:
            raise ValueError("empty range for randrange()")
        count = (stop - start - 1) // step + 1
    else:
        if start <= stop:
            raise ValueError("empty range for randrange()")
        count = (start - stop - 1) // (-step) + 1
    return start + (_ucharm_secure_word() % count) * step


def choice(sequence):
    if isinstance(sequence, str):
        if len(sequence) == 0:
            raise IndexError("Cannot choose from an empty sequence")
        return sequence[randrange(len(sequence))]
    return _rust_original_random_choice(sequence)


def sample(population, count):
    if not isinstance(population, list):
        raise TypeError("population must be a list")
    if count < 0 or count > len(population):
        raise ValueError("sample larger than population or is negative")
    working = population.copy()
    result = []
    for _ in range(count):
        index = randrange(len(working))
        result.append(working.pop(index))
    return result


def getrandbits(count):
    if count < 0:
        raise ValueError("number of bits must be non-negative")
    if count == 0:
        return 0
    if count > 62:
        raise ValueError("k too large (max 62)")
    return _ucharm_secure_word() & ((1 << count) - 1)
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"random",
    kind: NativeModuleKind::ImportAndExtend,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded random compatibility layer failed"
    );
}

unsafe extern "C" fn secure_word(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from PocketPy with its active callback argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(0, 0) {
        return false;
    }

    let mut bytes = [0_u8; 8];
    // SAFETY: `bytes` is writable for its exact length. All supported macOS
    // and Linux release targets provide `getentropy` and accept requests up
    // to 256 bytes.
    if unsafe { getentropy(bytes.as_mut_ptr().cast(), bytes.len()) } != 0 {
        return runtime_error(c"OS random source failed");
    }

    let value = u64::from_le_bytes(bytes) & ((1_u64 << 62) - 1);
    let mut roots = RootFrame::new();
    let value = roots.integer(value as i64);
    return_value(value)
}
