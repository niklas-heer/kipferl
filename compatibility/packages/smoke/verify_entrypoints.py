"""Parse json:loads, load and invoke the decoder, and reject a malformed declaration."""
from entrypoints import EntryPoint, BadEntryPoint
entry = EntryPoint.from_string("json:loads", "decode")
assert entry.name == "decode"
assert entry.module_name == "json"
assert entry.object_name == "loads"
assert entry.load()('{"answer": 42}') == {"answer": 42}
failed = False
try:
    EntryPoint.from_string("not a valid entry point!", "broken")
except BadEntryPoint:
    failed = True
assert failed
print("verified entrypoints")
