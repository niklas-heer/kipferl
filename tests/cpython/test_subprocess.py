"""
Simplified subprocess module tests for kipferl compatibility testing.
Works on both CPython and PocketPy.

Based on CPython's Lib/test/test_subprocess.py
"""

import subprocess
import sys

# Test tracking
_passed = 0
_failed = 0
_errors = []
_skipped = 0


def test(name, condition):
    global _passed, _failed, _errors
    if condition:
        _passed += 1
        print(f"  PASS: {name}")
    else:
        _failed += 1
        _errors.append(name)
        print(f"  FAIL: {name}")


def skip(name, reason):
    global _skipped
    _skipped += 1
    print(f"  SKIP: {name} ({reason})")


# Helper to get stdout from result (works with both dict and object)
def get_stdout(result):
    if isinstance(result, dict):
        raw = result.get("stdout", "")
    else:
        raw = result.stdout
    if isinstance(raw, bytes):
        return raw.decode().strip()
    return (raw or "").strip()


# Helper to get returncode from result (works with both dict and object)
def get_returncode(result):
    if isinstance(result, dict):
        return result.get("returncode", -1)
    return result.returncode


def get_stderr(result):
    if isinstance(result, dict):
        raw = result.get("stderr", "")
    else:
        raw = result.stderr
    if isinstance(raw, bytes):
        return raw.decode()
    return raw or ""


# ============================================================================
# subprocess.run() tests
# ============================================================================

print("\n=== subprocess.run() tests ===")

result = subprocess.run(["echo", "hello"], capture_output=True)
test("run echo hello - returncode", get_returncode(result) == 0)
test("run echo hello - stdout", get_stdout(result) == "hello")

result = subprocess.run(["echo", "hello world"], capture_output=True)
test("run echo hello world - stdout preserves space", get_stdout(result) == "hello world")

result = subprocess.run(["true"], capture_output=True)
test("run true - zero returncode", get_returncode(result) == 0)

result = subprocess.run(["false"], capture_output=True)
test("run false - non-zero returncode", get_returncode(result) != 0)

# Both pipes must be drained concurrently or one full pipe can deadlock the child.
result = subprocess.run(
    ["sh", "-c", "(yes o | head -c 70000) & (yes e | head -c 70000 >&2) & wait"],
    capture_output=True,
)
test("run drains large stdout", len(get_stdout(result)) >= 69999)
test("run drains large stderr", len(get_stderr(result)) >= 69999)


# ============================================================================
# subprocess.run() with shell=True tests
# ============================================================================

print("\n=== subprocess.run() with shell=True tests ===")

# Check if shell=True is supported by trying a simple call
_shell_supported = False
try:
    _test_result = subprocess.run("echo test", capture_output=True, shell=True)
    _shell_supported = True
except TypeError:
    pass

if _shell_supported:
    result = subprocess.run("echo hello", capture_output=True, shell=True)
    test("run shell echo - returncode", get_returncode(result) == 0)
    test("run shell echo - stdout", get_stdout(result) == "hello")
    result = subprocess.run("printf '%s' \"hello world\"", capture_output=True, shell=True)
    test("run shell printf - stdout preserves space", get_stdout(result) == "hello world")
else:
    skip("run shell echo - returncode", "shell=True not supported")
    skip("run shell echo - stdout", "shell=True not supported")


# ============================================================================
# subprocess.call() tests
# ============================================================================

print("\n=== subprocess.call() tests ===")

if hasattr(subprocess, "call"):
    ret = subprocess.call(["true"])
    test("call true", ret == 0)

    ret = subprocess.call(["false"])
    test("call false", ret != 0)
else:
    skip("call true", "subprocess.call not available")
    skip("call false", "subprocess.call not available")


# ============================================================================
# subprocess.check_output() tests
# ============================================================================

print("\n=== subprocess.check_output() tests ===")

if hasattr(subprocess, "check_output"):
    output = subprocess.check_output(["echo", "hello"])
    if isinstance(output, bytes):
        output = output.decode("utf-8")
    test("check_output echo", output.strip() == "hello")
    try:
        subprocess.check_output(["false"])
        test("check_output non-zero raises", False)
    except Exception:
        test("check_output non-zero raises", True)
else:
    skip("check_output echo", "subprocess.check_output not available")


# ============================================================================
# subprocess.Popen tests
# ============================================================================

print("\n=== subprocess.Popen tests ===")

if hasattr(subprocess, "Popen"):
    proc = subprocess.Popen(["echo", "hello"], stdout=subprocess.PIPE)
    stdout, stderr = proc.communicate()
    if isinstance(stdout, bytes):
        stdout = stdout.decode("utf-8")
    test("Popen echo - returncode", proc.returncode == 0)
    test("Popen echo - stdout", stdout.strip() == "hello")

    proc = subprocess.Popen(["true"])
    proc.wait()
    test("Popen true - wait returncode", proc.returncode == 0)
else:
    skip("Popen echo - returncode", "subprocess.Popen not available")
    skip("Popen echo - stdout", "subprocess.Popen not available")
    skip("Popen true - wait returncode", "subprocess.Popen not available")


# ============================================================================
# subprocess constants tests
# ============================================================================

print("\n=== subprocess constants tests ===")

if hasattr(subprocess, "PIPE"):
    test("PIPE constant exists", True)
else:
    skip("PIPE constant exists", "PIPE not available")

if hasattr(subprocess, "DEVNULL"):
    test("DEVNULL constant exists", True)
else:
    skip("DEVNULL constant exists", "DEVNULL not available")


# ============================================================================
# Summary
# ============================================================================

print("\n" + "=" * 50)
print(f"Results: {_passed} passed, {_failed} failed, {_skipped} skipped")
if _errors:
    print("Failed tests:")
    for e in _errors:
        print(f"  - {e}")
    sys.exit(1)
