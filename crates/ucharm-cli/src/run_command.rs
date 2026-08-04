use std::env;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const MAX_SCRIPT_SIZE: usize = 1024 * 1024;
const TRANSFORM_HEADER: &str = "#!/usr/bin/env pocketpy-ucharm\n\
# Transformed by ucharm run\n\n\
from charm import style, box, rule, success, error, warning, info, progress, spinner_frame, visible_len\n\
from input import select, multiselect, confirm, prompt, password\n\n\
# Stubs for functions not yet in native modules\n\
def spinner(msg, duration=1): pass\n\
def table(data, headers=None, header_style=None): pass\n\
def key_value(data): pass\n\
class Color: pass\n\n";

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const EMBEDDED_RUNTIME: &[u8] =
    include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-macos-aarch64");
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const EMBEDDED_RUNTIME_KEY: u64 = 0x7028_bea1_39aa_8ff5;
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const EMBEDDED_RUNTIME: &[u8] =
    include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-macos-x86_64");
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const EMBEDDED_RUNTIME_KEY: u64 = 0xcdea_1db9_0a87_1b1b;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const EMBEDDED_RUNTIME: &[u8] =
    include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-linux-aarch64");
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const EMBEDDED_RUNTIME_KEY: u64 = 0x5058_28b2_5c29_5d0f;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const EMBEDDED_RUNTIME: &[u8] =
    include_bytes!("../../../cli/src/stubs/pocketpy-ucharm-linux-x86_64");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const EMBEDDED_RUNTIME_KEY: u64 = 0xf398_30f6_c5da_6c0c;
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
const EMBEDDED_RUNTIME: &[u8] = &[];
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
const EMBEDDED_RUNTIME_KEY: u64 = 0;

pub(crate) fn embedded_runtime() -> &'static [u8] {
    EMBEDDED_RUNTIME
}

pub(crate) fn embedded_runtime_target() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "macos-aarch64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "macos-x86_64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-aarch64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x86_64";
    #[allow(unreachable_code)]
    "unsupported"
}

pub fn execute(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some(script) = arguments.first() else {
        writeln!(stderr, "{RED}Error:{RESET} No script specified")?;
        writeln!(stderr, "Usage: ucharm run <script.py> [args...]")?;
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
        Err(_) => {
            writeln!(stderr, "{RED}Error:{RESET} Failed to extract pocketpy")?;
            return Ok(1);
        }
    };
    let transformed_path = match prepare_transformed_script(&script_path) {
        Ok(path) => path,
        Err(_) => {
            writeln!(stderr, "{RED}Error:{RESET} Failed to transform script")?;
            return Ok(1);
        }
    };

    let _error = Command::new(runtime_path)
        .arg(transformed_path)
        .args(&arguments[1..])
        .current_dir(current_directory)
        .exec();
    writeln!(stderr, "{RED}Error:{RESET} Failed to exec pocketpy")?;
    Ok(1)
}

pub fn help() -> String {
    format!(
        "{BOLD}μcharm run{RESET} - Run a Python script with pocketpy-ucharm\n\n{DIM}USAGE:{RESET}\n    ucharm run <script.py> [args...]\n\n{DIM}ARGUMENTS:{RESET}\n    <script.py>    Python script to run\n    [args...]      Arguments passed to the script\n\n{DIM}DESCRIPTION:{RESET}\n    Runs your Python script using the embedded pocketpy-ucharm\n    interpreter with all native μcharm modules available.\n\n    The script is automatically transformed to use native modules\n    instead of the ucharm Python package.\n\n{DIM}EXAMPLES:{RESET}\n    ucharm run app.py\n    ucharm run app.py --verbose\n    ucharm run examples/demo.py\n"
    )
}

fn prepare_runtime() -> io::Result<PathBuf> {
    if EMBEDDED_RUNTIME.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no embedded runtime for this target",
        ));
    }

    let cache_directory = cache_directory();
    ensure_private_directory(&cache_directory)?;

    let runtime_path = cache_directory.join("pocketpy-ucharm");
    prepare_cached_file(&runtime_path, EMBEDDED_RUNTIME, 0o755)?;
    Ok(runtime_path)
}

fn prepare_transformed_script(script_path: &Path) -> io::Result<PathBuf> {
    let source = fs::read(script_path)?;
    if source.len() > MAX_SCRIPT_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "script exceeds the 1 MiB limit",
        ));
    }
    let source = String::from_utf8(source)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let transformed = transform_script(&source);

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
    let cache_root = env::var_os("UCHARM_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    cache_root.join(format!("ucharm-run-{EMBEDDED_RUNTIME_KEY:016x}"))
}

fn transform_script(source: &str) -> String {
    let mut transformed = String::with_capacity(TRANSFORM_HEADER.len() + source.len());
    transformed.push_str(TRANSFORM_HEADER);

    let mut in_multiline_import = false;
    for line in source.split('\n') {
        let trimmed = line.trim_matches([' ', '\t']);

        if in_multiline_import {
            if line.contains(')') {
                in_multiline_import = false;
            }
            continue;
        }

        if trimmed.starts_with("from ucharm import")
            || trimmed.starts_with("from ucharm.")
            || trimmed.starts_with("import ucharm")
        {
            if line.contains('(') && !line.contains(')') {
                in_multiline_import = true;
            }
            continue;
        }
        if line.contains("sys.path") {
            continue;
        }

        transformed.push_str(line);
        transformed.push('\n');
    }
    transformed
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
        Ok(metadata) => metadata.file_type().is_file() && metadata.len() == content.len() as u64,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    if valid {
        if fs::metadata(path)?.permissions().mode() & 0o777 != mode {
            fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        }
        return Ok(());
    }

    write_atomically(path, content, mode)
}

fn write_atomically(destination: &Path, content: &[u8], mode: u32) -> io::Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "cache path has no parent"))?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid cache filename"))?;

    loop {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(".{name}.{}.{}", std::process::id(), counter));
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
            temporary.write_all(content)?;
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

fn stable_hash(content: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{
        EMBEDDED_RUNTIME, EMBEDDED_RUNTIME_KEY, TRANSFORM_HEADER, stable_hash, transform_script,
    };

    #[test]
    fn transforms_imports_like_the_zig_cli() {
        let source = "#!/usr/bin/env python3\n\
import sys\n\
sys.path.insert(0, 'src')\n\
from ucharm import (\n\
    box,\n\
    success,\n\
)\n\
from ucharm.extra import thing\n\
import ucharm\n\
print(sys.argv)\n";

        assert_eq!(
            transform_script(source),
            format!("{TRANSFORM_HEADER}#!/usr/bin/env python3\nimport sys\nprint(sys.argv)\n\n")
        );
    }

    #[test]
    fn stable_hash_has_a_golden_vector() {
        assert_eq!(stable_hash(b"ucharm"), 0x8188_f7cc_f53b_1e53);
    }

    #[test]
    fn embedded_runtime_key_matches_its_content() {
        assert!(!EMBEDDED_RUNTIME.is_empty());
        assert_eq!(stable_hash(EMBEDDED_RUNTIME), EMBEDDED_RUNTIME_KEY);
    }
}
