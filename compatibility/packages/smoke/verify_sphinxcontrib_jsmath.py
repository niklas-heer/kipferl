"""Run the inline math visitor with a local translator double; require the expected escaped math span and SkipNode signal."""
from docutils import nodes
from sphinxcontrib.jsmath import html_visit_math
class Translator:
    def __init__(self):
        self.body = []
    def starttag(self, node, tag, suffix, CLASS):
        return '<' + tag + ' class="' + CLASS + '">'
    def encode(self, text):
        return text.replace("&", "&amp;").replace("<", "&lt;")
translator = Translator()
node = nodes.math("", "x < y")
skipped = False
try:
    html_visit_math(translator, node)
except nodes.SkipNode:
    skipped = True
assert skipped
assert "".join(translator.body) == '<span class="math notranslate nohighlight">x &lt; y</span>'
print("verified sphinxcontrib-jsmath")
