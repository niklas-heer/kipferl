mod ansi;
mod ansi_core;
mod args;
mod base64;
mod binascii;
mod charm;
mod charm_core;
mod encoding_core;
mod errno;
mod fnmatch;
mod fnmatch_core;
mod heapq;
mod input;
mod itertools;
mod statistics;
mod statistics_core;
mod string_methods;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;
mod typing;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    args::MODULE,
    base64::MODULE,
    binascii::MODULE,
    charm::MODULE,
    errno::MODULE,
    fnmatch::MODULE,
    heapq::MODULE,
    input::MODULE,
    itertools::MODULE,
    statistics::MODULE,
    term::MODULE,
    textwrap::MODULE,
    typing::MODULE,
];

pub(crate) fn register_all() {
    string_methods::register();
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    input::shutdown();
    term::shutdown();
}
