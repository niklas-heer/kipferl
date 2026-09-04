mod ansi;
mod ansi_core;
mod argparse;
mod args;
mod array;
mod base64;
mod binascii;
mod bytearray;
mod bytes_methods;
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
#[cfg(feature = "archives")]
mod gzip;
#[cfg(feature = "crypto")]
mod hash_core;
#[cfg(feature = "crypto")]
mod hashlib;
mod heapq;
#[cfg(feature = "crypto")]
mod hmac;
#[cfg(feature = "http")]
mod http_client;
#[cfg(feature = "interactive")]
mod input;
mod io;
mod itertools;
mod json;
#[cfg(feature = "formats")]
mod kdl;
mod logging;
mod math;
mod operator;
mod os;
mod os_path;
mod pathlib;
mod random;
#[cfg(feature = "regex")]
mod re;
mod secrets;
#[cfg(feature = "interactive")]
mod selection_tui;
mod shutil;
mod signal;
#[cfg(feature = "sqlite")]
mod sqlite3;
mod statistics;
mod statistics_core;
mod string_methods;
mod struct_module;
mod subprocess;
mod sys;
#[cfg(feature = "archives")]
mod tarfile;
mod tempfile;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;
#[cfg(feature = "timezone")]
mod time;
#[cfg(feature = "formats")]
mod toml;
#[cfg(feature = "formats")]
mod toml_core;
#[cfg(feature = "formats")]
mod tomllib;
mod tui;
mod tui_core;
mod typing;
mod unittest;
mod urllib_parse;
mod uuid;
mod xml_etree;
#[cfg(feature = "formats")]
mod yaml;
#[cfg(feature = "archives")]
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
    tui::MODULE,
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
    #[cfg(feature = "archives")]
    gzip::MODULE,
    #[cfg(feature = "crypto")]
    hashlib::MODULE,
    heapq::MODULE,
    #[cfg(feature = "crypto")]
    hmac::MODULE,
    #[cfg(feature = "http")]
    http_client::MODULE,
    #[cfg(feature = "interactive")]
    input::MODULE,
    itertools::MODULE,
    json::MODULE,
    #[cfg(feature = "formats")]
    kdl::MODULE,
    logging::MODULE,
    math::MODULE,
    operator::MODULE,
    random::MODULE,
    #[cfg(feature = "regex")]
    re::MODULE,
    secrets::MODULE,
    signal::MODULE,
    #[cfg(feature = "sqlite")]
    sqlite3::MODULE,
    statistics::MODULE,
    shutil::MODULE,
    #[cfg(feature = "archives")]
    tarfile::MODULE,
    tempfile::MODULE,
    term::MODULE,
    textwrap::MODULE,
    #[cfg(feature = "timezone")]
    time::MODULE,
    #[cfg(feature = "formats")]
    toml::MODULE,
    #[cfg(feature = "formats")]
    tomllib::MODULE,
    subprocess::MODULE,
    sys::MODULE,
    unittest::MODULE,
    urllib_parse::MODULE,
    uuid::MODULE,
    xml_etree::MODULE,
    #[cfg(feature = "formats")]
    yaml::MODULE,
    #[cfg(feature = "archives")]
    zipfile::MODULE,
];

pub fn register_all() {
    string_methods::register();
    bytes_methods::register();
    bytearray::register();
    register_modules(MODULES);
}

pub fn shutdown_all() {
    #[cfg(feature = "interactive")]
    input::shutdown();
    term::shutdown();
}
