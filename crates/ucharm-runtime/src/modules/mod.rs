mod ansi;
mod ansi_core;
mod args;
mod base64;
mod binascii;
mod charm;
mod charm_core;
mod encoding_core;
mod fnmatch;
mod fnmatch_core;
mod input;
mod statistics;
mod statistics_core;
mod term;
mod term_core;
mod textwrap;
mod textwrap_core;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    args::MODULE,
    base64::MODULE,
    binascii::MODULE,
    charm::MODULE,
    fnmatch::MODULE,
    input::MODULE,
    statistics::MODULE,
    term::MODULE,
    textwrap::MODULE,
];

pub(crate) fn register_all() {
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    input::shutdown();
    term::shutdown();
}
