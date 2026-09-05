"""Validate known and unknown classifiers, classifier enumeration, and one deprecated-classifier replacement."""
from trove_classifiers import classifiers, sorted_classifiers, deprecated_classifiers
assert "Programming Language :: Python :: 3" in classifiers
assert "License :: OSI Approved :: MIT License" in classifiers
assert "Kipferl :: invented classifier" not in classifiers
assert set(sorted_classifiers) == classifiers
assert deprecated_classifiers["Natural Language :: Ukranian"] == ["Natural Language :: Ukrainian"]
print("verified trove-classifiers")
