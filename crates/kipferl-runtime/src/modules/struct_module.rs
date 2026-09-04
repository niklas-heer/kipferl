use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, NativeTypeAlias, Value,
    execute_module, global_integer, global_number, global_tuple, return_bytes, return_value,
    runtime_error, type_error, value_error,
};

#[derive(Clone, Copy)]
enum Endian {
    Little,
    Big,
}

#[derive(Clone, Copy)]
enum Code {
    SignedByte,
    UnsignedByte,
    SignedShort,
    UnsignedShort,
    SignedInt,
    UnsignedInt,
    SignedLong,
    UnsignedLong,
    SignedLongLong,
    UnsignedLongLong,
    Float,
    Double,
    Padding,
}

#[derive(Clone, Copy)]
struct Item {
    code: Code,
    count: usize,
}

struct Format {
    endian: Endian,
    items: Vec<Item>,
    value_count: usize,
    size: usize,
}

enum Decoded {
    Integer(i64),
    Number(f64),
}

const SIGNATURES: &[NativeSignature] = &[
    NativeSignature {
        signature: c"pack(format, *values)",
        callback: pack,
    },
    NativeSignature {
        signature: c"unpack(format, buffer)",
        callback: unpack,
    },
    NativeSignature {
        signature: c"calcsize(format)",
        callback: calcsize,
    },
];

const TYPE_ALIASES: &[NativeTypeAlias] = &[NativeTypeAlias {
    name: c"error",
    value_type: ffi::py_PredefinedType_tp_ValueError,
}];

const COMPATIBILITY_SOURCE: &str = r#"
class Struct:
    def __init__(self, format):
        self.format = format
        self.size = calcsize(format)

    def pack(self, *values):
        return pack(self.format, *values)

    def unpack(self, buffer):
        return unpack(self.format, buffer)


def pack_into(format, buffer, offset, *values):
    encoded = pack(format, *values)
    if offset < 0:
        offset += len(buffer)
    if offset < 0 or offset + len(encoded) > len(buffer):
        raise error("pack_into requires a buffer of sufficient size")
    for index in range(len(encoded)):
        buffer[offset + index] = encoded[index]


def unpack_from(format, buffer, offset=0):
    size = calcsize(format)
    if offset < 0:
        offset += len(buffer)
    if offset < 0 or offset + size > len(buffer):
        raise error("unpack_from requires a buffer of sufficient size")
    return unpack(format, buffer[offset:offset + size])
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"struct",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: TYPE_ALIASES,
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded struct compatibility layer failed"
    );
}

unsafe extern "C" fn calcsize(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(format) = format_argument(&arguments) else {
        return false;
    };
    let Ok(size) = i64::try_from(format.size) else {
        return value_error(c"total struct size too long");
    };
    let value = global_integer(6, size);
    return_value(value)
}

#[expect(
    clippy::expect_used,
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "parse_format uses checked size/count accumulation; the exact argument count is validated and every output span follows the validated format sizes."
)]
unsafe extern "C" fn pack(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(format) = format_argument(&arguments) else {
        return false;
    };
    let Some(packed_values) = arguments.get(1) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };
    let Some(value_count) = packed_values.tuple_len() else {
        return type_error(c"pack values must be a tuple");
    };
    if value_count < format.value_count {
        return value_error(c"pack expected more items");
    }
    if value_count > format.value_count {
        return value_error(c"pack expected fewer items");
    }

    let mut output = Vec::new();
    if output.try_reserve_exact(format.size).is_err() {
        return runtime_error(c"unable to allocate packed result");
    }
    output.resize(format.size, 0);
    let mut offset = 0;
    let mut argument_index = 0;
    for item in &format.items {
        for _ in 0..item.count {
            let size = code_size(item.code);
            if matches!(item.code, Code::Padding) {
                offset += size;
                continue;
            }
            let value = packed_values
                .tuple_item(argument_index)
                .expect("value count checked");
            if !pack_value(
                item.code,
                format.endian,
                &mut output[offset..offset + size],
                value,
            ) {
                return false;
            }
            argument_index += 1;
            offset += size;
        }
    }
    return_bytes(&output)
}

#[expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "Input length equals the checked format size; each successive span is exactly the corresponding format item size."
)]
unsafe extern "C" fn unpack(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(format) = format_argument(&arguments) else {
        return false;
    };
    let Some(input) = arguments.get(1).and_then(Value::bytes) else {
        return type_error(c"a bytes-like object is required");
    };
    if input.len() != format.size {
        return value_error(c"unpack requires a buffer of the exact size");
    }

    let mut values = Vec::new();
    if values.try_reserve_exact(format.value_count).is_err() {
        return runtime_error(c"unable to allocate unpacked result");
    }
    let mut offset = 0;
    for item in &format.items {
        for _ in 0..item.count {
            let size = code_size(item.code);
            if !matches!(item.code, Code::Padding) {
                values.push(unpack_value(
                    item.code,
                    format.endian,
                    &input[offset..offset + size],
                ));
            }
            offset += size;
        }
    }

    let Some(tuple) = global_tuple(7, values.len()) else {
        return value_error(c"unpacked tuple is too large");
    };
    for (index, value) in values.into_iter().enumerate() {
        let value = match value {
            Decoded::Integer(value) => global_integer(6, value),
            Decoded::Number(value) => global_number(6, value),
        };
        assert!(tuple.tuple_set(index, value), "new tuple index is valid");
    }
    return_value(tuple)
}

fn format_argument(arguments: &Arguments) -> Option<Format> {
    let Some(source) = arguments.get(0).and_then(Value::string) else {
        type_error(c"format must be a string");
        return None;
    };
    parse_format(source.as_bytes()).map_or_else(
        || {
            value_error(c"bad char in struct format");
            None
        },
        Some,
    )
}

#[expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "The parser guards source indexes and ASCII digit subtraction; dynamic repeat counts and total sizes use checked arithmetic."
)]
fn parse_format(source: &[u8]) -> Option<Format> {
    let mut index = 0;
    let endian = source
        .first()
        .map_or_else(native_endian, |prefix| match prefix {
            b'<' => {
                index = 1;
                Endian::Little
            }
            b'>' | b'!' => {
                index = 1;
                Endian::Big
            }
            b'@' | b'=' => {
                index = 1;
                native_endian()
            }
            _ => native_endian(),
        });

    let mut items = Vec::new();
    let mut value_count = 0_usize;
    let mut size = 0_usize;
    while index < source.len() {
        let mut count = 0_usize;
        while index < source.len() && source[index].is_ascii_digit() {
            count = count
                .checked_mul(10)?
                .checked_add(usize::from(source[index] - b'0'))?;
            index += 1;
        }
        if count == 0 {
            count = 1;
        }
        let code = parse_code(*source.get(index)?)?;
        index += 1;
        size = size.checked_add(code_size(code).checked_mul(count)?)?;
        if !matches!(code, Code::Padding) {
            value_count = value_count.checked_add(count)?;
        }
        items.push(Item { code, count });
    }
    Some(Format {
        endian,
        items,
        value_count,
        size,
    })
}

const fn parse_code(code: u8) -> Option<Code> {
    Some(match code {
        b'b' => Code::SignedByte,
        b'B' => Code::UnsignedByte,
        b'h' => Code::SignedShort,
        b'H' => Code::UnsignedShort,
        b'i' => Code::SignedInt,
        b'I' => Code::UnsignedInt,
        b'l' => Code::SignedLong,
        b'L' => Code::UnsignedLong,
        b'q' => Code::SignedLongLong,
        b'Q' => Code::UnsignedLongLong,
        b'f' => Code::Float,
        b'd' => Code::Double,
        b'x' => Code::Padding,
        _ => return None,
    })
}

const fn native_endian() -> Endian {
    if cfg!(target_endian = "little") {
        Endian::Little
    } else {
        Endian::Big
    }
}

const fn code_size(code: Code) -> usize {
    match code {
        Code::SignedByte | Code::UnsignedByte | Code::Padding => 1,
        Code::SignedShort | Code::UnsignedShort => 2,
        Code::SignedInt | Code::UnsignedInt | Code::Float => 4,
        Code::SignedLong
        | Code::UnsignedLong
        | Code::SignedLongLong
        | Code::UnsignedLongLong
        | Code::Double => 8,
    }
}

#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "The f32 format explicitly requires IEEE-754 rounding from Python f64; narrowing is part of the struct format contract."
)]
fn pack_value(code: Code, endian: Endian, output: &mut [u8], value: Value) -> bool {
    match code {
        Code::SignedByte => integer_bytes::<i8>(value, output, endian),
        Code::UnsignedByte => integer_bytes::<u8>(value, output, endian),
        Code::SignedShort => integer_bytes::<i16>(value, output, endian),
        Code::UnsignedShort => integer_bytes::<u16>(value, output, endian),
        Code::SignedInt => integer_bytes::<i32>(value, output, endian),
        Code::UnsignedInt => integer_bytes::<u32>(value, output, endian),
        Code::SignedLong | Code::SignedLongLong => integer_bytes::<i64>(value, output, endian),
        Code::UnsignedLong | Code::UnsignedLongLong => integer_bytes::<u64>(value, output, endian),
        Code::Float => {
            let Ok(value) = value.cast_number() else {
                return false;
            };
            let narrowed = value as f32;
            if value.is_finite() && !narrowed.is_finite() {
                return value_error(c"float is too large for the f32 struct format");
            }
            write_bytes(output, narrowed.to_bits(), endian);
            true
        }
        Code::Double => {
            let Ok(value) = value.cast_number() else {
                return false;
            };
            write_bytes(output, value.to_bits(), endian);
            true
        }
        Code::Padding => true,
    }
}

trait PackedInteger: TryFrom<i64> + Copy {
    fn write(self, output: &mut [u8], endian: Endian);
}

macro_rules! packed_integer {
    ($($type:ty),* $(,)?) => {$(
        impl PackedInteger for $type {
            fn write(self, output: &mut [u8], endian: Endian) {
                match endian {
                    Endian::Little => output.copy_from_slice(&self.to_le_bytes()),
                    Endian::Big => output.copy_from_slice(&self.to_be_bytes()),
                }
            }
        }
    )*};
}

packed_integer!(i8, u8, i16, u16, i32, u32, i64, u64);

fn integer_bytes<T: PackedInteger>(value: Value, output: &mut [u8], endian: Endian) -> bool {
    let Ok(value) = value.cast_integer() else {
        return false;
    };
    let Ok(value) = T::try_from(value) else {
        return value_error(c"argument out of range");
    };
    value.write(output, endian);
    true
}

fn write_bytes<const N: usize>(output: &mut [u8], value: impl IntoBytes<N>, endian: Endian) {
    let bytes = match endian {
        Endian::Little => value.little(),
        Endian::Big => value.big(),
    };
    output.copy_from_slice(&bytes);
}

trait IntoBytes<const N: usize> {
    fn little(self) -> [u8; N];
    fn big(self) -> [u8; N];
}

impl IntoBytes<4> for u32 {
    fn little(self) -> [u8; 4] {
        self.to_le_bytes()
    }
    fn big(self) -> [u8; 4] {
        self.to_be_bytes()
    }
}

impl IntoBytes<8> for u64 {
    fn little(self) -> [u8; 8] {
        self.to_le_bytes()
    }
    fn big(self) -> [u8; 8] {
        self.to_be_bytes()
    }
}

#[expect(
    clippy::indexing_slicing,
    clippy::unreachable,
    reason = "The caller passes exactly code_size bytes and skips padding codes, so first-byte access is valid and the padding arm is impossible."
)]
fn unpack_value(code: Code, endian: Endian, input: &[u8]) -> Decoded {
    match code {
        Code::SignedByte => Decoded::Integer(i64::from(i8::from_ne_bytes([input[0]]))),
        Code::UnsignedByte => Decoded::Integer(i64::from(input[0])),
        Code::SignedShort => Decoded::Integer(i64::from(read_i16(input, endian))),
        Code::UnsignedShort => Decoded::Integer(i64::from(read_u16(input, endian))),
        Code::SignedInt => Decoded::Integer(i64::from(read_i32(input, endian))),
        Code::UnsignedInt => Decoded::Integer(i64::from(read_u32(input, endian))),
        Code::SignedLong | Code::SignedLongLong => Decoded::Integer(read_i64(input, endian)),
        Code::UnsignedLong | Code::UnsignedLongLong => {
            Decoded::Integer(i64::from_ne_bytes(read_u64(input, endian).to_ne_bytes()))
        }
        Code::Float => Decoded::Number(f64::from(f32::from_bits(read_u32(input, endian)))),
        Code::Double => Decoded::Number(f64::from_bits(read_u64(input, endian))),
        Code::Padding => unreachable!("padding produces no value"),
    }
}

macro_rules! read_integer {
    ($name:ident, $type:ty, $size:expr) => {
        fn $name(input: &[u8], endian: Endian) -> $type {
            let bytes: [u8; $size] = input.try_into().expect("format size is exact");
            match endian {
                Endian::Little => <$type>::from_le_bytes(bytes),
                Endian::Big => <$type>::from_be_bytes(bytes),
            }
        }
    };
}

read_integer!(read_i16, i16, 2);
read_integer!(read_u16, u16, 2);
read_integer!(read_i32, i32, 4);
read_integer!(read_u32, u32, 4);
read_integer!(read_i64, i64, 8);
read_integer!(read_u64, u64, 8);
