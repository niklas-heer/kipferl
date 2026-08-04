use std::ffi::{c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicI16, Ordering};

use ucharm_pocketpy_sys as ffi;

use crate::native::{
    Arguments, RootFrame, Value, bind_type_signature, create_type_with_destructor, return_value,
    type_error, type_object, value_error,
};

static BYTEARRAY_TYPE: AtomicI16 = AtomicI16::new(0);

#[repr(C)]
#[derive(Clone, Copy)]
struct ByteArrayState {
    data: *mut u8,
    length: usize,
}

impl ByteArrayState {
    const EMPTY: Self = Self {
        data: ptr::null_mut(),
        length: 0,
    };

    fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::EMPTY;
        }
        let mut boxed = bytes.to_vec().into_boxed_slice();
        let state = Self {
            data: boxed.as_mut_ptr(),
            length: boxed.len(),
        };
        Box::leak(boxed);
        state
    }

    fn as_slice(&self) -> &[u8] {
        if self.length == 0 {
            return &[];
        }
        // SAFETY: non-empty states own exactly `length` initialized bytes until
        // PocketPy invokes `bytearray_destructor`.
        unsafe { slice::from_raw_parts(self.data, self.length) }
    }
}

unsafe extern "C" fn bytearray_destructor(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: every instance initializes its userdata as `ByteArrayState`.
    let state = unsafe { &mut *userdata.cast::<ByteArrayState>() };
    if state.length != 0 {
        let allocation = ptr::slice_from_raw_parts_mut(state.data, state.length);
        // SAFETY: `from_bytes` leaked this exact boxed slice and ownership is
        // transferred back exactly once by PocketPy's object destructor.
        unsafe { drop(Box::from_raw(allocation)) };
    }
    *state = ByteArrayState::EMPTY;
}

pub(super) fn register() {
    // SAFETY: the builtins module is created by PocketPy initialization and
    // remains global for the VM lifetime.
    let builtins = unsafe { ffi::py_getmodule(c"builtins".as_ptr()) };
    assert!(!builtins.is_null(), "PocketPy builtins module is missing");
    // SAFETY: the non-null module reference is VM-global.
    let builtins = unsafe { Value::from_raw(builtins) };
    let value_type =
        create_type_with_destructor(builtins, c"bytearray", Some(bytearray_destructor));
    BYTEARRAY_TYPE.store(value_type, Ordering::Release);
    bind_type_signature(value_type, c"__new__(cls, source=None)", bytearray_new);
    bind_type_signature(value_type, c"__len__(self)", bytearray_len);
    bind_type_signature(value_type, c"__eq__(self, other)", bytearray_eq);
    builtins.set_attribute(c"bytearray", type_object(value_type));
}

unsafe extern "C" fn bytearray_new(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // The signature binder supplies `cls` plus zero or one source arguments.
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 2) {
        return false;
    }
    let value_type = BYTEARRAY_TYPE.load(Ordering::Acquire);
    assert_ne!(value_type, 0, "bytearray used before registration");
    let mut roots = RootFrame::new();
    let instance = roots
        .object_with_userdata(value_type, -1, ByteArrayState::EMPTY)
        .expect("bytearray state fits PocketPy userdata");

    let bytes = match arguments.get(1) {
        None => Vec::new(),
        Some(source) if source.is_none() => Vec::new(),
        Some(source) if source.is_type(ffi::py_PredefinedType_tp_bytes) => {
            source.bytes().expect("bytes checked")
        }
        Some(source) if source.is_type(ffi::py_PredefinedType_tp_int) => {
            let count = source.integer().expect("integer checked");
            if count < 0 {
                return value_error(c"negative count");
            }
            let Ok(count) = usize::try_from(count) else {
                return value_error(c"bytearray is too large");
            };
            vec![0; count]
        }
        Some(source) if source.is_type(ffi::py_PredefinedType_tp_str) => {
            source.string().expect("string checked").into_bytes()
        }
        Some(source) if source.is_type(ffi::py_PredefinedType_tp_list) => {
            let length = source.list_len().expect("list checked");
            let mut bytes = Vec::with_capacity(length);
            for index in 0..length {
                let Some(value) = source.list_item(index).and_then(Value::integer) else {
                    return type_error(c"an integer is required");
                };
                let Ok(value) = u8::try_from(value) else {
                    return value_error(c"byte must be in range(0, 256)");
                };
                bytes.push(value);
            }
            bytes
        }
        Some(source) if source.value_type() == value_type => {
            // SAFETY: the exact native type check establishes our userdata.
            unsafe { (&*source.userdata::<ByteArrayState>()).as_slice().to_vec() }
        }
        Some(_) => return type_error(c"cannot convert object to bytearray"),
    };

    // SAFETY: the freshly created native instance owns writable state userdata.
    unsafe { *instance.userdata::<ByteArrayState>() = ByteArrayState::from_bytes(&bytes) };
    return_value(instance)
}

unsafe extern "C" fn bytearray_len(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from the registered bytearray method.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let instance = arguments.get(0).expect("arity checked");
    // SAFETY: method binding guarantees a bytearray receiver.
    let length = unsafe { (&*instance.userdata::<ByteArrayState>()).length };
    let Ok(length) = i64::try_from(length) else {
        return value_error(c"bytearray is too large");
    };
    let mut roots = RootFrame::new();
    let length = roots.integer(length);
    return_value(length)
}

unsafe extern "C" fn bytearray_eq(argc: c_int, argv: ffi::py_StackRef) -> bool {
    // SAFETY: called only from the registered bytearray method.
    let arguments = unsafe { Arguments::from_raw(argc, argv) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let instance = arguments.get(0).expect("arity checked");
    let other = arguments.get(1).expect("arity checked");
    // SAFETY: method binding guarantees a bytearray receiver.
    let bytes = unsafe { (&*instance.userdata::<ByteArrayState>()).as_slice() };
    let equal = if other.value_type() == BYTEARRAY_TYPE.load(Ordering::Acquire) {
        // SAFETY: the exact native type check establishes our userdata.
        bytes == unsafe { (&*other.userdata::<ByteArrayState>()).as_slice() }
    } else if other.is_type(ffi::py_PredefinedType_tp_bytes) {
        bytes == other.bytes().expect("bytes checked")
    } else {
        false
    };
    let mut roots = RootFrame::new();
    let equal = roots.boolean(equal);
    return_value(equal)
}
