"""Lightweight runtime annotation compatibility, without type enforcement.

TypeVar retains only its name; constraints, bound and variance are accepted but
ignored. The registered decorators and cast return their argument unchanged.
get_type_hints, get_origin and get_args return {}, None and () respectively.
NewType, no_type_check_decorator and dataclass_transform are not provided.
"""

# Any and the special forms below are editor intrinsics. At runtime the
# aliases are placeholder types and the special forms are opaque sentinels.
# Keep these declarations self-contained: importing typing here is circular.
class Any: ...

class _SpecialForm:
    def __getitem__(self, parameters: Any, /) -> Any: ...

class _Alias:
    def __getitem__(self, parameters: Any, /) -> Any: ...

List = _Alias()
Dict = _Alias()
Set = _Alias()
FrozenSet = _Alias()
Tuple = _Alias()
Type = _Alias()
Callable = _Alias()
Generic = _Alias()
Protocol = _Alias()
Sequence = _Alias()
MutableSequence = _Alias()
Mapping = _Alias()
MutableMapping = _Alias()
Iterable = _Alias()
Iterator = _Alias()
Generator = _Alias()
Reversible = _Alias()
Container = _Alias()
Collection = _Alias()
Hashable = _Alias()
Sized = _Alias()
Awaitable = _Alias()
Coroutine = _Alias()
AsyncGenerator = _Alias()
AsyncIterator = _Alias()
AsyncIterable = _Alias()
IO = _Alias()
TextIO = _Alias()
BinaryIO = _Alias()

Optional: _SpecialForm
Union: _SpecialForm
ClassVar: _SpecialForm
Final: _SpecialForm
Literal: _SpecialForm
Annotated: _SpecialForm
NoReturn: _SpecialForm
Never: _SpecialForm
Self: _SpecialForm
LiteralString: _SpecialForm
TypeAlias: _SpecialForm
Concatenate: _SpecialForm
ParamSpec: _SpecialForm
TypeVarTuple: _SpecialForm
Unpack: _SpecialForm
Required: _SpecialForm
NotRequired: _SpecialForm
ReadOnly: _SpecialForm


TYPE_CHECKING: bool

class TypeVar:
    __name__: Any
    def __new__(cls, name: str, *constraints: Any, bound: Any = None,
                covariant: bool = False, contravariant: bool = False) -> "TypeVar": ...
    def __repr__(self) -> str: ...

def cast(typ: Any, value: Any, /) -> Any: ...
def get_type_hints(obj: Any, globalns: Any = None, localns: Any = None, /) -> dict[str, Any]: ...
def get_origin(value: Any, /) -> None: ...
def get_args(value: Any, /) -> tuple[()]: ...
def overload(value: Any, /) -> Any: ...
def no_type_check(value: Any, /) -> Any: ...
def runtime_checkable(value: Any, /) -> Any: ...
def final(value: Any, /) -> Any: ...
