import importlib.util
from pathlib import Path
import sys
import types
import unittest


RELEASE_SCRIPT = Path(__file__).with_name("release.py")
fake_kipferl = types.ModuleType("kipferl")
for function_name in [
    "box",
    "confirm",
    "error",
    "info",
    "rule",
    "select",
    "style",
    "success",
    "warning",
]:
    setattr(fake_kipferl, function_name, lambda *args, **kwargs: None)
sys.modules["kipferl"] = fake_kipferl
SPEC = importlib.util.spec_from_file_location("kipferl_release", RELEASE_SCRIPT)
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class ReleaseVersionTests(unittest.TestCase):
    def test_parses_stable_and_release_candidate_versions(self):
        self.assertEqual(release.parse_version("0.5.0"), (0, 5, 0, None))
        self.assertEqual(
            release.parse_version("0.6.0-rc.2"),
            (0, 6, 0, "rc.2"),
        )

    def test_starts_and_advances_release_candidate_series(self):
        self.assertEqual(release.next_release_candidate("0.5.0"), "0.6.0-rc.1")
        self.assertEqual(
            release.next_release_candidate("0.6.0-rc.1"),
            "0.6.0-rc.2",
        )
        self.assertEqual(release.final_version("0.6.0-rc.2"), "0.6.0")

    def test_rejects_unknown_prerelease_shapes(self):
        with self.assertRaisesRegex(ValueError, "only rc.N"):
            release.next_release_candidate("0.6.0-beta.1")

    def test_updates_only_the_workspace_package_version(self):
        manifest = """[workspace]\nversion = \"unrelated\"\n\n[workspace.package]\nversion = \"0.5.0\"\n"""
        self.assertEqual(
            release.workspace_manifest_with_version(manifest, "0.6.0-rc.1"),
            """[workspace]\nversion = \"unrelated\"\n\n[workspace.package]\nversion = \"0.6.0-rc.1\"\n""",
        )


if __name__ == "__main__":
    unittest.main()
