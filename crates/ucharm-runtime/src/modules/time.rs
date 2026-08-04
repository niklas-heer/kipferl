use std::ffi::{CStr, CString, c_int};
use std::sync::OnceLock;
use std::time::Instant;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, global_integer,
    global_tuple, return_number, return_string_bytes, return_value, runtime_error, type_error,
    value_error,
};

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"localtime(seconds=None)",
        callback: localtime,
    },
    NativeSignature {
        signature: c"gmtime(seconds=None)",
        callback: gmtime,
    },
    NativeSignature {
        signature: c"mktime(t)",
        callback: mktime,
    },
    NativeSignature {
        signature: c"strftime(format, t=None)",
        callback: strftime,
    },
    NativeSignature {
        signature: c"strptime(string, format)",
        callback: strptime,
    },
    NativeSignature {
        signature: c"monotonic()",
        callback: monotonic,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"time",
    kind: NativeModuleKind::Extend,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

fn timestamp(value: Option<Value>) -> Result<libc::time_t, &'static CStr> {
    if value.is_none_or(Value::is_none) {
        // SAFETY: a null output pointer asks libc to return the current time.
        return Ok(unsafe { libc::time(std::ptr::null_mut()) });
    }
    let seconds = value
        .expect("checked Some above")
        .cast_number()
        .map_err(|()| c"timestamp must be a number")?;
    if !seconds.is_finite() {
        return Err(c"timestamp out of range");
    }
    Ok(seconds as libc::time_t)
}

fn converted_time(
    value: Option<Value>,
    convert: unsafe extern "C" fn(*const libc::time_t, *mut libc::tm) -> *mut libc::tm,
) -> bool {
    let time = match timestamp(value) {
        Ok(value) => value,
        Err(message) => return type_error(message),
    };
    // SAFETY: libc::tm is a plain C value and zero is a valid initial state.
    let mut result: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: both pointers are valid for the synchronous libc conversion.
    if unsafe { convert(&time, &mut result) }.is_null() {
        return value_error(c"invalid time");
    }
    return_tm(&result)
}

unsafe extern "C" fn localtime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    converted_time(arguments.get(0), libc::localtime_r)
}

unsafe extern "C" fn gmtime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    converted_time(arguments.get(0), libc::gmtime_r)
}

fn return_tm(value: &libc::tm) -> bool {
    let values = [
        i64::from(value.tm_year) + 1900,
        i64::from(value.tm_mon) + 1,
        i64::from(value.tm_mday),
        i64::from(value.tm_hour),
        i64::from(value.tm_min),
        i64::from(value.tm_sec),
        i64::from((value.tm_wday + 6).rem_euclid(7)),
        i64::from(value.tm_yday) + 1,
        i64::from(value.tm_isdst),
    ];
    let Some(tuple) = global_tuple(0, values.len()) else {
        return runtime_error(c"failed to create struct_time");
    };
    for (index, value) in values.into_iter().enumerate() {
        let item = global_integer(1, value);
        let _ = tuple.tuple_set(index, item);
    }
    return_value(tuple)
}

fn tm_from_value(value: Value) -> Result<libc::tm, &'static CStr> {
    if value.tuple_len().is_none_or(|length| length < 9) {
        return Err(c"time tuple must have at least 9 elements");
    }
    let mut fields = [0_i64; 9];
    for (index, field) in fields.iter_mut().enumerate() {
        *field = value
            .tuple_item(index)
            .and_then(Value::integer)
            .ok_or(c"time tuple fields must be integers")?;
    }
    // SAFETY: libc::tm is plain C storage initialized field by field below.
    let mut result: libc::tm = unsafe { std::mem::zeroed() };
    result.tm_year = c_int::try_from(fields[0] - 1900).map_err(|_| c"year out of range")?;
    result.tm_mon = c_int::try_from(fields[1] - 1).map_err(|_| c"month out of range")?;
    result.tm_mday = c_int::try_from(fields[2]).map_err(|_| c"day out of range")?;
    result.tm_hour = c_int::try_from(fields[3]).map_err(|_| c"hour out of range")?;
    result.tm_min = c_int::try_from(fields[4]).map_err(|_| c"minute out of range")?;
    result.tm_sec = c_int::try_from(fields[5]).map_err(|_| c"second out of range")?;
    let weekday = c_int::try_from(fields[6]).map_err(|_| c"weekday out of range")?;
    result.tm_wday = (weekday + 1).rem_euclid(7);
    result.tm_yday = c_int::try_from(fields[7] - 1).map_err(|_| c"year day out of range")?;
    result.tm_isdst = c_int::try_from(fields[8]).map_err(|_| c"DST flag out of range")?;
    Ok(result)
}

unsafe extern "C" fn mktime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(value) = arguments.get(0) else {
        return type_error(c"mktime() requires a time tuple");
    };
    let mut value = match tm_from_value(value) {
        Ok(value) => value,
        Err(message) => return type_error(message),
    };
    // SAFETY: the value is a fully initialized libc::tm.
    let result = unsafe { libc::mktime(&mut value) };
    return_number(result as f64)
}

unsafe extern "C" fn strftime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(format) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"strftime() format must be a string");
    };
    let format = match CString::new(format) {
        Ok(value) => value,
        Err(_) => return value_error(c"format contains NUL"),
    };
    let time = if arguments.get(1).is_none_or(Value::is_none) {
        // SAFETY: a null output pointer asks libc to return the current time.
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        // SAFETY: libc::tm is plain C storage initialized by localtime_r.
        let mut value: libc::tm = unsafe { std::mem::zeroed() };
        // SAFETY: both pointers are valid for the synchronous conversion.
        if unsafe { libc::localtime_r(&now, &mut value) }.is_null() {
            return value_error(c"invalid time");
        }
        value
    } else {
        match tm_from_value(arguments.get(1).expect("checked Some above")) {
            Ok(value) => value,
            Err(message) => return type_error(message),
        }
    };
    let mut buffer = vec![0_i8; 4096];
    // SAFETY: buffer is writable, format is NUL terminated, and time is initialized.
    let length =
        unsafe { libc::strftime(buffer.as_mut_ptr(), buffer.len(), format.as_ptr(), &time) };
    if length == 0 {
        return_string_bytes(&[])
    } else {
        // SAFETY: libc initialized exactly length bytes in the output buffer.
        let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), length) };
        return_string_bytes(bytes)
    }
}

unsafe extern "C" fn strptime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(input) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"strptime() input must be a string");
    };
    let Some(format) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"strptime() format must be a string");
    };
    let input = match CString::new(input) {
        Ok(value) => value,
        Err(_) => return value_error(c"input contains NUL"),
    };
    let format = match CString::new(format) {
        Ok(value) => value,
        Err(_) => return value_error(c"format contains NUL"),
    };
    // SAFETY: libc::tm is plain C storage initialized by strptime.
    let mut result: libc::tm = unsafe { std::mem::zeroed() };
    result.tm_isdst = -1;
    // SAFETY: both inputs are NUL terminated and result is writable.
    if unsafe { libc::strptime(input.as_ptr(), format.as_ptr(), &mut result) }.is_null() {
        return value_error(c"time data does not match format");
    }
    return_tm(&result)
}

unsafe extern "C" fn monotonic(_argc: c_int, _argv: ffi::py_StackRef) -> bool {
    static START: OnceLock<Instant> = OnceLock::new();
    let elapsed = START.get_or_init(Instant::now).elapsed().as_secs_f64();
    return_number(elapsed)
}
