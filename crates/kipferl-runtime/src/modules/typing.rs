use std::ffi::{CStr, c_int};
use std::sync::atomic::{AtomicI16, Ordering};

use kipferl_pocketpy_sys as ffi;

use crate::native::{
    Arguments, NativeFunction, NativeModule, NativeModuleKind, RootFrame, Value, bind_type_method,
    bind_type_signature, create_type, execute_module, return_string, return_value, type_error,
    type_object,
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
        name: c"override",
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
    c"TypeGuard",
    c"TypeIs",
];

/// Special forms are subscriptable in annotations and aliases, such as
/// `Optional[int]` or `Union[int, str]`, and subscription returns the form
/// itself. `NamedTuple`, `TypedDict` and `NewType` cover their common uses;
/// `NamedTuple` instances are not tuples and their fields stay assignable.
const SOURCE: &str = r#"
class _SpecialForm:
    def __init__(self, name):
        self._name = name

    def __getitem__(self, parameters):
        return self

    def __call__(self, *args, **kwargs):
        raise TypeError("Cannot instantiate typing." + self._name)

    def __repr__(self):
        return "typing." + self._name


class TypedDict(dict):
    pass


def NewType(name, tp):
    def new_type(value):
        return value

    new_type.__name__ = name
    return new_type


class NamedTuple:
    def __init__(self, *args, **kwargs):
        fields = self._fields
        if len(args) > len(fields):
            raise TypeError(
                "__new__() takes %d positional arguments but %d were given"
                % (len(fields) + 1, len(args) + 1)
            )
        for index in range(len(fields)):
            name = fields[index]
            if index < len(args):
                if name in kwargs:
                    raise TypeError("got multiple values for argument '%s'" % name)
                value = args[index]
            elif name in kwargs:
                value = kwargs.pop(name)
            elif name in self._field_defaults:
                value = self._field_defaults[name]
            else:
                raise TypeError("__new__() missing required argument: '%s'" % name)
            setattr(self, name, value)
        for name in kwargs:
            raise TypeError("__new__() got an unexpected keyword argument '%s'" % name)

    @property
    def _fields(self):
        # the fields are those of the class that names NamedTuple as its base
        cls = type(self)
        while cls.__base__ is not NamedTuple:
            cls = cls.__base__
        return tuple(cls.__annotations__)

    @property
    def _field_defaults(self):
        defaults = {}
        for name in self._fields:
            if hasattr(type(self), name):
                defaults[name] = getattr(type(self), name)
        return defaults

    def __iter__(self):
        return iter(self._values())

    def __len__(self):
        return len(self._fields)

    def __getitem__(self, index):
        return self._values()[index]

    def __eq__(self, other):
        if isinstance(other, NamedTuple):
            other = other._values()
        return self._values() == other

    def __ne__(self, other):
        return not self == other

    def __lt__(self, other):
        return self._values() < tuple(other)

    def __hash__(self):
        return hash(self._values())

    def __repr__(self):
        parts = []
        for name in self._fields:
            parts.append(name + "=" + repr(getattr(self, name)))
        return type(self).__name__ + "(" + ", ".join(parts) + ")"

    def _values(self):
        return tuple(getattr(self, name) for name in self._fields)

    def _asdict(self):
        return {name: getattr(self, name) for name in self._fields}

    def _replace(self, **changes):
        values = self._asdict()
        values.update(changes)
        return type(self)(**values)
"#;

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
    let mut source = String::from(SOURCE);
    for &name in SENTINELS {
        let name = name.to_string_lossy();
        source.push_str(&name);
        source.push_str(" = _SpecialForm('");
        source.push_str(&name);
        source.push_str("')\n");
    }
    assert!(
        execute_module(module, &source),
        "embedded typing compatibility layer failed"
    );
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
