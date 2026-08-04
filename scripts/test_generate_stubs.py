import tempfile
import unittest
from pathlib import Path

from scripts import generate_stubs


class CanonicalStubTests(unittest.TestCase):
    def test_checked_in_manifest_matches_canonical_stubs(self):
        stubs = generate_stubs.load_stubs()
        self.assertEqual([stub.filename for stub in stubs], sorted(stub.filename for stub in stubs))
        generate_stubs.validate_runtime_modules(
            stubs, generate_stubs.load_registered_modules()
        )
        generate_stubs.check_manifest(generate_stubs.render_rust_manifest(stubs))

    def test_stub_without_a_runtime_module_is_rejected(self):
        stubs = (generate_stubs.Stub("missing.pyi", "missing: bool\n"),)
        with self.assertRaisesRegex(generate_stubs.StubError, "no registered"):
            generate_stubs.validate_runtime_modules(stubs, frozenset({"available"}))

    def test_invalid_syntax_and_missing_newline_are_rejected(self):
        with tempfile.TemporaryDirectory() as raw_directory:
            directory = Path(raw_directory)
            (directory / "broken.pyi").write_text("def broken(: ...\n", encoding="utf-8")
            with self.assertRaisesRegex(generate_stubs.StubError, "invalid Python syntax"):
                generate_stubs.load_stubs(directory)

            (directory / "broken.pyi").write_text("def valid() -> None: ...", encoding="utf-8")
            with self.assertRaisesRegex(generate_stubs.StubError, "end with a newline"):
                generate_stubs.load_stubs(directory)

    def test_export_is_an_exact_copy_of_the_canonical_set(self):
        stubs = generate_stubs.load_stubs()
        with tempfile.TemporaryDirectory() as raw_directory:
            output = Path(raw_directory)
            (output / "stale.pyi").write_text("stale: bool\n", encoding="utf-8")
            generate_stubs.write_output(stubs, output)
            generate_stubs.check_output(stubs, output)
            self.assertFalse((output / "stale.pyi").exists())

            first = output / stubs[0].filename
            first.write_text("drift: bool\n", encoding="utf-8")
            with self.assertRaisesRegex(generate_stubs.StubError, "stub export drift"):
                generate_stubs.check_output(stubs, output)


if __name__ == "__main__":
    unittest.main()
