use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::run_command;

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
    _stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some(options) = parse(arguments, stdout)? else {
        return Ok(0);
    };

    if options.compat {
        run_compat_tests(&options, current_directory, stdout)
    } else if let Some(test_file) = options.test_file {
        run_single_test(&test_file, current_directory, stdout)
    } else {
        write!(stdout, "{}", help())?;
        Ok(0)
    }
}

fn parse(arguments: &[String], stdout: &mut dyn Write) -> io::Result<Option<Options>> {
    let mut options = Options::default();
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "--compat" => options.compat = true,
            "--report" | "-r" => options.report = true,
            "--verbose" | "-v" => options.verbose = true,
            "--module" | "-m" => {
                index += 1;
                if let Some(module) = arguments.get(index) {
                    options.module = Some(module.clone());
                }
            }
            "-h" | "--help" => {
                write!(stdout, "{}", help())?;
                return Ok(None);
            }
            argument if !argument.starts_with('-') => {
                options.test_file = Some(argument.to_owned());
            }
            _ => {}
        }
        index += 1;
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
            "{DIM}Make sure you're running from the ucharm repository root.{RESET}"
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
    if let Some(path) = env::var_os("UCHARM_COMPAT_RUNNER").map(PathBuf::from)
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
    if let Some(path) = env::var_os("UCHARM_TEST_RUNTIME").map(PathBuf::from)
        && path.is_file()
    {
        return Ok(path);
    }

    if let Ok(executable) = env::current_exe()
        && let Some(binary_directory) = executable.parent()
    {
        let sibling = binary_directory.join("pocketpy-ucharm");
        if sibling.is_file() {
            return Ok(sibling);
        }
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for root in [current_directory, source_root.as_path()] {
        for profile in ["release", "debug"] {
            let candidate = root.join("target").join(profile).join("pocketpy-ucharm");
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
    let runtime_path = runtime_path(current_directory)?;
    writeln!(stdout, "Running {test_file} with pocketpy-ucharm...\n")?;
    stdout.flush()?;

    let status = Command::new(runtime_path)
        .arg(test_file)
        .current_dir(current_directory)
        .status()?;
    exit_code(status)
}

fn exit_code(status: ExitStatus) -> io::Result<u8> {
    match status.code() {
        Some(code) => u8::try_from(code).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("child process returned invalid exit code {code}"),
            )
        }),
        None => Ok(1),
    }
}

pub fn help() -> String {
    format!(
        "\n  {CYAN}{BOLD}μcharm test{RESET} - CPython Compatibility Testing\n\n{BOLD}USAGE{RESET}\n    ucharm test [options] [file]\n\n{BOLD}OPTIONS{RESET}\n    {CYAN}--compat{RESET}        Run full CPython compatibility test suite\n    {CYAN}--report{RESET}, -r    Generate compat_report.md\n    {CYAN}--verbose{RESET}, -v   Show failure details\n    {CYAN}--module{RESET}, -m    Test only specified module\n    {CYAN}-h{RESET}, --help      Show this help\n\n{BOLD}EXAMPLES{RESET}\n    {DIM}${RESET} ucharm test --compat              {DIM}# Full compatibility suite{RESET}\n    {DIM}${RESET} ucharm test --compat --report     {DIM}# Generate markdown report{RESET}\n    {DIM}${RESET} ucharm test --compat -m functools {DIM}# Test single module{RESET}\n    {DIM}${RESET} ucharm test mytest.py             {DIM}# Run with pocketpy-ucharm{RESET}\n\n{BOLD}ABOUT{RESET}\n    Tests μcharm's compatibility with CPython standard library.\n    Runs each test file with both CPython and pocketpy-ucharm,\n    comparing results to calculate compatibility percentages.\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{Options, help, parse};

    #[test]
    fn parses_legacy_options() {
        let arguments = [
            "--compat",
            "-r",
            "--verbose",
            "--module",
            "functools",
            "first.py",
            "last.py",
            "--unknown",
        ]
        .map(str::to_owned);
        let mut stdout = Vec::new();

        assert_eq!(
            parse(&arguments, &mut stdout).expect("parse options"),
            Some(Options {
                compat: true,
                report: true,
                verbose: true,
                module: Some("functools".to_owned()),
                test_file: Some("last.py".to_owned()),
            })
        );
        assert!(stdout.is_empty());
    }

    #[test]
    fn help_matches_the_zig_cli() {
        assert_eq!(
            help(),
            concat!(
                "\n  \x1b[36m\x1b[1mμcharm test\x1b[0m - CPython Compatibility Testing\n\n",
                "\x1b[1mUSAGE\x1b[0m\n",
                "    ucharm test [options] [file]\n\n",
                "\x1b[1mOPTIONS\x1b[0m\n",
                "    \x1b[36m--compat\x1b[0m        Run full CPython compatibility test suite\n",
                "    \x1b[36m--report\x1b[0m, -r    Generate compat_report.md\n",
                "    \x1b[36m--verbose\x1b[0m, -v   Show failure details\n",
                "    \x1b[36m--module\x1b[0m, -m    Test only specified module\n",
                "    \x1b[36m-h\x1b[0m, --help      Show this help\n\n",
                "\x1b[1mEXAMPLES\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm test --compat              \x1b[2m# Full compatibility suite\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm test --compat --report     \x1b[2m# Generate markdown report\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm test --compat -m functools \x1b[2m# Test single module\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm test mytest.py             \x1b[2m# Run with pocketpy-ucharm\x1b[0m\n\n",
                "\x1b[1mABOUT\x1b[0m\n",
                "    Tests μcharm's compatibility with CPython standard library.\n",
                "    Runs each test file with both CPython and pocketpy-ucharm,\n",
                "    comparing results to calculate compatibility percentages.\n\n",
            )
        );
    }
}
