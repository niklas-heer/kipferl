import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import check_recipes


class RecipeCheckTests(unittest.TestCase):
    def test_snippet_drift_missing_and_malformed_markers_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            recipes = root / "examples/recipes"
            docs = root / "website/content/docs"
            recipes.mkdir(parents=True)
            docs.mkdir(parents=True)
            (recipes / "sample.py").write_text("print('hello')\n")
            page = docs / "sample.mdx"
            page.write_text("{/* recipe:sample.py */}\n```python\nprint('hello')\n```\n{/* endrecipe */}\n")
            self.assertEqual(check_recipes.check_snippets(root), ["sample.py"])
            page.write_text(page.read_text().replace("hello", "stale"))
            with self.assertRaisesRegex(check_recipes.RecipeError, "snippet drift"):
                check_recipes.check_snippets(root)
            page.write_text("{/* recipe:sample.py */}\nmissing fence")
            with self.assertRaisesRegex(check_recipes.RecipeError, "Malformed"):
                check_recipes.check_snippets(root)
            page.write_text("No recipes")
            with self.assertRaisesRegex(check_recipes.RecipeError, "missing from docs"):
                check_recipes.check_snippets(root)

    def test_runtime_failure_cannot_pass_as_valid_output(self):
        result = subprocess.CompletedProcess([], -11, '{"ok": true}', "crashed")
        with patch.object(check_recipes.subprocess, "run", return_value=result):
            with self.assertRaisesRegex(check_recipes.RecipeError, "got -11"):
                check_recipes.run_recipe("runtime", "sample.py", [], ".")

    def test_new_recipe_requires_an_execution_fixture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "examples/recipes"
            directory.mkdir(parents=True)
            for name in check_recipes.EXECUTED_RECIPES | {"untested.py"}:
                (directory / name).write_text("print('hello')\n")
            with self.assertRaisesRegex(check_recipes.RecipeError, "execution fixtures"):
                check_recipes.check_execution("runtime", root)

    def test_missing_runtime_and_timeout_are_failures(self):
        for error in [FileNotFoundError("missing runtime"), subprocess.TimeoutExpired("runtime", 20)]:
            with self.subTest(error=type(error).__name__):
                with patch.object(check_recipes.subprocess, "run", side_effect=error):
                    with self.assertRaisesRegex(check_recipes.RecipeError, "could not complete"):
                        check_recipes.run_recipe("runtime", "sample.py", [], ".")

    def test_invalid_or_incorrect_output_is_a_failure(self):
        for output in ["not JSON", '{"wrong": true}']:
            result = subprocess.CompletedProcess([], 0, output, "")
            with self.assertRaises(check_recipes.RecipeError):
                check_recipes.expect_json(result, {"ok": True}, "sample.py")


if __name__ == "__main__":
    unittest.main()
