"""Temporary paths in the runtime's temporary directory.

mktemp returns an unused name without reserving it. mkstemp creates and closes
an empty file and returns its path, rather than a file-descriptor/path tuple.
mkdtemp's optional prefix is a complete path prefix; the default is <temp>/tmp.
Callers are responsible for removing created files and directories.
"""

from typing import Optional

def gettempdir() -> str: ...
def mktemp() -> str: ...
def mkstemp() -> str: ...
def mkdtemp(prefix: Optional[str] = None) -> str: ...
