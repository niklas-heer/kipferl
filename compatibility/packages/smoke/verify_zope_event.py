"""Deliver ordered dictionary events, preserve event identity, then remove the subscriber and verify delivery stops."""
from zope.event import notify, subscribers
seen = []
def record(event):
    seen.append(event)
subscribers.append(record)
first = {"kind": "created", "id": 7}
notify(first)
notify({"kind": "updated", "id": 7})
assert len(seen) == 2
assert seen[0] is first
assert seen[1]["kind"] == "updated"
subscribers.remove(record)
notify({"kind": "removed", "id": 7})
assert len(seen) == 2
print("verified zope-event")
