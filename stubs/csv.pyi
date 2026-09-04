"""Comma-separated text with eager readers and minimal quoting.

reader returns a list of rows and parses each input string independently;
quoted fields spanning input lines and dialect configuration are not supported.
Writer methods return the result of the supplied output object's write method.
"""

from typing import Any, Dict, Iterable, Iterator, List, Optional

QUOTE_MINIMAL: int
QUOTE_ALL: int
QUOTE_NONNUMERIC: int
QUOTE_NONE: int

def reader(csvfile: Iterable[str]) -> List[List[str]]: ...

class writer:
    def __init__(self, output: Any) -> None: ...
    def writerow(self, row: Iterable[Any]) -> Any: ...

class DictReader:
    def __init__(self, data: Iterable[str], fieldnames: Optional[List[str]] = None) -> None: ...
    def __iter__(self) -> Iterator[Dict[str, str]]: ...

class DictWriter:
    fieldnames: List[str]
    def __init__(self, output: Any, fieldnames: List[str]) -> None: ...
    def writeheader(self) -> Any: ...
    def writerow(self, row: Dict[str, Any]) -> Any: ...
