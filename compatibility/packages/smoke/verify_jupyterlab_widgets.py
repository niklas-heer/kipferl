"""Resolve the extension registration descriptor and read its package.json identity."""
import os
import json
import jupyterlab_widgets
paths = jupyterlab_widgets._jupyter_labextension_paths()
assert len(paths) == 1
assert paths[0]["dest"] == "@jupyter-widgets/jupyterlab-manager"
with open(os.path.join(paths[0]["src"], "package.json")) as metadata:
    package = json.load(metadata)
assert package["name"] == "@jupyter-widgets/jupyterlab-manager"
print("verified jupyterlab-widgets")
