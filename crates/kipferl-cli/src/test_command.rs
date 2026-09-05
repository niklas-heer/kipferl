use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::{project_config, run_command};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    compat: bool,
    report: bool,
    verbose: bool,
    module: Option<String>,
    test_file: Option<String>,
}

pub fn execute(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let options = match parse(arguments, stdout) {
        Ok(Some(options)) => options,
        Ok(None) => return Ok(0),
        Err(error) => {
            writeln!(stderr, "Error: {error}")?;
            return Ok(1);
        }
    };

    if options.compat {
        run_compat_tests(&options, current_directory, stdout)
    } else if let Some(test_file) = options.test_file {
        run_single_test(&test_file, current_directory, stdout)
    } else {
        match run_project_tests(current_directory, stdout) {
            Ok(code) => Ok(code),
            Err(error) => {
                writeln!(stderr, "Error: {error}")?;
                Ok(1)
            }
        }
    }
}

fn parse(arguments: &[String], stdout: &mut dyn Write) -> io::Result<Option<Options>> {
    let mut options = Options::default();
    let mut arguments = arguments.iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--compat" => options.compat = true,
            "--report" | "-r" => options.report = true,
            "--verbose" | "-v" => options.verbose = true,
            "--module" | "-m" => {
                let module = arguments
                    .next()
                    .filter(|v| !v.starts_with('-'))
                    .ok_or_else(|| invalid("--module requires a module name"))?;
                options.module = Some(module.clone());
            }
            "-h" | "--help" => {
                write!(stdout, "{}", help())?;
                return Ok(None);
            }
            argument if !argument.starts_with('-') => {
                if options.test_file.is_some() {
                    return Err(invalid("only one test file or directory may be specified"));
                }
                options.test_file = Some(argument.to_owned());
            }
            option => return Err(invalid(&format!("unknown option '{option}'"))),
        }
    }

    if options.compat && options.test_file.is_some() {
        return Err(invalid("--compat cannot be combined with a test path"));
    }
    if !options.compat && (options.report || options.verbose || options.module.is_some()) {
        return Err(invalid(
            "--report, --verbose, and --module require --compat",
        ));
    }
    Ok(Some(options))
}

fn run_compat_tests(
    options: &Options,
    current_directory: &Path,
    stdout: &mut dyn Write,
) -> io::Result<u8> {
    let Some(runner_path) = find_compat_runner(current_directory) else {
        writeln!(
            stdout,
            "{RED}Error:{RESET} Could not find tests/compat_runner.py"
        )?;
        writeln!(
            stdout,
            "{DIM}Make sure you're running from the kipferl repository root.{RESET}"
        )?;
        return Ok(1);
    };
    let runtime_path = runtime_path(current_directory)?;

    let mut command = Command::new("python3");
    command
        .arg(runner_path)
        .arg("--runtime")
        .arg(runtime_path)
        .current_dir(current_directory);
    if options.verbose {
        command.arg("--verbose");
    }
    if options.report {
        command.arg("--report");
    }
    if let Some(module) = &options.module {
        command.arg("--module").arg(module);
    }

    stdout.flush()?;
    exit_code(command.status()?)
}

fn find_compat_runner(current_directory: &Path) -> Option<PathBuf> {
    if let Some(path) = env::var_os("KIPFERL_COMPAT_RUNNER").map(PathBuf::from)
        && path.is_file()
    {
        return Some(path);
    }

    if let Ok(executable) = env::current_exe()
        && let Some(binary_directory) = executable.parent()
    {
        for ancestor in binary_directory.ancestors().take(4) {
            let candidate = ancestor.join("tests/compat_runner.py");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let current_path = current_directory.join("tests/compat_runner.py");
    if current_path.is_file() {
        return Some(current_path);
    }

    let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/compat_runner.py");
    source_path.is_file().then_some(source_path)
}

fn runtime_path(current_directory: &Path) -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("KIPFERL_TEST_RUNTIME").map(PathBuf::from)
        && path.is_file()
    {
        return Ok(path);
    }

    if let Ok(executable) = env::current_exe()
        && let Some(binary_directory) = executable.parent()
    {
        let sibling = binary_directory.join("pocketpy-kipferl");
        if sibling.is_file() {
            return Ok(sibling);
        }
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for root in [current_directory, source_root.as_path()] {
        for profile in ["release", "debug"] {
            let candidate = root.join("target").join(profile).join("pocketpy-kipferl");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    run_command::prepare_runtime()
}

fn run_single_test(
    test_file: &str,
    current_directory: &Path,
    stdout: &mut dyn Write,
) -> io::Result<u8> {
    let path = current_directory.join(test_file);
    if path.is_dir() {
        let mut files = Vec::new();
        discover_tests(&path, &mut files)?;
        return run_files(&mut files, current_directory, stdout);
    }
    writeln!(stdout, "Running {test_file} with pocketpy-kipferl...\n")?;
    stdout.flush()?;
    exit_code(
        Command::new(env::current_exe()?)
            .arg("run")
            .arg(path)
            .current_dir(current_directory)
            .status()?,
    )
}

fn run_project_tests(current_directory: &Path, stdout: &mut dyn Write) -> io::Result<u8> {
    let (root, paths) = match project_config::discover(current_directory)? {
        Some(config) => (config.root, config.tests),
        None => (current_directory.to_owned(), vec![PathBuf::from("tests")]),
    };
    let mut files = Vec::new();
    for path in paths {
        let path = root.join(path);
        if !path.exists() {
            return Err(invalid(&format!(
                "test path does not exist: {}; create tests/test_app.py with top-level assertions",
                path.display()
            )));
        }
        if path.is_dir() {
            discover_tests(&path, &mut files)?;
        } else if path.extension().is_some_and(|v| v == "py") {
            files.push(path);
        } else {
            return Err(invalid(&format!(
                "test path is not a Python file: {}",
                path.display()
            )));
        }
    }
    run_files(&mut files, &root, stdout)
}

fn discover_tests(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Do not follow directory symlinks or traverse generated files.
        if kind.is_dir() && !name.starts_with('.') && name != "__pycache__" {
            discover_tests(&entry.path(), files)?;
        } else if kind.is_file() && name.starts_with("test_") && name.ends_with(".py") {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn run_files(files: &mut Vec<PathBuf>, root: &Path, stdout: &mut dyn Write) -> io::Result<u8> {
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err(invalid(
            "no tests found; create tests/test_app.py with top-level assertions",
        ));
    }
    let mut failed = 0_usize;
    for file in files.iter() {
        if run_single_test(&file.to_string_lossy(), root, stdout)? != 0 {
            failed = failed.saturating_add(1);
        }
    }
    writeln!(
        stdout,
        "\n{} passed, {failed} failed ({} test files)",
        files.len().saturating_sub(failed),
        files.len()
    )?;
    Ok(u8::from(failed != 0))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_owned())
}

fn exit_code(status: ExitStatus) -> io::Result<u8> {
    status.code().map_or(Ok(1), |code| {
        u8::try_from(code).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("child process returned invalid exit code {code}"),
            )
        })
    })
}

pub fn help() -> String {
    format!(
        "\n  {CYAN}{BOLD}Kipferl test{RESET} - Project tests and CPython Compatibility Testing\n\n{BOLD}USAGE{RESET}\n    kipferl test [options] [file]\n\n{BOLD}OPTIONS{RESET}\n    {CYAN}--compat{RESET}        Run full CPython compatibility test suite\n    {CYAN}--report{RESET}, -r    Generate compat_report.md\n    {CYAN}--verbose{RESET}, -v   Show failure details\n    {CYAN}--module{RESET}, -m    Test only specified module\n    {CYAN}-h{RESET}, --help      Show this help\n\n{BOLD}EXAMPLES{RESET}\n    {DIM}${RESET} kipferl test --compat              {DIM}# Full compatibility suite{RESET}\n    {DIM}${RESET} kipferl test --compat --report     {DIM}# Generate markdown report{RESET}\n    {DIM}${RESET} kipferl test --compat -m functools {DIM}# Test single module{RESET}\n    {DIM}${RESET} kipferl test mytest.py             {DIM}# Run with pocketpy-kipferl{RESET}\n\n{BOLD}ABOUT{RESET}\n    Without --compat, discovers tests/test_*.py recursively or the paths\n    configured in kipferl.json. Tests use top-level assertions; each file\n    runs in an isolated interpreter. Any failed file fails the command.\n\n    --compat tests Kipferl's compatibility with CPython standard library.\n    Runs each test file with both CPython and pocketpy-kipferl,\n    comparing results to calculate compatibility percentages.\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn rejects_unknown_and_conflicting_test_arguments() {
        for args in [
            vec!["--unknown"],
            vec!["--module"],
            vec!["a.py", "b.py"],
            vec!["--compat", "a.py"],
            vec!["--report"],
        ] {
            assert!(
                parse(
                    &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                    &mut Vec::new()
                )
                .is_err()
            );
        }
        assert!(
            parse(
                &["--compat".into(), "--module".into(), "json".into()],
                &mut Vec::new()
            )
            .is_ok()
        );
    }
}
