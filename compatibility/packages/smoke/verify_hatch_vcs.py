"""Construct setuptools-scm options, preserve tag/fallback values, remove implicit file-writing options, and reject invalid tag-pattern types."""
from hatch_vcs.version_source import VCSVersionSource
source = VCSVersionSource(".", {"tag-pattern": "v(.*)", "fallback-version": "1.2.3", "raw-options": {"write_to": "version.py"}})
options = source.construct_setuptools_scm_config()
assert options["root"] == "."
assert options["tag_regex"] == "v(.*)"
assert options["fallback_version"] == "1.2.3"
assert "write_to" not in options
failed = False
try:
    VCSVersionSource(".", {"tag-pattern": 3}).construct_setuptools_scm_config()
except TypeError:
    failed = True
assert failed
print("verified hatch-vcs")
