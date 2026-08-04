mod ansi;
mod ansi_core;
mod args;
mod array;
mod base64;
mod binascii;
mod bytearray;
mod bytes_methods;
mod charm;
mod charm_core;
mod collections;
mod copy;
mod csv;
mod dataclasses;
mod datetime;
mod encoding_core;
mod errno;
mod fnmatch;
mod fnmatch_core;
mod functools;
mod gzip;
mod hash_core;
mod hashlib;
mod heapq;
mod hmac;
mod input;
mod io;
mod itertools;
mod json;
mod operator;
mod random;
mod secrets;
mod statistics;
mod statistics_core;
mod string_methods;
mod struct_module;
mod tarfile;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;
mod typing;
mod uuid;
mod zipfile;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    struct_module::MODULE,
    array::MODULE,
    args::MODULE,
    base64::MODULE,
    binascii::MODULE,
    charm::MODULE,
    copy::MODULE,
    typing::MODULE,
    collections::MODULE,
    io::MODULE,
    csv::MODULE,
    dataclasses::MODULE,
    datetime::MODULE,
    errno::MODULE,
    fnmatch::MODULE,
    functools::MODULE,
    gzip::MODULE,
    hashlib::MODULE,
    heapq::MODULE,
    hmac::MODULE,
    input::MODULE,
    itertools::MODULE,
    json::MODULE,
    operator::MODULE,
    random::MODULE,
    secrets::MODULE,
    statistics::MODULE,
    tarfile::MODULE,
    term::MODULE,
    textwrap::MODULE,
    uuid::MODULE,
    zipfile::MODULE,
];

pub(crate) fn register_all() {
    string_methods::register();
    bytes_methods::register();
    bytearray::register();
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    input::shutdown();
    term::shutdown();
}
