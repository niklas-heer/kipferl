use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const SOURCE: &str = r#"
class _GeneratorContextManager:
    def __init__(self, func, args, kwargs):
        self.gen = func(*args, **kwargs)

    def __enter__(self):
        try:
            return next(self.gen)
        except StopIteration:
            raise RuntimeError("generator didn't yield")

    def __exit__(self, *args):
        try:
            next(self.gen)
        except StopIteration:
            return False
        raise RuntimeError("generator didn't stop")

class _ContextManagerWrapper:
    def __init__(self, func):
        self.func = func

    def __call__(self, *args, **kwargs):
        return _GeneratorContextManager(self.func, args, kwargs)

def contextmanager(func):
    return _ContextManagerWrapper(func)

class closing:
    def __init__(self, thing):
        self.thing = thing

    def __enter__(self):
        return self.thing

    def __exit__(self, *exc_info):
        self.thing.close()
        return False

class suppress:
    def __init__(self, *exceptions):
        self._exceptions = exceptions

    def __enter__(self):
        return self

    def __exit__(self, *args):
        exctype = args[0] if len(args) > 0 else None
        if exctype is None:
            return False
        for exc in self._exceptions:
            if issubclass(exctype, exc):
                return True
        return False

class nullcontext:
    def __init__(self, enter_result=None):
        self.enter_result = enter_result

    def __enter__(self):
        return self.enter_result

    def __exit__(self, *excinfo):
        return False
"#;

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"contextlib",
    kind: NativeModuleKind::Create,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(execute_module(module, SOURCE), "embedded contextlib module");
}
