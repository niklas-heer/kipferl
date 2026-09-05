"""Release smoke tests use only local subprocesses and mocked package commands."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

import check_release_packages as smoke


class ValidationTests(unittest.TestCase):
    def record(self):
        return {"name": "tzdata", "version": "2025.2", "status": "tested",
                "target": "macos-aarch64", "runtime_sha256": "a" * 64,
                "wheel_sha256": "b" * 64, "wheel_filename": "tzdata.whl",
                "smoke": {"file": "tzdata.py", "sha256": smoke.digest(
                    smoke.ROOT / "compatibility/packages/smoke/tzdata.py")}}

    def lock(self):
        return {"schema": 1, "runtime_sha256": "a" * 64, "target": "macos-aarch64",
                "requirements": ["tzdata==2025.2"], "allow_unverified": False,
                "packages": [{"name": "tzdata", "version": "2025.2", "sha256": "b" * 64,
                              "filename": "tzdata.whl"}]}

    def test_versions_require_exact_release_and_accept_cli_color(self):
        smoke.validate_versions("\x1b[1mKipferl\x1b[0m v0.7.0-rc.1\n",
                                "Kipferl runtime 0.7.0-rc.1\n", "0.7.0-rc.1")
        for cli, runtime in [("Kipferl v0.7.0", "Kipferl runtime 0.7.0-rc.1"),
                             ("Kipferl v0.7.0-rc.1", "3.11.0")]:
            with self.assertRaises(smoke.SmokeFailure):
                smoke.validate_versions(cli, runtime, "0.7.0-rc.1")

    def test_catalog_requires_reviewed_exact_runtime_target_and_smoke(self):
        record = self.record()
        self.assertEqual(smoke.tested_record({"records": [record]}, "a" * 64,
                                            "macos-aarch64"), record)
        for field, value in [("runtime_sha256", "c" * 64), ("status", "unverified"),
                             ("target", "linux-x86_64"), ("version", "2025.3"),
                             ("smoke", {"file": "tzdata.py", "sha256": "c" * 64})]:
            changed = copy.deepcopy(record)
            changed[field] = value
            with self.subTest(field=field), self.assertRaises(smoke.SmokeFailure):
                smoke.tested_record({"records": [changed]}, "a" * 64, "macos-aarch64")

    def test_lock_cannot_opt_in_or_change_evidence(self):
        smoke.validate_lock(self.lock(), self.record(), "a" * 64, "macos-aarch64")
        for field, value in [("allow_unverified", True), ("runtime_sha256", "c" * 64),
                             ("target", "linux-x86_64"), ("requirements", []),
                             ("packages", [])]:
            lock = self.lock()
            lock[field] = value
            with self.subTest(field=field), self.assertRaises(smoke.SmokeFailure):
                smoke.validate_lock(lock, self.record(), "a" * 64, "macos-aarch64")
        lock = self.lock()
        lock["packages"][0]["sha256"] = "c" * 64
        with self.assertRaises(smoke.SmokeFailure):
            smoke.validate_lock(lock, self.record(), "a" * 64, "macos-aarch64")

    def test_explicit_cli_mode_does_not_claim_os_isolation(self):
        with patch.object(smoke.platform, "system", return_value="Linux"):
            self.assertEqual(smoke.offline_prefix("cli", cwd=Path.cwd(), env={}), [])
            with self.assertRaisesRegex(smoke.SmokeFailure, "explicitly tests only"):
                smoke.offline_prefix("required", cwd=Path.cwd(), env={})


class CommandTests(unittest.TestCase):
    def test_environment_is_private_and_drops_inherited_configuration(self):
        with tempfile.TemporaryDirectory() as temporary:
            with patch.dict(os.environ, {"HOME": "/user-home", "PYTHONPATH": "/user-python",
                                         "KIPFERL_CACHE_DIR": "/user-cache", "HTTPS_PROXY": "secret"}):
                env = smoke.environment(Path(temporary))
            self.assertNotIn("PYTHONPATH", env)
            self.assertNotIn("HTTPS_PROXY", env)
            for key in ("HOME", "KIPFERL_CACHE_DIR", "TMPDIR", "XDG_CACHE_HOME"):
                self.assertTrue(Path(env[key]).is_relative_to(temporary))

    def test_output_is_drained_but_bounded_and_failure_has_diagnostics(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            env = smoke.environment(cwd)
            with self.assertRaisesRegex(smoke.SmokeFailure, "output limit"):
                smoke.command([sys.executable, "-c", "print('x' * 1000000)"],
                              cwd=cwd, env=env, limit=100)
            with self.assertRaisesRegex(smoke.SmokeFailure, "actual diagnostic"):
                smoke.command([sys.executable, "-c", "import sys;sys.exit('actual diagnostic')"],
                              cwd=cwd, env=env)
            result = smoke.command([sys.executable, "-c", "raise SystemExit(7)"],
                                   cwd=cwd, env=env, success=False)
            self.assertEqual(result["returncode"], 7)

    def test_timeout_kills_command_and_inherited_pipe_children(self):
        with tempfile.TemporaryDirectory() as temporary:
            cwd = Path(temporary)
            env = smoke.environment(cwd)
            for source in ["import time;time.sleep(20)",
                           "import subprocess,sys;subprocess.Popen([sys.executable,'-c','import time;time.sleep(20)'])"]:
                started = time.monotonic()
                with self.assertRaisesRegex(smoke.SmokeFailure, "timed out"):
                    smoke.command([sys.executable, "-c", source], cwd=cwd, env=env, timeout=0.2)
                self.assertLess(time.monotonic() - started, 4)


class WorkflowTests(unittest.TestCase):
    def test_isolated_workflow_restores_offline_then_removes_project_and_all_caches(self):
        calls = []
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            cli, runtime = root / "cli", root / "runtime"
            for binary in (cli, runtime):
                binary.write_bytes(b"fixture binary")
                binary.chmod(0o700)
            wheel_bytes = b"fixture wheel"
            import hashlib
            wheel_hash = hashlib.sha256(wheel_bytes).hexdigest()
            record = ValidationTests().record()
            record.update(runtime_sha256=smoke.digest(runtime), wheel_sha256=wheel_hash)
            original_project = None

            def fake_command(arguments, *, cwd, env, **options):
                nonlocal original_project
                calls.append(arguments)
                self.assertTrue(Path(env["HOME"]).is_dir())
                output, code = "", 0
                if arguments[-1] == "--version":
                    version = (smoke.ROOT / "VERSION").read_text().strip()
                    output = f"Kipferl v{version}" if arguments[0] == str(cli) else f"Kipferl runtime {version}"
                elif "catalog" in arguments:
                    output = json.dumps({"records": [record]})
                elif "add" in arguments:
                    self.assertEqual(arguments[1:], ["add", "tzdata==2025.2"])
                    original_project = cwd
                    (cwd / ".kipferl/packages").mkdir(parents=True)
                    cache = cwd / ".kipferl/cache"
                    cache.mkdir()
                    (cache / (wheel_hash + ".whl")).write_bytes(wheel_bytes)
                    lock = ValidationTests().lock()
                    lock["runtime_sha256"] = smoke.digest(runtime)
                    lock["packages"][0]["sha256"] = wheel_hash
                    (cwd / "kipferl.lock").write_text(json.dumps(lock))
                elif "sync" in arguments:
                    self.assertEqual(arguments[1:], ["sync", "--locked", "--offline"])
                    self.assertFalse((cwd / ".kipferl/packages").exists())
                    if (cwd / ".kipferl/cache" / (wheel_hash + ".whl")).is_file():
                        (cwd / ".kipferl/packages").mkdir()
                    else:
                        code = 1
                        output = "wheel is missing from the offline cache"
                elif "build" in arguments:
                    self.assertIn("universal", arguments)
                    (cwd / "program").write_bytes(b"standalone")
                elif "run" in arguments:
                    output = smoke.SUCCESS
                elif len(arguments) == 1:
                    self.assertFalse(original_project.exists())
                    self.assertFalse(original_project.parent.exists())
                    self.assertTrue(Path(arguments[0]).is_file())
                    output = smoke.SUCCESS
                return {"stdout": output, "stderr": "", "returncode": code, "seconds": 0}

            with patch.object(smoke, "command", side_effect=fake_command), \
                    patch.object(smoke, "host_target", return_value="macos-aarch64"):
                result = smoke.run_smoke(cli, runtime, "macos-aarch64", "cli")
            self.assertEqual(result["status"], "passed")
            self.assertEqual(result["offline_isolation"], "cli-offline-flag-only")
            self.assertEqual(result["runtime_sha256"], smoke.digest(runtime))
            self.assertEqual(result["wheel_sha256"], wheel_hash)
            self.assertIn("offline-missing-cache-rejected", [step["name"] for step in result["steps"]])
            self.assertEqual(sum("sync" in call for call in calls), 2)


if __name__ == "__main__":
    unittest.main()
