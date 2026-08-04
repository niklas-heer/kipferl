mod ansi;
mod ansi_core;
mod args;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[ansi::MODULE, args::MODULE];

pub(crate) fn register_all() {
    register_modules(MODULES);
}
