use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

include!("generated_stubs.rs");

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";

const PYRIGHT_CONFIG: &str = r#"{
  "include": ["."],
  "exclude": [".kipferl"],
  "stubPath": ".kipferl/stubs",
  "extraPaths": [".kipferl/packages"],
  "reportMissingImports": false,
  "reportMissingModuleSource": false,
  "pythonVersion": "3.11",
  "typeCheckingMode": "basic"
}
"#;

const CLI_TEMPLATE: &str = r#"#!/usr/bin/env python3
"""__APP_NAME__ - Built with kipferl"""
import sys


def greeting(name):
    return f"Hello, {name}!"


def main():
    argv = sys.argv[1:]
    if argv == ["--help"] or argv == ["-h"]:
        print("Usage: __APP_NAME__ [--name NAME]")
        print("Greet someone. Default name: World.")
        return
    if not argv:
        name = "World"
    elif len(argv) == 2 and argv[0] == "--name":
        name = argv[1]
    else:
        print("Error: expected --name NAME. Run with --help for usage.")
        sys.exit(2)
    print(greeting(name))


if __name__ == "__main__":
    main()
"#;

const INTERACTIVE_TEMPLATE: &str = r#"#!/usr/bin/env python3
"""__APP_NAME__ - Built with kipferl"""
import sys
import tui
import input


def greeting(name):
    return f"Hello, {name}!"


def main():
    if sys.argv[1:] == ["--help"] or sys.argv[1:] == ["-h"]:
        print("Usage: __APP_NAME__")
        print("An interactive greeting tool. Prompts for your name.")
        return
    if len(sys.argv) > 1:
        print("Error: unexpected arguments. Run with --help for usage.")
        sys.exit(2)
    tui.box("__APP_NAME__\nBuilt with kipferl", title="Welcome", border_color="cyan")
    name = input.prompt("What's your name?", default="World")
    tui.success(greeting(name))


if __name__ == "__main__":
    main()
"#;

const API_TEMPLATE: &str = r#"#!/usr/bin/env python3
"""__APP_NAME__ - Built with kipferl"""
import sys
import json
from http.client import HTTPSConnection


def format_payload(payload):
    return json.dumps(payload, indent=2)


def main():
    argv = sys.argv[1:]
    if not argv or argv == ["--help"] or argv == ["-h"]:
        print("Usage: __APP_NAME__ HOST [PATH]")
        print("Fetch JSON over HTTPS; for example: api.github.com /zen")
        print("Non-JSON responses are printed as text. PATH defaults to /.")
        return
    if len(argv) > 2 or argv[0].startswith("-") or "://" in argv[0]:
        print("Error: specify a host without https:// and an optional path.")
        sys.exit(2)
    path = argv[1] if len(argv) > 1 else "/"
    if not path.startswith("/"):
        print("Error: PATH must start with /.")
        sys.exit(2)
    connection = HTTPSConnection(argv[0], timeout=15)
    try:
        connection.request("GET", path, headers={"User-Agent": "kipferl-api-starter", "Accept": "application/json"})
        response = connection.getresponse()
        body = response.read().decode("utf-8")
        if response.status >= 400:
            print(f"Error: HTTP {response.status} {response.reason}")
            sys.exit(1)
        try:
            print(format_payload(json.loads(body)))
        except ValueError:
            print(body)
    except Exception as error:
        print(f"Request failed: {error}")
        sys.exit(1)


if __name__ == "__main__":
    main()
"#;

#[derive(Debug, Clone, Copy, Default)]
pub enum Template {
    #[default]
    Cli,
    Api,
    Interactive,
}

impl Template {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cli" => Some(Self::Cli),
            "api" => Some(Self::Api),
            "interactive" => Some(Self::Interactive),
            _ => None,
        }
    }

    const fn source(self) -> &'static str {
        match self {
            Self::Cli => CLI_TEMPLATE,
            Self::Api => API_TEMPLATE,
            Self::Interactive => INTERACTIVE_TEMPLATE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiInstructions {
    Agents,
    Claude,
    Copilot,
    All,
}

impl AiInstructions {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "agents" => Some(Self::Agents),
            "claude" => Some(Self::Claude),
            "copilot" => Some(Self::Copilot),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectOptions {
    pub add_stubs: bool,
    pub template: Template,
    pub project_files: bool,
    pub ai: Option<AiInstructions>,
    pub create_app: bool,
    pub app_name: Option<String>,
}

pub fn sanitize_name(name: &str) -> Result<String, &'static str> {
    if name.is_empty() {
        return Err("project name cannot be empty");
    }
    if name.len() > 240 {
        return Err("project name is too long");
    }
    if name.chars().any(|character| {
        character == '/' || character == '\\' || character == '\0' || character.is_control()
    }) {
        return Err("project name cannot contain path separators or control characters");
    }

    let sanitized: String = name
        .chars()
        .map(|character| match character {
            ' ' | '-' => '_',
            'A'..='Z' => character.to_ascii_lowercase(),
            _ => character,
        })
        .collect();
    if sanitized == "." || sanitized == ".." || sanitized.is_empty() {
        return Err("project name is not valid");
    }
    Ok(sanitized)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The fixed starter workflow creates at most ten logical files/groups, independent of user input length"
)]
pub fn initialize(
    directory: &Path,
    options: &ProjectOptions,
    output: &mut dyn Write,
) -> io::Result<usize> {
    let mut files_created = 0;

    if options.create_app
        && let Some(name) = &options.app_name
    {
        files_created += usize::from(create_app(directory, name, options.template, output)?);
        if options.project_files {
            files_created += create_project_files(directory, name, options.template, output)?;
        }
    }

    if options.add_stubs {
        let stub_directory = directory.join(".kipferl/stubs");
        fs::create_dir_all(&stub_directory)?;
        for (name, content) in STUBS {
            fs::write(stub_directory.join(name), content)?;
        }
        writeln!(
            output,
            "{GREEN}+{RESET} Created .kipferl/stubs/ ({} stub files)",
            STUBS.len()
        )?;
        files_created += 1;
        files_created += usize::from(write_if_absent(
            &directory.join("pyrightconfig.json"),
            PYRIGHT_CONFIG,
            "pyrightconfig.json",
            output,
        )?);
    }

    if let Some(ai) = options.ai {
        if matches!(ai, AiInstructions::Agents | AiInstructions::All) {
            files_created += usize::from(write_if_absent(
                &directory.join("AGENTS.md"),
                include_str!("../templates/AGENTS.md"),
                "AGENTS.md",
                output,
            )?);
        }
        if matches!(ai, AiInstructions::Claude | AiInstructions::All) {
            files_created += usize::from(write_if_absent(
                &directory.join("CLAUDE.md"),
                include_str!("../templates/CLAUDE.md"),
                "CLAUDE.md",
                output,
            )?);
        }
        if matches!(ai, AiInstructions::Copilot | AiInstructions::All) {
            let copilot_path = directory.join(".github/copilot-instructions.md");
            fs::create_dir_all(directory.join(".github"))?;
            files_created += usize::from(write_if_absent(
                &copilot_path,
                include_str!("../templates/copilot-instructions.md"),
                ".github/copilot-instructions.md",
                output,
            )?);
        }
    }

    Ok(files_created)
}

fn create_app(
    directory: &Path,
    name: &str,
    template: Template,
    output: &mut dyn Write,
) -> io::Result<bool> {
    let filename = format!("{}.py", sanitize_name(name).map_err(io::Error::other)?);
    let path = directory.join(&filename);
    // The name appears inside both a quoted string and a triple-quoted
    // docstring. Escape quotes so every accepted project name stays data.
    let content = template
        .source()
        .replace("__APP_NAME__", &name.replace('"', "\\\""));
    let created = write_if_absent(&path, &content, &filename, output)?;
    if created {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(created)
}

#[expect(
    clippy::arithmetic_side_effects,
    reason = "The loop counts successful writes in a fixed four-element array"
)]
fn create_project_files(
    directory: &Path,
    name: &str,
    template: Template,
    output: &mut dyn Write,
) -> io::Result<usize> {
    let sanitized = sanitize_name(name).map_err(io::Error::other)?;
    let filename = format!("{sanitized}.py");
    let config = serde_json::json!({
        "entry": filename,
        "output": format!("dist/{sanitized}"),
        "assets": [],
        "tests": ["tests"]
    });
    let config = format!("{}\n", serde_json::to_string_pretty(&config)?);
    let example = match template {
        Template::Cli => "kipferl run -- --name Ada",
        Template::Api => "kipferl run -- api.github.com /zen",
        Template::Interactive => "kipferl run",
    };
    let readme = format!(
        "# {name}\n\nBuilt with Kipferl. Edit `{filename}` to make this tool your own.\n\n```sh\nkipferl run -- --help\n{example}\nkipferl dev\nkipferl test\nkipferl build\n./dist/{sanitized} --help\n```\n\n`kipferl.json` selects the entry script, output path, bundled assets, and test directories. Paths are relative to this project. Add application data files or directories to `assets` to ship them alongside your code.\n\nUse `kipferl deps catalog` to inspect package compatibility and `kipferl add <requirement>` to add a PyPI dependency. Commit `kipferl.json` and `kipferl.lock`; restore with `kipferl sync --locked`. Installed packages live in `.kipferl/packages`, which is ignored by Git and included in editor import paths. See https://kipferl.dev/docs/guides/packages for current limits.\n\nTests are ordinary Python scripts named `test_*.py`, with top-level assertions. Each runs in its own interpreter; a failed assertion makes `kipferl test` fail.\n\nIDE autocomplete is configured through `pyrightconfig.json` and `.kipferl/stubs`. The runtime supports a Python subset; see https://kipferl.dev/docs/modules for supported modules.\n"
    );
    let filename_literal = serde_json::to_string(&filename)?;
    let assertion = match template {
        Template::Api => "assert 'ok' in format_payload({'ok': True})",
        _ => "assert greeting('Ada') == 'Hello, Ada!'\nassert greeting('World') == 'Hello, World!'",
    };
    let test = format!(
        "# Run with: kipferl test\n# Loading the app with a module name keeps interactive/network work in main().\n__name__ = 'app_test'\nwith open({filename_literal}, 'r') as source_file:\n    source = source_file.read()\nexec(compile(source, {filename_literal}, 'exec'), globals())\n{assertion}\nprint('Application tests passed')\n"
    );
    fs::create_dir_all(directory.join("tests"))?;
    let mut created = 0;
    for (path, content) in [
        ("kipferl.json", config.as_str()),
        ("README.md", readme.as_str()),
        ("tests/test_app.py", test.as_str()),
        (".gitignore", "dist/\n.kipferl/\n__pycache__/\n"),
    ] {
        created += usize::from(write_if_absent(
            &directory.join(path),
            content,
            path,
            output,
        )?);
    }
    Ok(created)
}

fn write_if_absent(
    path: &Path,
    content: &str,
    display_path: &str,
    output: &mut dyn Write,
) -> io::Result<bool> {
    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            writeln!(
                output,
                "{DIM}-{RESET} {display_path} already exists (skipped)"
            )?;
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    file.write_all(content.as_bytes())?;
    writeln!(output, "{GREEN}+{RESET} Created {display_path}")?;
    Ok(true)
}
