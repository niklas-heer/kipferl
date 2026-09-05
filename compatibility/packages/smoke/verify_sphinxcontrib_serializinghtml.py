"""Round-trip a page-context dictionary through the package JSON encoder, including a UserString translation proxy."""
from collections import UserString
from sphinxcontrib.serializinghtml import jsonimpl
encoded = jsonimpl.dumps({"title": UserString("Café"), "body": "<p>Hello</p>"})
decoded = jsonimpl.loads(encoded)
assert decoded == {"title": "Café", "body": "<p>Hello</p>"}
print("verified sphinxcontrib-serializinghtml")
