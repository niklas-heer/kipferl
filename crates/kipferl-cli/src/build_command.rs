use crate::encoding::base64_encode;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use kipferl_format::Trailer;
use sha2::{Digest, Sha256};

use crate::tree_shake::{self, RuntimeProfile};
use crate::{bundle, embedded_runtime, run_command};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const HEADER_LINE: &str = "\x1b[2m─────────────────────────────────────────\x1b[0m";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/kipferl-loader-macos-aarch64");
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/kipferl-loader-macos-x86_64");
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/kipferl-loader-linux-x86_64");
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const EMBEDDED_LOADER: &[u8] = include_bytes!("../assets/kipferl-loader-linux-aarch64");
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

    fn host() -> io::Result<Self> {
        Self::parse(run_command::embedded_runtime_target())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "unsupported Kipferl host"))
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

    const fn runtime_filename(self, profile: RuntimeProfile) -> &'static str {
        match (self, profile) {
            (Self::MacosAarch64, RuntimeProfile::Core) => "pocketpy-kipferl-core-macos-aarch64",
            (Self::MacosX86_64, RuntimeProfile::Core) => "pocketpy-kipferl-core-macos-x86_64",
            (Self::LinuxX86_64, RuntimeProfile::Core) => "pocketpy-kipferl-core-linux-x86_64",
            (Self::LinuxAarch64, RuntimeProfile::Core) => "pocketpy-kipferl-core-linux-aarch64",
            (Self::MacosAarch64, RuntimeProfile::Full) => "pocketpy-kipferl-macos-aarch64",
            (Self::MacosX86_64, RuntimeProfile::Full) => "pocketpy-kipferl-macos-x86_64",
            (Self::LinuxX86_64, RuntimeProfile::Full) => "pocketpy-kipferl-linux-x86_64",
            (Self::LinuxAarch64, RuntimeProfile::Full) => "pocketpy-kipferl-linux-aarch64",
        }
    }

    const fn loader_filename(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "kipferl-loader-macos-aarch64",
            Self::MacosX86_64 => "kipferl-loader-macos-x86_64",
            Self::LinuxX86_64 => "kipferl-loader-linux-x86_64",
            Self::LinuxAarch64 => "kipferl-loader-linux-aarch64",
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
    let mut assets = Vec::new();
    let mut output = None;
    let mut mode = Mode::Universal;
    let mut target = None;
    let mut full_runtime = false;
    let mut arguments = arguments.iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-o" | "--output" => {
                let Some(value) = arguments.next() else {
                    return error(stderr, " -o requires an argument");
                };
                output = Some(value.clone());
            }
            "--asset" => {
                let Some(value) = arguments.next() else {
                    return error(
                        stderr,
                        " --asset requires a project-relative file or directory",
                    );
                };
                assets.push(PathBuf::from(value));
            }
            "-m" | "--mode" => {
                let Some(value) = arguments.next() else {
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
                let Some(value) = arguments.next() else {
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
            "--full-runtime" => full_runtime = true,
            "-h" | "--help" => {
                write!(stdout, "{}", help())?;
                return Ok(0);
            }
            option if option.starts_with('-') => {
                return error(stderr, &format!(" Unknown option '{option}'"));
            }
            value if script.is_none() => script = Some(value.to_owned()),
            value => {
                return error(
                    stderr,
                    &format!(" Unexpected argument '{value}'; specify one input script"),
                );
            }
        }
    }

    finish_build(
        BuildOptions {
            script,
            assets,
            output,
            mode,
            target,
            full_runtime,
        },
        current_directory,
        stdout,
        stderr,
    )
}

struct BuildOptions {
    script: Option<String>,
    assets: Vec<PathBuf>,
    output: Option<String>,
    mode: Mode,
    target: Option<Target>,
    full_runtime: bool,
}

fn finish_build(
    options: BuildOptions,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let BuildOptions {
        script,
        assets,
        output,
        mode,
        target,
        full_runtime,
    } = options;
    let Some(script) = script else {
        writeln!(stderr, "{RED}Error:{RESET}  No input script specified")?;
        writeln!(
            stderr,
            "Usage: kipferl build <script.py> -o <output> [--mode <mode>] [--target <target>]"
        )?;
        return Ok(1);
    };
    let Some(output) = output else {
        return error(stderr, " No output path specified (-o)");
    };
    let build_target = match target {
        Some(target) => target,
        None => Target::host()?,
    };
    if full_runtime && mode != Mode::Universal {
        return error(stderr, " --full-runtime requires --mode universal");
    }
    let script_path = current_directory.join(&script);
    if !script_path.exists() {
        return error(stderr, &format!(" Script not found: {script}"));
    }
    let output_path = current_directory.join(&output);

    writeln!(stdout)?;
    writeln!(stdout, "{CYAN}{BOLD}Kipferl build{RESET}")?;
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

    let bundle = match bundle::build(&script_path, &assets) {
        Ok(bundle) => bundle,
        Err(build_error) => return error(stderr, &format!(" Build failed: {build_error}")),
    };
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Bundled {} Python modules and {} assets",
        bundle.module_count, bundle.asset_count
    )?;
    if bundle.has_dependencies && (mode != Mode::Universal || build_target != Target::host()?) {
        writeln!(
            stdout,
            "Dependency compatibility was checked with the embedded {} runtime. Test this artifact on its destination runtime before distributing it.",
            run_command::embedded_runtime_target()
        )?;
    }
    let result = match mode {
        Mode::Single => build_single(&bundle.python, &output_path, stdout),
        Mode::Executable => build_executable(&bundle.python, &output_path, &output, stdout),
        Mode::Universal => build_universal(
            &bundle,
            &output_path,
            &output,
            build_target,
            full_runtime,
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
        "{BOLD}Kipferl build{RESET} - Build standalone binaries from Python scripts\n\n{DIM}USAGE:{RESET}\n    kipferl build <script.py> -o <output> [OPTIONS]\n\n{DIM}OPTIONS:{RESET}\n    -o, --output <path>    Output file path (required)\n    -m, --mode <mode>      Build mode: universal, executable, single\n                           (default: universal)\n    -t, --target <target>  Target platform for cross-compilation\n                           (default: current platform)\n    --full-runtime         Disable tree shaking for universal builds\n    --asset <path>         Bundle a project-relative file/directory (repeatable)\n    --targets              List available targets\n    -h, --help             Show this help\n\n{DIM}TARGETS:{RESET}\n    macos-aarch64          macOS on Apple Silicon\n    macos-x86_64           macOS on Intel\n    linux-x86_64           Linux on x86_64\n    linux-aarch64          Linux on ARM64\n\n{DIM}MODES:{RESET}\n    universal              Tree-shaken standalone binary, no dependencies\n    executable             Shell wrapper (requires pocketpy-kipferl)\n    single                 Transformed .py file (requires pocketpy-kipferl)\n\n{DIM}EXAMPLES:{RESET}\n    kipferl build app.py -o app\n    kipferl build app.py -o app-linux --target linux-x86_64\n    kipferl build app.py -o app-full --full-runtime\n    kipferl build app.py -o app.py --mode single\n"
    )
}

fn build_single(transformed: &[u8], output: &Path, stdout: &mut dyn Write) -> io::Result<()> {
    write_executable(output, &[transformed])?;
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Transformed Python code {DIM}({} bytes){RESET}",
        transformed.len()
    )?;
    writeln!(
        stdout,
        "\n{DIM}Note: Requires pocketpy-kipferl with native modules{RESET}"
    )
}

fn build_executable(
    transformed: &[u8],
    output_path: &Path,
    output_display: &str,
    stdout: &mut dyn Write,
) -> io::Result<()> {
    let encoded = base64_encode(transformed);
    let wrapper = format!(
        "#!/bin/bash\n# Built with Kipferl - https://github.com/niklas-heer/kipferl\n# Requires pocketpy-kipferl with native modules\n\nPOCKETPY=\"pocketpy-kipferl\"\nif ! command -v \"$POCKETPY\" &> /dev/null; then\n    POCKETPY=\"pocketpy\"\n    if ! command -v \"$POCKETPY\" &> /dev/null; then\n        echo \"Error: pocketpy not found\" >&2\n        exit 1\n    fi\nfi\necho \"{encoded}\" | base64 -d | \"$POCKETPY\" /dev/stdin \"$@\"\n"
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

#[expect(
    clippy::too_many_arguments,
    reason = "The final universal-build boundary receives the explicit target/profile, destinations, and separate diagnostic writers without hiding them in global state"
)]
fn build_universal(
    bundle: &bundle::Bundle,
    output_path: &Path,
    output_display: &str,
    target: Target,
    force_full_runtime: bool,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let python = &bundle.python;
    let forced_analysis;
    let analysis = if force_full_runtime {
        forced_analysis = tree_shake::Analysis::forced_full();
        &forced_analysis
    } else {
        &bundle.analysis
    };
    let runtime = runtime_for(target, analysis.profile, current_directory, stdout, stderr)?;
    let loader = loader_for(target, current_directory, stdout, stderr)?;

    writeln!(
        stdout,
        "{GREEN}✓{RESET} Runtime profile {BOLD}{}{RESET}{DIM} ({}){RESET}",
        analysis.profile.name(),
        if analysis.profile == RuntimeProfile::Core {
            "tree-shaken"
        } else {
            "complete compatibility"
        }
    )?;
    for reason in &analysis.reasons {
        writeln!(stdout, "{DIM}  Full runtime: {reason}{RESET}")?;
    }
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Using {BOLD}pocketpy-kipferl{RESET}{DIM} for {} ({} KB){RESET}",
        target.name(),
        runtime.len() / 1024
    )?;
    writeln!(
        stdout,
        "{GREEN}✓{RESET} Selected loader {BOLD}{}{RESET}{DIM} ({} KB){RESET}",
        target.name(),
        loader.len() / 1024
    )?;

    let size = |length| {
        u64::try_from(length).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "component size exceeds the executable format limit",
            )
        })
    };
    let runtime_offset = size(loader.len())?;
    let runtime_size = size(runtime.len())?;
    let python_offset = runtime_offset.checked_add(runtime_size).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "combined component size exceeds the executable format limit",
        )
    })?;
    let trailer = Trailer {
        runtime_offset,
        runtime_size,
        python_offset,
        python_size: size(python.len())?,
    }
    .encode();
    write_executable(
        output_path,
        &[loader.as_ref(), runtime.as_ref(), python, &trailer],
    )?;

    let total_size = [loader.len(), runtime.len(), python.len(), trailer.len()]
        .into_iter()
        .try_fold(0_usize, |total, size| {
            total
                .checked_add(size)
                .ok_or_else(|| io::Error::other("executable size exceeds platform limits"))
        })?;
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
    profile: RuntimeProfile,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<Cow<'static, [u8]>> {
    component_for(
        target,
        target.runtime_filename(profile),
        match profile {
            RuntimeProfile::Core => embedded_runtime::core()?,
            RuntimeProfile::Full => run_command::embedded_runtime()?,
        },
        match profile {
            RuntimeProfile::Core => "tree-shaken PocketPy runtime",
            RuntimeProfile::Full => "full PocketPy runtime",
        },
        match profile {
            RuntimeProfile::Core => "~1-2MB",
            RuntimeProfile::Full => "~5MB",
        },
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

#[expect(
    clippy::too_many_arguments,
    reason = "Component selection receives immutable build context and separate output writers; keeping these explicit avoids mutable shared download state"
)]
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
    if target == Target::host()? {
        if embedded.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("no embedded {description} for this target"),
            ));
        }
        return Ok(Cow::Borrowed(embedded));
    }

    if let Some(directory) = env::var_os("KIPFERL_RUNTIME_DIR") {
        let component = PathBuf::from(directory).join(filename);
        if component.is_file() {
            return fs::read(component).map(Cow::Owned);
        }
    }

    for source_path in [
        current_directory
            .join("crates/kipferl-cli/assets")
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
    if let Some(directory) = env::var_os("KIPFERL_RUNTIME_CACHE_DIR") {
        return PathBuf::from(directory);
    }
    env::var_os("HOME").map_or_else(
        || env::temp_dir().join("kipferl-runtimes"),
        |home_directory| PathBuf::from(home_directory).join(".kipferl/runtimes"),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "The download boundary receives the exact component/version/cache destinations and diagnostic writers so each filesystem write remains reviewable"
)]
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
        "https://github.com/niklas-heer/kipferl/releases/download/v{}/{filename}",
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
    for (byte, pair) in digest.iter_mut().zip(token.as_bytes().chunks_exact(2)) {
        let [high, low] = pair else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid SHA-256 length",
            ));
        };
        let decode = |digit: u8| match digit {
            b'0'..=b'9' => Ok(digit.saturating_sub(b'0')),
            b'a'..=b'f' => Ok(digit.saturating_sub(b'a').saturating_add(10)),
            b'A'..=b'F' => Ok(digit.saturating_sub(b'A').saturating_add(10)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SHA-256 must contain only ASCII hexadecimal digits",
            )),
        };
        *byte = (decode(*high)? << 4) | decode(*low)?;
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    run_command::write_atomically(path, pieces, 0o755)
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
            (b"kipferl".as_slice(), "a2lwZmVybA=="),
            (b"\xff\xff\xff".as_slice(), "////"),
            (b"\x00\xff\x10".as_slice(), "AP8Q"),
            (b"\xff\xff".as_slice(), "//8="),
            (b"\xff".as_slice(), "/w=="),
        ] {
            assert_eq!(base64_encode(input), expected);
        }
    }

    #[test]
    fn rejects_non_ascii_checksums_without_panicking() {
        for token in [
            format!("aé{}", "0".repeat(61)),
            "💥".repeat(16),
            "g".repeat(64),
        ] {
            assert_eq!(token.len(), 64);
            assert_eq!(
                parse_sha256(token.as_bytes())
                    .expect_err("invalid digest")
                    .kind(),
                std::io::ErrorKind::InvalidData
            );
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
