mod project;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use project::{AiInstructions, ProjectOptions};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

#[must_use]
pub fn version() -> &'static str {
    include_str!("../../../VERSION").trim()
}

pub fn run(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some(command) = arguments.first().map(String::as_str) else {
        print_logo(stdout)?;
        write!(stdout, "{}", main_help())?;
        return Ok(0);
    };

    match command {
        "-h" | "--help" => {
            print_logo(stdout)?;
            write!(stdout, "{}", main_help())?;
            Ok(0)
        }
        "-v" | "--version" => {
            writeln!(
                stdout,
                "{CYAN}{BOLD}μcharm{RESET} {DIM}v{}{RESET}",
                version()
            )?;
            Ok(0)
        }
        "new" => run_new(&arguments[1..], current_directory, stdout, stderr),
        "init" => run_init(&arguments[1..], current_directory, stdout, stderr),
        "build" | "run" | "test" => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Command '{BOLD}{command}{RESET}' has not migrated to Rust yet"
            )?;
            writeln!(
                stderr,
                "{DIM}Use the production Zig CLI for this command during the migration.{RESET}"
            )?;
            Ok(1)
        }
        unknown => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Unknown command '{BOLD}{unknown}{RESET}'"
            )?;
            writeln!(
                stderr,
                "{DIM}Run '{RESET}ucharm --help{DIM}' for usage.{RESET}"
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
        ..ProjectOptions::default()
    };
    let mut name = None;
    let mut minimal = false;
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
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
            "--ai" => {
                index += 1;
                let Some(value) = arguments.get(index) else {
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
        index += 1;
    }

    let Some(name) = name else {
        writeln!(stderr, "{RED}Error:{RESET} No project name specified")?;
        writeln!(stderr, "{DIM}Usage: {RESET}ucharm new <name>")?;
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
        project::initialize(current_directory, &options, stdout)?;
        return Ok(0);
    }

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
        Err(error) => return Err(with_context(error, "failed to create project directory")),
    }
    writeln!(stdout, "{GREEN}+{RESET} Created {sanitized}/")?;
    project::initialize(&project_directory, &options, stdout)?;

    writeln!(stdout, "\n\x1b[32mDone!{RESET} Project created.\n")?;
    write!(stdout, "{BOLD}Next steps:\n{RESET}")?;
    writeln!(stdout, "  {DIM}${RESET} {CYAN}cd {sanitized}{RESET}")?;
    writeln!(
        stdout,
        "  {DIM}${RESET} {CYAN}ucharm run {sanitized}.py{RESET}\n"
    )?;
    write!(stdout, "{BOLD}Build standalone binary:\n{RESET}")?;
    writeln!(
        stdout,
        "  {DIM}${RESET} {CYAN}ucharm build {sanitized}.py -o {sanitized} --mode universal{RESET}"
    )?;
    Ok(0)
}

fn run_init(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut options = ProjectOptions::default();
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
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
                index += 1;
                let Some(value) = arguments.get(index) else {
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
        index += 1;
    }

    if !options.add_stubs && options.ai.is_none() {
        writeln!(
            stdout,
            "{YELLOW}No options specified.{RESET} Use --stubs, --ai, or --all\n"
        )?;
        write!(stdout, "{}", init_help())?;
        return Ok(0);
    }

    let files_created = project::initialize(current_directory, &options, stdout)?;
    writeln!(
        stdout,
        "\n\x1b[32mDone!{RESET} Initialized ucharm in current directory."
    )?;
    if options.add_stubs && files_created > 0 {
        writeln!(
            stdout,
            "\n{DIM}IDE autocomplete should now work for ucharm modules.{RESET}"
        )?;
    }
    Ok(0)
}

fn usage_error(stderr: &mut dyn Write, message: &str) -> io::Result<u8> {
    writeln!(stderr, "{RED}Error:{RESET} {message}")?;
    Ok(1)
}

fn with_context(error: io::Error, context: &str) -> io::Error {
    io::Error::new(error.kind(), format!("{context}: {error}"))
}

fn print_logo(output: &mut dyn Write) -> io::Result<()> {
    let tagline = "Beautiful CLIs with PocketPy";
    let box_width = tagline.len() + 6;
    let title_width = 8 + version().len();
    let title_pad_left = (box_width - title_width) / 2;
    let title_pad_right = box_width - title_width - title_pad_left;
    let tagline_pad_left = (box_width - tagline.len()) / 2;
    let tagline_pad_right = box_width - tagline.len() - tagline_pad_left;

    writeln!(output)?;
    write!(output, "{CYAN}{BOLD}  ╭{}╮\n{RESET}", "─".repeat(box_width))?;
    write!(
        output,
        "{CYAN}{BOLD}  │{RESET}{}{CYAN}{BOLD}μcharm{RESET} {DIM}v{}{RESET}{}{CYAN}{BOLD}│\n{RESET}",
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
        "{BOLD}USAGE{RESET}\n    ucharm {CYAN}<command>{RESET} [options]\n\n{BOLD}COMMANDS{RESET}\n    {CYAN}new{RESET} {DIM}<name>{RESET}        Create a new project\n    {CYAN}run{RESET} {DIM}<file>{RESET}        Run a script with pocketpy\n    {CYAN}build{RESET} {DIM}<file>{RESET}      Build a standalone binary\n    {CYAN}init{RESET}              Initialize ucharm in current directory\n    {CYAN}test{RESET}              Run compatibility tests\n\n{BOLD}OPTIONS{RESET}\n    {CYAN}-h{RESET}, {CYAN}--help{RESET}        Show this help\n    {CYAN}-v{RESET}, {CYAN}--version{RESET}     Show version\n\n{BOLD}EXAMPLES{RESET}\n    {DIM}${RESET} ucharm new myapp                  {DIM}# Create new project{RESET}\n    {DIM}${RESET} ucharm run app.py                 {DIM}# Run with pocketpy{RESET}\n    {DIM}${RESET} ucharm build app.py -o app        {DIM}# Build universal binary{RESET}\n    {DIM}${RESET} ucharm init --stubs --ai claude   {DIM}# Add IDE support{RESET}\n\n{DIM}    Docs: https://github.com/ucharmdev/ucharm{RESET}\n"
    )
}

fn new_help() -> String {
    format!(
        "{BOLD}ucharm new{RESET} - Create a new ucharm project\n\n{DIM}USAGE:{RESET}\n    ucharm new <name> [options]\n\n{DIM}ARGUMENTS:{RESET}\n    <name>           Project name (creates <name>/ directory)\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add stubs and AI instructions (agents + claude)\n    --minimal        Just create the .py file (no directory)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    ucharm new myapp\n    ucharm new myapp --all\n    ucharm new myapp --stubs --ai claude\n    ucharm new myapp --minimal\n\n{DIM}FILES CREATED:{RESET}\n    myapp/\n      myapp.py                       Main application file\n      .ucharm/stubs/                 Type stubs (with --stubs)\n      pyrightconfig.json             Pyright config (with --stubs)\n      AGENTS.md                      AI instructions (with --ai)\n"
    )
}

fn init_help() -> String {
    format!(
        "{BOLD}ucharm init{RESET} - Initialize ucharm in current directory\n\n{DIM}USAGE:{RESET}\n    ucharm init [options]\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add both stubs and AI instructions (agents + claude)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    ucharm init --stubs\n    ucharm init --ai agents\n    ucharm init --all\n\n{DIM}FILES CREATED:{RESET}\n    .ucharm/stubs/                   Type stubs for runtime modules\n    pyrightconfig.json               Pyright configuration\n    AGENTS.md                        Universal (Cursor, Windsurf, Zed)\n    CLAUDE.md                        Claude Code\n    .github/copilot-instructions.md  GitHub Copilot\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{init_help, main_help, new_help, project::sanitize_name};

    #[test]
    fn help_text_matches_the_zig_cli_snapshots() {
        assert_eq!(
            main_help(),
            concat!(
                "\x1b[1mUSAGE\x1b[0m\n",
                "    ucharm \x1b[36m<command>\x1b[0m [options]\n\n",
                "\x1b[1mCOMMANDS\x1b[0m\n",
                "    \x1b[36mnew\x1b[0m \x1b[2m<name>\x1b[0m        Create a new project\n",
                "    \x1b[36mrun\x1b[0m \x1b[2m<file>\x1b[0m        Run a script with pocketpy\n",
                "    \x1b[36mbuild\x1b[0m \x1b[2m<file>\x1b[0m      Build a standalone binary\n",
                "    \x1b[36minit\x1b[0m              Initialize ucharm in current directory\n",
                "    \x1b[36mtest\x1b[0m              Run compatibility tests\n\n",
                "\x1b[1mOPTIONS\x1b[0m\n",
                "    \x1b[36m-h\x1b[0m, \x1b[36m--help\x1b[0m        Show this help\n",
                "    \x1b[36m-v\x1b[0m, \x1b[36m--version\x1b[0m     Show version\n\n",
                "\x1b[1mEXAMPLES\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm new myapp                  \x1b[2m# Create new project\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm run app.py                 \x1b[2m# Run with pocketpy\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm build app.py -o app        \x1b[2m# Build universal binary\x1b[0m\n",
                "    \x1b[2m$\x1b[0m ucharm init --stubs --ai claude   \x1b[2m# Add IDE support\x1b[0m\n\n",
                "\x1b[2m    Docs: https://github.com/ucharmdev/ucharm\x1b[0m\n",
            )
        );
        assert_eq!(
            new_help(),
            concat!(
                "\x1b[1mucharm new\x1b[0m - Create a new ucharm project\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n    ucharm new <name> [options]\n\n",
                "\x1b[2mARGUMENTS:\x1b[0m\n",
                "    <name>           Project name (creates <name>/ directory)\n\n",
                "\x1b[2mOPTIONS:\x1b[0m\n",
                "    --stubs          Add type stubs for IDE autocomplete\n",
                "    --ai <type>      Add AI assistant instructions\n",
                "                     Types: agents, claude, copilot, all\n",
                "    --all            Add stubs and AI instructions (agents + claude)\n",
                "    --minimal        Just create the .py file (no directory)\n",
                "    -h, --help       Show this help\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    ucharm new myapp\n",
                "    ucharm new myapp --all\n",
                "    ucharm new myapp --stubs --ai claude\n",
                "    ucharm new myapp --minimal\n\n",
                "\x1b[2mFILES CREATED:\x1b[0m\n",
                "    myapp/\n",
                "      myapp.py                       Main application file\n",
                "      .ucharm/stubs/                 Type stubs (with --stubs)\n",
                "      pyrightconfig.json             Pyright config (with --stubs)\n",
                "      AGENTS.md                      AI instructions (with --ai)\n",
            )
        );
        assert_eq!(
            init_help(),
            concat!(
                "\x1b[1mucharm init\x1b[0m - Initialize ucharm in current directory\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n    ucharm init [options]\n\n",
                "\x1b[2mOPTIONS:\x1b[0m\n",
                "    --stubs          Add type stubs for IDE autocomplete\n",
                "    --ai <type>      Add AI assistant instructions\n",
                "                     Types: agents, claude, copilot, all\n",
                "    --all            Add both stubs and AI instructions (agents + claude)\n",
                "    -h, --help       Show this help\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    ucharm init --stubs\n",
                "    ucharm init --ai agents\n",
                "    ucharm init --all\n\n",
                "\x1b[2mFILES CREATED:\x1b[0m\n",
                "    .ucharm/stubs/                   Type stubs for runtime modules\n",
                "    pyrightconfig.json               Pyright configuration\n",
                "    AGENTS.md                        Universal (Cursor, Windsurf, Zed)\n",
                "    CLAUDE.md                        Claude Code\n",
                "    .github/copilot-instructions.md  GitHub Copilot\n",
            )
        );
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
