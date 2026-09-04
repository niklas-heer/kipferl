"""Return a sorted list of matching paths.

root_dir and dir_fd are accepted but ignored. recursive=True enables the
runtime's limited ** traversal, capped at 65 directory levels and matching
files by the final pattern component. iglob and rglob are not provided.
"""

from typing import List, Optional

def glob(pathname: str, root_dir: Optional[str] = None, dir_fd: Optional[int] = None,
         recursive: bool = False) -> List[str]: ...
