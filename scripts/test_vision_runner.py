"""Keep vision diagnostics available when a runtime fails or hangs."""

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "tests/vision/run_vision.py"
SPEC = importlib.util.spec_from_file_location("vision_runner", SCRIPT)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class VisionRunnerTests(unittest.TestCase):
    def test_timeout_is_reported_and_remaining_tests_run(self):
        with mock.patch.object(runner, "SUPPORTED_TEST_SCRIPTS", ["first.py", "second.py"]), \
             mock.patch.object(runner.subprocess, "run", side_effect=[
                 subprocess.TimeoutExpired("runtime", 2),
                 subprocess.CompletedProcess("runtime", 0, "ok\n", ""),
             ]):
            results, failures = runner.run_tests("runtime", 2)
        self.assertEqual(results, {"first.py": False, "second.py": True})
        self.assertIn("Timed out", failures["first.py"])

    def test_missing_runtime_is_a_reportable_failure(self):
        with mock.patch.object(runner.subprocess, "run", side_effect=FileNotFoundError("missing runtime")):
            status, _, error = runner.run_cmd(["runtime"], 1)
        self.assertNotEqual(status, 0)
        self.assertIn("missing runtime", error)

    def test_success_marker_must_be_exact(self):
        with mock.patch.object(runner, "SUPPORTED_TEST_SCRIPTS", ["test.py"]), \
             mock.patch.object(runner, "run_cmd", return_value=(0, "okay but broken\n", "")):
            results, failures = runner.run_tests("runtime", 1)
        self.assertFalse(results["test.py"])
        self.assertIn("test.py", failures)

    def test_single_successful_benchmark_sample_has_percentile(self):
        with mock.patch.object(runner, "BENCH_SCRIPTS", ["empty.py"]), \
             mock.patch.object(runner, "run_cmd", return_value=(0, "", "")):
            timings, failures = runner.benchmark("runtime", 1, 0, 1)
        self.assertEqual(failures, {})
        self.assertEqual(timings["empty.py"]["runs"], 1)
        self.assertEqual(timings["empty.py"]["p90_ms"], timings["empty.py"]["med_ms"])

    def test_partial_benchmark_failure_is_not_hidden(self):
        with mock.patch.object(runner, "BENCH_SCRIPTS", ["empty.py"]), \
             mock.patch.object(runner, "run_cmd", side_effect=[(-1, "", "timeout"), (0, "", "")]):
            timings, failures = runner.benchmark("runtime", 2, 0, 1)
        self.assertEqual(timings["empty.py"]["runs"], 1)
        self.assertEqual(failures["empty.py"], "1 runs failed")

    def test_failed_warmup_is_reported_with_successful_measurements(self):
        with mock.patch.object(runner, "BENCH_SCRIPTS", ["empty.py"]), \
             mock.patch.object(runner.subprocess, "run", side_effect=[
                 subprocess.TimeoutExpired("runtime", 2),
                 subprocess.CompletedProcess("runtime", 0, "", ""),
             ]):
            timings, failures = runner.benchmark("runtime", 1, 1, 2)
        self.assertEqual(timings["empty.py"]["runs"], 1)
        self.assertEqual(failures["empty.py"], "1 warmup runs failed")

    def test_invalid_benchmark_options_are_rejected(self):
        for option, value in [("--runs", "0"), ("--runs", "-1"), ("--warmup", "-1"), ("--timeout", "0")]:
            with self.subTest(option=option, value=value), \
                 mock.patch.object(sys, "argv", [str(SCRIPT), option, value]), \
                 mock.patch("sys.stderr"), \
                 mock.patch.object(runner, "run_tests") as run_tests:
                with self.assertRaises(SystemExit) as raised:
                    runner.main()
                self.assertEqual(raised.exception.code, 2)
                run_tests.assert_not_called()


if __name__ == "__main__":
    unittest.main()
