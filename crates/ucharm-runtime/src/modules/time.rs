use std::ffi::{CStr, c_int};
use std::sync::OnceLock;
use std::time::Instant;

use jiff::{Timestamp, Zoned, civil::DateTime, fmt::strtime::BrokenDownTime, tz::TimeZone};
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

fn timestamp(value: Option<Value>) -> Result<Timestamp, &'static CStr> {
    if value.is_none_or(Value::is_none) {
        return Ok(Timestamp::now());
    }
    let seconds = value
        .expect("checked Some above")
        .cast_number()
        .map_err(|()| c"timestamp must be a number")?;
    if !seconds.is_finite() {
        return Err(c"timestamp out of range");
    }
    Timestamp::from_second(seconds as i64).map_err(|_| c"timestamp out of range")
}

unsafe extern "C" fn localtime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let timestamp = match timestamp(arguments.get(0)) {
        Ok(value) => value,
        Err(message) => return type_error(message),
    };
    let time_zone = TimeZone::system();
    let datetime = time_zone.to_datetime(timestamp);
    let is_dst = i64::from(time_zone.to_offset_info(timestamp).dst().is_dst());
    return_datetime(datetime, is_dst)
}

unsafe extern "C" fn gmtime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let timestamp = match timestamp(arguments.get(0)) {
        Ok(value) => value,
        Err(message) => return type_error(message),
    };
    return_datetime(TimeZone::UTC.to_datetime(timestamp), 0)
}

fn return_datetime(value: DateTime, is_dst: i64) -> bool {
    let values = [
        i64::from(value.year()),
        i64::from(value.month()),
        i64::from(value.day()),
        i64::from(value.hour()),
        i64::from(value.minute()),
        i64::from(value.second()),
        i64::from(value.weekday().to_monday_zero_offset()),
        i64::from(value.day_of_year()),
        is_dst,
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

struct TimeFields {
    datetime: DateTime,
    is_dst: i64,
}

fn time_fields(value: Value) -> Result<TimeFields, &'static CStr> {
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
    let datetime = DateTime::new(
        i16::try_from(fields[0]).map_err(|_| c"year out of range")?,
        i8::try_from(fields[1]).map_err(|_| c"month out of range")?,
        i8::try_from(fields[2]).map_err(|_| c"day out of range")?,
        i8::try_from(fields[3]).map_err(|_| c"hour out of range")?,
        i8::try_from(fields[4]).map_err(|_| c"minute out of range")?,
        i8::try_from(fields[5]).map_err(|_| c"second out of range")?,
        0,
    )
    .map_err(|_| c"calendar field out of range")?;
    Ok(TimeFields {
        datetime,
        is_dst: fields[8],
    })
}

fn zoned_is_dst(value: &Zoned) -> bool {
    value
        .time_zone()
        .to_offset_info(value.timestamp())
        .dst()
        .is_dst()
}

fn resolve_local(fields: &TimeFields) -> Result<Zoned, ()> {
    let time_zone = TimeZone::system();
    let ambiguous = time_zone.to_ambiguous_zoned(fields.datetime);
    if fields.is_dst < 0 {
        return ambiguous.compatible().map_err(|_| ());
    }

    let desired_dst = fields.is_dst > 0;
    let earlier = ambiguous.clone().earlier().map_err(|_| ())?;
    if zoned_is_dst(&earlier) == desired_dst {
        return Ok(earlier);
    }
    let later = ambiguous.clone().later().map_err(|_| ())?;
    if zoned_is_dst(&later) == desired_dst {
        return Ok(later);
    }
    ambiguous.compatible().map_err(|_| ())
}

unsafe extern "C" fn mktime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(value) = arguments.get(0) else {
        return type_error(c"mktime() requires a time tuple");
    };
    let fields = match time_fields(value) {
        Ok(value) => value,
        Err(message) => return type_error(message),
    };
    let zoned = match resolve_local(&fields) {
        Ok(value) => value,
        Err(()) => return value_error(c"mktime argument out of range"),
    };
    return_number(zoned.timestamp().as_second() as f64)
}

unsafe extern "C" fn strftime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(format) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"strftime() format must be a string");
    };
    let zoned = if arguments.get(1).is_none_or(Value::is_none) {
        Timestamp::now().to_zoned(TimeZone::system())
    } else {
        let fields = match time_fields(arguments.get(1).expect("checked Some above")) {
            Ok(value) => value,
            Err(message) => return type_error(message),
        };
        match resolve_local(&fields) {
            Ok(value) => value,
            Err(()) => return value_error(c"invalid time"),
        }
    };
    let output = zoned.strftime(&format).to_string();
    return_string_bytes(output.as_bytes())
}

unsafe extern "C" fn strptime(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(input) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"strptime() input must be a string");
    };
    let Some(format) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"strptime() format must be a string");
    };
    let mut parsed = match BrokenDownTime::parse(format.as_bytes(), input.as_bytes()) {
        Ok(value) => value,
        Err(_) => return value_error(c"time data does not match format"),
    };
    if parsed.year().is_none()
        && parsed.iso_week_year().is_none()
        && parsed.set_year(Some(1900)).is_err()
    {
        return value_error(c"time data does not match format");
    }
    let has_alternate_date = parsed.day_of_year().is_some()
        || parsed.iso_week().is_some()
        || parsed.sunday_based_week().is_some()
        || parsed.monday_based_week().is_some();
    if !has_alternate_date {
        if parsed.month().is_none() && parsed.set_month(Some(1)).is_err() {
            return value_error(c"time data does not match format");
        }
        if parsed.day().is_none() && parsed.set_day(Some(1)).is_err() {
            return value_error(c"time data does not match format");
        }
    }
    if parsed.hour().is_none() && parsed.set_hour(Some(0)).is_err() {
        return value_error(c"time data does not match format");
    }
    if parsed.minute().is_none() && parsed.set_minute(Some(0)).is_err() {
        return value_error(c"time data does not match format");
    }
    if parsed.second().is_none() && parsed.set_second(Some(0)).is_err() {
        return value_error(c"time data does not match format");
    }
    let datetime = match parsed.to_datetime() {
        Ok(value) => value,
        Err(_) => return value_error(c"time data does not match format"),
    };
    return_datetime(datetime, -1)
}

unsafe extern "C" fn monotonic(_argc: c_int, _argv: ffi::py_StackRef) -> bool {
    static START: OnceLock<Instant> = OnceLock::new();
    let elapsed = START.get_or_init(Instant::now).elapsed().as_secs_f64();
    return_number(elapsed)
}
