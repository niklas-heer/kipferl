use std::env;
use std::ffi::CString;
use std::fs;
use std::process::ExitCode;

use ucharm_runtime::{ExecuteError, Vm};

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
    let arguments: Vec<String> = env::args().collect();
    let vm = Vm::initialize().map_err(|error| error.to_string())?;

    let (source, filename, python_arguments) = match arguments.as_slice() {
        [program] => {
            return Err(format!("usage: {program} [-c code | script.py] [args...]"));
        }
        [_, flag, code, rest @ ..] if flag == "-c" => {
            let argv: Vec<String> = ["-c"]
                .into_iter()
                .chain(rest.iter().map(String::as_str))
                .map(str::to_owned)
                .collect();
            (code.clone(), "<string>".to_owned(), argv)
        }
        [_, flag] if flag == "-c" => return Err("-c requires an argument".to_owned()),
        [_, script, rest @ ..] => {
            let source = fs::read_to_string(script)
                .map_err(|error| format!("cannot read '{script}': {error}"))?;
            let argv = std::iter::once(script.clone())
                .chain(rest.iter().cloned())
                .collect();
            (source, script.clone(), argv)
        }
        _ => unreachable!("the no-argument case is handled above"),
    };

    let python_arguments: Vec<CString> = python_arguments
        .iter()
        .map(|argument| CString::new(argument.as_str()))
        .collect::<Result<_, _>>()
        .map_err(|_| "an argument contains a NUL byte".to_owned())?;
    vm.set_argv(&python_arguments);

    match vm.execute_str(&source, &filename) {
        Ok(()) => Ok(()),
        Err(ExecuteError::PythonException) => {
            vm.print_exception();
            Err("Python execution failed".to_owned())
        }
        Err(error) => Err(error.to_string()),
    }
}
