#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::exit,
        clippy::panic_in_result_fn
    )
)]

use std::env;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = match env::args_os()
        .skip(1)
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "arguments must be valid UTF-8")
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::FAILURE;
        }
    };
    let current_directory = match env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!("error: cannot determine current directory: {error}");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let stderr = io::stderr();
    match kipferl_cli::run(
        &arguments,
        &current_directory,
        &mut stdout.lock(),
        &mut stderr.lock(),
    ) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
