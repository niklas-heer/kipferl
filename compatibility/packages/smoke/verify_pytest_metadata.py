"""Run the metadata configuration hook with a local pytest configuration double; merge CLI and JSON metadata and format the report header."""
from pytest_metadata.plugin import pytest_configure, pytest_report_header, metadata_key
class Plugins:
    def list_plugin_distinfo(self):
        return []
class Hooks:
    def pytest_metadata(self, metadata, config):
        metadata["hook_seen"] = True
class Config:
    def __init__(self):
        self.stash = {}
        self.pluginmanager = Plugins()
        self.hook = Hooks()
    def getoption(self, name):
        options = {"metadata": [("Build", "42")], "metadata_from_json": '{"Region":"eu"}', "metadata_from_json_file": None, "verbose": 1}
        return options[name]
config = Config()
pytest_configure(config)
metadata = config.stash[metadata_key]
assert metadata["Build"] == "42"
assert metadata["Region"] == "eu"
assert metadata["hook_seen"] is True
assert "Python" in metadata and "Packages" in metadata
assert "metadata:" in pytest_report_header(config)
print("verified pytest-metadata")
