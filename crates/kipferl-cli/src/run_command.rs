use std::env;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::embedded_runtime;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn embedded_runtime() -> io::Result<&'static [u8]> {
    embedded_runtime::full()
}

pub const fn embedded_runtime_target() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "macos-aarch64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "macos-x86_64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-aarch64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x86_64";
    #[allow(
        unreachable_code,
        reason = "Each supported target returns its cfg-selected constant; this fallback only compiles for other targets"
    )]
    "unsupported"
}

pub fn execute(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some((script, script_arguments)) = arguments.split_first() else {
        writeln!(stderr, "{RED}Error:{RESET} No script specified")?;
        writeln!(stderr, "Usage: kipferl run <script.py> [args...]")?;
        return Ok(1);
    };

    if matches!(script.as_str(), "-h" | "--help") {
        write!(stdout, "{}", help())?;
        return Ok(0);
    }

    let script_path = current_directory.join(script);
    if !script_path.exists() {
        writeln!(stderr, "{RED}Error:{RESET} Script not found: {script}")?;
        return Ok(1);
    }

    let runtime_path = match prepare_runtime() {
        Ok(path) => path,
        Err(error) => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Cannot prepare the embedded runtime: {error}"
            )?;
            writeln!(
                stderr,
                "Check that KIPFERL_CACHE_DIR points to a writable directory."
            )?;
            return Ok(1);
        }
    };
    let transformed_path = match prepare_transformed_script(&script_path) {
        Ok(path) => path,
        Err(error) => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Cannot prepare '{script}': {error}"
            )?;
            return Ok(1);
        }
    };

    let error = Command::new(&runtime_path)
        .arg(transformed_path)
        .args(script_arguments)
        .current_dir(current_directory)
        .exec();
    writeln!(
        stderr,
        "{RED}Error:{RESET} Cannot start runtime '{}': {error}",
        runtime_path.display()
    )?;
    Ok(1)
}

pub fn help() -> String {
    format!(
        "{BOLD}Kipferl run{RESET} - Run a Python script with pocketpy-kipferl\n\n{DIM}USAGE:{RESET}\n    kipferl run <script.py> [args...]\n\n{DIM}ARGUMENTS:{RESET}\n    <script.py>    Python script to run\n    [args...]      Arguments passed to the script\n\n{DIM}DESCRIPTION:{RESET}\n    Runs your Python script using the embedded pocketpy-kipferl\n    interpreter with all native Kipferl modules available.\n\n    The script is automatically transformed to use native modules\n    instead of the kipferl Python package.\n\n{DIM}EXAMPLES:{RESET}\n    kipferl run app.py\n    kipferl run app.py --verbose\n    kipferl run examples/demo.py\n"
    )
}

pub fn prepare_runtime() -> io::Result<PathBuf> {
    let runtime = embedded_runtime()?;
    if runtime.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no embedded runtime for this target",
        ));
    }

    let cache_directory = cache_directory();
    ensure_private_directory(&cache_directory)?;

    let runtime_path = cache_directory.join("pocketpy-kipferl");
    prepare_cached_file(&runtime_path, runtime, 0o755)?;
    Ok(runtime_path)
}

pub fn prepare_transformed_script(script_path: &Path) -> io::Result<PathBuf> {
    let transformed = crate::bundle::development_source(script_path)?;

    let cache_directory = cache_directory();
    ensure_private_directory(&cache_directory)?;

    let transformed_path = cache_directory.join(format!(
        "script-{:016x}.py",
        stable_hash(transformed.as_bytes())
    ));
    prepare_cached_file(&transformed_path, transformed.as_bytes(), 0o600)?;
    Ok(transformed_path)
}

fn cache_directory() -> PathBuf {
    let cache_root = env::var_os("KIPFERL_CACHE_DIR").map_or_else(env::temp_dir, PathBuf::from);
    cache_root.join(format!("kipferl-run-{:016x}", embedded_runtime::full_key()))
}

/// Transform legacy imports in place: generated code occupies the same physical
/// lines as the user's source, so `compile()` preserves useful traceback locations.
pub fn transform_source(source: &str) -> io::Result<String> {
    let masked = code_without_strings(source)?;
    let mut transformed = source.to_owned();
    // The shared lexer identifies import spans even inside inline suites or
    // after semicolons, while skipping comments and all string literals.
    for statement in crate::tree_shake::import_statements(source)
        .into_iter()
        .rev()
    {
        if !statement
            .modules
            .iter()
            .any(|name| name == "kipferl" || name.starts_with("kipferl."))
        {
            continue;
        }
        let span = statement.start..statement.end;
        let original = source
            .get(span.clone())
            .ok_or_else(|| io::Error::other("invalid import source span"))?;
        let code = masked
            .get(span)
            .ok_or_else(|| io::Error::other("invalid masked import span"))?
            .replace("\\\n", " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let mut replacement = legacy_import(&code)?;
        let newline_count = original.bytes().filter(|&byte| byte == b'\n').count();
        // A semicolon belongs to the final physical line of a continued
        // import. Keep it on that line without turning it into a new suite.
        let padding = if source.as_bytes().get(statement.end) == Some(&b';') {
            "\\\n"
        } else {
            "\n"
        };
        replacement.push_str(&padding.repeat(newline_count));
        transformed.replace_range(statement.start..statement.end, &replacement);
    }
    Ok(transformed)
}

fn legacy_import(code: &str) -> io::Result<String> {
    let unsupported = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Unsupported legacy import '{}'. Import native modules such as tui, input, or args directly.",
                code.trim()
            ),
        )
    };
    let Some(rest) = code.strip_prefix("from ") else {
        return Err(unsupported());
    };
    let Some((module, imports)) = rest.split_once(" import") else {
        return Err(unsupported());
    };
    let suffix = module.strip_prefix("kipferl").ok_or_else(unsupported)?;
    let mapped = match suffix {
        "" => None,
        ".components" | ".style" | ".table" | ".tui" => Some("tui"),
        ".input" => Some("input"),
        ".args" => Some("args"),
        _ => return Err(unsupported()),
    };
    let imports = imports
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    if imports == "*" {
        return Ok(mapped.map_or_else(
            || "from tui import *; from input import *".to_owned(),
            |module| format!("from {module} import *"),
        ));
    }
    let mut statements = Vec::new();
    for import in imports
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parts: Vec<_> = import.split_whitespace().collect();
        let (name, alias) = match parts.as_slice() {
            [name] => (*name, None),
            [name, "as", alias] => (*name, Some(*alias)),
            _ => return Err(unsupported()),
        };
        if !valid_identifier(name) || alias.is_some_and(|alias| !valid_identifier(alias)) {
            return Err(unsupported());
        }
        let alias = alias
            .map(|alias| format!(" as {alias}"))
            .unwrap_or_default();
        let module = mapped.or(match name {
            "select" | "multiselect" | "confirm" | "prompt" | "password" => Some("input"),
            "style" | "box" | "rule" | "success" | "error" | "warning" | "info" | "progress"
            | "progress_done" | "spinner" | "spinner_frame" | "visible_len" | "table" => {
                Some("tui")
            }
            _ => None,
        });
        if let Some(module) = module {
            statements.push(format!("from {module} import {name}{alias}"));
        } else if matches!(name, "tui" | "input" | "args" | "ansi" | "term") {
            statements.push(format!("import {name}{alias}"));
        } else {
            return Err(unsupported());
        }
    }
    if statements.is_empty() {
        return Err(unsupported());
    }
    Ok(statements.join("; "))
}

fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && characters.all(|character| character == '_' || character.is_alphanumeric())
}

/// A Python string literal, including arbitrary Unicode and source newlines.
pub fn python_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str("\\x");
                output.push(char::from_digit(u32::from(character) >> 4, 16).unwrap_or('0'));
                output.push(char::from_digit(u32::from(character) & 15, 16).unwrap_or('0'));
            }
            character => output.push(character),
        }
    }
    output.push('\"');
    output
}

// Mask string literals and comments without moving line or byte boundaries.
// In particular, quote state must survive physical lines of triple-quoted and
// backslash-continued strings before import-looking text can be classified.
#[expect(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    reason = "The scanner guards byte accesses with index < len, uses get for lookahead, and masks ranges from the previous cursor through the current bounded cursor"
)]
fn code_without_strings(source: &str) -> io::Result<String> {
    let bytes = source.as_bytes();
    let mut code = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        match bytes[index] {
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            quote @ (b'\'' | b'"') => {
                let triple =
                    bytes.get(index + 1) == Some(&quote) && bytes.get(index + 2) == Some(&quote);
                index += if triple { 3 } else { 1 };
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if triple
                        && bytes.get(index) == Some(&quote)
                        && bytes.get(index + 1) == Some(&quote)
                        && bytes.get(index + 2) == Some(&quote)
                    {
                        index += 3;
                        break;
                    } else if !triple && bytes[index] == quote {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
            }
            _ => {
                index += 1;
                continue;
            }
        }
        for byte in &mut code[start..index] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    // Whole literals/comments are replaced with ASCII, preserving all other
    // complete UTF-8 sequences from the source.
    String::from_utf8(code).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn ensure_private_directory(directory: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true).mode(0o700).create(directory)?;
    if !fs::symlink_metadata(directory)?.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "runtime cache path is not a directory",
        ));
    }
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
}

fn prepare_cached_file(path: &Path, content: &[u8], mode: u32) -> io::Result<()> {
    let valid = match fs::symlink_metadata(path) {
        Ok(metadata) => {
            metadata.file_type().is_file()
                && usize::try_from(metadata.len()).ok() == Some(content.len())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    if valid {
        let mut file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                return write_atomically(path, &[content], mode);
            }
            Err(error) => return Err(error),
        };
        let mut buffer = [0_u8; 8192];
        let mut matches = true;
        for expected in content.chunks(buffer.len()) {
            let actual = buffer
                .get_mut(..expected.len())
                .ok_or_else(|| io::Error::other("cache chunk exceeds read buffer"))?;
            if let Err(error) = file.read_exact(actual) {
                if error.kind() != io::ErrorKind::UnexpectedEof {
                    return Err(error);
                }
                matches = false;
                break;
            }
            if actual != expected {
                matches = false;
                break;
            }
        }
        if matches && file.read(&mut buffer[..1])? == 0 {
            if file.metadata()?.permissions().mode() & 0o777 != mode {
                file.set_permissions(fs::Permissions::from_mode(mode))?;
            }
            return Ok(());
        }
    }

    write_atomically(path, &[content], mode)
}

pub fn write_atomically(destination: &Path, pieces: &[&[u8]], mode: u32) -> io::Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output path has no parent"))?;

    loop {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        // Keep the staging name short even when the output name is near the
        // filesystem's per-component limit.
        let temporary_path = parent.join(format!(
            ".kipferl-output.{}.{}",
            std::process::id(),
            counter
        ));
        let mut temporary = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(mode)
            .open(&temporary_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            for piece in pieces {
                temporary.write_all(piece)?;
            }
            temporary.flush()?;
            temporary.set_permissions(fs::Permissions::from_mode(mode))?;
            drop(temporary);
            fs::rename(&temporary_path, destination)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        return result;
    }
}

pub fn stable_hash(content: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{embedded_runtime, python_string, stable_hash, transform_source};

    #[test]
    fn transforms_legacy_imports_without_moving_source_lines() {
        let source = "from kipferl import (\n    box,\n    prompt as ask,\n)\nraise ValueError('original line 5')\n";
        let result = transform_source(source).expect("transform");
        assert_eq!(
            result,
            "from tui import box; from input import prompt as ask\n\n\n\nraise ValueError('original line 5')\n"
        );
        assert_eq!(source.lines().count(), result.lines().count());
        assert!(transform_source("from kipferl.extra import missing").is_err());
    }

    #[test]
    fn preserves_native_source_strings_and_search_paths() {
        let source = "\nimport sys\nsys.path.insert(0, 'src')\ntext = \"\"\"\nfrom kipferl import (\n\"\"\"\n";
        assert_eq!(transform_source(source).expect("transform"), source);
        assert_eq!(python_string("a\n\"\\é"), "\"a\\n\\\"\\\\é\"");
    }

    #[test]
    fn stable_hash_has_a_golden_vector() {
        assert_eq!(stable_hash(b"kipferl"), 0xb423_e7c9_e3f2_8f40);
    }

    #[test]
    fn embedded_runtime_key_matches_its_content() {
        assert!(!embedded_runtime().expect("embedded runtime").is_empty());
        assert_eq!(
            stable_hash(embedded_runtime().expect("embedded runtime")),
            crate::embedded_runtime::full_key()
        );
    }
}
