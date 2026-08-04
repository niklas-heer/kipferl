mod ansi;
mod ansi_core;
mod args;
mod base64;
mod binascii;
mod bytearray;
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
mod heapq;
mod input;
mod itertools;
mod json;
mod operator;
mod random;
mod statistics;
mod statistics_core;
mod string_methods;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;
mod typing;
mod uuid;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    args::MODULE,
    base64::MODULE,
    binascii::MODULE,
    charm::MODULE,
    copy::MODULE,
    typing::MODULE,
    collections::MODULE,
    csv::MODULE,
    dataclasses::MODULE,
    datetime::MODULE,
    errno::MODULE,
    fnmatch::MODULE,
    functools::MODULE,
    heapq::MODULE,
    input::MODULE,
    itertools::MODULE,
    json::MODULE,
    operator::MODULE,
    random::MODULE,
    statistics::MODULE,
    term::MODULE,
    textwrap::MODULE,
    uuid::MODULE,
];

pub(crate) fn register_all() {
    string_methods::register();
    bytearray::register();
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    input::shutdown();
    term::shutdown();
}
