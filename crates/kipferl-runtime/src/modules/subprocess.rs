use std::ffi::c_int;
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Stdio};

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeIntConstant, NativeModule, NativeModuleKind, NativeSignature, RootFrame,
    Value, execute_module, return_value, runtime_error, type_error,
};

const MAX_CAPTURE_BYTES: usize = 1024 * 1024;

const CONSTANTS: &[NativeIntConstant] = &[
    NativeIntConstant {
        name: c"PIPE",
        value: -1,
    },
    NativeIntConstant {
        name: c"DEVNULL",
        value: -2,
    },
];

const SIGNATURES: &[NativeSignature] = &[NativeSignature {
    signature: c"_run(args, capture_output=False, shell=False)",
    callback: run,
}];

const COMPATIBILITY_SOURCE: &str = r#"
class Popen:
    def __init__(self, args, stdout=None, stderr=None, shell=False, text=False):
        self.args = args
        self._stdout_pipe = stdout == PIPE
        self._stderr_pipe = stderr == PIPE
        self.shell = shell
        self.text = text
        self.returncode = None
        self._stdout = None
        self._stderr = None

    def communicate(self):
        if self.returncode is None:
            result = _run(self.args, self._stdout_pipe or self._stderr_pipe, self.shell)
            self.returncode = result["returncode"]
            self._stdout = result["stdout"] if self._stdout_pipe else None
            self._stderr = result["stderr"] if self._stderr_pipe else None
            if self.text:
                if self._stdout is not None:
                    self._stdout = self._stdout.decode().replace("\r\n", "\n").replace("\r", "\n")
                if self._stderr is not None:
                    self._stderr = self._stderr.decode().replace("\r\n", "\n").replace("\r", "\n")
        return (self._stdout, self._stderr)

    def wait(self):
        if self.returncode is None:
            self.communicate()
        return self.returncode


def run(args, capture_output=False, shell=False):
    return _run(args, capture_output, shell)


def call(args):
    return _run(args, False, False)["returncode"]


def check_output(args):
    result = _run(args, True, False)
    if result["returncode"] != 0:
        raise RuntimeError("process returned non-zero exit status")
    return result["stdout"]


def getstatusoutput(command):
    result = _run(command, True, True)
    output = result["stdout"].decode()
    if len(output) > 0 and output[-1] == "\n":
        output = output[:-1]
    return (result["returncode"], output)


def getoutput(command):
    return getstatusoutput(command)[1]
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"subprocess",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: SIGNATURES,
    int_constants: CONSTANTS,
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
        panic!("embedded subprocess compatibility layer failed");
    }
}

unsafe extern "C" fn run(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let capture = arguments.get(1).and_then(Value::boolean).unwrap_or(false);
    let shell = arguments.get(2).and_then(Value::boolean).unwrap_or(false);
    let Some(argument) = arguments.get(0) else {
        return type_error(c"args are required");
    };

    let command = if shell {
        let Some(command) = argument.string() else {
            return type_error(c"args must be a string when shell=True");
        };
        vec!["sh".to_owned(), "-c".to_owned(), command]
    } else {
        let Some(command) = command_arguments(argument) else {
            return false;
        };
        command
    };

    let Ok(result) = execute(&command, capture) else {
        return runtime_error(c"failed to run process");
    };
    return_result(result)
}

struct ProcessResult {
    status: i64,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

#[expect(
    clippy::expect_used,
    reason = "The child was successfully spawned with stdout and stderr explicitly piped, which guarantees both handles are present."
)]
fn execute(arguments: &[String], capture: bool) -> io::Result<ProcessResult> {
    let Some((program, parameters)) = arguments.split_first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    };
    let mut command = Command::new(program);
    command.args(parameters).stdin(Stdio::null());
    if !capture {
        let status = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        return Ok(ProcessResult {
            status: return_code(status),
            stdout: None,
            stderr: None,
        });
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout is available");
    let stderr = child.stderr.take().expect("piped stderr is available");
    let (status, stdout, stderr) = std::thread::scope(|scope| -> io::Result<_> {
        let stdout_reader = scope.spawn(move || read_capped(stdout));
        let stderr_reader = scope.spawn(move || read_capped(stderr));
        let status = child.wait()?;
        let stdout = stdout_reader
            .join()
            .map_err(|_| std::io::Error::other("stdout reader panicked"))??;
        let stderr = stderr_reader
            .join()
            .map_err(|_| std::io::Error::other("stderr reader panicked"))??;
        Ok((status, stdout, stderr))
    })?;
    Ok(ProcessResult {
        status: return_code(status),
        stdout: Some(stdout),
        stderr: Some(stderr),
    })
}

fn return_code(status: ExitStatus) -> i64 {
    if let Some(code) = status.code() {
        return i64::from(code);
    }
    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return i64::from(signal).wrapping_neg();
    }
    -1
}

fn read_capped(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len() < MAX_CAPTURE_BYTES {
            let remaining = MAX_CAPTURE_BYTES.saturating_sub(output.len());
            output.extend(buffer.iter().copied().take(count.min(remaining)));
        }
    }
}

fn command_arguments(value: Value) -> Option<Vec<String>> {
    if let Some(command) = value.string() {
        return Some(vec![command]);
    }
    let length = value.list_len().or_else(|| value.tuple_len());
    let Some(length) = length else {
        type_error(c"args must be a string or sequence");
        return None;
    };
    if length == 0 {
        type_error(c"args must be a non-empty sequence");
        return None;
    }
    let mut arguments = Vec::with_capacity(length);
    for index in 0..length {
        let item = value
            .list_item(index)
            .or_else(|| value.tuple_item(index))
            .and_then(Value::string);
        let Some(item) = item else {
            type_error(c"args must be strings");
            return None;
        };
        arguments.push(item);
    }
    Some(arguments)
}

#[expect(
    clippy::expect_used,
    reason = "Output reads are capped at 1 MiB per stream and dictionary keys are short literals, all strictly below the VM signed-int string/bytes limit."
)]
fn return_result(result: ProcessResult) -> bool {
    let mut roots = RootFrame::new();
    let output = roots.dict();
    let stdout_key = roots.string("stdout").expect("short key fits PocketPy");
    let stderr_key = roots.string("stderr").expect("short key fits PocketPy");
    let status_key = roots.string("returncode").expect("short key fits PocketPy");
    let stdout = match result.stdout {
        Some(bytes) => roots.bytes(&bytes).expect("captured output is bounded"),
        None => roots.none(),
    };
    let stderr = match result.stderr {
        Some(bytes) => roots.bytes(&bytes).expect("captured output is bounded"),
        None => roots.none(),
    };
    let status = roots.integer(result.status);
    if !output.dict_set(stdout_key, stdout)
        || !output.dict_set(stderr_key, stderr)
        || !output.dict_set(status_key, status)
    {
        return false;
    }
    return_value(output)
}
