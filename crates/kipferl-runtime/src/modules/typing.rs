use std::ffi::{CStr, c_int};
use std::sync::atomic::{AtomicI16, Ordering};

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, bind_type_method,
    bind_type_signature, create_type, return_string, return_value, type_error, type_object,
};

static TYPEVAR_TYPE: AtomicI16 = AtomicI16::new(0);

const FUNCTIONS: &[NativeFunction] = &[
    NativeFunction {
        name: c"cast",
        callback: cast,
    },
    NativeFunction {
        name: c"overload",
        callback: identity,
    },
    NativeFunction {
        name: c"final",
        callback: identity,
    },
    NativeFunction {
        name: c"no_type_check",
        callback: identity,
    },
    NativeFunction {
        name: c"runtime_checkable",
        callback: identity,
    },
    NativeFunction {
        name: c"get_args",
        callback: get_args,
    },
    NativeFunction {
        name: c"get_origin",
        callback: get_origin,
    },
    NativeFunction {
        name: c"get_type_hints",
        callback: get_type_hints,
    },
];

const TYPE_ALIASES: &[&CStr] = &[
    c"List",
    c"Dict",
    c"Set",
    c"FrozenSet",
    c"Tuple",
    c"Type",
    c"Callable",
    c"Generic",
    c"Protocol",
    c"Sequence",
    c"MutableSequence",
    c"Mapping",
    c"MutableMapping",
    c"Iterable",
    c"Iterator",
    c"Generator",
    c"Reversible",
    c"Container",
    c"Collection",
    c"Hashable",
    c"Sized",
    c"Awaitable",
    c"Coroutine",
    c"AsyncGenerator",
    c"AsyncIterator",
    c"AsyncIterable",
    c"IO",
    c"TextIO",
    c"BinaryIO",
];

const SENTINELS: &[&CStr] = &[
    c"Any",
    c"Optional",
    c"Union",
    c"ClassVar",
    c"Final",
    c"Literal",
    c"Annotated",
    c"NoReturn",
    c"Never",
    c"Self",
    c"LiteralString",
    c"TypeAlias",
    c"Concatenate",
    c"ParamSpec",
    c"TypeVarTuple",
    c"Unpack",
    c"Required",
    c"NotRequired",
    c"ReadOnly",
];

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"typing",
    kind: NativeModuleKind::Create,
    functions: FUNCTIONS,
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    let typevar = create_type(module, c"TypeVar");
    TYPEVAR_TYPE.store(typevar, Ordering::Release);
    bind_type_signature(
        typevar,
        c"__new__(cls, name, *constraints, bound=None, covariant=False, contravariant=False)",
        typevar_new,
    );
    bind_type_method(typevar, c"__repr__", typevar_repr);
    module.set_attribute(c"TypeVar", type_object(typevar));

    for &name in TYPE_ALIASES {
        let alias = create_type(module, name);
        module.set_attribute(name, type_object(alias));
    }
    for &name in SENTINELS {
        let mut roots = RootFrame::new();
        let sentinel = roots.object(crate::native::predefined_type(
            ffi::py_PredefinedType_tp_object,
        ));
        module.set_attribute(name, sentinel);
    }
    let mut roots = RootFrame::new();
    let type_checking = roots.boolean(false);
    module.set_attribute(c"TYPE_CHECKING", type_checking);
}

unsafe extern "C" fn typevar_new(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // The declaration binder validates and expands the TypeVar signature.
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    let Some(name) = arguments.get(1) else {
        return false;
    };
    let value_type = TYPEVAR_TYPE.load(Ordering::Acquire);
    assert_ne!(value_type, 0, "TypeVar used before module initialization");
    let mut roots = RootFrame::new();
    let instance = roots.object(value_type);
    instance.set_attribute(c"__name__", name);
    return_value(instance)
}

unsafe extern "C" fn typevar_repr(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let name = arguments
        .get(0)
        .and_then(|value| value.attribute(c"__name__"))
        .and_then(Value::string);
    if let Some(name) = name
        && name.len() < 127
    {
        return return_string(&format!("~{name}"));
    }
    return_string("~T")
}

unsafe extern "C" fn cast(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(2, 2) {
        return false;
    }
    let Some(value) = arguments.get(1) else {
        return type_error(c"cast() requires a value");
    };
    return_value(value)
}

unsafe extern "C" fn identity(argc: c_int, stack: ffi::py_StackRef) -> bool {
    // SAFETY: PocketPy supplies an active callback stack containing `argc` values.
    let arguments = unsafe { Arguments::from_raw(argc, stack) };
    if !arguments.require_arity(1, 1) {
        return false;
    }
    let Some(value) = arguments.get(0) else {
        return type_error(c"decorator requires an argument");
    };
    return_value(value)
}

#[expect(
    clippy::expect_used,
    reason = "A zero-length tuple is always representable by the VM signed-int length API."
)]
unsafe extern "C" fn get_args(argc: c_int, stack: ffi::py_StackRef) -> bool {
    if !require_arity(argc, stack, 1, 1) {
        return false;
    }
    let mut roots = RootFrame::new();
    let value = roots.tuple(0).expect("zero-length tuple is representable");
    return_value(value)
}

unsafe extern "C" fn get_origin(argc: c_int, stack: ffi::py_StackRef) -> bool {
    if !require_arity(argc, stack, 1, 1) {
        return false;
    }
    let mut roots = RootFrame::new();
    let value = roots.none();
    return_value(value)
}

unsafe extern "C" fn get_type_hints(argc: c_int, stack: ffi::py_StackRef) -> bool {
    if !require_arity(argc, stack, 1, 3) {
        return false;
    }
    let mut roots = RootFrame::new();
    let value = roots.dict();
    return_value(value)
}

fn require_arity(argc: c_int, stack: ffi::py_StackRef, minimum: usize, maximum: usize) -> bool {
    // SAFETY: called only from a PocketPy callback with its active argument stack.
    unsafe { Arguments::from_raw(argc, stack) }.require_arity(minimum, maximum)
}
