use std::ffi::c_int;
use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::{DeflateDecoder, GzDecoder};
use flate2::write::GzEncoder;
use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, Value, return_bytes, runtime_error,
    type_error, value_error,
};

const MAX_DATA_SIZE: usize = 64 * 1024 * 1024;

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"compress",
        callback: compress,
    },
    NativeFunction {
        name: c"decompress",
        callback: decompress,
    },
    NativeFunction {
        name: c"_inflate_raw",
        callback: inflate_raw,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"gzip",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn compress(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = bytes_argument(argc, argv) else {
        return false;
    };
    if input.len() > MAX_DATA_SIZE {
        return value_error(c"data too large");
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if encoder.write_all(&input).is_err() {
        return runtime_error(c"gzip compression failed");
    }
    match encoder.finish() {
        Ok(output) => return_bytes(&output),
        Err(_) => runtime_error(c"gzip compression failed"),
    }
}

unsafe extern "C" fn decompress(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = bytes_argument(argc, argv) else {
        return false;
    };
    decode_limited(GzDecoder::new(input.as_slice()))
}

unsafe extern "C" fn inflate_raw(argc: c_int, argv: ffi::py_StackRef) -> bool {
    let Some(input) = bytes_argument(argc, argv) else {
        return false;
    };
    decode_limited(DeflateDecoder::new(input.as_slice()))
}

fn decode_limited(mut decoder: impl Read) -> bool {
    let mut output = Vec::new();
    match decoder
        .by_ref()
        .take(u64::try_from(MAX_DATA_SIZE + 1).expect("fixed limit fits u64"))
        .read_to_end(&mut output)
    {
        Ok(_) if output.len() <= MAX_DATA_SIZE => return_bytes(&output),
        Ok(_) => value_error(c"decompressed data too large"),
        Err(_) => value_error(c"invalid compressed data"),
    }
}

fn bytes_argument(argc: c_int, argv: ffi::py_StackRef) -> Option<Vec<u8>> {
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return None;
    }
    let Some(input) = arguments.get(0).and_then(Value::bytes) else {
        type_error(c"a bytes-like object is required");
        return None;
    };
    Some(input)
}
