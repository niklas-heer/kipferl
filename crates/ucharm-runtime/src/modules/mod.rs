mod ansi;
mod ansi_core;

use crate::native::{NativeModule, register_modules};

const MODULES: &[NativeModule] = &[ansi::MODULE];

pub(crate) fn register_all() {
    register_modules(MODULES);
}
