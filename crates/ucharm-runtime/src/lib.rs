use std::error::Error;
use std::ffi::{CStr, CString, NulError, c_char};
use std::fmt;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};

use ucharm_pocketpy_sys as ffi;

const VM_NEVER_INITIALIZED: u8 = 0;
const VM_ACTIVE: u8 = 1;
const VM_FINALIZED: u8 = 2;

static VM_STATE: AtomicU8 = AtomicU8::new(VM_NEVER_INITIALIZED);

/// The process-wide PocketPy virtual machine.
///
/// PocketPy exposes global VM state and cannot be initialized again after
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

impl Vm {
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
                vm.register_probe_module();
                Ok(vm)
            }
            Err(VM_ACTIVE) => Err(InitializeError::AlreadyActive),
            Err(_) => Err(InitializeError::AlreadyFinalized),
        }
    }

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

    pub fn execute_str(&self, source: &str, filename: &str) -> Result<(), ExecuteError> {
        let source = CString::new(source)?;
        let filename = CString::new(filename)?;
        self.execute(&source, &filename)
    }

    pub fn set_argv(&self, arguments: &[CString]) {
        let mut pointers: Vec<*mut c_char> = arguments
            .iter()
            .map(|argument| argument.as_ptr().cast_mut())
            .collect();

        // SAFETY: every pointer is NUL-terminated and remains valid throughout
        // the call. PocketPy converts the arguments into VM-owned strings.
        unsafe { ffi::py_sys_setargv(pointers.len() as i32, pointers.as_mut_ptr()) };
    }

    pub fn print_exception(&self) {
        // SAFETY: `self` proves the VM is active. Call this only after an API
        // reports a pending Python exception.
        unsafe { ffi::py_printexc() };
    }

    fn register_probe_module(&self) {
        // SAFETY: `self` proves the VM is active. The C string literals have
        // static storage, and the callback uses PocketPy's required ABI.
        unsafe {
            let module = ffi::py_newmodule(c"_ucharm_rust".as_ptr());
            ffi::py_bindfunc(module, c"answer".as_ptr(), Some(probe_answer));
        }
    }
}

unsafe extern "C" fn probe_answer(_argc: i32, _argv: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy invokes this callback with an active VM and owns the
    // return register. `py_newint` writes a complete Python integer into it.
    unsafe { ffi::py_newint(ffi::py_retval(), 42) };
    true
}

impl Drop for Vm {
    fn drop(&mut self) {
        // SAFETY: `Vm` is the unique process-wide owner and finalization occurs
        // exactly once. PocketPy documents finalization as irreversible.
        unsafe { ffi::py_finalize() };
        VM_STATE.store(VM_FINALIZED, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::Vm;

    #[test]
    fn executes_python_through_pocketpy() {
        let vm = Vm::initialize().expect("initialize PocketPy");
        let arguments = [CString::new("-c").expect("valid C string")];
        vm.set_argv(&arguments);
        vm.execute_str(
            "import _ucharm_rust, sys\nassert _ucharm_rust.answer() == 42\nassert sys.argv == ['-c']",
            "<rust-test>",
        )
        .expect("execute Python");
    }
}
