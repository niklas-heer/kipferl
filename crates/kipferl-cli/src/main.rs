use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut process_arguments = env::args_os();
    let executable = process_arguments.next();
    if invoked_as_legacy_ucharm(executable.as_deref()) {
        eprintln!(
            "warning: `ucharm` was renamed to `kipferl`; the compatibility alias will be removed after the 0.6 release"
        );
    }

    let arguments = match process_arguments
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

fn invoked_as_legacy_ucharm(executable: Option<&OsStr>) -> bool {
    executable
        .and_then(|value| Path::new(value).file_name())
        .is_some_and(|name| name == "ucharm")
}

#[cfg(test)]
mod tests {
    use super::invoked_as_legacy_ucharm;
    use std::ffi::OsStr;

    #[test]
    fn recognizes_only_the_legacy_command_name() {
        assert!(invoked_as_legacy_ucharm(Some(OsStr::new(
            "/usr/local/bin/ucharm"
        ))));
        assert!(!invoked_as_legacy_ucharm(Some(OsStr::new(
            "/usr/local/bin/kipferl"
        ))));
        assert!(!invoked_as_legacy_ucharm(None));
    }
}
