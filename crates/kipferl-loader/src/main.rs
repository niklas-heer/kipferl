#![deny(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unreachable,
    clippy::string_slice,
    clippy::panic_in_result_fn,
    clippy::panic,
    clippy::exit,
    clippy::as_conversions
)]

use std::env;
use std::process::{Command, ExitCode};

use kipferl_loader::{LoaderError, prepare_path};
use std::os::unix::process::CommandExt;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kipferl: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), LoaderError> {
    let executable = env::current_exe()?;
    let cache_root = env::var_os("KIPFERL_CACHE_DIR")
        .or_else(|| env::var_os("UCHARM_CACHE_DIR"))
        .map_or_else(env::temp_dir, Into::into);
    let prepared = prepare_path(&executable, &cache_root)?;

    let error = Command::new(&prepared.runtime_path)
        .arg(&prepared.python_path)
        .args(env::args_os().skip(1))
        .exec();
    Err(error.into())
}
