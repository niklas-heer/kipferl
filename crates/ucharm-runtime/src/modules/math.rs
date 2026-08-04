use std::ffi::c_int;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    global_integer, global_number, global_tuple, return_number, return_value, type_error,
    value_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"sinh(x)",
        callback: sinh,
    },
    NativeSignature {
        signature: c"cosh(x)",
        callback: cosh,
    },
    NativeSignature {
        signature: c"tanh(x)",
        callback: tanh,
    },
    NativeSignature {
        signature: c"asinh(x)",
        callback: asinh,
    },
    NativeSignature {
        signature: c"acosh(x)",
        callback: acosh,
    },
    NativeSignature {
        signature: c"atanh(x)",
        callback: atanh,
    },
    NativeSignature {
        signature: c"frexp(x)",
        callback: frexp,
    },
    NativeSignature {
        signature: c"ldexp(x, i)",
        callback: ldexp,
    },
    NativeSignature {
        signature: c"expm1(x)",
        callback: expm1,
    },
    NativeSignature {
        signature: c"log1p(x)",
        callback: log1p,
    },
    NativeSignature {
        signature: c"hypot(x, y)",
        callback: hypot,
    },
    NativeSignature {
        signature: c"cbrt(x)",
        callback: cbrt,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"math",
    kind: NativeModuleKind::Extend,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, "tau = 6.283185307179586"),
        "extend math constants"
    );
}

fn unary(argc: c_int, argv: ffi::py_StackRef, operation: impl FnOnce(f64) -> f64) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    return_number(operation(value))
}

unsafe extern "C" fn sinh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::sinh)
}

unsafe extern "C" fn cosh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::cosh)
}

unsafe extern "C" fn tanh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::tanh)
}

unsafe extern "C" fn asinh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::asinh)
}

unsafe extern "C" fn acosh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    if value < 1.0 {
        return value_error(c"math domain error");
    }
    return_number(value.acosh())
}

unsafe extern "C" fn atanh(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    if !(-1.0..1.0).contains(&value) {
        return value_error(c"math domain error");
    }
    return_number(value.atanh())
}

unsafe extern "C" fn frexp(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    let (mantissa, exponent) = if value == 0.0 || !value.is_finite() {
        (value, 0)
    } else {
        let exponent = value.abs().log2().floor() as i32 + 1;
        (value / 2.0_f64.powi(exponent), exponent)
    };
    let Some(result) = global_tuple(0, 2) else {
        return value_error(c"failed to create frexp result");
    };
    let mantissa = global_number(1, mantissa);
    let _ = result.tuple_set(0, mantissa);
    let exponent = global_integer(1, i64::from(exponent));
    let _ = result.tuple_set(1, exponent);
    return_value(result)
}

unsafe extern "C" fn ldexp(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    let Some(exponent) = arguments.get(1).and_then(Value::integer) else {
        return type_error(c"expected int");
    };
    let Ok(exponent) = i32::try_from(exponent) else {
        return value_error(c"math range error");
    };
    return_number(value * 2.0_f64.powi(exponent))
}

unsafe extern "C" fn expm1(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::exp_m1)
}

unsafe extern "C" fn log1p(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(value) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    if value <= -1.0 {
        return value_error(c"math domain error");
    }
    return_number(value.ln_1p())
}

unsafe extern "C" fn hypot(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Ok(x) = arguments.get(0).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    let Ok(y) = arguments.get(1).ok_or(()).and_then(|v| v.cast_number()) else {
        return type_error(c"expected number");
    };
    return_number(x.hypot(y))
}

unsafe extern "C" fn cbrt(argc: c_int, argv: ffi::py_StackRef) -> bool {
    unary(argc, argv, f64::cbrt)
}
