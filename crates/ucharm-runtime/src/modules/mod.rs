mod ansi;
mod ansi_core;
mod argparse;
mod args;
mod array;
mod base64;
mod binascii;
mod bytearray;
mod bytes_methods;
mod charm;
mod charm_core;
mod collections;
mod configparser;
mod contextlib;
mod copy;
mod csv;
mod dataclasses;
mod datetime;
mod encoding_core;
mod errno;
mod filesystem_core;
mod fnmatch;
mod fnmatch_core;
mod functools;
mod glob;
mod gzip;
mod hash_core;
mod hashlib;
mod heapq;
mod hmac;
mod input;
mod io;
mod itertools;
mod json;
mod logging;
mod math;
mod operator;
mod os;
mod os_path;
mod pathlib;
mod random;
mod re;
mod secrets;
mod shutil;
mod signal;
mod statistics;
mod statistics_core;
mod string_methods;
mod struct_module;
mod subprocess;
mod sys;
mod tarfile;
mod tempfile;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;
mod time;
mod tomllib;
mod typing;
mod unittest;
mod urllib_parse;
mod uuid;
mod xml_etree;
mod zipfile;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    argparse::MODULE,
    struct_module::MODULE,
    array::MODULE,
    args::MODULE,
    base64::MODULE,
    binascii::MODULE,
    charm::MODULE,
    copy::MODULE,
    typing::MODULE,
    collections::MODULE,
    configparser::MODULE,
    contextlib::MODULE,
    io::MODULE,
    csv::MODULE,
    dataclasses::MODULE,
    datetime::MODULE,
    errno::MODULE,
    fnmatch::MODULE,
    os::MODULE,
    os_path::MODULE,
    pathlib::MODULE,
    glob::MODULE,
    functools::MODULE,
    gzip::MODULE,
    hashlib::MODULE,
    heapq::MODULE,
    hmac::MODULE,
    input::MODULE,
    itertools::MODULE,
    json::MODULE,
    logging::MODULE,
    math::MODULE,
    operator::MODULE,
    random::MODULE,
    re::MODULE,
    secrets::MODULE,
    signal::MODULE,
    statistics::MODULE,
    shutil::MODULE,
    tarfile::MODULE,
    tempfile::MODULE,
    term::MODULE,
    textwrap::MODULE,
    time::MODULE,
    tomllib::MODULE,
    subprocess::MODULE,
    sys::MODULE,
    unittest::MODULE,
    urllib_parse::MODULE,
    uuid::MODULE,
    xml_etree::MODULE,
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
