mod ansi;
mod ansi_core;
mod args;
mod charm;
mod charm_core;
mod input;
mod term;
mod term_core;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[
    ansi::MODULE,
    args::MODULE,
    charm::MODULE,
    input::MODULE,
    term::MODULE,
];

pub(crate) fn register_all() {
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    input::shutdown();
    term::shutdown();
}
