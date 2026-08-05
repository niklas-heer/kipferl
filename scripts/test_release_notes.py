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
        guidance = release_notes.installation_section(
            "v0.6.0-rc.2", "niklas-heer/kipferl"
        )

        self.assertIn("prerelease", guidance)
        self.assertIn(".sha256", guidance)
        self.assertIn("kipferl-macos-aarch64", guidance)
        self.assertIn("kipferl-linux-x86_64", guidance)
        self.assertIn(
            "https://github.com/niklas-heer/kipferl/releases/tag/v0.6.0-rc.2",
            guidance,
        )
        self.assertNotIn("brew install", guidance)

    def test_stable_release_uses_homebrew(self):
        guidance = release_notes.installation_section(
            "v0.6.0", "niklas-heer/kipferl"
        )

        self.assertIn("brew install niklas-heer/tap/kipferl", guidance)
        self.assertIn("brew upgrade kipferl", guidance)

    def test_assembly_owns_links_and_asset_names(self):
        notes = release_notes.assemble_release_notes(
            "A concise generated summary.",
            "v0.6.0-rc.2",
            "v0.6.0-rc.1",
            "niklas-heer/kipferl",
        )

        self.assertTrue(notes.startswith("A concise generated summary."))
        self.assertIn("kipferl-linux-aarch64", notes)
        self.assertIn(
            "https://github.com/niklas-heer/kipferl/compare/"
            "v0.6.0-rc.1...v0.6.0-rc.2",
            notes,
        )

    def test_generated_summary_cannot_override_release_metadata(self):
        release_notes.validate_generated_summary("Add a reliable watch mode.")

        for invalid in (
            "### Installation\nDownload kipferl-macos-arm64",
            "See https://github.com/wrong/project/releases",
            "brew install something/else",
            "Full changelog: made up",
        ):
            with self.subTest(invalid=invalid):
                with self.assertRaises(ValueError):
                    release_notes.validate_generated_summary(invalid)


if __name__ == "__main__":
    unittest.main()
