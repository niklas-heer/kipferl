//! Dependency discovery and portable Python payloads for the universal loader.
use crate::encoding::base64_encode;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

use crate::{dependencies, project_config, run_command, tree_shake};

const MAX_FILES: usize = 1024;
const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;
const HELPER: &str = "__kipferl_bundle_runtime";

pub struct Bundle {
    pub(crate) python: Vec<u8>,
    pub(crate) analysis: tree_shake::Analysis,
    pub(crate) module_count: usize,
    pub(crate) asset_count: usize,
    pub(crate) has_dependencies: bool,
}

struct Module {
    path: PathBuf,
    import_root: PathBuf,
    source: String,
}

struct Collector {
    root: PathBuf,
    import_roots: Vec<PathBuf>,
    entry: PathBuf,
    modules: BTreeMap<String, Module>,
    assets: BTreeMap<PathBuf, Vec<u8>>,
    asset_directories: BTreeSet<PathBuf>,
    asset_paths: BTreeSet<PathBuf>,
    total_bytes: usize,
    read_paths: BTreeSet<PathBuf>,
    development: bool,
    analysis_source: String,
}

pub fn build(script: &Path, assets: &[PathBuf]) -> io::Result<Bundle> {
    collect(script, assets, false)
}

pub fn development_source(script: &Path) -> io::Result<String> {
    String::from_utf8(collect(script, &[], true)?.python)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn collect(script: &Path, explicit_assets: &[PathBuf], development: bool) -> io::Result<Bundle> {
    if fs::symlink_metadata(script)?.file_type().is_symlink() {
        return Err(invalid("input scripts must not be symbolic links"));
    }
    let absolute = script.canonicalize()?;
    let parent = absolute
        .parent()
        .ok_or_else(|| invalid("script has no parent directory"))?;
    let config = project_config::discover(parent)?;
    let root = config
        .as_ref()
        .map_or_else(|| parent.to_owned(), |config| config.root.clone());
    let mut collector = Collector {
        entry: absolute
            .strip_prefix(&root)
            .map_err(|_| invalid("script is outside project root"))?
            .to_owned(),
        root,
        import_roots: vec![parent.to_owned()],
        modules: BTreeMap::new(),
        assets: BTreeMap::new(),
        asset_directories: BTreeSet::new(),
        asset_paths: BTreeSet::new(),
        total_bytes: 0,
        read_paths: BTreeSet::new(),
        development,
        analysis_source: String::new(),
    };
    if parent != collector.root {
        collector.import_roots.push(collector.root.clone());
    }
    let installed = dependencies::validate_installation(&collector.root)?;
    if let Some(packages) = &installed {
        collector.import_roots.push(packages.clone());
    }
    let entry_path = collector.entry.clone();
    let entry_source = collector.read_source(&entry_path)?;
    let entry_source = collector.prepare_source(&entry_path, None, &entry_source)?;
    if let Some(packages) = &installed {
        // Keep package resources available in both development and standalone
        // execution. Imported Python sources are replaced by prepared wrappers.
        let relative = packages
            .strip_prefix(&collector.root)
            .map_err(|_| invalid("installed packages must be inside the project"))?;
        collector.add_asset(relative, 0)?;
    }
    // prepare_source discovers imports recursively, including imports inside functions.
    if !development {
        let configured = config
            .as_ref()
            .map_or(&[][..], |config| config.assets.as_slice());
        for asset in configured.iter().chain(explicit_assets) {
            validate_relative(asset)?;
            collector.add_asset(asset, 0)?;
        }
    }
    let mut analysis = tree_shake::analyze(&collector.analysis_source);
    analysis
        .reasons
        .retain(|reason| reason != "relative import cannot be resolved statically");
    if analysis.reasons.is_empty() {
        analysis.profile = tree_shake::RuntimeProfile::Core;
    }
    if installed.is_some() {
        analysis.profile = tree_shake::RuntimeProfile::Full;
        analysis.reasons.push(
            "PyPI dependencies use the full runtime against which compatibility is checked"
                .to_owned(),
        );
    }
    let python = collector.bootstrap(&entry_source)?;
    if !development {
        collector.preflight(&entry_source, &python)?;
    }
    Ok(Bundle {
        python: python.into_bytes(),
        analysis,
        module_count: collector.modules.len(),
        has_dependencies: installed.is_some(),
        asset_count: collector
            .assets
            .len()
            .saturating_add(collector.asset_directories.len()),
    })
}

impl Collector {
    fn checked_path(&self, relative: &Path) -> io::Result<PathBuf> {
        validate_relative(relative)?;
        let mut path = self.root.clone();
        for component in relative.components() {
            path.push(component);
            if fs::symlink_metadata(&path)?.file_type().is_symlink() {
                return Err(invalid(&format!(
                    "{}: symbolic links cannot be bundled",
                    relative.display()
                )));
            }
        }
        Ok(path)
    }

    fn read_file(&mut self, relative: &Path, limit: usize) -> io::Result<Vec<u8>> {
        let path = self.checked_path(relative)?;
        if !fs::symlink_metadata(&path)?.is_file() {
            return Err(invalid(&format!(
                "{}: expected a regular file",
                relative.display()
            )));
        }
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)?;
        if !file.metadata()?.is_file() {
            return Err(invalid(&format!(
                "{}: expected a regular file",
                relative.display()
            )));
        }
        let mut data = Vec::new();
        file.take(
            u64::try_from(limit)
                .map_err(io::Error::other)?
                .saturating_add(1),
        )
        .read_to_end(&mut data)?;
        if data.len() > limit {
            return Err(invalid(&format!(
                "{} exceeds the {} MiB file limit",
                relative.display(),
                limit / (1024 * 1024)
            )));
        }
        if self.read_paths.insert(relative.to_owned()) {
            self.total_bytes = self
                .total_bytes
                .checked_add(data.len())
                .ok_or_else(|| invalid("bundle byte count overflow"))?;
            if self.read_paths.len() > MAX_FILES || self.total_bytes > MAX_TOTAL_BYTES {
                return Err(invalid(
                    "bundle exceeds 1,024 files or 32 MiB; include only application resources",
                ));
            }
        }
        Ok(data)
    }

    fn read_source(&mut self, relative: &Path) -> io::Result<String> {
        let bytes = self.read_file(relative, 1024 * 1024)?;
        let source = String::from_utf8(bytes).map_err(|_| {
            invalid(&format!(
                "{}: Python source must be UTF-8",
                relative.display()
            ))
        })?;
        if source.contains('\0') {
            return Err(invalid(&format!(
                "{}: Python source contains a NUL byte",
                relative.display()
            )));
        }
        let transformed = run_command::transform_source(&source)?;
        self.analysis_source.push_str(&transformed);
        self.analysis_source.push('\n');
        Ok(transformed)
    }

    fn resolve(&self, name: &str) -> io::Result<Option<(PathBuf, PathBuf)>> {
        if name.len() > 63 {
            return Err(invalid(&format!(
                "module name '{name}' exceeds PocketPy's 63-byte limit; shorten the package/module name"
            )));
        }
        if name == HELPER {
            return Err(invalid(
                "__kipferl_bundle_runtime is reserved by the packager; rename this module",
            ));
        }
        if name.is_empty()
            || name.split('.').any(|part| {
                part.is_empty()
                    || !part
                        .chars()
                        .all(|character| character == '_' || character.is_ascii_alphanumeric())
            })
        {
            return Err(invalid(&format!("invalid local module name '{name}'")));
        }
        for root in &self.import_roots {
            for suffix in [
                format!("{}.py", name.replace('.', "/")),
                format!("{}/__init__.py", name.replace('.', "/")),
            ] {
                let candidate = root.join(suffix);
                match fs::symlink_metadata(&candidate) {
                    Ok(_) => {
                        let relative = candidate
                            .strip_prefix(&self.root)
                            .map_err(|_| invalid("module escapes project root"))?
                            .to_owned();
                        self.checked_path(&relative)?;
                        return Ok(Some((
                            relative,
                            root.strip_prefix(&self.root)
                                .unwrap_or_else(|_| Path::new(""))
                                .to_owned(),
                        )));
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(None)
    }

    fn add_module(&mut self, name: &str) -> io::Result<bool> {
        if self.modules.contains_key(name) {
            return Ok(true);
        }
        let Some((path, import_root)) = self.resolve(name)? else {
            return Ok(false);
        };
        let source = self.read_source(&path)?;
        // Mark before recursing so circular imports terminate.
        self.modules.insert(
            name.to_owned(),
            Module {
                path: path.clone(),
                import_root,
                source: String::new(),
            },
        );
        let parts: Vec<_> = name.split('.').collect();
        for end in 1..parts.len() {
            let parent = parts
                .iter()
                .take(end)
                .copied()
                .collect::<Vec<_>>()
                .join(".");
            if !self.add_module(&parent)? {
                return Err(invalid(&format!(
                    "{}: local package '{parent}' needs an __init__.py file",
                    path.display()
                )));
            }
        }
        let source = self.prepare_source(&path, Some(name), &source)?;
        self.modules
            .get_mut(name)
            .ok_or_else(|| {
                io::Error::other("discovered module was lost during dependency collection")
            })?
            .source = source;
        Ok(true)
    }

    fn prepare_source(
        &mut self,
        path: &Path,
        module: Option<&str>,
        source: &str,
    ) -> io::Result<String> {
        let mut insertions = Vec::new();
        for statement in tree_shake::import_statements(source) {
            let mut loads = Vec::new();
            for imported in &statement.modules {
                let name = resolve_relative(imported, module, path)?;
                if supported_module(&name) {
                    if matches!(name.as_str(), "http" | "urllib" | "xml" | "xml.etree") {
                        for member in &statement.members {
                            let child = format!("{name}.{member}");
                            if !supported_module(&child) {
                                return Err(invalid(&format!(
                                    "{}: unsupported import '{child}'; use a supported Kipferl submodule",
                                    path.display()
                                )));
                            }
                        }
                    }
                    if self.resolve(&name)?.is_some() {
                        return Err(invalid(&format!(
                            "{}: local module '{name}' conflicts with a built-in Kipferl module; rename the local file/package",
                            path.display()
                        )));
                    }
                    continue;
                }
                if !self.add_module(&name)? {
                    if self.development {
                        // Let the interpreter report runtime-only/dynamic availability in run/dev.
                        continue;
                    }
                    let prefix = source
                        .get(..statement.start)
                        .ok_or_else(|| invalid("invalid import source span"))?;
                    return Err(unsupported_import(path, prefix, &name));
                }
                loads.push((name.clone(), false));
                for member in &statement.members {
                    let child = format!("{name}.{member}");
                    if self.add_module(&child)? {
                        loads.push((child, true));
                    }
                }
            }
            if !statement.is_from && statement.modules.iter().any(|name| name.contains('.')) {
                let mut replacement = String::new();
                for (name, alias) in statement.modules.iter().zip(&statement.aliases) {
                    let root = name.split('.').next().unwrap_or(name);
                    let load =
                        |module: &str| module_load(module, self.modules.contains_key(module));
                    if let Some(alias) = alias {
                        // Native alias traversal preserves overwritten parent attributes
                        // and falls back to the module cache when attributes are deleted.
                        write!(replacement, "{}; import {name} as {alias}; ", load(name))
                            .map_err(io::Error::other)?;
                    } else {
                        if root != name {
                            write!(replacement, "{}; ", load(name)).map_err(io::Error::other)?;
                        }
                        // A failed dotted import must not bind the root in the caller.
                        write!(replacement, "{root} = {}; ", load(root))
                            .map_err(io::Error::other)?;
                    }
                }
                let newline_count = source
                    .get(statement.start..statement.end)
                    .ok_or_else(|| invalid("invalid import source span"))?
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count();
                replacement.truncate(replacement.len().saturating_sub(2));
                let continuation = if source.as_bytes().get(statement.end) == Some(&b';') {
                    "\\\n"
                } else {
                    "\n"
                };
                replacement.push_str(&continuation.repeat(newline_count));
                insertions.push((statement.start, statement.end, replacement));
            } else if !loads.is_empty() {
                let mut prefix = String::new();
                for (name, member) in loads {
                    let method = if member { "load_member" } else { "load" };
                    write!(
                        prefix,
                        "__import__('{HELPER}').{method}({}); ",
                        run_command::python_string(&name)
                    )
                    .map_err(io::Error::other)?;
                }
                insertions.push((statement.start, statement.start, prefix));
            }
        }
        let mut transformed = source.to_owned();
        for (start, end, replacement) in insertions.into_iter().rev() {
            transformed.replace_range(start..end, &replacement);
        }
        Ok(transformed)
    }

    fn add_asset(&mut self, relative: &Path, depth: usize) -> io::Result<()> {
        if depth > 32 {
            return Err(invalid("asset directories may be at most 32 levels deep"));
        }
        if !self.asset_paths.insert(relative.to_owned()) {
            return Ok(());
        }
        if self.asset_paths.len() > MAX_FILES {
            return Err(invalid("asset selection exceeds 1,024 files/directories"));
        }
        let path = self.checked_path(relative)?;
        if path.is_dir() {
            self.asset_directories.insert(relative.to_owned());
            let mut entries = fs::read_dir(&path)?
                .take(MAX_FILES + 1)
                .collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(std::fs::DirEntry::file_name);
            if entries.len() > MAX_FILES {
                return Err(invalid("asset directory exceeds 1,024 entries"));
            }
            for entry in entries {
                self.add_asset(&relative.join(entry.file_name()), depth.saturating_add(1))?;
            }
        } else {
            if relative == self.entry || self.modules.values().any(|module| module.path == relative)
            {
                return Ok(());
            }
            if relative == Path::new(&format!("{HELPER}.py")) {
                return Err(invalid("asset name is reserved by the packager"));
            }
            let data = self.read_file(relative, MAX_FILE_BYTES)?;
            self.assets.insert(relative.to_owned(), data);
        }
        Ok(())
    }

    fn preflight(&self, entry_source: &str, bootstrap: &str) -> io::Result<()> {
        crate::syntax_check::check_sources(
            std::iter::once((self.entry.as_path(), entry_source))
                .chain(
                    self.modules
                        .values()
                        .map(|module| (module.path.as_path(), module.source.as_str())),
                )
                .chain(std::iter::once((
                    Path::new("<kipferl bootstrap>"),
                    bootstrap,
                ))),
        )
    }

    fn bootstrap(&self, entry_source: &str) -> io::Result<String> {
        let filename = |path: &Path| -> io::Result<String> {
            if self.development {
                path_text(&self.root.join(path))
            } else {
                path_text(path)
            }
        };
        if self.modules.is_empty() && self.assets.is_empty() && self.asset_directories.is_empty() {
            let original = filename(&self.entry)?;
            let argv = if self.development {
                format!(
                    "import sys\nsys.argv[0] = {}\n",
                    run_command::python_string(&original)
                )
            } else {
                String::new()
            };
            return Ok(format!(
                "#!/usr/bin/env pocketpy-kipferl\n{argv}__file__ = {}\n__kipferl_source = {}\n__kipferl_code = __import__('sys')._kipferl_compile_module(__kipferl_source, {})\ndel __kipferl_source\ntry:\n    exec(__kipferl_code)\n{}",
                run_command::python_string(&original),
                run_command::python_string(entry_source),
                run_command::python_string(&original),
                exit_handler()
            ));
        }
        let mut files = self.assets.clone();
        files.insert(self.entry.clone(), entry_source.as_bytes().to_vec());
        let mut mappings = String::from("{\n");
        for (name, module) in &self.modules {
            let original = filename(&module.path)?;
            let file_expr = if self.development {
                run_command::python_string(&path_text(&self.root.join(&module.path))?)
            } else {
                format!(
                    "__import__('{HELPER}').root + '/' + {}",
                    run_command::python_string(&path_text(&module.path)?)
                )
            };
            let wrapper = format!(
                "import os as __kipferl_os\n__kipferl_os.chdir(__import__('{HELPER}').caller_directory)\n__file__ = {file_expr}\n__kipferl_source = {}\n__kipferl_code = __import__('sys')._kipferl_compile_module(__kipferl_source, {})\ndel __kipferl_source\nexec(__kipferl_code)\n",
                run_command::python_string(&module.source),
                run_command::python_string(&original)
            );
            files.insert(module.path.clone(), wrapper.into_bytes());
            writeln!(
                mappings,
                "{}: {},",
                run_command::python_string(name),
                run_command::python_string(&path_text(&module.import_root)?)
            )
            .map_err(io::Error::other)?;
        }
        mappings.push('}');
        let helper = format!(
            "import os\nroot = os.getcwd()\ncaller_directory = root\npaths = {mappings}\nclass Scope:\n    def __init__(self, directory):\n        self.directory = directory\n    def __enter__(self):\n        self.previous = os.getcwd()\n        os.chdir(self.directory)\n    def __exit__(self, *args):\n        os.chdir(self.previous)\n        return False\ndef load(name):\n    global caller_directory\n    parts = name.split('.')\n    if len(parts) > 1:\n        load('.'.join(parts[:-1]))\n    caller_directory = os.getcwd()\n    with Scope(root + '/' + paths.get(name, '')):\n        module = __import__(name, None, None, ['__name__'])\n    return module\ndef load_member(name):\n    parts = name.split('.')\n    parent = load('.'.join(parts[:-1]))\n    if not hasattr(parent, parts[-1]):\n        load(name)\n"
        );
        files.insert(PathBuf::from(format!("{HELPER}.py")), helper.into_bytes());
        let mut data = String::from("[\n");
        for (path, bytes) in files {
            writeln!(
                data,
                "({}, '{}'),",
                run_command::python_string(&path_text(&path)?),
                base64_encode(&bytes)
            )
            .map_err(io::Error::other)?;
        }
        data.push(']');
        let directories = self
            .asset_directories
            .iter()
            .map(|path| path_text(path).map(|path| run_command::python_string(&path)))
            .collect::<io::Result<Vec<_>>>()?
            .join(", ");
        let original = filename(&self.entry)?;
        let argv = if self.development {
            "        __kipferl_sys.argv[0] = __file__\n"
        } else {
            ""
        };
        let exit = exit_handler();
        let file_expr = if self.development {
            run_command::python_string(&path_text(&self.root.join(&self.entry))?)
        } else {
            format!(
                "__kipferl_root + '/' + {}",
                run_command::python_string(&path_text(&self.entry)?)
            )
        };
        Ok(format!(
            "#!/usr/bin/env pocketpy-kipferl\n# Built with Kipferl: bundled local modules and application assets.\nimport os as __kipferl_os, tempfile as __kipferl_tempfile, shutil as __kipferl_shutil, base64 as __kipferl_base64, sys as __kipferl_sys\n__kipferl_cwd = __kipferl_os.getcwd()\n__kipferl_root = __kipferl_tempfile.mkdtemp()\nclass __KipferlResources:\n    def __enter__(self):\n        pass\n    def __exit__(self, *args):\n        __kipferl_os.chdir(__kipferl_cwd)\n        __kipferl_shutil.rmtree(__kipferl_root)\n        return False\ntry:\n    with __KipferlResources():\n        for __kipferl_directory in [{directories}]:\n            __kipferl_os.makedirs(__kipferl_root + '/' + __kipferl_directory, exist_ok=True)\n        for __kipferl_name, __kipferl_data in {data}:\n            __kipferl_destination = __kipferl_root + '/' + __kipferl_name\n            __kipferl_parent = __kipferl_os.path.dirname(__kipferl_destination)\n            __kipferl_os.makedirs(__kipferl_parent, exist_ok=True)\n            with open(__kipferl_destination, 'wb') as __kipferl_file:\n                __kipferl_file.write(__kipferl_base64.b64decode(__kipferl_data))\n        __kipferl_os.chdir(__kipferl_root)\n        import {HELPER}\n        __kipferl_os.chdir(__kipferl_cwd)\n        __file__ = {file_expr}\n{argv}\n        __kipferl_source = {}\n        __kipferl_code = __import__('sys')._kipferl_compile_module(__kipferl_source, {})\n        del __kipferl_source\n        exec(__kipferl_code)\n{exit}",
            run_command::python_string(entry_source),
            run_command::python_string(&original)
        ))
    }
}

fn module_load(module: &str, bundled: bool) -> String {
    let name = run_command::python_string(module);
    if bundled {
        format!("__import__('{HELPER}').load({name})")
    } else {
        format!("__import__({name}, None, None, ['__name__'])")
    }
}

fn resolve_relative(name: &str, module: Option<&str>, path: &Path) -> io::Result<String> {
    let dots = name.bytes().take_while(|byte| *byte == b'.').count();
    if dots == 0 {
        return Ok(name.to_owned());
    }
    let Some(module) = module else {
        return Err(invalid(&format!(
            "{}: relative imports require a package; use an absolute local import in the entry script",
            path.display()
        )));
    };
    let mut parts: Vec<_> = module.split('.').collect();
    if path.file_name().is_none_or(|name| name != "__init__.py") {
        parts.pop();
    }
    if dots > parts.len() {
        return Err(invalid(&format!(
            "{}: relative import '{name}' escapes the package",
            path.display()
        )));
    }
    parts.truncate(parts.len().saturating_sub(dots.saturating_sub(1)));
    let tail = name.trim_start_matches('.');
    if !tail.is_empty() {
        parts.push(tail);
    }
    Ok(parts.join("."))
}

fn unsupported_import(path: &Path, source_prefix: &str, name: &str) -> io::Error {
    let line = source_prefix
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        .saturating_add(1);
    invalid(&format!(
        "{}:{line}: unsupported import '{name}'. Add a local {}/__init__.py or {}.py, use 'kipferl add <distribution>' for a compatible PyPI package, or use a supported Kipferl module. --full-runtime does not install dependencies.",
        path.display(),
        name.replace('.', "/"),
        name.replace('.', "/")
    ))
}

fn supported_module(name: &str) -> bool {
    // Exact submodule names matter: urllib.request is not urllib.parse.
    matches!(
        name,
        "ansi"
            | "argparse"
            | "args"
            | "array"
            | "array2d"
            | "base64"
            | "binascii"
            | "bisect"
            | "builtins"
            | "cmath"
            | "collections"
            | "colorcvt"
            | "configparser"
            | "conio"
            | "contextlib"
            | "copy"
            | "csv"
            | "dataclasses"
            | "datetime"
            | "dis"
            | "easing"
            | "enum"
            | "errno"
            | "fnmatch"
            | "functools"
            | "gc"
            | "glob"
            | "gzip"
            | "hashlib"
            | "heapq"
            | "hmac"
            | "http"
            | "http.client"
            | "importlib"
            | "input"
            | "inspect"
            | "io"
            | "itertools"
            | "json"
            | "kdl"
            | "linalg"
            | "logging"
            | "lz4"
            | "math"
            | "operator"
            | "os"
            | "os.path"
            | "pathlib"
            | "pickle"
            | "pkpy"
            | "random"
            | "re"
            | "secrets"
            | "shutil"
            | "signal"
            | "sqlite3"
            | "statistics"
            | "struct"
            | "subprocess"
            | "sys"
            | "tarfile"
            | "tempfile"
            | "term"
            | "textwrap"
            | "time"
            | "toml"
            | "tomllib"
            | "traceback"
            | "tui"
            | "typing"
            | "unicodedata"
            | "unittest"
            | "urllib"
            | "urllib.parse"
            | "uuid"
            | "vmath"
            | "xml"
            | "xml.etree"
            | "xml.etree.ElementTree"
            | "yaml"
            | "zipfile"
    )
}

fn validate_relative(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid(&format!(
            "{}: asset/module paths must be relative to the project root and cannot contain '.' or '..'",
            path.display()
        )));
    }
    path_text(path)?;
    Ok(())
}

fn path_text(path: &Path) -> io::Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid("bundle paths must be UTF-8"))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

const fn exit_handler() -> &'static str {
    "except SystemExit as __kipferl_exit:\n    import builtins as __kipferl_builtins\n    import sys as __kipferl_sys\n    __kipferl_status = __kipferl_exit.args[0] if __kipferl_exit.args else 0\n    if __kipferl_status is None:\n        __kipferl_status = 0\n    if not isinstance(__kipferl_status, int):\n        __kipferl_sys.stderr.write(str(__kipferl_status) + '\\n')\n        __kipferl_status = 1\n    __kipferl_builtins.exit(__kipferl_status)\n"
}
