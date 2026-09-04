use crate::native::{NativeModule, NativeModuleKind, Value, execute_module};

const COMPATIBILITY_SOURCE: &str = r"
partial.func = property(lambda self: self.f)
partial.keywords = property(lambda self: self.kwargs)

class _RustCmpKey:
    def __init__(self, obj, compare):
        self.obj = obj
        self._compare = compare

    def __lt__(self, other):
        return self._compare(self.obj, other.obj) < 0

    def __le__(self, other):
        return self._compare(self.obj, other.obj) <= 0

    def __eq__(self, other):
        return self._compare(self.obj, other.obj) == 0

    def __ne__(self, other):
        return self._compare(self.obj, other.obj) != 0

    def __gt__(self, other):
        return self._compare(self.obj, other.obj) > 0

    def __ge__(self, other):
        return self._compare(self.obj, other.obj) >= 0


class _RustCmpKeyFactory:
    def __init__(self, compare):
        self._compare = compare

    def __call__(self, obj):
        return _RustCmpKey(obj, self._compare)


def cmp_to_key(compare):
    return _RustCmpKeyFactory(compare)


class _RustWrapsDecorator:
    def __init__(self, wrapped):
        self._wrapped = wrapped

    def __call__(self, wrapper):
        try:
            wrapper.__name__ = self._wrapped.__name__
        except AttributeError:
            pass
        try:
            wrapper.__doc__ = self._wrapped.__doc__
        except AttributeError:
            pass
        return wrapper


def wraps(wrapped):
    return _RustWrapsDecorator(wrapped)


class _RustKeywordMarker:
    pass


_rust_keyword_marker = _RustKeywordMarker()


class _RustLruCacheWrapper:
    def __init__(self, function, maxsize, typed=False):
        self._function = function
        if maxsize is not None and maxsize < 0:
            maxsize = 0
        self._maxsize = maxsize
        self._typed = typed
        self._cache = {}
        self._hits = 0
        self._misses = 0

    def _make_key(self, args, kwargs):
        key = args
        pairs = []
        if len(kwargs) != 0:
            for name in kwargs:
                pairs.append((name, kwargs[name]))
            pairs.sort()
            key = (args, _rust_keyword_marker, tuple(pairs))
        if self._typed:
            argument_types = []
            for value in args:
                argument_types.append(type(value))
            keyword_types = []
            for name, value in pairs:
                keyword_types.append(type(value))
            key = (key, tuple(argument_types), tuple(keyword_types))
        return key

    def __call__(self, *args, **kwargs):
        key = self._make_key(args, kwargs)
        if key in self._cache:
            self._hits += 1
            result = self._cache.pop(key)
            self._cache[key] = result
            return result

        self._misses += 1
        result = self._function(*args, **kwargs)
        if self._maxsize != 0:
            if self._maxsize is not None and len(self._cache) >= self._maxsize:
                oldest = next(iter(self._cache))
                self._cache.pop(oldest)
            self._cache[key] = result
        return result

    def cache_info(self):
        return (self._hits, self._misses, self._maxsize, len(self._cache))

    def cache_clear(self):
        self._cache = {}
        self._hits = 0
        self._misses = 0


def lru_cache(maxsize=128, typed=False):
    if callable(maxsize) and not isinstance(maxsize, type):
        return _RustLruCacheWrapper(maxsize, 128, typed)

    def decorator(function):
        return _RustLruCacheWrapper(function, maxsize, typed)

    return decorator


def cache(function):
    return _RustLruCacheWrapper(function, None, False)
";

pub(super) const MODULE: NativeModule = NativeModule {
    name: c"functools",
    kind: NativeModuleKind::ImportAndExtend,
    functions: &[],
    signatures: &[],
    int_constants: &[],
    type_aliases: &[],
    initializer: Some(initialize),
};

fn initialize(module: Value) {
    assert!(
        execute_module(module, COMPATIBILITY_SOURCE),
        "embedded functools compatibility layer failed"
    );
}
