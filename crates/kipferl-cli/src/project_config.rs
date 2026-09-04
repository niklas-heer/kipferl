//! Small, explicit project defaults shared by commands and the packager.
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value};

const MAX_CONFIG_SIZE: u64 = 64 * 1024;

#[derive(Debug)]
pub struct ProjectConfig {
    pub root: PathBuf,
    pub entry: PathBuf,
    pub output: PathBuf,
    pub assets: Vec<PathBuf>,
    pub tests: Vec<PathBuf>,
}

pub fn discover(start: &Path) -> io::Result<Option<ProjectConfig>> {
    let start = start.canonicalize()?;
    for root in start.ancestors() {
        let path = root.join("kipferl.json");
        match std::fs::metadata(&path) {
            Ok(metadata) if !metadata.is_file() => {
                return Err(invalid(&format!(
                    "{}: configuration must be a regular file",
                    path.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(io::Error::new(
                    error.kind(),
                    format!("{}: {error}", path.display()),
                ));
            }
        }
        // A nonblocking open also protects against a file becoming a FIFO
        // between the metadata check and open. Check the opened handle again.
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            if !file.metadata()?.is_file() {
                return Err(invalid("configuration must be a regular file"));
            }
            let mut bytes = Vec::new();
            file.take(MAX_CONFIG_SIZE + 1).read_to_end(&mut bytes)?;
            if bytes.len() > 64 * 1024 {
                return Err(invalid("configuration exceeds 64 KiB"));
            }
            let value: Value =
                serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
            let object = value
                .as_object()
                .ok_or_else(|| invalid("expected a JSON object"))?;
            for key in object.keys() {
                if !["entry", "output", "assets", "tests"].contains(&key.as_str()) {
                    return Err(invalid(&format!(
                        "unknown setting '{key}'; use entry, output, assets, or tests"
                    )));
                }
            }
            Ok(ProjectConfig {
                root: root.to_owned(),
                entry: object.get("entry").map_or_else(
                    || Ok(PathBuf::from("app.py")),
                    |value| path_value(value, "entry"),
                )?,
                output: object.get("output").map_or_else(
                    || Ok(PathBuf::from("dist/app")),
                    |value| path_value(value, "output"),
                )?,
                assets: path_list(object, "assets", &[])?,
                tests: path_list(object, "tests", &["tests"])?,
            })
        })();
        return result
            .map(Some)
            .map_err(|e: io::Error| invalid(&format!("{}: {e}", path.display())));
    }
    Ok(None)
}

fn path_value(value: &Value, key: &str) -> io::Result<PathBuf> {
    let text = value
        .as_str()
        .ok_or_else(|| invalid(&format!("'{key}' must contain a path string")))?;
    let path = PathBuf::from(text);
    if text.is_empty()
        || text.contains('\\')
        || text.chars().any(char::is_control)
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
        || !path.components().any(|c| matches!(c, Component::Normal(_)))
    {
        return Err(invalid(&format!(
            "'{key}' must be a nonempty project-relative path without '..'"
        )));
    }
    Ok(path)
}

fn path_list(
    object: &Map<String, Value>,
    key: &str,
    defaults: &[&str],
) -> io::Result<Vec<PathBuf>> {
    match object.get(key) {
        None => Ok(defaults.iter().map(PathBuf::from).collect()),
        Some(Value::Array(values)) => values.iter().map(|value| path_value(value, key)).collect(),
        Some(_) => Err(invalid(&format!(
            "'{key}' must be an array of path strings"
        ))),
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_owned())
}
