"""Resolve the Python keyword style and extension registration descriptor."""
from jupyterlab_pygments import JupyterStyle, _jupyter_labextension_paths
from pygments.token import Keyword
assert "--jp-mirror-editor-keyword-color" in JupyterStyle.styles[Keyword]
paths = _jupyter_labextension_paths()
assert paths == [{"src": "labextension", "dest": "jupyterlab_pygments"}]
print("verified jupyterlab-pygments")
