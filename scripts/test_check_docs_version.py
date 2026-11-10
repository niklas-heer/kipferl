import tempfile
import unittest
from pathlib import Path

from scripts import check_docs_version

RELEASES = "https://github.com/niklas-heer/kipferl/releases"


class DocsVersionCheckTests(unittest.TestCase):
    def check_fixture(self, version, constant="0.7.3", files=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            contents = {
                "VERSION": version + "\n",
                check_docs_version.CONSTANT: f'export const stableVersion = "{constant}";\n',
                "README.md": f"[v0.7.3]({RELEASES}/tag/v0.7.3)\n",
                "website/content/docs/getting-started/installation.mdx": (
                    f"release={RELEASES}/download/v0.7.3\n\n```\nKipferl v0.7.3\n```\n"
                    f"Historical [v0.6.0]({RELEASES}/tag/v0.6.0) downloads.\n"
                ),
            }
            contents.update(files or {})
            for name, text in contents.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            return check_docs_version.check(root)

    def test_current_documentation_passes_with_historical_links(self):
        self.assertEqual(self.check_fixture("0.7.3"), [])

    def test_version_bump_without_documentation_fails(self):
        findings = self.check_fixture("0.7.4")
        self.assertTrue(any("stableVersion is 0.7.3" in finding for finding in findings))
        self.assertTrue(any("README.md: does not link the v0.7.4" in finding for finding in findings))
        self.assertTrue(any("installation.mdx:4: claims v0.7.3" in finding for finding in findings))

    def test_stale_version_claim_in_any_docs_page_fails(self):
        findings = self.check_fixture(
            "0.7.3", files={"website/content/docs/guides/packages.mdx": "verify it reports `v0.7.2`.\n"}
        )
        self.assertEqual(findings, ["website/content/docs/guides/packages.mdx:1: claims v0.7.2, VERSION is 0.7.3"])

    def test_link_to_unreleased_version_fails(self):
        findings = self.check_fixture(
            "0.7.3", files={"compatibility/packages/README.md": f"[next]({RELEASES}/tag/v0.8.0)\n"}
        )
        self.assertEqual(findings, ["compatibility/packages/README.md: links unreleased v0.8.0, VERSION is 0.7.3"])

    def test_release_candidate_version_is_skipped(self):
        self.assertEqual(self.check_fixture("0.8.0-rc.1"), [])


if __name__ == "__main__":
    unittest.main()
