"""Kipferl's synchronous process API.

run returns a dictionary with returncode, stdout, and stderr keys, rather than
CPython's CompletedProcess. Captured streams contain bytes; uncaptured streams
are None and are discarded. Each captured stream retains its first 1 MiB while
the remainder is drained. shell=True requires a command string.

Popen starts execution when communicate() or wait() is first called. Subsequent
calls reuse the result; text=True decodes piped output and normalizes newlines.
check_output raises RuntimeError for a nonzero exit status. Termination by a
signal produces the negative signal number on supported Unix targets.
"""

from typing import Any, Dict, List, Optional, Tuple, Union

_Args = Union[str, List[str], Tuple[str, ...]]
_Stream = Optional[Union[bytes, str]]

PIPE: int
DEVNULL: int

def run(args: _Args, capture_output: bool = False, shell: bool = False) -> Dict[str, Any]: ...
def call(args: _Args) -> int: ...
def check_output(args: _Args) -> bytes: ...
def getoutput(command: str) -> str: ...
def getstatusoutput(command: str) -> Tuple[int, str]: ...

class Popen:
    args: _Args
    returncode: Optional[int]
    def __init__(self, args: _Args, stdout: Optional[int] = None, stderr: Optional[int] = None, shell: bool = False, text: bool = False) -> None: ...
    def communicate(self) -> Tuple[_Stream, _Stream]: ...
    def wait(self) -> int: ...
