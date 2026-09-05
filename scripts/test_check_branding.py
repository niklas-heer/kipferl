import tempfile
import unittest
from pathlib import Path

from scripts import check_branding


class BrandingCheckTests(unittest.TestCase):
    def check_fixture(self, filename, content):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content.encode() if isinstance(content, str) else content)
            return check_branding.check_paths(root, [filename])

    def test_case_insensitive_old_names_rejected_in_text(self):
        for name in ("mcharm", "MCHARM", "uChArM", "µcharm", "μCHARM", "μ charm", "M charm", "kipval"):
            with self.subTest(name=name):
                findings = self.check_fixture("README.md", f"Use {name} now.\n")
                self.assertEqual(len(findings), 1)
                self.assertIn("README.md:1", findings[0])

    def test_retired_names_rejected_in_paths_even_with_binary_content(self):
        for filename in ("bin/MCHARM", "assets/uChArM.png", "docs/µcharm.md", "KIPVAL.txt"):
            with self.subTest(filename=filename):
                findings = self.check_fixture(filename, b"\x00\xff")
                self.assertEqual(len(findings), 1)
                self.assertIn("filename", findings[0])

    def test_frozen_magic_exact_lines_allowed_only_in_format_source(self):
        source = "crates/kipferl-format/src/lib.rs"
        declaration = 'pub const TRAILER_MAGIC: [u8; 8] = *b"MCHARM01";'
        assertion = 'assert_eq!(TRAILER_MAGIC, *b"MCHARM01");'
        self.assertEqual(self.check_fixture(source, declaration + "\n    " + assertion), [])
        self.assertTrue(self.check_fixture("new.rs", declaration))
        self.assertTrue(self.check_fixture(source, declaration + " // MCHARM product"))
        self.assertTrue(self.check_fixture(source, declaration + '\nprintln!("mcharm");'))

    def test_retired_drawing_is_rejected_without_plaintext_name(self):
        self.assertTrue(self.check_fixture("logo.rs", check_branding.RETIRED_BANNER))

    def test_historical_exception_does_not_exempt_new_line(self):
        # Use an actual reviewed historical line when the migration list exists.
        candidates = [(path, lines) for path, lines in check_branding.ALLOWED_LINES.items()
                      if path != "crates/kipferl-format/src/lib.rs"]
        self.assertTrue(candidates, "Historical exceptions must be explicit and tested")
        path, lines = candidates[0]
        approved = sorted(lines)[0]
        self.assertEqual(self.check_fixture(path, approved), [])
        self.assertTrue(self.check_fixture(path, approved + "\nInstall mcharm today."))

    def test_current_branding_and_nontext_assets_pass(self):
        self.assertEqual(self.check_fixture("README.md", "Use Kipferl and kipferl.dev."), [])
        self.assertEqual(self.check_fixture("app.bin", b"\x00mcharm\xff"), [])
        self.assertEqual(self.check_fixture("image.png", b"\xffmcharm"), [])


if __name__ == "__main__":
    unittest.main()
