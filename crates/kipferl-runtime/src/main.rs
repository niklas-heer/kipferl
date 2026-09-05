// Native callbacks must report failures through Python or structured Rust
// errors; implicit unwraps and process exits bypass that boundary.
#![deny(clippy::unwrap_used, clippy::exit, clippy::panic_in_result_fn)]

use std::env;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::process::ExitCode;

use kipferl_runtime::{CompileError, ExecuteError, Vm};

const MAX_SYNTAX_FILES: usize = 1024;
const MAX_SYNTAX_BYTES: u64 = 128 * 1024 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args_os()
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "arguments must use UTF-8".to_owned())
        })
        .collect::<Result<_, _>>()?;
    let vm = Vm::initialize().map_err(|error| error.to_string())?;
    if let [_, flag, paths @ ..] = arguments.as_slice()
        && flag == "--check-syntax"
    {
        let paths = if let [separator, remaining @ ..] = paths
            && separator == "--"
        {
            remaining
        } else {
            paths
        };
        return check_syntax(&vm, paths);
    }

    let (source, filename, python_arguments, script_file) = match arguments.as_slice() {
        [program] => {
            return Err(format!("usage: {program} [-c code | script.py] [args...] | --check-syntax [--] file.py [...]"));
        }
        [_, flag, code, rest @ ..] if flag == "-c" => {
            let argv: Vec<String> = std::iter::once("-c")
                .chain(rest.iter().map(String::as_str))
                .map(str::to_owned)
                .collect();
            (code.clone(), "<string>".to_owned(), argv, None)
        }
        [_, flag] if flag == "-c" => return Err("-c requires an argument".to_owned()),
        [_, script, rest @ ..] => {
            let source = fs::read_to_string(script)
                .map_err(|error| format!("cannot read '{script}': {error}"))?;
            let argv = std::iter::once(script.clone())
                .chain(rest.iter().cloned())
                .collect();
            (source, script.clone(), argv, Some(script.clone()))
        }
        [] => return Err("usage: pocketpy-kipferl [-c code | script.py] [args...] | --check-syntax [--] file.py [...]".to_owned()),
    };

    let python_arguments: Vec<CString> = python_arguments
        .iter()
        .map(|argument| CString::new(argument.as_str()))
        .collect::<Result<_, _>>()
        .map_err(|_| "an argument contains a NUL byte".to_owned())?;
    vm.set_argv(&python_arguments)
        .map_err(|error| error.to_string())?;
    if let Some(script_file) = script_file {
        vm.set_file(&script_file)
            .map_err(|error| error.to_string())?;
    }

    match vm.execute_str(&source, &filename) {
        Ok(()) => Ok(()),
        Err(ExecuteError::PythonException) => {
            vm.print_exception();
            Err("Python execution failed".to_owned())
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Read bounded regular UTF-8 files and compile them as modules, without execution.
fn check_syntax(vm: &Vm, paths: &[String]) -> Result<(), String> {
    if paths.is_empty() || paths.len() > MAX_SYNTAX_FILES {
        return Err(
            "usage: pocketpy-kipferl --check-syntax [--] file.py [...] (1–1,024 files)".to_owned(),
        );
    }
    for path in paths {
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
            .map_err(|error| format!("cannot read '{path}': {error}"))?;
        let metadata = file
            .metadata()
            .map_err(|error| format!("cannot inspect '{path}': {error}"))?;
        if !metadata.is_file() || metadata.len() > MAX_SYNTAX_BYTES {
            return Err(format!(
                "'{path}': syntax checking requires a regular file of at most 128 MiB"
            ));
        }
        let mut source = String::new();
        file.take(MAX_SYNTAX_BYTES.saturating_add(1))
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read UTF-8 source '{path}': {error}"))?;
        if u64::try_from(source.len()).map_or(true, |length| length > MAX_SYNTAX_BYTES) {
            return Err(format!("'{path}': syntax source exceeds 128 MiB"));
        }
        match vm.compile_str(&source, path) {
            Ok(()) => {}
            Err(CompileError::PythonException) => {
                vm.print_exception();
                return Err(format!(
                    "Python syntax check failed for '{path}' (no source was executed)"
                ));
            }
            Err(error) => return Err(format!("cannot compile '{path}': {error}")),
        }
    }
    Ok(())
}
