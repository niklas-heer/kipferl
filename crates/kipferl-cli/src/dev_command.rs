use std::collections::HashSet;
use std::ffi::OsStr;
use std::io::{self, IsTerminal, Write};
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecursiveMode, Watcher};

use crate::run_command;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const DEFAULT_DEBOUNCE_MS: u64 = 150;
const LOOP_INTERVAL: Duration = Duration::from_millis(25);
const IGNORED_COMPONENTS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".kipferl",
    ".venv",
    "venv",
    "target",
    "__pycache__",
    "node_modules",
];
const PROJECT_EXTENSIONS: &[&str] = &[
    "py", "pyi", "toml", "json", "yaml", "yml", "xml", "csv", "kdl", "ini", "cfg", "conf", "html",
    "htm", "css", "jinja", "jinja2", "j2",
];

#[derive(Debug, Eq, PartialEq)]
struct Options {
    script: PathBuf,
    script_arguments: Vec<String>,
    watch_paths: Vec<PathBuf>,
    clear: bool,
    debounce: Duration,
}

#[derive(Debug)]
struct WatchTarget {
    requested: PathBuf,
    root: PathBuf,
    exact_file: Option<PathBuf>,
    project_files_only: bool,
}

pub fn execute(
    arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    if arguments
        .first()
        .is_some_and(|value| matches!(value.as_str(), "-h" | "--help"))
    {
        write!(stdout, "{}", help())?;
        return Ok(0);
    }

    let options = match parse_options(arguments) {
        Ok(options) => options,
        Err(message) => {
            writeln!(stderr, "{RED}Error:{RESET} {message}")?;
            writeln!(
                stderr,
                "Usage: kipferl dev [OPTIONS] <script.py> [--] [args...]"
            )?;
            return Ok(1);
        }
    };
    let script_path = current_directory.join(&options.script);
    if !script_path.is_file() {
        writeln!(
            stderr,
            "{RED}Error:{RESET} Script not found: {}",
            options.script.display()
        )?;
        return Ok(1);
    }

    let targets = match resolve_targets(&script_path, &options.watch_paths, current_directory) {
        Ok(targets) => targets,
        Err(message) => {
            writeln!(stderr, "{RED}Error:{RESET} {message}")?;
            return Ok(1);
        }
    };
    let runtime_path = match run_command::prepare_runtime() {
        Ok(path) => path,
        Err(error) => {
            writeln!(
                stderr,
                "{RED}Error:{RESET} Failed to extract pocketpy: {error}"
            )?;
            return Ok(1);
        }
    };

    watch(
        &options,
        &script_path,
        &targets,
        &runtime_path,
        current_directory,
        stdout,
        stderr,
    )
}

fn watch(
    options: &Options,
    script_path: &Path,
    targets: &[WatchTarget],
    runtime_path: &Path,
    current_directory: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(sender).map_err(watch_error)?;
    let mut watched_roots = HashSet::new();
    for target in targets {
        if watched_roots.insert(target.root.clone()) {
            watcher
                .watch(&target.root, RecursiveMode::Recursive)
                .map_err(watch_error)?;
        }
    }

    writeln!(
        stdout,
        "{GREEN}{BOLD}Kipferl dev{RESET} {DIM}watching {}{}",
        targets
            .first()
            .ok_or_else(|| io::Error::other("no watch targets"))?
            .requested
            .display(),
        if targets.len() == 1 {
            String::new()
        } else {
            format!(" and {} more", targets.len().saturating_sub(1))
        }
    )?;
    stdout.flush()?;

    let terminal = TerminalState::capture();
    let mut child = Some(spawn_script(
        runtime_path,
        script_path,
        &options.script_arguments,
        current_directory,
        stdout,
    )?);
    let mut restart_at = None;
    let mut reported_exit = false;

    loop {
        match receiver.recv_timeout(LOOP_INTERVAL) {
            Ok(Ok(event)) if event_requires_restart(&event, targets) => {
                restart_at = Some(Instant::now().checked_add(options.debounce).ok_or_else(
                    || {
                        io::Error::other(
                            "watch debounce deadline exceeds the monotonic clock range",
                        )
                    },
                )?);
            }
            Ok(Err(error)) => {
                writeln!(stderr, "{YELLOW}Watch warning:{RESET} {error}")?;
            }
            Ok(Ok(_)) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                stop_child(&mut child, terminal.as_ref());
                writeln!(
                    stderr,
                    "{RED}Error:{RESET} File watcher stopped unexpectedly"
                )?;
                return Ok(1);
            }
        }

        if let Some(running) = child.as_mut()
            && let Some(status) = running.try_wait()?
        {
            terminal.as_ref().map(TerminalState::restore);
            if !reported_exit {
                report_exit(stdout, status)?;
                reported_exit = true;
            }
            child = None;
        }

        if restart_at.is_some_and(|deadline| Instant::now() >= deadline) {
            restart_at = None;
            stop_child(&mut child, terminal.as_ref());
            if options.clear {
                write!(stdout, "\x1b[2J\x1b[H")?;
            }
            writeln!(stdout, "{CYAN}{BOLD}↻{RESET} Change detected, restarting…")?;
            stdout.flush()?;
            child = match spawn_script(
                runtime_path,
                script_path,
                &options.script_arguments,
                current_directory,
                stdout,
            ) {
                Ok(running) => {
                    reported_exit = false;
                    Some(running)
                }
                Err(error) => {
                    writeln!(
                        stderr,
                        "{YELLOW}Restart failed:{RESET} {error}. Waiting for changes…"
                    )?;
                    reported_exit = true;
                    None
                }
            };
        }
    }
}

fn parse_options(arguments: &[String]) -> Result<Options, String> {
    let mut watch_paths = Vec::new();
    let mut clear = false;
    let mut debounce = Duration::from_millis(DEFAULT_DEBOUNCE_MS);
    let mut arguments = arguments.iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-w" | "--watch" => {
                let path = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a path"))?;
                watch_paths.push(PathBuf::from(path));
            }
            "--clear" => clear = true,
            "--debounce" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--debounce requires milliseconds".to_owned())?;
                let milliseconds = value
                    .parse::<u64>()
                    .map_err(|_| "--debounce must be a non-negative integer".to_owned())?;
                if milliseconds > 60_000 {
                    return Err("--debounce cannot exceed 60000 milliseconds".to_owned());
                }
                debounce = Duration::from_millis(milliseconds);
            }
            "--" => return Err("no script specified before '--'".to_owned()),
            option if option.starts_with('-') => {
                return Err(format!("unknown option '{option}'"));
            }
            script => {
                let mut script_arguments = arguments.cloned().collect::<Vec<_>>();
                if script_arguments.first().is_some_and(|value| value == "--") {
                    script_arguments.remove(0);
                }
                return Ok(Options {
                    script: PathBuf::from(script),
                    script_arguments,
                    watch_paths,
                    clear,
                    debounce,
                });
            }
        }
    }

    Err("no script specified".to_owned())
}

fn resolve_targets(
    script_path: &Path,
    extra_paths: &[PathBuf],
    current_directory: &Path,
) -> Result<Vec<WatchTarget>, String> {
    let script_parent = script_path
        .parent()
        .ok_or_else(|| "script has no parent directory".to_owned())?;
    let mut targets = vec![directory_target(script_parent, true)?];

    for path in extra_paths {
        let requested = current_directory.join(path);
        let metadata = requested
            .metadata()
            .map_err(|error| format!("cannot watch {}: {error}", path.display()))?;
        if metadata.is_dir() {
            targets.push(directory_target(&requested, false)?);
        } else if metadata.is_file() {
            let exact_file = canonical(&requested)?;
            let root = exact_file
                .parent()
                .ok_or_else(|| format!("watch path has no parent: {}", path.display()))?
                .to_owned();
            targets.push(WatchTarget {
                requested: path.clone(),
                root,
                exact_file: Some(exact_file),
                project_files_only: false,
            });
        } else {
            return Err(format!(
                "watch path is not a file or directory: {}",
                path.display()
            ));
        }
    }
    Ok(targets)
}

fn directory_target(path: &Path, project_files_only: bool) -> Result<WatchTarget, String> {
    let root = canonical(path)?;
    Ok(WatchTarget {
        requested: root.clone(),
        root,
        exact_file: None,
        project_files_only,
    })
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("cannot watch {}: {error}", path.display()))
}

fn event_requires_restart(event: &Event, targets: &[WatchTarget]) -> bool {
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }
    event
        .paths
        .iter()
        .any(|path| targets.iter().any(|target| target_matches(path, target)))
}

fn target_matches(path: &Path, target: &WatchTarget) -> bool {
    if let Some(file) = &target.exact_file {
        return path == file;
    }
    path.strip_prefix(&target.root).is_ok_and(|relative| {
        !is_ignored(relative) && (!target.project_files_only || is_project_file(relative))
    })
}

fn is_project_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == ".env")
        || path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| {
                PROJECT_EXTENSIONS
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
            })
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str();
        IGNORED_COMPONENTS
            .iter()
            .any(|ignored| value == OsStr::new(ignored))
            || value == OsStr::new(".DS_Store")
    })
}

fn spawn_script(
    runtime_path: &Path,
    script_path: &Path,
    script_arguments: &[String],
    current_directory: &Path,
    stdout: &mut dyn Write,
) -> io::Result<Child> {
    let transformed_path = run_command::prepare_transformed_script(script_path)?;
    writeln!(
        stdout,
        "{CYAN}→{RESET} Running {BOLD}{}{RESET}",
        script_path.display()
    )?;
    stdout.flush()?;
    Command::new(runtime_path)
        .arg(transformed_path)
        .args(script_arguments)
        .current_dir(current_directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| io::Error::new(error.kind(), format!("failed to start script: {error}")))
}

fn stop_child(child: &mut Option<Child>, terminal: Option<&TerminalState>) {
    if let Some(mut running) = child.take() {
        let _ = running.kill();
        let _ = running.wait();
    }
    if let Some(terminal) = terminal {
        terminal.restore();
    }
}

fn report_exit(stdout: &mut dyn Write, status: ExitStatus) -> io::Result<()> {
    let status = status.code().map_or_else(
        || "from a signal".to_owned(),
        |code| format!("with code {code}"),
    );
    writeln!(
        stdout,
        "{DIM}Process exited {status}. Waiting for changes…{RESET}"
    )?;
    stdout.flush()
}

fn watch_error(error: notify::Error) -> io::Error {
    io::Error::other(error)
}

struct TerminalState(libc::termios);

impl TerminalState {
    fn capture() -> Option<Self> {
        if !io::stdin().is_terminal() {
            return None;
        }
        let mut settings = MaybeUninit::<libc::termios>::uninit();
        // SAFETY: `settings` points to writable storage and stdin is a valid
        // descriptor. A zero return initializes the entire value.
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, settings.as_mut_ptr()) } != 0 {
            return None;
        }
        // SAFETY: the successful `tcgetattr` call initialized `settings`.
        Some(Self(unsafe { settings.assume_init() }))
    }

    fn restore(&self) {
        // SAFETY: the settings were captured from this terminal and remain
        // initialized for the duration of the call.
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw const self.0) };
        eprint!("\x1b[0m\x1b[?25h");
    }
}

pub fn help() -> String {
    format!(
        "{BOLD}Kipferl dev{RESET} - Restart a script when project files change\n\n{DIM}USAGE:{RESET}\n    kipferl dev [OPTIONS] <script.py> [--] [args...]\n\n{DIM}OPTIONS:{RESET}\n    -w, --watch <path>    Watch an additional file or directory\n    --clear               Clear the terminal before each restart\n    --debounce <ms>       Wait for writes to settle (default: 150)\n    -h, --help            Show this help\n\n{DIM}DESCRIPTION:{RESET}\n    Runs the script immediately, then watches its directory recursively.\n    The watcher stays alive when the script exits so the next edit runs it\n    again. Generated, cache, virtual-environment, and VCS paths are ignored.\n\n{DIM}EXAMPLES:{RESET}\n    kipferl dev app.py\n    kipferl dev --clear app.py\n    kipferl dev --watch templates --watch settings.toml app.py -- --verbose\n\n{DIM}DOCS:{RESET}\n    https://kipferl.dev/docs/commands/dev\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{Options, WatchTarget, event_requires_restart, is_ignored, parse_options};
    use notify::{Event, EventKind};
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn parses_options_and_preserves_script_arguments() {
        assert_eq!(
            parse_options(&[
                "--clear".into(),
                "--watch".into(),
                "templates".into(),
                "--debounce".into(),
                "75".into(),
                "app.py".into(),
                "--".into(),
                "--verbose".into(),
            ]),
            Ok(Options {
                script: PathBuf::from("app.py"),
                script_arguments: vec!["--verbose".into()],
                watch_paths: vec![PathBuf::from("templates")],
                clear: true,
                debounce: Duration::from_millis(75),
            })
        );
    }

    #[test]
    fn rejects_incomplete_or_unknown_options() {
        assert_eq!(parse_options(&[]), Err("no script specified".into()));
        assert_eq!(
            parse_options(&["--watch".into()]),
            Err("--watch requires a path".into())
        );
        assert_eq!(
            parse_options(&["--debounce".into(), "soon".into(), "app.py".into()]),
            Err("--debounce must be a non-negative integer".into())
        );
        assert_eq!(
            parse_options(&["--debounce".into(), "60001".into(), "app.py".into()]),
            Err("--debounce cannot exceed 60000 milliseconds".into())
        );
        assert_eq!(
            parse_options(&["--wat".into(), "app.py".into()]),
            Err("unknown option '--wat'".into())
        );
    }

    #[test]
    fn filters_access_and_generated_path_events() {
        let target = WatchTarget {
            requested: PathBuf::from("/project"),
            root: PathBuf::from("/project"),
            exact_file: None,
            project_files_only: true,
        };
        let modified = Event::new(EventKind::Any).add_path(PathBuf::from("/project/app.py"));
        assert!(event_requires_restart(&modified, &[target]));

        let ignored =
            Event::new(EventKind::Any).add_path(PathBuf::from("/project/__pycache__/app.pyc"));
        assert!(is_ignored(&ignored.paths[0]));
        assert!(!event_requires_restart(
            &ignored,
            &[WatchTarget {
                requested: PathBuf::from("/project"),
                root: PathBuf::from("/project"),
                exact_file: None,
                project_files_only: true,
            }]
        ));

        let accessed = Event::new(EventKind::Access(notify::event::AccessKind::Any))
            .add_path(PathBuf::from("/project/app.py"));
        assert!(!event_requires_restart(
            &accessed,
            &[WatchTarget {
                requested: PathBuf::from("/project"),
                root: PathBuf::from("/project"),
                exact_file: None,
                project_files_only: true,
            }]
        ));
    }

    #[test]
    fn exact_file_targets_do_not_restart_for_neighbors() {
        let target = WatchTarget {
            requested: PathBuf::from("settings.toml"),
            root: PathBuf::from("/project"),
            exact_file: Some(PathBuf::from("/project/settings.toml")),
            project_files_only: false,
        };
        let neighbor = Event::new(EventKind::Any).add_path(PathBuf::from("/project/other.toml"));
        assert!(!event_requires_restart(&neighbor, &[target]));
    }

    #[test]
    fn ignored_parent_names_do_not_hide_the_watched_root() {
        let target = WatchTarget {
            requested: PathBuf::from("/target/project"),
            root: PathBuf::from("/target/project"),
            exact_file: None,
            project_files_only: true,
        };
        let modified = Event::new(EventKind::Any).add_path(PathBuf::from("/target/project/app.py"));
        assert!(event_requires_restart(&modified, &[target]));
    }

    #[test]
    fn default_watch_ignores_application_output_but_keeps_project_files() {
        let target = || WatchTarget {
            requested: PathBuf::from("/project"),
            root: PathBuf::from("/project"),
            exact_file: None,
            project_files_only: true,
        };
        let database = Event::new(EventKind::Any).add_path(PathBuf::from("/project/app.db"));
        assert!(!event_requires_restart(&database, &[target()]));
        let module = Event::new(EventKind::Any).add_path(PathBuf::from("/project/lib/helpers.py"));
        assert!(event_requires_restart(&module, &[target()]));
        let config = Event::new(EventKind::Any).add_path(PathBuf::from("/project/config.toml"));
        assert!(event_requires_restart(&config, &[target()]));
        for extension in ["json", "yaml", "yml", "xml", "csv", "kdl", "ini", "cfg"] {
            let config = Event::new(EventKind::Any)
                .add_path(PathBuf::from(format!("/project/config.{extension}")));
            assert!(event_requires_restart(&config, &[target()]));
        }
    }
}
