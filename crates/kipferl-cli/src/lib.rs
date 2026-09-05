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

mod build_command;
mod bundle;
mod completions;
mod dependencies;
mod dev_command;
mod embedded_json;
mod embedded_runtime;
mod encoding;
mod package_compat;
mod project;
mod project_config;
mod run_command;
mod syntax_check;
mod test_command;
mod tree_shake;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use project::{AiInstructions, ProjectOptions, Template};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";

#[must_use]
pub fn version() -> &'static str {
    include_str!("../../../VERSION").trim()
}

/// Execute a CLI command.
///
/// # Errors
/// Returns filesystem, process, or output errors encountered while running the command.
pub fn run(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some((command, command_arguments)) = arguments.split_first() else {
        print_logo(stdout)?;
        write!(stdout, "{}", main_help())?;
        return Ok(0);
    };

    match command.as_str() {
        "-h" | "--help" => {
            print_logo(stdout)?;
            write!(stdout, "{}", main_help())?;
            Ok(0)
        }
        "-v" | "--version" => {
            writeln!(
                stdout,
                "{CYAN}{BOLD}Kipferl{RESET} {DIM}v{}{RESET}",
                version()
            )?;
            Ok(0)
        }
        "new" => run_new(command_arguments, current_directory, stdout, stderr),
        "init" => run_init(command_arguments, current_directory, stdout, stderr),
        "run" => run_project_command(
            ProjectCommand::Run,
            command_arguments,
            current_directory,
            stdout,
            stderr,
        ),
        "dev" => run_project_command(
            ProjectCommand::Dev,
            command_arguments,
            current_directory,
            stdout,
            stderr,
        ),
        "build" => run_project_command(
            ProjectCommand::Build,
            command_arguments,
            current_directory,
            stdout,
            stderr,
        ),
        "completions" => completions::execute(command_arguments, stdout, stderr),
        "test" => test_command::execute(command_arguments, current_directory, stdout, stderr),
        "add" | "sync" | "deps" => dependencies::execute(
            command,
            command_arguments,
            current_directory,
            stdout,
            stderr,
        ),
        unknown => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Unknown command '{BOLD}{unknown}{RESET}'"
            )?;
            writeln!(
                stderr,
                "{DIM}Run '{RESET}kipferl --help{DIM}' for usage.{RESET}"
            )?;
            Ok(1)
        }
    }
}

fn run_new(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut options = ProjectOptions {
        create_app: true,
        project_files: true,
        ..ProjectOptions::default()
    };
    let mut name = None;
    let mut minimal = false;
    let mut arguments = arguments.iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => {
                write!(stdout, "{}", new_help())?;
                return Ok(0);
            }
            "--stubs" => options.add_stubs = true,
            "--all" => {
                options.add_stubs = true;
                options.ai = Some(AiInstructions::All);
            }
            "--minimal" => minimal = true,
            "--template" => {
                let Some(value) = arguments.next() else {
                    return usage_error(stderr, "--template requires cli, api, or interactive");
                };
                let Some(template) = Template::parse(value) else {
                    return usage_error(stderr, "invalid --template; use cli, api, or interactive");
                };
                options.template = template;
            }
            "--ai" => {
                let Some(value) = arguments.next() else {
                    return usage_error(
                        stderr,
                        "--ai requires a type (agents, claude, copilot, all)",
                    );
                };
                options.ai = match AiInstructions::parse(value) {
                    Some(ai) => Some(ai),
                    None => {
                        return usage_error(
                            stderr,
                            "invalid --ai type; use agents, claude, copilot, or all",
                        );
                    }
                };
            }
            option if option.starts_with('-') => {
                return usage_error(stderr, &format!("unknown option '{option}'"));
            }
            value if name.is_none() => name = Some(value.to_owned()),
            value => return usage_error(stderr, &format!("unexpected argument '{value}'")),
        }
    }

    let Some(name) = name else {
        writeln!(stderr, "{RED}Error:{RESET} No project name specified")?;
        writeln!(stderr, "{DIM}Usage: {RESET}kipferl new <name>")?;
        return Ok(1);
    };
    let sanitized = match project::sanitize_name(&name) {
        Ok(name) => name,
        Err(message) => return usage_error(stderr, message),
    };
    options.app_name = Some(name.clone());

    print_new_logo(stdout)?;
    writeln!(stdout, "Creating new project: {BOLD}{name}{RESET}\n")?;

    if minimal {
        options.project_files = false;
        project::initialize(current_directory, &options, stdout)?;
        return Ok(0);
    }

    options.add_stubs = true;
    let project_directory = current_directory.join(&sanitized);
    match fs::create_dir(&project_directory) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Directory '{sanitized}' already exists"
            )?;
            return Ok(1);
        }
        Err(error) => return Err(with_context(&error, "failed to create project directory")),
    }
    writeln!(stdout, "{GREEN}+{RESET} Created {sanitized}/")?;
    project::initialize(&project_directory, &options, stdout)?;

    writeln!(stdout, "\n\x1b[32mDone!{RESET} Project created.\n")?;
    write!(stdout, "{BOLD}Next steps:\n{RESET}")?;
    writeln!(stdout, "  {DIM}${RESET} {CYAN}cd {sanitized}{RESET}")?;
    writeln!(stdout, "  {DIM}${RESET} {CYAN}kipferl run{RESET}\n")?;
    write!(stdout, "{BOLD}Build standalone binary:\n{RESET}")?;
    writeln!(stdout, "  {DIM}${RESET} {CYAN}kipferl build{RESET}")?;
    Ok(0)
}

fn run_init(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut options = ProjectOptions::default();
    let mut arguments = arguments.iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => {
                write!(stdout, "{}", init_help())?;
                return Ok(0);
            }
            "--stubs" => options.add_stubs = true,
            "--all" => {
                options.add_stubs = true;
                options.ai = Some(AiInstructions::All);
            }
            "--ai" => {
                let Some(value) = arguments.next() else {
                    return usage_error(
                        stderr,
                        "--ai requires a type (agents, claude, copilot, all)",
                    );
                };
                options.ai = match AiInstructions::parse(value) {
                    Some(ai) => Some(ai),
                    None => {
                        return usage_error(
                            stderr,
                            "invalid --ai type; use agents, claude, copilot, or all",
                        );
                    }
                };
            }
            option => return usage_error(stderr, &format!("unknown option '{option}'")),
        }
    }

    if !options.add_stubs && options.ai.is_none() {
        options.add_stubs = true;
    }

    let files_created = project::initialize(current_directory, &options, stdout)?;
    writeln!(
        stdout,
        "\n\x1b[32mDone!{RESET} Initialized kipferl in current directory."
    )?;
    if options.add_stubs && files_created > 0 {
        writeln!(
            stdout,
            "\n{DIM}IDE autocomplete should now work for kipferl modules.{RESET}"
        )?;
    }
    Ok(0)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectCommand {
    Run,
    Dev,
    Build,
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The argument cursor advances by at most two only while a value exists, and value-taking options verify the next argument before advancing"
)]
fn run_project_command(
    command: ProjectCommand,
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut args = arguments.to_vec();
    let mut index = 0;
    let mut explicit_script = false;
    let mut script_args = None;
    while let Some(value) = args.get(index).map(String::as_str) {
        if matches!(value, "--help" | "-h" | "--targets") {
            break;
        }
        if value == "--" {
            script_args = Some(index);
            break;
        }
        if !value.starts_with('-') {
            explicit_script = true;
            break;
        }
        let takes_value = match command {
            ProjectCommand::Build => matches!(
                value,
                "-o" | "--output" | "-m" | "--mode" | "-t" | "--target" | "--asset"
            ),
            ProjectCommand::Dev => matches!(value, "-w" | "--watch" | "--debounce"),
            ProjectCommand::Run => false,
        };
        if takes_value && args.get(index + 1).is_none_or(|v| v.starts_with('-')) {
            return usage_error(stderr, &format!("{value} requires a value"));
        }
        if command == ProjectCommand::Run {
            return usage_error(
                stderr,
                &format!(
                    "unknown option '{value}'; use 'kipferl run -- [arguments]' to pass arguments to the project"
                ),
            );
        }
        index += if takes_value { 2 } else { 1 };
    }
    let informational = args
        .first()
        .is_some_and(|v| matches!(v.as_str(), "--help" | "-h" | "--targets"));
    let needs_output = command == ProjectCommand::Build
        && !args.iter().any(|v| matches!(v.as_str(), "-o" | "--output"));
    if (!explicit_script || needs_output) && !informational {
        let config = match project_config::discover(current_directory) {
            Ok(config) => config,
            Err(error) => return usage_error(stderr, &error.to_string()),
        };
        if let Some(config) = config {
            let entry = config
                .root
                .join(config.entry)
                .to_string_lossy()
                .into_owned();
            if command == ProjectCommand::Build {
                if !explicit_script {
                    args.push(entry);
                }
                if !args.iter().any(|v| matches!(v.as_str(), "-o" | "--output")) {
                    args.extend([
                        "--output".to_owned(),
                        config
                            .root
                            .join(config.output)
                            .to_string_lossy()
                            .into_owned(),
                    ]);
                }
            } else if command == ProjectCommand::Run {
                if args.first().is_some_and(|v| v == "--") {
                    args.remove(0);
                }
                args.insert(0, entry);
            } else {
                args.insert(script_args.unwrap_or(args.len()), entry);
            }
        } else if command == ProjectCommand::Run && args.first().is_some_and(|v| v == "--") {
            args.remove(0);
        }
    }
    if command == ProjectCommand::Run && args.get(1).is_some_and(|v| v == "--") {
        args.remove(1);
    }
    match command {
        ProjectCommand::Run => run_command::execute(&args, current_directory, stdout, stderr),
        ProjectCommand::Dev => dev_command::execute(&args, current_directory, stdout, stderr),
        ProjectCommand::Build => build_command::execute(&args, current_directory, stdout, stderr),
    }
}

fn usage_error(stderr: &mut dyn Write, message: &str) -> io::Result<u8> {
    writeln!(stderr, "{RED}Error:{RESET} {message}")?;
    Ok(1)
}

fn with_context(error: &io::Error, context: &str) -> io::Error {
    io::Error::new(error.kind(), format!("{context}: {error}"))
}

const fn logo_padding(width: usize, content_width: usize) -> (usize, usize) {
    let total = width.saturating_sub(content_width);
    let left = total / 2;
    (left, total.saturating_sub(left))
}

fn print_logo(output: &mut dyn Write) -> io::Result<()> {
    let tagline = "Beautiful CLIs with PocketPy";
    let title_width = 8_usize.saturating_add(version().len());
    let box_width = tagline.len().max(title_width).saturating_add(6);
    let (title_pad_left, title_pad_right) = logo_padding(box_width, title_width);
    let (tagline_pad_left, tagline_pad_right) = logo_padding(box_width, tagline.len());

    writeln!(output)?;
    write!(output, "{CYAN}{BOLD}  ╭{}╮\n{RESET}", "─".repeat(box_width))?;
    write!(
        output,
        "{CYAN}{BOLD}  │{RESET}{}{CYAN}{BOLD}Kipferl{RESET} {DIM}v{}{RESET}{}{CYAN}{BOLD}│\n{RESET}",
        " ".repeat(title_pad_left),
        version(),
        " ".repeat(title_pad_right)
    )?;
    write!(
        output,
        "{CYAN}{BOLD}  │{RESET}{}{DIM}{tagline}{RESET}{}{CYAN}{BOLD}│\n{RESET}",
        " ".repeat(tagline_pad_left),
        " ".repeat(tagline_pad_right)
    )?;
    write!(output, "{CYAN}{BOLD}  ╰{}╯\n{RESET}", "─".repeat(box_width))?;
    writeln!(output)
}

fn print_new_logo(output: &mut dyn Write) -> io::Result<()> {
    writeln!(
        output,
        "\n{CYAN}┌┬┐┌─┐┬ ┬┌─┐┬─┐┌┬┐{RESET}\n{CYAN}││││  ├─┤├─┤├┬┘│││{RESET}\n{CYAN}┴ ┴└─┘┴ ┴┴ ┴┴└─┴ ┴{RESET}"
    )?;
    writeln!(output, "{DIM}Beautiful CLIs with PocketPy{RESET}\n")
}

fn main_help() -> String {
    format!(
        "{BOLD}USAGE{RESET}\n    kipferl {CYAN}<command>{RESET} [options]\n\n{BOLD}COMMANDS{RESET}\n    {CYAN}new{RESET} {DIM}<name>{RESET}        Create a new project\n    {CYAN}run{RESET} {DIM}[file]{RESET}        Run a script with pocketpy\n    {CYAN}dev{RESET} {DIM}[file]{RESET}        Run and restart when files change\n    {CYAN}build{RESET} {DIM}[file]{RESET}      Build a standalone binary\n    {CYAN}init{RESET}              Initialize kipferl in current directory\n    {CYAN}test{RESET}              Run project tests (or --compat)\n    {CYAN}add{RESET} <requirement> Add a compatible PyPI dependency\n    {CYAN}sync{RESET} --locked     Restore locked dependencies\n    {CYAN}deps{RESET}              Check installed packages or view the catalog\n    {CYAN}completions{RESET} <shell> Generate bash, zsh, or fish completions\n\n{BOLD}OPTIONS{RESET}\n    {CYAN}-h{RESET}, {CYAN}--help{RESET}        Show this help\n    {CYAN}-v{RESET}, {CYAN}--version{RESET}     Show version\n\n    With kipferl.json, run/dev/build use the configured entry and output.\n    Pass project arguments with: kipferl run -- [arguments]\n\n{BOLD}EXAMPLES{RESET}\n    {DIM}${RESET} kipferl new myapp                  {DIM}# Create new project{RESET}\n    {DIM}${RESET} kipferl dev app.py                 {DIM}# Develop with live restart{RESET}\n    {DIM}${RESET} kipferl build app.py -o app        {DIM}# Build universal binary{RESET}\n    {DIM}${RESET} kipferl init --stubs --ai claude   {DIM}# Add IDE support{RESET}\n\n{DIM}    Docs: https://kipferl.dev{RESET}\n"
    )
}

fn new_help() -> String {
    format!(
        "{BOLD}kipferl new{RESET} - Create a new kipferl project\n\n{DIM}USAGE:{RESET}\n    kipferl new <name> [options]\n\n{DIM}ARGUMENTS:{RESET}\n    <name>           Project name (creates <name>/ directory)\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete (default for new)\n    --template <type> Starter: cli (default), api, interactive\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add stubs and AI instructions (agents + claude)\n    --minimal        Just create the .py file (no directory)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    kipferl new myapp\n    kipferl new myapp --all\n    kipferl new myapp --stubs --ai claude\n    kipferl new myapp --minimal\n\n{DIM}FILES CREATED:{RESET}\n    myapp/\n      myapp.py                       Main application file\n      kipferl.json                   Project defaults\n      README.md                      Run, test, and ship instructions\n      tests/test_app.py              Runnable application test\n      .kipferl/stubs/                 Type stubs (included)\n      pyrightconfig.json             Pyright config (included)\n      AGENTS.md                      AI instructions (with --ai)\n"
    )
}

fn init_help() -> String {
    format!(
        "{BOLD}kipferl init{RESET} - Initialize kipferl in current directory\n\n{DIM}USAGE:{RESET}\n    kipferl init [options]\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete (default)\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add both stubs and AI instructions (agents + claude)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    kipferl init --stubs\n    kipferl init --ai agents\n    kipferl init --all\n\n{DIM}FILES CREATED:{RESET}\n    .kipferl/stubs/                   Type stubs for runtime modules\n    pyrightconfig.json               Pyright configuration\n    AGENTS.md                        Universal (Cursor, Windsurf, Zed)\n    CLAUDE.md                        Claude Code\n    .github/copilot-instructions.md  GitHub Copilot\n"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn logo_padding_handles_titles_wider_than_the_box() {
        assert_eq!(super::logo_padding(10, 20), (0, 0));
        assert_eq!(super::logo_padding(20, 9), (5, 6));
    }

    use super::{
        build_command, dev_command, init_help, main_help, new_help, project::sanitize_name,
        run_command,
    };

    #[test]
    fn help_describes_project_workflows() {
        assert!(main_help().contains("completions"));
        assert!(new_help().contains("--template"));
        assert!(init_help().contains("--stubs"));
        assert!(run_command::help().contains("kipferl run"));
        assert!(dev_command::help().contains("--watch"));
        assert!(build_command::help().contains("--output"));
    }

    #[test]
    fn project_name_sanitization_matches_the_zig_cli() {
        for (input, expected) in [
            ("My App", "my_app"),
            ("hello-world", "hello_world"),
            ("TEST", "test"),
            ("Simple", "simple"),
        ] {
            assert_eq!(sanitize_name(input), Ok(expected.to_owned()));
        }
    }

    #[test]
    fn project_names_cannot_escape_the_target_directory() {
        for invalid in [
            "",
            ".",
            "..",
            "../app",
            "path/app",
            "path\\app",
            "bad\0name",
        ] {
            assert!(sanitize_name(invalid).is_err(), "accepted {invalid:?}");
        }
    }
}
