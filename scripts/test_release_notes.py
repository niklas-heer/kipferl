import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / ".github/scripts/generate_release_notes.py"
SPEC = importlib.util.spec_from_file_location("generate_release_notes", SCRIPT)
release_notes = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(release_notes)


class InstallationGuidanceTests(unittest.TestCase):
    def test_prerelease_uses_direct_downloads(self):
        guidance = release_notes.installation_guidance("v0.6.0-rc.1")

        self.assertIn("prerelease", guidance)
        self.assertIn(".sha256", guidance)
        self.assertNotIn("brew install", guidance)

    def test_stable_release_uses_homebrew(self):
        guidance = release_notes.installation_guidance("v0.6.0")

        self.assertIn("brew install niklas-heer/tap/kipferl", guidance)
        self.assertIn("brew upgrade kipferl", guidance)


if __name__ == "__main__":
    unittest.main()
