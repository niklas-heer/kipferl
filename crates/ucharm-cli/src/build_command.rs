use std::borrow::Cow;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};
use ucharm_format::Trailer;

use crate::run_command;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const HEADER_LINE: &str = "\x1b[2m─────────────────────────────────────────\x1b[0m";
const MAX_SCRIPT_SIZE: usize = 1024 * 1024;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/loader-macos-aarch64");
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/loader-macos-x86_64");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/loader-linux-x86_64");
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/loader-linux-aarch64");
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
const EMBEDDED_LOADER: &[u8] = &[];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Single,
    Executable,
    Universal,
}

impl Mode {
    const fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Executable => "executable",
            Self::Universal => "universal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    MacosAarch64,
    MacosX86_64,
    LinuxX86_64,
    LinuxAarch64,
}

impl Target {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "macos-aarch64" | "macos-arm64" => Some(Self::MacosAarch64),
            "macos-x86_64" | "macos-amd64" => Some(Self::MacosX86_64),
            "linux-x86_64" | "linux-amd64" => Some(Self::LinuxX86_64),
            "linux-aarch64" | "linux-arm64" => Some(Self::LinuxAarch64),
            _ => None,
        }
    }

    fn host() -> Self {
        Self::parse(run_command::embedded_runtime_target())
            .expect("the Rust CLI only builds for supported release targets")
    }

    const fn name(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "macos-aarch64",
            Self::MacosX86_64 => "macos-x86_64",
            Self::LinuxX86_64 => "linux-x86_64",
            Self::LinuxAarch64 => "linux-aarch64",
        }
    }

    const fn display_name(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "macOS (Apple Silicon)",
            Self::MacosX86_64 => "macOS (Intel)",
            Self::LinuxX86_64 => "Linux (x86_64)",
            Self::LinuxAarch64 => "Linux (ARM64)",
        }
    }

    const fn runtime_filename(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "pocketpy-ucharm-macos-aarch64",
            Self::MacosX86_64 => "pocketpy-ucharm-macos-x86_64",
            Self::LinuxX86_64 => "pocketpy-ucharm-linux-x86_64",
            Self::LinuxAarch64 => "pocketpy-ucharm-linux-aarch64",
        }
    }

    const fn loader_filename(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "loader-macos-aarch64",
            Self::MacosX86_64 => "loader-macos-x86_64",
            Self::LinuxX86_64 => "loader-linux-x86_64",
            Self::LinuxAarch64 => "loader-linux-aarch64",
        }
    }
}

pub fn execute(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut script = None;
    let mut output = None;
    let mut mode = Mode::Universal;
    let mut target = None;
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "-o" | "--output" => {
                index += 1;
                let Some(value) = arguments.get(index) else {
                    return error(stderr, " -o requires an argument");
                };
                output = Some(value.clone());
            }
            "-m" | "--mode" => {
                index += 1;
                let Some(value) = arguments.get(index) else {
                    return error(stderr, " --mode requires an argument");
                };
                mode = match value.as_str() {
                    "single" => Mode::Single,
                    "executable" => Mode::Executable,
                    "universal" => Mode::Universal,
                    unknown => {
                        return error(
                            stderr,
                            &format!(
                                " Unknown mode '{unknown}'. Use: single, executable, universal"
                            ),
                        );
                    }
                };
            }
            "-t" | "--target" => {
                index += 1;
                let Some(value) = arguments.get(index) else {
                    return error(stderr, " --target requires an argument");
                };
                let Some(parsed) = Target::parse(value) else {
                    writeln!(stderr, "{RED}Error:{RESET}  Unknown target '{value}'")?;
                    write!(stderr, "\n{}", targets_text())?;
                    return Ok(1);
                };
                target = Some(parsed);
            }
            "--targets" => {
                write!(stdout, "{}", targets_text())?;
                return Ok(0);
            }
            "-h" | "--help" => {
                write!(stdout, "{}", help())?;
                return Ok(0);
            }
            option if option.starts_with('-') => {
                return error(stderr, &format!(" Unknown option '{option}'"));
            }
            value => script = Some(value.to_owned()),
        }
        index += 1;
    }

    let Some(script) = script else {
        writeln!(stderr, "{RED}Error:{RESET}  No input script specified")?;
        writeln!(
            stderr,
            "Usage: ucharm build <script.py> -o <output> [--mode <mode>] [--target <target>]"
        )?;
        return Ok(1);
    };
    let Some(output) = output else {
        return error(stderr, " No output path specified (-o)");
    };
    let build_target = target.unwrap_or_else(Target::host);
    let script_path = current_directory.join(&script);
    if !script_path.exists() {
        return error(stderr, &format!(" Script not found: {script}"));
    }
    let output_path = current_directory.join(&output);

    writeln!(stdout)?;
    writeln!(stdout, "{CYAN}{BOLD}μcharm build{RESET}")?;
    writeln!(stdout, "{HEADER_LINE}")?;
    writeln!(stdout, "{DIM}  Input:  {RESET}{script}")?;
    writeln!(stdout, "{DIM}  Output: {RESET}{output}")?;
    writeln!(stdout, "{DIM}  Mode:   {RESET}{CYAN}{}{RESET}", mode.name())?;
    if mode == Mode::Universal {
        writeln!(
            stdout,
            "{DIM}  Target: {RESET}{CYAN}{}{RESET}{DIM} ({}){RESET}",
            build_target.name(),
            build_target.display_name()
        )?;
    }
    writeln!(stdout, "{HEADER_LINE}\n")?;

    let result = match mode {
        Mode::Single => build_single(&script_path, &output_path, stdout),
        Mode::Executable => build_executable(&script_path, &output_path, &output, stdout),
        Mode::Universal => build_universal(
            &script_path,
            &output_path,
            &output,
            build_target,
            current_directory,
            stdout,
            stderr,
        ),
    };
    match result {
        Ok(()) => Ok(0),
        Err(build_error) => {
            writeln!(stderr, "{RED}Error:{RESET}  Build failed: {build_error}")?;
            Ok(1)
        }
    }
}

fn error(stderr: &mut dyn Write, message: &str) -> io::Result<u8> {
    writeln!(stderr, "{RED}Error:{RESET} {message}")?;
    Ok(1)
}

fn targets_text() -> String {
    format!(
        "Available targets:\n  macos-aarch64  {DIM}(macOS Apple Silicon){RESET}\n  macos-x86_64   {DIM}(macOS Intel){RESET}\n  linux-x86_64   {DIM}(Linux x86_64){RESET}\n  linux-aarch64  {DIM}(Linux ARM64){RESET}\n"
    )
}

pub fn help() -> String {
    format!(
        "{BOLD}μcharm build{RESET} - Build standalone binaries from Python scripts\n\n{DIM}USAGE:{RESET}\n    ucharm build <script.py> -o <output> [OPTIONS]\n\n{DIM}OPTIONS:{RESET}\n    -o, --output <path>    Output file path (required)\n    -m, --mode <mode>      Build mode: universal, executable, single\n                           (default: universal)\n    -t, --target <target>  Target platform for cross-compilation\n                           (default: current platform)\n    --targets              List available targets\n    -h, --help             Show this help\n\n{DIM}TARGETS:{RESET}\n    macos-aarch64          macOS on Apple Silicon\n    macos-x86_64           macOS on Intel\n    linux-x86_64           Linux on x86_64\n    linux-aarch64          Linux on ARM64\n\n{DIM}MODES:{RESET}\n    universal              Standalone binary (~4-5MB, no dependencies)\n    executable             Shell wrapper (requires pocketpy-ucharm)\n    single                 Transformed .py file (requires pocketpy-ucharm)\n\n{DIM}EXAMPLES:{RESET}\n    ucharm build app.py -o app\n    ucharm build app.py -o app-linux --target linux-x86_64\n    ucharm build app.py -o app.py --mode single\n"
    )
}

fn transform_script(script_path: &Path) -> io::Result<Vec<u8>> {
    let source = fs::read(script_path)?;
    if source.len() > MAX_SCRIPT_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::FileTooLarge,
            "script exceeds the 1 MiB limit",
        ));
    }
    let source = String::from_utf8(source)
        .map_err(|invalid| io::Error::new(io::ErrorKind::InvalidData, invalid))?;
    let mut needs_tui = false;
    let mut needs_input = false;

    for line in source.split('\n') {
        let trimmed = line.trim_matches([' ', '\t']);
        if let Some(imports) = trimmed.strip_prefix("from ucharm import") {
            needs_tui |= contains_any(
                imports,
                &[
                    "style", "box", "rule", "success", "error", "warning", "info", "progress",
                ],
            );
            needs_input |= contains_any(
                imports,
                &["select", "multiselect", "confirm", "prompt", "password"],
            );
        } else if trimmed.starts_with("from ucharm.input import") {
            needs_input = true;
        } else if trimmed.starts_with("from ucharm.components import")
            || trimmed.starts_with("from ucharm.style import")
            || trimmed.starts_with("from ucharm.table import")
        {
            needs_tui = true;
        } else if trimmed.starts_with("import ucharm") {
            needs_tui = true;
            needs_input = true;
        }
    }

    let mut transformed = String::with_capacity(source.len() + 256);
    transformed.push_str("#!/usr/bin/env pocketpy-ucharm\n");
    transformed.push_str("# Built with ucharm - native modules edition\n\n");
    if needs_tui {
        transformed.push_str(
            "from tui import style, box, rule, success, error, warning, info, progress\n",
        );
    }
    if needs_input {
        transformed.push_str("from input import select, multiselect, confirm, prompt, password\n");
    }
    if needs_tui || needs_input {
        transformed.push('\n');
    }

    for line in source.split('\n') {
        let trimmed = line.trim_matches([' ', '\t']);
        if trimmed.starts_with("from ucharm import")
            || trimmed.starts_with("import ucharm")
            || trimmed.starts_with("from ucharm.")
        {
            continue;
        }
        if line.contains("sys.path") && line.contains("ucharm") {
            continue;
        }
        transformed.push_str(line);
        transformed.push('\n');
    }
    Ok(transformed.into_bytes())
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn build_single(script: &Path, output: &Path, stdout: &mut dyn Write) -> io::Result<()> {
    let transformed = transform_script(script)?;
    write_executable(output, &[&transformed])?;
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Transformed Python code {DIM}({} bytes){RESET}",
        transformed.len()
    )?;
    writeln!(
        stdout,
        "\n{DIM}Note: Requires pocketpy-ucharm with native modules{RESET}"
    )
}

fn build_executable(
    script: &Path,
    output_path: &Path,
    output_display: &str,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    let transformed = transform_script(script)?;
    let encoded = base64_encode(&transformed);
    let wrapper = format!(
        "#!/bin/bash\n# Built with μcharm - https://github.com/ucharmdev/ucharm\n# Requires pocketpy-ucharm with native modules\n\nPOCKETPY=\"pocketpy-ucharm\"\nif ! command -v \"$POCKETPY\" &> /dev/null; then\n    POCKETPY=\"pocketpy\"\n    if ! command -v \"$POCKETPY\" &> /dev/null; then\n        echo \"Error: pocketpy not found\" >&2\n        exit 1\n    fi\nfi\necho \"{encoded}\" | base64 -d | \"$POCKETPY\" /dev/stdin \"$@\"\n"
    );
    write_executable(output_path, &[wrapper.as_bytes()])?;

    writeln!(
        stdout,
        "{GREEN}✓{RESET} Created shell wrapper {DIM}({} bytes){RESET}",
        wrapper.len()
    )?;
    writeln!(stdout, "\n{HEADER_LINE}")?;
    writeln!(stdout, "{GREEN}{BOLD}✓ Built successfully!{RESET}")?;
    write_run_hint(stdout, output_display, false)
}

#[allow(clippy::too_many_arguments)]
fn build_universal(
    script: &Path,
    output_path: &Path,
    output_display: &str,
    target: Target,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let python = transform_script(script)?;
    let runtime = runtime_for(target, current_directory, stdout, stderr)?;
    let loader = loader_for(target, current_directory, stdout, stderr)?;

    writeln!(
        stdout,
        "{GREEN}✓{RESET} Using {BOLD}pocketpy-ucharm{RESET}{DIM} for {} ({} KB){RESET}",
        target.name(),
        runtime.len() / 1024
    )?;
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Selected loader {BOLD}{}{RESET}{DIM} ({} KB){RESET}",
        target.name(),
        loader.len() / 1024
    )?;

    let runtime_offset = loader.len() as u64;
    let runtime_size = runtime.len() as u64;
    let python_offset = runtime_offset + runtime_size;
    let trailer = Trailer {
        runtime_offset,
        runtime_size,
        python_offset,
        python_size: python.len() as u64,
    }
    .encode();
    write_executable(
        output_path,
        &[loader.as_ref(), runtime.as_ref(), &python, &trailer],
    )?;

    let total_size = loader.len() + runtime.len() + python.len() + trailer.len();
    let total_kb = total_size / 1024;
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Wrote universal binary {DIM}({total_kb} KB){RESET}"
    )?;
    writeln!(stdout, "\n{HEADER_LINE}")?;
    writeln!(stdout, "{GREEN}{BOLD}✓ Built successfully!{RESET}")?;
    writeln!(stdout, "{DIM}  Output:  {RESET}{output_display}")?;
    writeln!(stdout, "{DIM}  Target:  {RESET}{}", target.display_name())?;
    writeln!(
        stdout,
        "{DIM}  Size:    {RESET}{total_kb} KB {DIM}(standalone, no dependencies){RESET}"
    )?;
    writeln!(stdout, "{DIM}  Startup: {RESET}~8ms {DIM}(instant){RESET}")?;
    write_run_hint(stdout, output_display, true)
}

fn write_run_hint(stdout: &mut dyn Write, output: &str, leading_blank: bool) -> io::Result<()> {
    if leading_blank {
        writeln!(stdout)?;
    }
    if Path::new(output).is_absolute() {
        writeln!(stdout, "{DIM}  Run with: {RESET}{output}\n")
    } else {
        writeln!(stdout, "{DIM}  Run with: {RESET}./{output}\n")
    }
}

fn runtime_for(
    target: Target,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<Cow<'static, [u8]>> {
    component_for(
        target,
        target.runtime_filename(),
        run_command::embedded_runtime(),
        "PocketPy runtime",
        "~4MB",
        current_directory,
        stdout,
        stderr,
    )
}

fn loader_for(
    target: Target,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<Cow<'static, [u8]>> {
    component_for(
        target,
        target.loader_filename(),
        EMBEDDED_LOADER,
        "universal loader",
        "~450KB",
        current_directory,
        stdout,
        stderr,
    )
}

#[allow(clippy::too_many_arguments)]
fn component_for(
    target: Target,
    filename: &'static str,
    embedded: &'static [u8],
    description: &str,
    download_size: &str,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<Cow<'static, [u8]>> {
    if target == Target::host() {
        if embedded.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("no embedded {description} for this target"),
            ));
        }
        return Ok(Cow::Borrowed(embedded));
    }

    if let Some(directory) = env::var_os("UCHARM_RUNTIME_DIR") {
        let component = PathBuf::from(directory).join(filename);
        if component.is_file() {
            return fs::read(component).map(Cow::Owned);
        }
    }

    for source_path in [
        current_directory
            .join("crates/ucharm-cli/assets")
            .join(filename),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(filename),
    ] {
        if source_path.is_file() {
            return fs::read(source_path).map(Cow::Owned);
        }
    }

    let cache_directory = runtime_cache_directory();
    let component_path = cache_directory.join(filename);
    let version_path = cache_directory.join(format!("{filename}.version"));
    if component_path.is_file()
        && fs::read_to_string(&version_path).is_ok_and(|cached| cached.trim() == crate::version())
    {
        return fs::read(component_path).map(Cow::Owned);
    }

    download_component(
        target,
        filename,
        description,
        download_size,
        &cache_directory,
        &component_path,
        &version_path,
        stdout,
        stderr,
    )
    .map(Cow::Owned)
}

fn runtime_cache_directory() -> PathBuf {
    if let Some(directory) = env::var_os("UCHARM_RUNTIME_CACHE_DIR") {
        return PathBuf::from(directory);
    }
    env::var_os("HOME").map_or_else(
        || env::temp_dir().join("ucharm-runtimes"),
        |home_directory| PathBuf::from(home_directory).join(".ucharm/runtimes"),
    )
}

#[allow(clippy::too_many_arguments)]
fn download_component(
    target: Target,
    filename: &str,
    description: &str,
    download_size: &str,
    cache_directory: &Path,
    component_path: &Path,
    version_path: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<Vec<u8>> {
    let url = format!(
        "https://github.com/ucharmdev/ucharm/releases/download/v{}/{filename}",
        crate::version()
    );
    let checksum_url = format!("{url}.sha256");

    writeln!(
        stdout,
        "{YELLOW}?{RESET} {description} for {BOLD}{}{RESET} not found locally.",
        target.name()
    )?;
    write!(
        stdout,
        "  Download version {BOLD}{}{RESET} from GitHub? {DIM}({download_size}){RESET} [Y/n] ",
        crate::version(),
    )?;
    stdout.flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if matches!(answer.trim().chars().next(), Some('n' | 'N')) {
        writeln!(stdout, "\n{DIM}To download manually:{RESET}")?;
        writeln!(stdout, "  mkdir -p {}", cache_directory.display())?;
        writeln!(stdout, "  curl -L {url} -o {}", component_path.display())?;
        writeln!(
            stdout,
            "  echo '{}' > {}\n",
            crate::version(),
            version_path.display()
        )?;
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{description} download declined"),
        ));
    }

    ensure_private_directory(cache_directory)?;
    let checksum_output = Command::new("curl")
        .args(["-fsSL", &checksum_url])
        .output()
        .map_err(|failure| {
            io::Error::new(failure.kind(), format!("failed to run curl: {failure}"))
        })?;
    if !checksum_output.status.success() {
        writeln!(
            stderr,
            "{RED}Error:{RESET}  Failed to fetch {description} checksum"
        )?;
        return Err(io::Error::other("checksum download failed"));
    }
    let expected = parse_sha256(&checksum_output.stdout)?;
    let temporary_path = component_path.with_extension(format!("download.{}", std::process::id()));
    let status = Command::new("curl")
        .args(["-fSL", "--progress-bar", "-o"])
        .arg(&temporary_path)
        .arg(&url)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&temporary_path);
        return Err(io::Error::other(format!("{description} download failed")));
    }

    let downloaded = fs::read(&temporary_path)?;
    let actual: [u8; 32] = Sha256::digest(&downloaded).into();
    if actual != expected {
        let _ = fs::remove_file(&temporary_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("checksum mismatch for {filename}"),
        ));
    }
    fs::set_permissions(&temporary_path, fs::Permissions::from_mode(0o755))?;
    fs::rename(&temporary_path, component_path)?;
    fs::write(version_path, format!("{}\n", crate::version()))?;
    Ok(downloaded)
}

fn parse_sha256(output: &[u8]) -> io::Result<[u8; 32]> {
    let output = std::str::from_utf8(output)
        .map_err(|invalid| io::Error::new(io::ErrorKind::InvalidData, invalid))?;
    let token = output
        .split_whitespace()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "empty checksum response"))?;
    if token.len() != 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid SHA-256 length",
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&token[index * 2..index * 2 + 2], 16)
            .map_err(|invalid| io::Error::new(io::ErrorKind::InvalidData, invalid))?;
    }
    Ok(digest)
}

fn ensure_private_directory(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    if !fs::symlink_metadata(directory)?.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "runtime cache path is not a directory",
        ));
    }
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
}

fn write_executable(path: &Path, pieces: &[&[u8]]) -> io::Result<()> {
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o755)
        .open(path)?;
    for piece in pieces {
        output.write_all(piece)?;
    }
    output.flush()?;
    output.set_permissions(fs::Permissions::from_mode(0o755))
}

fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{Target, base64_encode, parse_sha256};

    #[test]
    fn target_aliases_are_stable() {
        assert_eq!(Target::parse("macos-arm64"), Some(Target::MacosAarch64));
        assert_eq!(Target::parse("linux-amd64"), Some(Target::LinuxX86_64));
        assert_eq!(Target::parse("windows-x86_64"), None);
    }

    #[test]
    fn base64_vectors_match_the_standard_encoding() {
        for (input, expected) in [
            (b"".as_slice(), ""),
            (b"f".as_slice(), "Zg=="),
            (b"fo".as_slice(), "Zm8="),
            (b"foo".as_slice(), "Zm9v"),
            (b"ucharm".as_slice(), "dWNoYXJt"),
        ] {
            assert_eq!(base64_encode(input), expected);
        }
    }

    #[test]
    fn parses_release_checksum_assets() {
        let digest = parse_sha256(
            b"ea817b8b2879159f7ced0d4b590f0e2eb8938d9fb24539aa7293e2aedf77f6dc  runtime\n",
        )
        .expect("parse checksum");
        assert_eq!(digest[0..4], [0xea, 0x81, 0x7b, 0x8b]);
        assert!(parse_sha256(b"invalid\n").is_err());
    }
}
