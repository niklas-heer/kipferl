use kipferl_pocketpy_sys as ffi;

use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r#"
import os


def exists(path):
    try:
        os.stat(path)
        return True
    except OSError:
        return False


def isdir(path):
    try:
        return (os.stat(path)[0] & 0o170000) == 0o040000
    except OSError:
        return False


def isfile(path):
    try:
        return (os.stat(path)[0] & 0o170000) == 0o100000
    except OSError:
        return False


def isabs(path):
    return len(path) > 0 and path[0] == "/"


def join(*paths):
    result = ""
    for path in paths:
        if path == "":
            continue
        if isabs(path):
            result = path
        elif result == "" or result[-1] == "/":
            result += path
        else:
            result += "/" + path
    return result


def split(path):
    index = len(path) - 1
    while index >= 0 and path[index] != "/":
        index -= 1
    if index < 0:
        return ("", path)
    head = path[:index]
    if head == "" and index == 0:
        head = "/"
    return (head, path[index + 1:])


def basename(path):
    return split(path)[1]


def dirname(path):
    return split(path)[0]


def splitext(path):
    slash = -1
    dot = -1
    index = 0
    while index < len(path):
        if path[index] == "/":
            slash = index
        elif path[index] == ".":
            dot = index
        index += 1
    if dot <= slash + 1:
        return (path, "")
    return (path[:dot], path[dot:])


def normpath(path):
    if path == "":
        return "."
    absolute = isabs(path)
    output = []
    for part in path.split("/"):
        if part == "" or part == ".":
            continue
        if part == "..":
            if len(output) > 0 and output[-1] != "..":
                output.pop()
            elif not absolute:
                output.append(part)
        else:
            output.append(part)
    result = "/".join(output)
    if absolute:
        result = "/" + result
    if result == "":
        return "/" if absolute else "."
    return result


def abspath(path):
    if isabs(path):
        return normpath(path)
    return normpath(join(os.getcwd(), path))
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"os.path",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::panic,
    reason = "Initialization runs before user code; failure to compile the checked-in compatibility source is a fatal runtime build defect."
)]
fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ffi::py_printexc() };
        panic!("embedded os.path compatibility layer failed");
    }
    // SAFETY: importing `os` earlier in registration created a VM-global
    // module which remains live for the process lifetime.
    let os = unsafe { ffi::py_getmodule(c"os".as_ptr()) };
    assert!(!os.is_null(), "PocketPy os module is missing");
    // SAFETY: the non-null module reference is VM-global.
    let os = unsafe { Value::from_raw(os) };
    os.set_attribute(c"path", module);
}
