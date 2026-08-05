mod build_command;
mod dev_command;
mod project;
mod run_command;
mod test_command;

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
                "{CYAN}{BOLD}Kipferl{RESET} {DIM}v{}{RESET}",
                version()
            )?;
            Ok(0)
        }
        "new" => run_new(&arguments[1..], current_directory, stdout, stderr),
        "init" => run_init(&arguments[1..], current_directory, stdout, stderr),
        "run" => run_command::execute(&arguments[1..], current_directory, stdout, stderr),
        "dev" => dev_command::execute(&arguments[1..], current_directory, stdout, stderr),
        "build" => build_command::execute(&arguments[1..], current_directory, stdout, stderr),
        "test" => test_command::execute(&arguments[1..], current_directory, stdout, stderr),
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
        "  {DIM}${RESET} {CYAN}kipferl run {sanitized}.py{RESET}\n"
    )?;
    write!(stdout, "{BOLD}Build standalone binary:\n{RESET}")?;
    writeln!(
        stdout,
        "  {DIM}${RESET} {CYAN}kipferl build {sanitized}.py -o {sanitized} --mode universal{RESET}"
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
        "{BOLD}USAGE{RESET}\n    kipferl {CYAN}<command>{RESET} [options]\n\n{BOLD}COMMANDS{RESET}\n    {CYAN}new{RESET} {DIM}<name>{RESET}        Create a new project\n    {CYAN}run{RESET} {DIM}<file>{RESET}        Run a script with pocketpy\n    {CYAN}dev{RESET} {DIM}<file>{RESET}        Run and restart when files change\n    {CYAN}build{RESET} {DIM}<file>{RESET}      Build a standalone binary\n    {CYAN}init{RESET}              Initialize kipferl in current directory\n    {CYAN}test{RESET}              Run compatibility tests\n\n{BOLD}OPTIONS{RESET}\n    {CYAN}-h{RESET}, {CYAN}--help{RESET}        Show this help\n    {CYAN}-v{RESET}, {CYAN}--version{RESET}     Show version\n\n{BOLD}EXAMPLES{RESET}\n    {DIM}${RESET} kipferl new myapp                  {DIM}# Create new project{RESET}\n    {DIM}${RESET} kipferl dev app.py                 {DIM}# Develop with live restart{RESET}\n    {DIM}${RESET} kipferl build app.py -o app        {DIM}# Build universal binary{RESET}\n    {DIM}${RESET} kipferl init --stubs --ai claude   {DIM}# Add IDE support{RESET}\n\n{DIM}    Docs: https://kipferl.dev{RESET}\n"
    )
}

fn new_help() -> String {
    format!(
        "{BOLD}kipferl new{RESET} - Create a new kipferl project\n\n{DIM}USAGE:{RESET}\n    kipferl new <name> [options]\n\n{DIM}ARGUMENTS:{RESET}\n    <name>           Project name (creates <name>/ directory)\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add stubs and AI instructions (agents + claude)\n    --minimal        Just create the .py file (no directory)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    kipferl new myapp\n    kipferl new myapp --all\n    kipferl new myapp --stubs --ai claude\n    kipferl new myapp --minimal\n\n{DIM}FILES CREATED:{RESET}\n    myapp/\n      myapp.py                       Main application file\n      .kipferl/stubs/                 Type stubs (with --stubs)\n      pyrightconfig.json             Pyright config (with --stubs)\n      AGENTS.md                      AI instructions (with --ai)\n"
    )
}

fn init_help() -> String {
    format!(
        "{BOLD}kipferl init{RESET} - Initialize kipferl in current directory\n\n{DIM}USAGE:{RESET}\n    kipferl init [options]\n\n{DIM}OPTIONS:{RESET}\n    --stubs          Add type stubs for IDE autocomplete\n    --ai <type>      Add AI assistant instructions\n                     Types: agents, claude, copilot, all\n    --all            Add both stubs and AI instructions (agents + claude)\n    -h, --help       Show this help\n\n{DIM}EXAMPLES:{RESET}\n    kipferl init --stubs\n    kipferl init --ai agents\n    kipferl init --all\n\n{DIM}FILES CREATED:{RESET}\n    .kipferl/stubs/                   Type stubs for runtime modules\n    pyrightconfig.json               Pyright configuration\n    AGENTS.md                        Universal (Cursor, Windsurf, Zed)\n    CLAUDE.md                        Claude Code\n    .github/copilot-instructions.md  GitHub Copilot\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_command, dev_command, init_help, main_help, new_help, project::sanitize_name,
        run_command,
    };

    #[test]
    fn help_text_matches_the_zig_cli_snapshots() {
        assert_eq!(
            main_help(),
            concat!(
                "\x1b[1mUSAGE\x1b[0m\n",
                "    kipferl \x1b[36m<command>\x1b[0m [options]\n\n",
                "\x1b[1mCOMMANDS\x1b[0m\n",
                "    \x1b[36mnew\x1b[0m \x1b[2m<name>\x1b[0m        Create a new project\n",
                "    \x1b[36mrun\x1b[0m \x1b[2m<file>\x1b[0m        Run a script with pocketpy\n",
                "    \x1b[36mdev\x1b[0m \x1b[2m<file>\x1b[0m        Run and restart when files change\n",
                "    \x1b[36mbuild\x1b[0m \x1b[2m<file>\x1b[0m      Build a standalone binary\n",
                "    \x1b[36minit\x1b[0m              Initialize kipferl in current directory\n",
                "    \x1b[36mtest\x1b[0m              Run compatibility tests\n\n",
                "\x1b[1mOPTIONS\x1b[0m\n",
                "    \x1b[36m-h\x1b[0m, \x1b[36m--help\x1b[0m        Show this help\n",
                "    \x1b[36m-v\x1b[0m, \x1b[36m--version\x1b[0m     Show version\n\n",
                "\x1b[1mEXAMPLES\x1b[0m\n",
                "    \x1b[2m$\x1b[0m kipferl new myapp                  \x1b[2m# Create new project\x1b[0m\n",
                "    \x1b[2m$\x1b[0m kipferl dev app.py                 \x1b[2m# Develop with live restart\x1b[0m\n",
                "    \x1b[2m$\x1b[0m kipferl build app.py -o app        \x1b[2m# Build universal binary\x1b[0m\n",
                "    \x1b[2m$\x1b[0m kipferl init --stubs --ai claude   \x1b[2m# Add IDE support\x1b[0m\n\n",
                "\x1b[2m    Docs: https://kipferl.dev\x1b[0m\n",
            )
        );
        assert_eq!(
            new_help(),
            concat!(
                "\x1b[1mkipferl new\x1b[0m - Create a new kipferl project\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n    kipferl new <name> [options]\n\n",
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
                "    kipferl new myapp\n",
                "    kipferl new myapp --all\n",
                "    kipferl new myapp --stubs --ai claude\n",
                "    kipferl new myapp --minimal\n\n",
                "\x1b[2mFILES CREATED:\x1b[0m\n",
                "    myapp/\n",
                "      myapp.py                       Main application file\n",
                "      .kipferl/stubs/                 Type stubs (with --stubs)\n",
                "      pyrightconfig.json             Pyright config (with --stubs)\n",
                "      AGENTS.md                      AI instructions (with --ai)\n",
            )
        );
        assert_eq!(
            init_help(),
            concat!(
                "\x1b[1mkipferl init\x1b[0m - Initialize kipferl in current directory\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n    kipferl init [options]\n\n",
                "\x1b[2mOPTIONS:\x1b[0m\n",
                "    --stubs          Add type stubs for IDE autocomplete\n",
                "    --ai <type>      Add AI assistant instructions\n",
                "                     Types: agents, claude, copilot, all\n",
                "    --all            Add both stubs and AI instructions (agents + claude)\n",
                "    -h, --help       Show this help\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    kipferl init --stubs\n",
                "    kipferl init --ai agents\n",
                "    kipferl init --all\n\n",
                "\x1b[2mFILES CREATED:\x1b[0m\n",
                "    .kipferl/stubs/                   Type stubs for runtime modules\n",
                "    pyrightconfig.json               Pyright configuration\n",
                "    AGENTS.md                        Universal (Cursor, Windsurf, Zed)\n",
                "    CLAUDE.md                        Claude Code\n",
                "    .github/copilot-instructions.md  GitHub Copilot\n",
            )
        );
        assert_eq!(
            run_command::help(),
            concat!(
                "\x1b[1mKipferl run\x1b[0m - Run a Python script with pocketpy-kipferl\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n",
                "    kipferl run <script.py> [args...]\n\n",
                "\x1b[2mARGUMENTS:\x1b[0m\n",
                "    <script.py>    Python script to run\n",
                "    [args...]      Arguments passed to the script\n\n",
                "\x1b[2mDESCRIPTION:\x1b[0m\n",
                "    Runs your Python script using the embedded pocketpy-kipferl\n",
                "    interpreter with all native Kipferl modules available.\n\n",
                "    The script is automatically transformed to use native modules\n",
                "    instead of the kipferl Python package.\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    kipferl run app.py\n",
                "    kipferl run app.py --verbose\n",
                "    kipferl run examples/demo.py\n",
            )
        );
        assert_eq!(
            dev_command::help(),
            concat!(
                "\x1b[1mKipferl dev\x1b[0m - Restart a script when project files change\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n",
                "    kipferl dev [OPTIONS] <script.py> [--] [args...]\n\n",
                "\x1b[2mOPTIONS:\x1b[0m\n",
                "    -w, --watch <path>    Watch an additional file or directory\n",
                "    --clear               Clear the terminal before each restart\n",
                "    --debounce <ms>       Wait for writes to settle (default: 150)\n",
                "    -h, --help            Show this help\n\n",
                "\x1b[2mDESCRIPTION:\x1b[0m\n",
                "    Runs the script immediately, then watches its directory recursively.\n",
                "    The watcher stays alive when the script exits so the next edit runs it\n",
                "    again. Generated, cache, virtual-environment, and VCS paths are ignored.\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    kipferl dev app.py\n",
                "    kipferl dev --clear app.py\n",
                "    kipferl dev --watch templates --watch settings.toml app.py -- --verbose\n",
            )
        );
        assert_eq!(
            build_command::help(),
            concat!(
                "\x1b[1mKipferl build\x1b[0m - Build standalone binaries from Python scripts\n\n",
                "\x1b[2mUSAGE:\x1b[0m\n",
                "    kipferl build <script.py> -o <output> [OPTIONS]\n\n",
                "\x1b[2mOPTIONS:\x1b[0m\n",
                "    -o, --output <path>    Output file path (required)\n",
                "    -m, --mode <mode>      Build mode: universal, executable, single\n",
                "                           (default: universal)\n",
                "    -t, --target <target>  Target platform for cross-compilation\n",
                "                           (default: current platform)\n",
                "    --targets              List available targets\n",
                "    -h, --help             Show this help\n\n",
                "\x1b[2mTARGETS:\x1b[0m\n",
                "    macos-aarch64          macOS on Apple Silicon\n",
                "    macos-x86_64           macOS on Intel\n",
                "    linux-x86_64           Linux on x86_64\n",
                "    linux-aarch64          Linux on ARM64\n\n",
                "\x1b[2mMODES:\x1b[0m\n",
                "    universal              Standalone binary (~5-6MB, no dependencies)\n",
                "    executable             Shell wrapper (requires pocketpy-kipferl)\n",
                "    single                 Transformed .py file (requires pocketpy-kipferl)\n\n",
                "\x1b[2mEXAMPLES:\x1b[0m\n",
                "    kipferl build app.py -o app\n",
                "    kipferl build app.py -o app-linux --target linux-x86_64\n",
                "    kipferl build app.py -o app.py --mode single\n",
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
