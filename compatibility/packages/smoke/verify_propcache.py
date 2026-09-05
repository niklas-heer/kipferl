"""Compute each regular/under-cache property once across repeat reads and verify its explicit cache entry."""
from propcache.api import cached_property, under_cached_property
class Summary:
    def __init__(self):
        self.calls = 0
        self._cache = {}
    @cached_property
    def total(self):
        self.calls += 1
        return 42
    @under_cached_property
    def label(self):
        self.calls += 1
        return "ready"
summary = Summary()
assert summary.total == 42 and summary.total == 42
assert summary.calls == 1
assert summary.label == "ready" and summary.label == "ready"
assert summary.calls == 2
assert summary._cache["label"] == "ready"
print("verified propcache")
