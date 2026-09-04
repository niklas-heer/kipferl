"""Shallow and recursive copying for Kipferl values.

Deep copying preserves shared references and circular containers. Lists and
dictionaries are snapshotted before custom copy hooks run, so hooks may change
the source without invalidating the traversal. Hook exceptions propagate.
The optional memo argument is accepted for compatibility; each call creates
its own internal memo and a fresh dictionary passed to __deepcopy__ hooks.
"""

from typing import Any

Error = RuntimeError

def copy(x: Any) -> Any: ...
def deepcopy(x: Any, memo: Any = None) -> Any: ...
