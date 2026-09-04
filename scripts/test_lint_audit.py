import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from scripts.lint_audit import diagnostics, exception_inventory, exception_issues


class AuditTests(unittest.TestCase):
    def test_deduplicates_lib_and_test_messages_but_keeps_distinct_locations(self):
        event = {"reason": "compiler-message", "message": {
            "level": "warning", "code": {"code": "clippy::string_slice"},
            "message": "slicing may panic", "rendered": "diagnostic",
            "spans": [{"is_primary": True, "file_name": "src/lib.rs", "line_start": 3, "column_start": 4}],
        }}
        first = json.dumps(event)
        event["message"]["spans"][0]["line_start"] = 8
        found = diagnostics([first, first, json.dumps(event), "not json"], "full")
        self.assertEqual(len(found), 2)
        self.assertEqual({r["line"] for r in found.values()}, {3, 8})

    def test_retains_compiler_errors_without_a_source_span_or_lint_code(self):
        event = {"reason": "compiler-message", "message": {
            "level": "error", "code": None, "message": "unknown compiler failure", "spans": [],
        }}
        found = diagnostics([json.dumps(event)], "core")
        self.assertEqual(len(found), 1)
        record = next(iter(found.values()))
        self.assertEqual((record["level"], record["file"], record["lint"]), ("error", "<compiler>", "compiler"))

    def test_inventory_retains_multiline_and_conditional_exceptions(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "crates").mkdir()
            (root / "crates/lib.rs").write_text(
                '#![allow(non_camel_case_types)]\n'
                '#[expect(\n clippy::indexing_slicing,\n reason = "index in [0, 8)"\n)]\n'
                '#[cfg_attr(unix, expect(clippy::exit, reason = "process handoff"))]\n'
                '#[deny(clippy::panic)]\n'
            )
            found = exception_inventory(root)
        self.assertEqual(len(found), 3)
        self.assertEqual(found[1]["line"], 2)
        self.assertEqual(found[1]["reason"], "index in [0, 8)")
        self.assertEqual(found[2]["lints"], ["clippy::exit"])
        self.assertEqual(exception_issues(found), [])

    def test_exception_policy_rejects_missing_reasons_and_blanket_groups(self):
        records = [
            {"file": "lib.rs", "line": 1, "lints": ["clippy::expect_used"], "reason": None},
            {"file": "lib.rs", "line": 2, "lints": ["clippy::pedantic"], "reason": "legacy code"},
        ]
        issues = exception_issues(records)
        self.assertEqual(len(issues), 2)
        self.assertIn("concrete reason", issues[0])
        self.assertIn("blanket", issues[1])


    def inventory(self, source):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "crates").mkdir()
            (root / "crates/lib.rs").write_text(source)
            return exception_inventory(root)

    def test_blanket_warning_and_all_clippy_groups_are_rejected(self):
        for name in ("warnings", "clippy::all", "clippy::style", "clippy::complexity",
                     "clippy::correctness", "clippy::suspicious", "clippy::perf",
                     "clippy::restriction", "clippy::cargo", "clippy::nursery"):
            with self.subTest(name=name):
                found = self.inventory(f'#![allow({name}, reason = "legacy code")]\n')
                self.assertEqual(len(found), 1)
                self.assertTrue(any("blanket" in issue for issue in exception_issues(found)))

    def test_each_conditional_exception_requires_its_own_reason(self):
        found = self.inventory(
            '#[cfg_attr(unix,\n'
            ' expect(clippy::unwrap_used, reason = "fixture setup"),\n'
            ' cfg_attr(test, allow(clippy::indexing_slicing)),\n'
            ' expect(clippy::panic, reason = "bounded test wait"))]\n'
        )
        self.assertEqual(len(found), 3)
        self.assertEqual([record["line"] for record in found], [2, 3, 4])
        self.assertEqual([record["reason"] for record in found],
                         ["fixture setup", None, "bounded test wait"])
        issues = exception_issues(found)
        self.assertEqual(len(issues), 1)
        self.assertIn("lib.rs:3", issues[0])

    def test_inline_attributes_are_found_but_comments_and_literals_are_ignored(self):
        found = self.inventory(r'''
// #[allow(clippy::unwrap_used)]
/* outer /* #[allow(clippy::panic)] */ #[expect(clippy::exit)] */
const NORMAL: &str = "\n#[allow(clippy::expect_used)]";
const RAW: &str = r###"first " quoted
#[allow(clippy::string_slice)]
"###;
const BYTES: &[u8] = br##"first " quoted
#[allow(clippy::indexing_slicing)]
"##;
const C: &std::ffi::CStr = cr##"first " quoted
#[allow(clippy::exit)]
"##;
fn borrow<'a>(text: &'a str) -> &'a str { text }
mod actual { #[expect(clippy::unwrap_used, reason = "fixture setup")] fn fixture() {} }
''')
        self.assertEqual(len(found), 1)
        self.assertEqual(found[0]["lints"], ["clippy::unwrap_used"])
        self.assertEqual(exception_issues(found), [])

    def test_reason_strings_keep_brackets_quotes_and_rust_escapes(self):
        found = self.inventory(r'''
#[expect(clippy::indexing_slicing, reason = "index [0, 8); quote \"checked\"; \x41\u{42}\'s")]
#[expect(clippy::panic, reason = r##"literal "quote" and [brackets], clippy::all"##)]
''')
        self.assertEqual(len(found), 2)
        self.assertEqual(found[0]["reason"], 'index [0, 8); quote "checked"; AB\'s')
        self.assertEqual(found[1]["reason"], 'literal "quote" and [brackets], clippy::all')
        self.assertEqual(found[1]["lints"], ["clippy::panic"])
        self.assertEqual(exception_issues(found), [])

    def test_whitespace_reason_is_rejected(self):
        found = self.inventory('#[expect(clippy::panic, reason = "  ")]\n')
        self.assertEqual(len(exception_issues(found)), 1)

    def test_macro_arguments_are_not_lint_declarations(self):
        found = self.inventory('#[some_macro(allow(clippy::panic))]\n')
        self.assertEqual(found, [])


if __name__ == "__main__":
    unittest.main()
