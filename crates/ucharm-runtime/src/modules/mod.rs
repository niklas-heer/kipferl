mod ansi;
mod ansi_core;
mod args;
mod term;
mod term_core;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[ansi::MODULE, args::MODULE, term::MODULE];

pub(crate) fn register_all() {
    register_modules(MODULES);
}

pub(crate) fn shutdown_all() {
    term::shutdown();
}
