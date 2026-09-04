use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, execute_module,
    global_integer, global_number, global_tuple, return_number, return_value, type_error,
    value_error,
};

unsafe extern "C" {
    #[link_name = "frexp"]
    fn c_frexp(value: f64, exponent: *mut c_int) -> f64;
    #[link_name = "ldexp"]
    fn c_ldexp(value: f64, exponent: c_int) -> f64;
}

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

fn unary(argc: c_int, stack: ffi::py_StackRef, operation: impl FnOnce(f64) -> f64) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    return_number(operation(value))
}

unsafe extern "C" fn sinh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::sinh)
}

unsafe extern "C" fn cosh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::cosh)
}

unsafe extern "C" fn tanh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::tanh)
}

unsafe extern "C" fn asinh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::asinh)
}

unsafe extern "C" fn acosh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    if value < 1.0 {
        return value_error(c"math domain error");
    }
    return_number(value.acosh())
}

unsafe extern "C" fn atanh(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    if value.abs() >= 1.0 {
        return value_error(c"math domain error");
    }
    return_number(value.atanh())
}

unsafe extern "C" fn frexp(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    let (mantissa, exponent) = decompose(value);
    let Some(result) = global_tuple(0, 2) else {
        return value_error(c"failed to create frexp result");
    };
    let mantissa = global_number(1, mantissa);
    let _ = result.tuple_set(0, mantissa);
    let exponent = global_integer(1, i64::from(exponent));
    let _ = result.tuple_set(1, exponent);
    return_value(result)
}

unsafe extern "C" fn ldexp(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    let Some(exponent) = arguments.get(1).and_then(Value::integer) else {
        return type_error(c"expected int");
    };
    scale(value, exponent).map_or_else(|| value_error(c"math range error"), return_number)
}

unsafe extern "C" fn expm1(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::exp_m1)
}

unsafe extern "C" fn log1p(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(value) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    if value <= -1.0 {
        return value_error(c"math domain error");
    }
    return_number(value.ln_1p())
}

unsafe extern "C" fn hypot(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Ok(x) = arguments
        .get(0)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    let Ok(y) = arguments
        .get(1)
        .ok_or(())
        .and_then(super::super::native::Value::cast_number)
    else {
        return type_error(c"expected number");
    };
    return_number(x.hypot(y))
}

unsafe extern "C" fn cbrt(argc: c_int, stack: ffi::py_StackRef) -> bool {
    unary(argc, stack, f64::cbrt)
}

// Unlike log2/powi, libm preserves subnormals and does not first construct an
// overflowing power of two when the final scaled value is representable.
#[deny(clippy::as_conversions, clippy::arithmetic_side_effects)]
fn decompose(value: f64) -> (f64, c_int) {
    if value == 0.0 || !value.is_finite() {
        return (value, 0);
    }
    let mut exponent = 0;
    // SAFETY: frexp accepts every finite double and writes one initialized int
    // through this valid, exclusively borrowed local pointer. It retains none.
    let mantissa = unsafe { c_frexp(value, &raw mut exponent) };
    (mantissa, exponent)
}

#[deny(clippy::as_conversions, clippy::arithmetic_side_effects)]
fn scale(value: f64, exponent: i64) -> Option<f64> {
    if value == 0.0 || !value.is_finite() {
        return Some(value);
    }
    let Ok(exponent) = c_int::try_from(exponent) else {
        return if exponent < 0 {
            Some(0.0_f64.copysign(value))
        } else {
            None
        };
    };
    // SAFETY: ldexp accepts every double and int and owns no external memory.
    let result = unsafe { c_ldexp(value, exponent) };
    result.is_finite().then_some(result)
}

#[cfg(test)]
mod tests {
    use super::{decompose, scale};

    #[test]
    fn preserves_binary_exponents_at_float_boundaries() {
        for value in [
            f64::MAX,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            1.0,
            0.5,
            -0.0,
        ] {
            for value in [value, -value] {
                let (mantissa, exponent) = decompose(value);
                assert_eq!(
                    scale(mantissa, i64::from(exponent)).map(f64::to_bits),
                    Some(value.to_bits())
                );
                if value != 0.0 {
                    assert!((0.5..1.0).contains(&mantissa.abs()));
                }
            }
        }
        assert_eq!(scale(0.5, 1024), Some(2.0_f64.powi(1023)));
        assert_eq!(scale(1.0, i64::MAX), None);
        assert_eq!(
            scale(-1.0, i64::MIN).map(f64::to_bits),
            Some((-0.0_f64).to_bits())
        );
        assert_eq!(scale(f64::INFINITY, i64::MAX), Some(f64::INFINITY));
        assert!(scale(f64::NAN, 0).is_some_and(f64::is_nan));
    }
}
