"""Register the theme, locate its configuration file, and apply show_powered_by context conversion."""
import os
import alabaster
class Config:
    html_theme_options = {"show_powered_by": "false"}
class App:
    def __init__(self):
        self.config = Config()
        self.events = []
        self.themes = {}
    def require_sphinx(self, version):
        assert version == "6.2"
    def add_html_theme(self, name, path):
        self.themes[name] = path
    def connect(self, name, callback):
        self.events.append(name)
app = App()
result = alabaster.setup(app)
assert result["version"] == "1.0.0"
assert app.events == ["html-page-context"]
with open(os.path.join(app.themes["alabaster"], "theme.conf")) as config:
    assert "[theme]" in config.read()
context = {}
alabaster.update_context(app, "index", "page.html", context, None)
assert context["show_sphinx"] is False
assert context["alabaster_version"] == "1.0.0"
print("verified alabaster")
