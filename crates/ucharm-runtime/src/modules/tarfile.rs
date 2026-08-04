use std::ffi::c_int;
use std::fs::File;
use std::io::Read;

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, execute_module,
    return_bytes, return_value, runtime_error, type_error, value_error,
};

const MAX_ARCHIVE_SIZE: usize = 64 * 1024 * 1024;

const COMPATIBILITY_SOURCE: &str = r#"
import io


ReadError = ValueError


def is_tarfile(name):
    try:
        _names(name)
        return True
    except Exception:
        return False


class TarFile:
    def __init__(self, name, mode="r"):
        if mode != "r":
            raise ValueError("only read mode is supported")
        self._name = name
        self._entries = _names(name)
        self.closed = False

    def getnames(self):
        return list(self._entries)

    def extractfile(self, member):
        if member in self._entries:
            return io.BytesIO(_read_member(self._name, member))
        return None

    def close(self):
        self.closed = True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


def open(name, mode="r"):
    return TarFile(name, mode)
"#;

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"_names",
        callback: names,
    },
    NativeFunction {
        name: c"_read_member",
        callback: read_member,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"tarfile",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    if !execute_module(module, COMPATIBILITY_SOURCE) {
        // SAFETY: initialization failed with a live PocketPy exception.
        unsafe { ucharm_pocketpy_sys::py_printexc() };
        panic!("embedded tarfile compatibility layer failed");
    }
}

fn open_archive(path: &str) -> Result<tar::Archive<File>, ()> {
    let file = File::open(path).map_err(|_| ())?;
    let length = file.metadata().map_err(|_| ())?.len();
    if !(1024..=MAX_ARCHIVE_SIZE as u64).contains(&length) {
        return Err(());
    }
    Ok(tar::Archive::new(file))
}

unsafe extern "C" fn names(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"filename must be a string");
    };
    let Ok(mut archive) = open_archive(&path) else {
        return value_error(c"not a tar archive");
    };
    let Ok(entries) = archive.entries() else {
        return value_error(c"not a tar archive");
    };
    let mut names = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            return value_error(c"not a tar archive");
        };
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let Ok(path) = entry.path() else {
            return value_error(c"invalid tar member name");
        };
        let Some(path) = path.to_str() else {
            return value_error(c"invalid tar member name");
        };
        names.push(path.to_owned());
    }
    let mut roots = RootFrame::new();
    let output = roots.list();
    for name in names {
        let Some(name) = roots.string(&name) else {
            return value_error(c"tar member name is too large");
        };
        output.list_append(name);
    }
    return_value(output)
}

unsafe extern "C" fn read_member(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(path) = arguments.get(0).and_then(Value::string) else {
        return type_error(c"filename must be a string");
    };
    let Some(name) = arguments.get(1).and_then(Value::string) else {
        return type_error(c"member name must be a string");
    };
    let Ok(mut archive) = open_archive(&path) else {
        return value_error(c"not a tar archive");
    };
    let Ok(entries) = archive.entries() else {
        return value_error(c"not a tar archive");
    };
    for entry in entries {
        let Ok(entry) = entry else {
            return value_error(c"not a tar archive");
        };
        let Ok(entry_path) = entry.path() else {
            return value_error(c"invalid tar member name");
        };
        if entry_path != std::path::Path::new(&name) {
            continue;
        }
        if entry.size() > MAX_ARCHIVE_SIZE as u64 {
            return value_error(c"tar member is too large");
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        if entry
            .take(MAX_ARCHIVE_SIZE as u64 + 1)
            .read_to_end(&mut bytes)
            .is_err()
        {
            return runtime_error(c"unable to read tar member");
        }
        if bytes.len() > MAX_ARCHIVE_SIZE {
            return value_error(c"tar member is too large");
        }
        return return_bytes(&bytes);
    }
    runtime_error(c"unable to read tar member")
}

#[cfg(test)]
mod tests {
    use super::open_archive;

    #[test]
    fn rejects_short_non_archives_before_parsing() {
        let path = std::env::temp_dir().join(format!(
            "ucharm-invalid-tar-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(&path, [0_u8; 512]).expect("write invalid archive");
        assert!(open_archive(path.to_str().expect("UTF-8 temp path")).is_err());
        std::fs::remove_file(path).expect("remove invalid archive");
    }
}
