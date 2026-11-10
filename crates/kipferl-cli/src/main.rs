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
use std::io::{self, IsTerminal};
use std::process::ExitCode;

use kipferl_cli::color;

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
    let mut styled_stdout = color::Output::new(stdout.lock(), color::enabled(stdout.is_terminal()));
    let mut styled_stderr = color::Output::new(stderr.lock(), color::enabled(stderr.is_terminal()));
    match kipferl_cli::run(
        &arguments,
        &current_directory,
        &mut styled_stdout,
        &mut styled_stderr,
    ) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
