"""
typing - Native typing module for kipferl
"""

from typing import Optional, Any, List, Callable, TypeVar, Iterable, Iterator

def TypeVar() -> None:
    ...

def cast() -> None:
    ...

def get_type_hints() -> None:
    ...

def get_origin(value: Any) -> None:
    """
    get_origin(tp) -> None
    """
    ...

def get_args(value: Any) -> None:
    """
    get_args(tp) -> () (empty tuple)
    """
    ...

def NewType() -> None:
    ...

def overload(value: Any) -> None:
    """
    Decorator functions - return argument unchanged
    """
    ...

def no_type_check(value: Any) -> None:
    ...

def no_type_check_decorator(value: Any) -> None:
    ...

def runtime_checkable(value: Any) -> None:
    ...

def final(value: Any) -> None:
    ...

def dataclass_transform(value: Any) -> None:
    ...
