use std::ffi::c_int;
use std::path::{Path, PathBuf};

use kipferl_pocketpy_sys as ffi;

use super::{filesystem_core, fnmatch_core};
use crate::native::{
    Arguments, NativeModule, NativeModuleKind, NativeSignature, Value, os_error,
    return_string_list, type_error,
};

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"glob(pathname, root_dir=None, dir_fd=None, recursive=False)",
    callback: glob,
}];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"glob",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: &[],
    type_aliases: &[],
    initializer: None,
};

unsafe extern "C" fn glob(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    let Some(pattern) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"pattern must be a string");
    };
    let recursive = arguments.get(3).and_then(Value::boolean).unwrap_or(false);
    let mut results = Vec::new();

    if recursive && let Some(marker) = pattern.find("**") {
        let base = pattern[..marker].trim_end_matches('/');
        let tail = pattern[marker + 2..].trim_start_matches('/');
        let name_pattern = tail.rsplit('/').next().unwrap_or(tail);
        let root = if base.is_empty() { "." } else { base };
        if walk(Path::new(root), name_pattern, 0, &mut results).is_err() {
            return os_error(c"failed to walk directory");
        }
        results.sort();
        return return_string_list(&results);
    }

    let (directory, name_pattern) = match pattern.rsplit_once('/') {
        Some(("", name)) => ("/", name),
        Some((directory, name)) => (directory, name),
        None => (".", pattern.as_str()),
    };
    let Ok(entries) = std::fs::read_dir(directory) else {
        return return_string_list(&[]);
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if !fnmatch_core::matches(name_pattern, &name) {
            continue;
        }
        let path = if directory == "/" {
            PathBuf::from(format!("/{name}"))
        } else {
            Path::new(directory).join(name)
        };
        if let Some(path) = filesystem_core::path_string(&path) {
            results.push(path);
        }
    }
    results.sort();
    return_string_list(&results)
}

fn walk(
    directory: &Path,
    name_pattern: &str,
    depth: usize,
    results: &mut Vec<String>,
) -> std::io::Result<()> {
    if depth > 64 {
        return Ok(());
    }
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            walk(&path, name_pattern, depth + 1, results)?;
        } else if file_type.is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| fnmatch_core::matches(name_pattern, name))
            && let Some(path) = filesystem_core::path_string(&path)
        {
            results.push(path);
        }
    }
    Ok(())
}
