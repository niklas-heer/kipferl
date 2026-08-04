use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";

const STUBS: &[(&str, &str)] = &[
    ("ansi.pyi", include_str!("../../../stubs/ansi.pyi")),
    ("args.pyi", include_str!("../../../stubs/args.pyi")),
    ("base64.pyi", include_str!("../../../stubs/base64.pyi")),
    ("tui.pyi", include_str!("../../../stubs/tui.pyi")),
    ("copy.pyi", include_str!("../../../stubs/copy.pyi")),
    ("csv.pyi", include_str!("../../../stubs/csv.pyi")),
    ("datetime.pyi", include_str!("../../../stubs/datetime.pyi")),
    ("fnmatch.pyi", include_str!("../../../stubs/fnmatch.pyi")),
    (
        "functools.pyi",
        include_str!("../../../stubs/functools.pyi"),
    ),
    ("glob.pyi", include_str!("../../../stubs/glob.pyi")),
    ("heapq.pyi", include_str!("../../../stubs/heapq.pyi")),
    ("input.pyi", include_str!("../../../stubs/input.pyi")),
    (
        "itertools.pyi",
        include_str!("../../../stubs/itertools.pyi"),
    ),
    ("logging.pyi", include_str!("../../../stubs/logging.pyi")),
    ("operator.pyi", include_str!("../../../stubs/operator.pyi")),
    ("random.pyi", include_str!("../../../stubs/random.pyi")),
    ("shutil.pyi", include_str!("../../../stubs/shutil.pyi")),
    ("signal.pyi", include_str!("../../../stubs/signal.pyi")),
    (
        "statistics.pyi",
        include_str!("../../../stubs/statistics.pyi"),
    ),
    (
        "subprocess.pyi",
        include_str!("../../../stubs/subprocess.pyi"),
    ),
    ("tempfile.pyi", include_str!("../../../stubs/tempfile.pyi")),
    ("term.pyi", include_str!("../../../stubs/term.pyi")),
    ("textwrap.pyi", include_str!("../../../stubs/textwrap.pyi")),
    ("typing.pyi", include_str!("../../../stubs/typing.pyi")),
];

const PYRIGHT_CONFIG: &str = r#"{
  "include": ["."],
  "exclude": [".ucharm"],
  "stubPath": ".ucharm/stubs",
  "reportMissingImports": false,
  "reportMissingModuleSource": false,
  "pythonVersion": "3.11",
  "typeCheckingMode": "basic"
}
"#;

const APP_TEMPLATE: &str = r##"#!/usr/bin/env python3
"""
__APP_NAME__ - Built with ucharm
"""
from ucharm import box, success, error, warning, info
from ucharm import select, confirm, prompt


def main():
    box(
        "__APP_NAME__\n"
        "Built with ucharm",
        title="Welcome",
        border_color="cyan"
    )
    print()

    choice = select("What would you like to do?", [
        "Say hello",
        "Show status messages",
        "Exit"
    ])

    if choice == "Say hello":
        name = prompt("What's your name?", default="World")
        print()
        success(f"Hello, {name}!")
    elif choice == "Show status messages":
        print()
        success("This is a success message")
        warning("This is a warning message")
        error("This is an error message")
        info("This is an info message")
    else:
        info("Goodbye!")


if __name__ == "__main__":
    main()
"##;

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

pub fn initialize(
    directory: &Path,
    options: &ProjectOptions,
    output: &mut dyn Write,
) -> io::Result<usize> {
    let mut files_created = 0;

    if options.create_app
        && let Some(name) = &options.app_name
    {
        files_created += create_app(directory, name, output)? as usize;
    }

    if options.add_stubs {
        let stub_directory = directory.join(".ucharm/stubs");
        fs::create_dir_all(&stub_directory)?;
        for (name, content) in STUBS {
            fs::write(stub_directory.join(name), content)?;
        }
        writeln!(
            output,
            "{GREEN}+{RESET} Created .ucharm/stubs/ ({} stub files)",
            STUBS.len()
        )?;
        files_created += 1;
        files_created += write_if_absent(
            &directory.join("pyrightconfig.json"),
            PYRIGHT_CONFIG,
            "pyrightconfig.json",
            output,
        )? as usize;
    }

    if let Some(ai) = options.ai {
        if matches!(ai, AiInstructions::Agents | AiInstructions::All) {
            files_created += write_if_absent(
                &directory.join("AGENTS.md"),
                include_str!("../templates/AGENTS.md"),
                "AGENTS.md",
                output,
            )? as usize;
        }
        if matches!(ai, AiInstructions::Claude | AiInstructions::All) {
            files_created += write_if_absent(
                &directory.join("CLAUDE.md"),
                include_str!("../templates/CLAUDE.md"),
                "CLAUDE.md",
                output,
            )? as usize;
        }
        if matches!(ai, AiInstructions::Copilot | AiInstructions::All) {
            let copilot_path = directory.join(".github/copilot-instructions.md");
            fs::create_dir_all(copilot_path.parent().expect("Copilot path has a parent"))?;
            files_created += write_if_absent(
                &copilot_path,
                include_str!("../templates/copilot-instructions.md"),
                ".github/copilot-instructions.md",
                output,
            )? as usize;
        }
    }

    Ok(files_created)
}

fn create_app(directory: &Path, name: &str, output: &mut dyn Write) -> io::Result<bool> {
    let filename = format!("{}.py", sanitize_name(name).map_err(io::Error::other)?);
    let path = directory.join(&filename);
    let content = APP_TEMPLATE.replace("__APP_NAME__", name);
    let created = write_if_absent(&path, &content, &filename, output)?;
    if created {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
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
