use ucharm_pocketpy_sys as ffi;

use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
import os


def _join_paths(paths):
    result = ""
    first = True
    for raw in paths:
        if not isinstance(raw, str):
            raise TypeError("path must be a string")
        part = raw
        if first:
            result = part
            first = False
            continue
        while len(part) > 0 and part[0] == "/":
            part = part[1:]
        if result != "" and result[-1] != "/":
            result += "/"
        result += part
    while len(result) > 1 and result[-1] == "/":
        result = result[:-1]
    return result


class Path:
    def __init__(self, *paths):
        self.path = _join_paths(paths)
        display = self.path if self.path != "" else "."
        if display == "/":
            self.name = ""
        else:
            self.name = os.path.basename(display)
        root, suffix = os.path.splitext(self.name)
        self.suffix = suffix
        self.stem = root if suffix != "" else self.name
        parent_path = os.path.dirname(display)
        if parent_path == "":
            parent_path = "."
        if parent_path == display:
            self.parent = self
        else:
            self.parent = Path(parent_path)

    def __str__(self):
        return self.path if self.path != "" else "."

    def __repr__(self):
        return "Path('" + str(self) + "')"

    def __truediv__(self, other):
        return Path(self.path, other)

    def joinpath(self, *others):
        values = [self.path]
        for other in others:
            values.append(other)
        return Path(*values)

    def with_name(self, name):
        return Path(str(self.parent), name)

    def with_suffix(self, suffix):
        if not isinstance(suffix, str):
            raise TypeError("suffix must be a string")
        base = self.path
        if self.suffix != "":
            base = base[:-len(self.suffix)]
        return Path(base + suffix)

    def is_absolute(self):
        return os.path.isabs(self.path)

    def exists(self):
        return os.path.exists(str(self))

    def is_file(self):
        return os.path.isfile(str(self))

    def is_dir(self):
        return os.path.isdir(str(self))

    def resolve(self):
        return Path(os.path.abspath(str(self)))

    def cwd():
        return Path(os.getcwd())
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"pathlib",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded pathlib compatibility layer failed");
    }
}
