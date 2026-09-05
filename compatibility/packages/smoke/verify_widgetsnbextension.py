"""Validate the extension registration descriptor and read its bundled JavaScript entry point."""
import os
import widgetsnbextension
paths = widgetsnbextension._jupyter_nbextension_paths()
assert len(paths) == 1
entry = paths[0]
assert entry["section"] == "notebook"
assert entry["dest"] == "jupyter-js-widgets"
assert entry["require"] == "jupyter-js-widgets/extension"
filename = os.path.join(os.path.dirname(widgetsnbextension.__file__), entry["src"], "extension.js")
with open(filename, "rb") as script:
    assert len(script.read(1024)) == 1024
print("verified widgetsnbextension")
