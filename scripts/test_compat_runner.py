"""Regression tests for compatibility-gate failures, independent of a Rust build."""

import contextlib
import io
import importlib.util
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "tests/compat_runner.py"
SPEC = importlib.util.spec_from_file_location("compat_runner", SCRIPT)
runner = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = runner
SPEC.loader.exec_module(runner)


class CompatibilityGateTests(unittest.TestCase):
    def test_process_failure_overrides_successful_output(self):
        outputs = [
            "Results: 4 passed, 0 failed, 0 skipped",
            "Ran 4 tests\n\nOK\n",
            "PASS: completed check\n",
            "SKIP: unavailable dependency\n",
        ]
        for output in outputs:
            for status in (1, -11):
                with self.subTest(output=output, status=status):
                    _, failed, _, failures = runner.parse_test_output(
                        output, "fatal error", status
                    )
                    self.assertEqual(failed, 1)
                    self.assertIn(str(status), failures[0])
                    self.assertIn("fatal error", failures[0])

    def test_existing_failures_are_not_double_counted(self):
        self.assertEqual(
            runner.parse_test_output("Results: 4 passed, 2 failed", "", 1)[:3],
            (4, 2, 0),
        )

    def test_skip_only_output_does_not_invent_a_pass(self):
        self.assertEqual(
            runner.parse_test_output("SKIP: unavailable module\n", "", 0)[:3],
            (0, 0, 1),
        )

    def test_ok_in_test_output_does_not_hide_unittest_failures(self):
        self.assertEqual(
            runner.parse_test_output(
                "the token is OK", "Ran 3 tests\nFAILED (failures=1, errors=1)\n", 1
            )[:3],
            (1, 2, 0),
        )

    def test_successful_unittest_with_skips(self):
        self.assertEqual(
            runner.parse_test_output("", "Ran 3 tests\n\nOK (skipped=1)\n", 0)[:3],
            (2, 0, 1),
        )

    def test_actual_crash_after_summary_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            script = Path(directory) / "crash.py"
            script.write_text(
                "print('Results: 1 passed, 0 failed', flush=True)\n"
                "raise RuntimeError('failure after summary')\n"
            )
            stdout, stderr, code, _ = runner.run_test_file(sys.executable, str(script))
            self.assertEqual(runner.parse_test_output(stdout, stderr, code)[1], 1)

    def test_report_does_not_label_a_crashed_module_fully_compatible(self):
        result = runner.ModuleResult(
            name="io", category="stdlib", cpython_passed=4,
            kipferl_passed=4, kipferl_failed=1, failures=["Process exited with status 1"],
        )
        with tempfile.TemporaryDirectory() as directory, mock.patch("builtins.print"):
            report = Path(directory) / "report.md"
            runner.generate_report([result], report)
            content = report.read_text()
        self.assertIn("1 failing", content)
        self.assertNotIn("✅ Full", content)
        self.assertIn("Process exited with status 1", content)

    def test_report_and_terminal_exclude_external_groups_from_stdlib_coverage(self):
        with mock.patch.object(runner, "STDLIB_MODULES", ["json", "toml"]), \
             mock.patch.object(runner, "CPYTHON_STDLIB_ALL", {"json": "JSON", "csv": "CSV"}), \
             tempfile.TemporaryDirectory() as directory, \
             contextlib.redirect_stdout(io.StringIO()) as terminal:
            report = Path(directory) / "report.md"
            runner.print_summary([])
            runner.generate_report([], report)
            content = report.read_text()
        self.assertIn("2 compatibility groups", terminal.getvalue())
        self.assertIn("Modules targeted: 1/2 (50.0%)", terminal.getvalue())
        self.assertIn("Not yet started: 1 modules", terminal.getvalue())
        self.assertIn("**Modules targeted**: 1/2 (50.0%)", content)
        self.assertIn("**Not yet started**: 1 modules", content)

    def test_relative_runtime_remains_valid_after_fixture_directory_change(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--runtime",
             os.path.relpath(sys.executable), "--module", "io"],
            capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_noop_runtime_is_rejected(self):
        with mock.patch.object(sys, "argv", [str(SCRIPT), "--runtime", sys.executable]), \
             mock.patch.object(runner.subprocess, "run", return_value=subprocess.CompletedProcess("runtime", 0, b"", b"")), \
             mock.patch.object(runner, "run_all_tests") as run_all, \
             mock.patch("builtins.print"):
            with self.assertRaises(SystemExit) as raised:
                runner.main()
            self.assertEqual(raised.exception.code, 1)
            run_all.assert_not_called()

    def test_missing_module_fails_cli(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--runtime", sys.executable,
             "--module", "nonexistent_fixture"],
            capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

    def test_unsuccessful_runtime_smoke_test_stops_before_fixtures(self):
        with mock.patch.object(sys, "argv", [str(SCRIPT), "--runtime", sys.executable]), \
             mock.patch.object(runner.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "runtime")), \
             mock.patch.object(runner, "run_all_tests") as run_all, \
             mock.patch("builtins.print"):
            with self.assertRaises(SystemExit) as raised:
                runner.main()
            self.assertEqual(raised.exception.code, 1)
            run_all.assert_not_called()

    def test_broken_cpython_baseline_fails_unless_report_only_is_explicit(self):
        result = runner.ModuleResult(name="io", category="stdlib", cpython_failed=1, kipferl_passed=1)
        for extra_args, expected in [([], 1), (["--ci"], 0)]:
            with self.subTest(extra_args=extra_args), \
                 mock.patch.object(sys, "argv", [str(SCRIPT), "--runtime", sys.executable] + extra_args), \
                 mock.patch.object(runner, "run_all_tests", return_value=[result]), \
                 mock.patch("builtins.print"):
                with self.assertRaises(SystemExit) as raised:
                    runner.main()
                self.assertEqual(raised.exception.code, expected)

    def test_runtime_name_resolves_through_path(self):
        with mock.patch.object(sys, "argv", [str(SCRIPT), "--runtime", "test-runtime"]), \
             mock.patch.object(runner.shutil, "which", return_value=sys.executable), \
             mock.patch.object(runner, "run_all_tests", return_value=[]) as run_all, \
             mock.patch("builtins.print"):
            with self.assertRaises(SystemExit) as raised:
                runner.main()
            self.assertEqual(raised.exception.code, 0)
            self.assertEqual(run_all.call_args.args[1], str(Path(sys.executable).resolve()))


if __name__ == "__main__":
    unittest.main()
