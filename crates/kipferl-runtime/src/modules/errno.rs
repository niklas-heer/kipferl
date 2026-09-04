use std::ffi::c_int;

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeIntConstant, NativeModule, NativeModuleKind, RootFrame, Value,
    bind_type_method, call, type_error, type_magic,
};

#[expect(
    clippy::as_conversions,
    reason = "libc errno constants are signed C ints; widening them to i64 is lossless on all supported targets."
)]
const CONSTANTS: &[NativeIntConstant] = &[
    NativeIntConstant {
        name: c"EPERM",
        value: libc::EPERM as i64,
    },
    NativeIntConstant {
        name: c"ENOENT",
        value: libc::ENOENT as i64,
    },
    NativeIntConstant {
        name: c"ESRCH",
        value: libc::ESRCH as i64,
    },
    NativeIntConstant {
        name: c"EINTR",
        value: libc::EINTR as i64,
    },
    NativeIntConstant {
        name: c"EIO",
        value: libc::EIO as i64,
    },
    NativeIntConstant {
        name: c"EBADF",
        value: libc::EBADF as i64,
    },
    NativeIntConstant {
        name: c"ECHILD",
        value: libc::ECHILD as i64,
    },
    NativeIntConstant {
        name: c"EAGAIN",
        value: libc::EAGAIN as i64,
    },
    NativeIntConstant {
        name: c"ENOMEM",
        value: libc::ENOMEM as i64,
    },
    NativeIntConstant {
        name: c"EACCES",
        value: libc::EACCES as i64,
    },
    NativeIntConstant {
        name: c"EEXIST",
        value: libc::EEXIST as i64,
    },
    NativeIntConstant {
        name: c"ENOTDIR",
        value: libc::ENOTDIR as i64,
    },
    NativeIntConstant {
        name: c"EISDIR",
        value: libc::EISDIR as i64,
    },
    NativeIntConstant {
        name: c"EINVAL",
        value: libc::EINVAL as i64,
    },
    NativeIntConstant {
        name: c"ENOSPC",
        value: libc::ENOSPC as i64,
    },
    NativeIntConstant {
        name: c"EPIPE",
        value: libc::EPIPE as i64,
    },
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"errno",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: CONSTANTS,
    type_aliases: &[],
    initializer: Some(initialize),
};

#[expect(
    clippy::expect_used,
    reason = "The initializer just created errorcode; constant errno names are short ASCII C strings and fit the VM string length."
)]
fn initialize(module: Value) {
    let mut roots = RootFrame::new();
    let errorcode = roots.dict();
    module.set_attribute(c"errorcode", errorcode);
    drop(roots);

    let errorcode = module
        .attribute(c"errorcode")
        .expect("errno.errorcode was just installed");
    for constant in CONSTANTS {
        let mut roots = RootFrame::new();
        let name = roots
            .string(constant.name.to_str().expect("errno names are UTF-8"))
            .expect("short errno name fits PocketPy");
        let name = name.snapshot();
        let key = roots.integer(constant.value);
        assert!(
            errorcode.dict_set(key, name.value()),
            "errno reverse mapping insertion"
        );
    }

    bind_type_method(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_OSError),
        c"__init__",
        oserror_init,
    );
}

unsafe extern "C" fn oserror_init(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 3) {
        return false;
    }
    let Some(initializer) = type_magic(
        crate::native::predefined_type(ffi::py_PredefinedType_tp_BaseException),
        c"__init__",
    ) else {
        return type_error(c"BaseException.__init__ is unavailable");
    };
    let Some(self_value) = arguments.get(0) else {
        crate::native::type_error(c"missing native argument");
        return false;
    };
    arguments.get(1).map_or_else(
        || call(initializer, &[self_value]),
        |errno| call(initializer, &[self_value, errno]),
    )
}
