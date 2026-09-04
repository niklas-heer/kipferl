// Native callbacks must report failures through Python or structured Rust
// errors; implicit unwraps and process exits bypass that boundary.
#![deny(clippy::unwrap_used, clippy::exit, clippy::panic_in_result_fn)]

pub mod args_core;
pub mod input_core;
mod modules;
mod native;

use std::error::Error;
use std::ffi::{CStr, CString, NulError, c_char};
use std::fmt;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};

use kipferl_pocketpy_sys as ffi;

const VM_NEVER_INITIALIZED: u8 = 0;
const VM_ACTIVE: u8 = 1;
const VM_FINALIZED: u8 = 2;

static VM_STATE: AtomicU8 = AtomicU8::new(VM_NEVER_INITIALIZED);

/// The process-wide `PocketPy` virtual machine.
///
/// `PocketPy` exposes global VM state and cannot be initialized again after
/// finalization. This owner is therefore deliberately neither `Send` nor
/// `Sync`, and only one instance can exist during the process lifetime.
pub struct Vm {
    _not_send_or_sync: PhantomData<Rc<()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializeError {
    AlreadyActive,
    AlreadyFinalized,
}

impl fmt::Display for InitializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyActive => formatter.write_str("PocketPy is already initialized"),
            Self::AlreadyFinalized => {
                formatter.write_str("PocketPy cannot be initialized after finalization")
            }
        }
    }
}

impl Error for InitializeError {}

#[derive(Debug)]
pub enum ExecuteError {
    InteriorNul(NulError),
    PythonException,
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul(_) => formatter.write_str("Python source contains a NUL byte"),
            Self::PythonException => formatter.write_str("PocketPy execution failed"),
        }
    }
}

impl Error for ExecuteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InteriorNul(error) => Some(error),
            Self::PythonException => None,
        }
    }
}

impl From<NulError> for ExecuteError {
    fn from(error: NulError) -> Self {
        Self::InteriorNul(error)
    }
}

/// Failure to install script metadata into the VM.
#[derive(Debug)]
pub enum ContextError {
    InteriorNul(NulError),
    TooLarge,
    MissingMainModule,
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul(_) => formatter.write_str("script path contains a NUL byte"),
            Self::TooLarge => formatter.write_str("script metadata exceeds PocketPy's C ABI limit"),
            Self::MissingMainModule => formatter.write_str("PocketPy main module is unavailable"),
        }
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InteriorNul(error) => Some(error),
            Self::TooLarge | Self::MissingMainModule => None,
        }
    }
}

fn context_length(length: usize) -> Result<i32, ContextError> {
    i32::try_from(length).map_err(|_| ContextError::TooLarge)
}

impl Vm {
    /// Create the process-wide VM.
    ///
    /// # Errors
    /// Returns an error if a VM is already active or was previously finalized.
    pub fn initialize() -> Result<Self, InitializeError> {
        match VM_STATE.compare_exchange(
            VM_NEVER_INITIALIZED,
            VM_ACTIVE,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                // SAFETY: the state transition above grants this value unique,
                // process-wide ownership of PocketPy initialization.
                unsafe { ffi::py_initialize() };
                let vm = Self {
                    _not_send_or_sync: PhantomData,
                };
                modules::register_all();
                Ok(vm)
            }
            Err(VM_ACTIVE) => Err(InitializeError::AlreadyActive),
            Err(_) => Err(InitializeError::AlreadyFinalized),
        }
    }

    /// Execute source in the main module.
    ///
    /// # Errors
    /// Returns an error when Python raises an exception; the VM retains it for reporting.
    pub fn execute(&self, source: &CStr, filename: &CStr) -> Result<(), ExecuteError> {
        // SAFETY: `self` proves the process-wide VM is active. Both C strings
        // remain alive for the duration of the call, and a null module selects
        // PocketPy's main module as required by the C API.
        let succeeded = unsafe {
            ffi::py_exec(
                source.as_ptr(),
                filename.as_ptr(),
                ffi::py_CompileMode_EXEC_MODE,
                ptr::null_mut(),
            )
        };

        if succeeded {
            Ok(())
        } else {
            Err(ExecuteError::PythonException)
        }
    }

    /// Execute UTF-8 source with its diagnostic filename.
    ///
    /// # Errors
    /// Returns an error for embedded NUL bytes or a Python exception.
    pub fn execute_str(&self, source: &str, filename: &str) -> Result<(), ExecuteError> {
        let source = CString::new(source)?;
        let filename = CString::new(filename)?;
        self.execute(&source, &filename)
    }

    /// Set Python's argument vector, copying every argument into the VM.
    ///
    /// # Errors
    ///
    /// Returns an error if the argument count exceeds `PocketPy`'s C ABI limit.
    #[deny(
        clippy::as_conversions,
        clippy::expect_used,
        clippy::panic_in_result_fn
    )]
    pub fn set_argv(&self, arguments: &[CString]) -> Result<(), ContextError> {
        let count = context_length(arguments.len())?;
        let mut pointers: Vec<*mut c_char> = arguments
            .iter()
            .map(|argument| argument.as_ptr().cast_mut())
            .collect();

        // SAFETY: every pointer is NUL-terminated and remains valid throughout
        // the call; `count` was checked against the C ABI's signed length.
        // PocketPy converts the arguments into VM-owned strings.
        unsafe { ffi::py_sys_setargv(count, pointers.as_mut_ptr()) };
        Ok(())
    }

    /// Set `__main__.__file__` to the script's original filename.
    ///
    /// # Errors
    ///
    /// Returns an error for embedded NUL bytes, filenames exceeding `PocketPy`'s
    /// C ABI string limit, or an unavailable main module.
    #[deny(clippy::expect_used, clippy::panic_in_result_fn)]
    pub fn set_file(&self, filename: &str) -> Result<(), ContextError> {
        let length = context_length(filename.len())?;
        let filename = CString::new(filename).map_err(ContextError::InteriorNul)?;
        let bytes = filename.as_bytes();
        // SAFETY: the VM is active, `__main__` is a process-global module, and
        // register zero is a stable VM root used synchronously for assignment.
        unsafe {
            let main = ffi::py_getmodule(c"__main__".as_ptr());
            if main.is_null() {
                return Err(ContextError::MissingMainModule);
            }
            let value = ffi::py_getreg(0);
            let destination = ffi::py_newstrn(value, length);
            ptr::copy_nonoverlapping(bytes.as_ptr(), destination.cast::<u8>(), bytes.len());
            ffi::py_setdict(main, ffi::py_name(c"__file__".as_ptr()), value);
        }
        Ok(())
    }

    pub fn print_exception(&self) {
        // SAFETY: `self` proves the VM is active. Call this only after an API
        // reports a pending Python exception.
        unsafe { ffi::py_printexc() };
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        modules::shutdown_all();
        // SAFETY: `Vm` is the unique process-wide owner and finalization occurs
        // exactly once. PocketPy documents finalization as irreversible.
        unsafe { ffi::py_finalize() };
        VM_STATE.store(VM_FINALIZED, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::{ContextError, InitializeError, Vm, context_length};

    #[test]
    fn rejects_metadata_lengths_outside_the_c_abi_without_allocating() {
        assert_eq!(context_length(0).expect("zero fits"), 0);
        assert_eq!(
            context_length(2_147_483_647).expect("maximum fits"),
            i32::MAX
        );
        assert!(matches!(
            context_length(2_147_483_648),
            Err(ContextError::TooLarge)
        ));
        assert!(matches!(
            context_length(usize::MAX),
            Err(ContextError::TooLarge)
        ));
    }

    #[test]
    fn enforces_the_process_lifecycle_and_executes_python() {
        let vm = Vm::initialize().expect("initialize PocketPy");
        assert!(matches!(
            Vm::initialize(),
            Err(InitializeError::AlreadyActive)
        ));
        let arguments = [CString::new("-c").expect("valid C string")];
        vm.set_argv(&arguments).expect("set Python arguments");
        assert!(vm.set_file("invalid\0path").is_err());
        vm.set_file("original.py")
            .expect("set original script path");
        vm.execute_str(
            "import ansi, sys\n\
assert sys.argv == ['-c']\n\
assert __file__ == 'original.py'\n\
assert ansi.fg('red') == '\\x1b[31m'\n\
assert ansi.bg('#f50') == '\\x1b[48;2;255;85;0m'\n\
assert ansi.rgb(1, 2, 3, True) == '\\x1b[48;2;1;2;3m'\n\
assert ansi.strikethrough() == '\\x1b[9m'",
            "<rust-test>",
        )
        .expect("execute Python");

        drop(vm);
        assert!(matches!(
            Vm::initialize(),
            Err(InitializeError::AlreadyFinalized)
        ));
    }
}
