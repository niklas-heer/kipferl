#!/usr/bin/env python3
"""
ucharm CPython Compatibility Test Runner

This script runs the full CPython test suite against pocketpy-ucharm
and generates a detailed compatibility report.

Usage:
    python tests/compat_runner.py [--module MODULE] [--verbose] [--report]

Like Bun does with Node.js tests, we:
1. Run the same tests on both CPython and pocketpy-ucharm
2. Track what passes/fails/skips on each
3. Calculate compatibility percentages
4. Generate a detailed report with specific failure information
"""

import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional

# ANSI colors
RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
RED = "\033[31m"
GREEN = "\033[32m"
YELLOW = "\033[33m"
BLUE = "\033[34m"
CYAN = "\033[36m"
WHITE = "\033[37m"
BG_RED = "\033[41m"
BG_GREEN = "\033[42m"

# Progress bar characters
BAR_FULL = "█"
BAR_EMPTY = "░"


@dataclass
class TestResult:
    """Result of a single test."""

    name: str
    passed: bool
    skipped: bool = False
    error: Optional[str] = None
    duration_ms: float = 0


@dataclass
class ModuleResult:
    """Result of testing a module."""

    name: str
    category: str  # 'stdlib' or 'ucharm'
    cpython_tests: list = field(default_factory=list)
    ucharm_tests: list = field(default_factory=list)
    cpython_passed: int = 0
    cpython_failed: int = 0
    cpython_skipped: int = 0
    ucharm_passed: int = 0
    ucharm_failed: int = 0
    ucharm_skipped: int = 0
    failures: list = field(default_factory=list)
    skipped_reasons: list = field(default_factory=list)
    duration_ms: float = 0
    error: Optional[str] = None

    @property
    def cpython_total(self) -> int:
        return self.cpython_passed + self.cpython_failed

    @property
    def ucharm_compared_passed(self) -> int:
        # When running this suite on an older CPython (e.g. 3.9),
        # μcharm may legitimately exercise more tests than CPython.
        # Cap to the CPython baseline so totals/parity remain meaningful.
        if self.cpython_total == 0:
            return 0
        return min(self.ucharm_passed, self.cpython_total)

    @property
    def ucharm_total(self) -> int:
        return self.ucharm_passed + self.ucharm_failed

    @property
    def parity_percent(self) -> float:
        if self.cpython_total == 0:
            return 100.0
        return (self.ucharm_compared_passed / self.cpython_total) * 100


# All modules available in pocketpy-ucharm
# Organized by category for testing

# CPython standard library modules we are ACTIVELY TARGETING for compatibility
# These are modules where we have tests and are tracking CPython compatibility
STDLIB_MODULES = [
    # CPython stdlib modules (core coverage)
    "argparse",
    "array",
    "binascii",
    "bisect",
    "collections",
    "configparser",
    "contextlib",
    "dataclasses",
    "enum",
    "errno",
    "hashlib",
    "heapq",
    "io",
    "json",
    "math",
    "operator",
    "os",
    "random",
    "re",
    "struct",
    "sys",
    "time",
    "urllib_parse",
    "uuid",
    # Modules with native μcharm Rust implementations
    "base64",
    "copy",
    "csv",
    "datetime",
    "fnmatch",
    "functools",
    "glob",
    "gzip",
    "hmac",
    "http.client",
    "itertools",
    "logging",
    "pathlib",
    "secrets",
    "shutil",
    "signal",
    "sqlite3",
    "statistics",
    "subprocess",
    "tarfile",
    "tempfile",
    "textwrap",
    "tomllib",
    "toml",
    "typing",
    "unittest",
    "xml.etree.ElementTree",
    "zipfile",
]

# Complete list of CPython standard library modules (Python 3.11+)
# These are ALL the modules in CPython's stdlib that we could potentially implement
# Organized by category for reference
CPYTHON_STDLIB_ALL = {
    # Text Processing
    "string": "Common string operations",
    "re": "Regular expressions",
    "difflib": "Helpers for computing deltas",
    "textwrap": "Text wrapping and filling",
    "unicodedata": "Unicode database",
    "stringprep": "Internet string preparation",
    "readline": "GNU readline interface",
    "rlcompleter": "Completion function for readline",
    # Binary Data
    "struct": "Interpret bytes as packed binary data",
    "codecs": "Codec registry and base classes",
    # Data Types
    "datetime": "Date and time types",
    "zoneinfo": "IANA time zone support",
    "calendar": "Calendar-related functions",
    "collections": "Container datatypes",
    "collections.abc": "Abstract base classes for containers",
    "heapq": "Heap queue algorithm",
    "bisect": "Array bisection algorithm",
    "array": "Efficient arrays of numeric values",
    "weakref": "Weak references",
    "types": "Dynamic type creation",
    "copy": "Shallow and deep copy operations",
    "pprint": "Pretty-print data structures",
    "reprlib": "Alternate repr() implementation",
    "enum": "Enumeration support",
    "graphlib": "Topological sorting",
    # Numeric and Mathematical
    "numbers": "Numeric abstract base classes",
    "math": "Mathematical functions",
    "cmath": "Math for complex numbers",
    "decimal": "Decimal fixed point arithmetic",
    "fractions": "Rational numbers",
    "random": "Generate pseudo-random numbers",
    "statistics": "Mathematical statistics functions",
    # Functional Programming
    "itertools": "Iterator building blocks",
    "functools": "Higher-order functions",
    "operator": "Standard operators as functions",
    # File and Directory Access
    "pathlib": "Object-oriented filesystem paths",
    "os.path": "Common pathname manipulations",
    "fileinput": "Iterate over lines from input",
    "stat": "Interpreting stat() results",
    "filecmp": "File and directory comparisons",
    "tempfile": "Temporary files and directories",
    "glob": "Unix style pathname pattern expansion",
    "fnmatch": "Unix filename pattern matching",
    "linecache": "Random access to text lines",
    "shutil": "High-level file operations",
    # Data Persistence
    "pickle": "Python object serialization",
    "copyreg": "Register pickle support functions",
    "shelve": "Python object persistence",
    "marshal": "Internal Python object serialization",
    "dbm": "Interfaces to Unix databases",
    "sqlite3": "DB-API 2.0 interface for SQLite",
    # Data Compression
    "zlib": "Compression compatible with gzip",
    "gzip": "Support for gzip files",
    "bz2": "Support for bzip2 compression",
    "lzma": "Compression using LZMA algorithm",
    "zipfile": "Work with ZIP archives",
    "tarfile": "Read and write tar archives",
    # File Formats
    "csv": "CSV file reading and writing",
    "configparser": "Configuration file parser",
    "tomllib": "Parse TOML files",
    "toml": "TOML parse/serialize (compat)",
    "netrc": "netrc file processing",
    "plistlib": "Generate and parse Apple plist files",
    # Cryptographic
    "hashlib": "Secure hashes and message digests",
    "hmac": "Keyed-hashing for message auth",
    "secrets": "Generate secure random numbers",
    # OS Services
    "os": "Miscellaneous OS interfaces",
    "io": "Core tools for working with streams",
    "time": "Time access and conversions",
    "argparse": "Parser for command-line options",
    "getopt": "C-style parser for command line",
    "logging": "Logging facility",
    "logging.config": "Logging configuration",
    "logging.handlers": "Logging handlers",
    "getpass": "Portable password input",
    "curses": "Terminal handling for character-cell",
    "curses.textpad": "Text input widget for curses",
    "curses.ascii": "ASCII character utilities",
    "curses.panel": "Panel stack extension for curses",
    "platform": "Access to platform's identifying data",
    "errno": "Standard errno system symbols",
    "ctypes": "Foreign function library",
    # Concurrent Execution
    "threading": "Thread-based parallelism",
    "multiprocessing": "Process-based parallelism",
    "multiprocessing.shared_memory": "Shared memory",
    "concurrent.futures": "Launching parallel tasks",
    "subprocess": "Subprocess management",
    "sched": "Event scheduler",
    "queue": "Synchronized queue class",
    "contextvars": "Context variables",
    # Networking
    "asyncio": "Asynchronous I/O",
    "socket": "Low-level networking interface",
    "ssl": "TLS/SSL wrapper for sockets",
    "select": "Waiting for I/O completion",
    "selectors": "High-level I/O multiplexing",
    "signal": "Set handlers for async events",
    "mmap": "Memory-mapped file support",
    # Internet Data Handling
    "email": "Email and MIME handling",
    "json": "JSON encoder and decoder",
    "mailbox": "Manipulate mailboxes",
    "mimetypes": "Map filenames to MIME types",
    "base64": "Base16, Base32, Base64 encodings",
    "binascii": "Convert between binary and ASCII",
    "quopri": "Encode and decode MIME quoted-printable",
    # HTML/XML
    "html": "HyperText Markup Language support",
    "html.parser": "Simple HTML and XHTML parser",
    "html.entities": "HTML entity definitions",
    "xml.etree.ElementTree": "ElementTree XML API",
    "xml.dom": "Document Object Model API",
    "xml.dom.minidom": "Minimal DOM implementation",
    "xml.sax": "SAX2 parser support",
    # Internet Protocols
    "urllib": "URL handling modules",
    "urllib.request": "URL opening library",
    "urllib.parse": "Parse URLs into components",
    "http": "HTTP modules",
    "http.client": "HTTP protocol client",
    "http.server": "HTTP servers",
    "http.cookies": "HTTP cookie handling",
    "ftplib": "FTP protocol client",
    "poplib": "POP3 protocol client",
    "imaplib": "IMAP4 protocol client",
    "smtplib": "SMTP protocol client",
    "uuid": "UUID objects",
    "socketserver": "Framework for network servers",
    "ipaddress": "IPv4/IPv6 manipulation",
    # Development Tools
    "typing": "Support for type hints",
    "pydoc": "Documentation generator",
    "doctest": "Test interactive Python examples",
    "unittest": "Unit testing framework",
    "unittest.mock": "Mock object library",
    "test": "Regression test package",
    # Debugging and Profiling
    "bdb": "Debugger framework",
    "faulthandler": "Dump Python tracebacks",
    "pdb": "Python debugger",
    "timeit": "Measure execution time",
    "trace": "Trace Python statement execution",
    "tracemalloc": "Trace memory allocations",
    # Runtime Services
    "sys": "System-specific parameters",
    "sysconfig": "Python's configuration info",
    "builtins": "Built-in objects",
    "__main__": "Top-level code environment",
    "warnings": "Warning control",
    "dataclasses": "Data class decorator",
    "contextlib": "Context manager utilities",
    "abc": "Abstract base classes",
    "atexit": "Exit handlers",
    "traceback": "Print or retrieve a traceback",
    "__future__": "Future statement definitions",
    "gc": "Garbage collector interface",
    "inspect": "Inspect live objects",
    "site": "Site-specific configuration hook",
    # Custom Python Interpreters
    "code": "Interpreter base classes",
    "codeop": "Compile Python code",
    # Importing
    "zipimport": "Import modules from ZIP archives",
    "pkgutil": "Package extension utilities",
    "modulefinder": "Find modules used by a script",
    "runpy": "Locate and run Python modules",
    "importlib": "Import machinery",
    "importlib.resources": "Package resources",
    "importlib.metadata": "Package metadata",
}

# ucharm-specific runtime modules - these are OUR libraries, not CPython stdlib
# No compatibility comparison needed - they're accepted as-is
UCHARM_MODULES = [
    "ansi",  # ANSI color codes
    "args",  # CLI argument parsing
    "charm",  # TUI components (box, rule, progress, etc.)
    "input",  # Interactive input (select, confirm, prompt)
    "path",  # Path manipulation utilities
    "term",  # Terminal control
]

# Modules we explicitly skip from testing:
# - Network modules: select, socket, ssl (platform-specific)
# - Asyncio: asyncio (complex, separate testing)
# - GC: gc (not available in PocketPy)
SKIP_MODULES = [
    "gc",
    "select",
    "socket",
    "ssl",
    "asyncio",
]


def get_runtime_path() -> str:
    """Find pocketpy-ucharm binary."""
    script_dir = Path(__file__).resolve().parent.parent

    # Canonical optimized Rust build
    release_path = script_dir / "target" / "release" / "pocketpy-ucharm"
    if release_path.exists():
        return str(release_path.resolve())

    # Development Rust build
    debug_path = script_dir / "target" / "debug" / "pocketpy-ucharm"
    if debug_path.exists():
        return str(debug_path.resolve())

    # Fallback to PATH
    return "pocketpy-ucharm"


def print_header():
    """Print the test runner header."""
    print()
    print(f"{CYAN}{BOLD}╭{'─' * 58}╮{RESET}")
    print(
        f"{CYAN}{BOLD}│{RESET}  {BOLD}μcharm CPython Compatibility Test Suite{RESET}                 {CYAN}{BOLD}│{RESET}"
    )
    print(
        f"{CYAN}{BOLD}│{RESET}  {DIM}Testing against CPython test suite{RESET}                      {CYAN}{BOLD}│{RESET}"
    )
    print(f"{CYAN}{BOLD}╰{'─' * 58}╯{RESET}")
    print()


def progress_bar(current: int, total: int, width: int = 30) -> str:
    """Create a progress bar string."""
    if total == 0:
        return BAR_EMPTY * width
    # Cap at 100% to prevent overflow
    ratio = min(current / total, 1.0)
    filled = int(width * ratio)
    return BAR_FULL * filled + BAR_EMPTY * (width - filled)


def format_percent(value: float) -> str:
    """Format percentage with color."""
    if value >= 100:
        return f"{GREEN}{BOLD}100%{RESET}"
    elif value >= 90:
        return f"{GREEN}{value:.1f}%{RESET}"
    elif value >= 70:
        return f"{YELLOW}{value:.1f}%{RESET}"
    else:
        return f"{RED}{value:.1f}%{RESET}"


def format_duration(ms: float) -> str:
    """Format duration in human-readable form."""
    if ms < 1000:
        return f"{ms:.0f}ms"
    elif ms < 60000:
        return f"{ms / 1000:.1f}s"
    else:
        return f"{ms / 60000:.1f}m"


def run_test_file(
    interpreter: str, test_file: str, timeout: int = 60
) -> tuple[str, str, int, float]:
    """Run a test file and capture output."""
    start = time.time()
    env = os.environ.copy()

    try:
        test_path = Path(test_file)
        result = subprocess.run(
            [interpreter, test_path.name],
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=test_path.parent,
            env=env,
        )
        duration = (time.time() - start) * 1000
        return result.stdout, result.stderr, result.returncode, duration
    except subprocess.TimeoutExpired:
        duration = (time.time() - start) * 1000
        return "", "TIMEOUT", -1, duration
    except Exception as e:
        duration = (time.time() - start) * 1000
        return "", str(e), -1, duration


def parse_test_output(
    stdout: str, stderr: str, returncode: int
) -> tuple[int, int, int, list]:
    """Parse test output to extract pass/fail/skip counts."""
    passed = 0
    failed = 0
    skipped = 0
    failures = []

    # Try to find our custom format first: "Results: X passed, Y failed, Z skipped"
    results_match = re.search(
        r"Results:\s*(\d+)\s*passed,\s*(\d+)\s*failed(?:,\s*(\d+)\s*skipped)?", stdout
    )
    if results_match:
        passed = int(results_match.group(1))
        failed = int(results_match.group(2))
        skipped = int(results_match.group(3) or 0)

        # Extract failure details
        for line in stdout.split("\n"):
            if "FAIL:" in line:
                failures.append(line.strip())

        return passed, failed, skipped, failures

    # Try unittest format: "Ran X tests" ... "OK" or "FAILED (failures=Y)"
    ran_match = re.search(r"Ran\s+(\d+)\s+test", stdout + stderr)
    if ran_match:
        total = int(ran_match.group(1))

        if "OK" in stdout or "OK" in stderr:
            # Check for skips
            skip_match = re.search(r"skipped=(\d+)", stdout + stderr)
            skipped = int(skip_match.group(1)) if skip_match else 0
            passed = total - skipped
        else:
            fail_match = re.search(r"failures?=(\d+)", stdout + stderr)
            error_match = re.search(r"errors?=(\d+)", stdout + stderr)
            skip_match = re.search(r"skipped=(\d+)", stdout + stderr)

            failed = int(fail_match.group(1)) if fail_match else 0
            failed += int(error_match.group(1)) if error_match else 0
            skipped = int(skip_match.group(1)) if skip_match else 0
            passed = total - failed - skipped

            # Extract failure info
            for match in re.finditer(r"(FAIL|ERROR):\s*(\S+)", stdout + stderr):
                failures.append(f"{match.group(1)}: {match.group(2)}")

        return passed, failed, skipped, failures

    # Fallback: count PASS/FAIL lines
    for line in stdout.split("\n"):
        stripped = line.strip()
        if stripped.startswith("PASS:"):
            passed += 1
        elif stripped.startswith("FAIL:"):
            failed += 1
            failures.append(line.strip())
        elif stripped.startswith("SKIP:"):
            skipped += 1

    # If still nothing, use return code
    if passed == 0 and failed == 0:
        if returncode == 0:
            passed = 1
        else:
            failed = 1
            if stderr:
                failures.append(stderr[:200])

    return passed, failed, skipped, failures


def test_module(
    module: str, category: str, test_dir: Path, mpy_path: str, verbose: bool = False
) -> ModuleResult:
    """Test a single module against both CPython and pocketpy-ucharm."""
    result = ModuleResult(name=module, category=category)

    # Find test file
    test_stem = module.replace(".", "_").lower()
    test_file = test_dir / f"test_{test_stem}.py"
    if not test_file.exists():
        result.error = "Test file not found"
        return result

    start_time = time.time()

    # Print module name
    print(f"  {BOLD}{module:15}{RESET} ", end="", flush=True)

    # Run with CPython
    stdout, stderr, code, duration = run_test_file("python3", str(test_file))
    passed, failed, skipped, failures = parse_test_output(stdout, stderr, code)
    result.cpython_passed = passed
    result.cpython_failed = failed
    result.cpython_skipped = skipped

    # Run with pocketpy-ucharm
    stdout, stderr, code, duration = run_test_file(mpy_path, str(test_file))
    passed, failed, skipped, failures = parse_test_output(stdout, stderr, code)
    result.ucharm_passed = passed
    result.ucharm_failed = failed
    result.ucharm_skipped = skipped
    result.failures = failures

    result.duration_ms = (time.time() - start_time) * 1000

    # Print result
    cpython_str = f"{result.cpython_passed}/{result.cpython_total}"
    ucharm_str = f"{result.ucharm_compared_passed}/{result.cpython_total}"
    parity = result.parity_percent

    bar = progress_bar(result.ucharm_compared_passed, result.cpython_total, 20)

    if parity >= 100:
        status = f"{GREEN}✓{RESET}"
        bar_color = GREEN
    elif parity >= 90:
        status = f"{YELLOW}○{RESET}"
        bar_color = YELLOW
    else:
        status = f"{RED}✗{RESET}"
        bar_color = RED

    # Format parity - need to account for ANSI codes in format_percent
    parity_formatted = format_percent(parity)
    print(
        f"{bar_color}{bar}{RESET}  {cpython_str:>7} → {ucharm_str:>7}  {parity_formatted}  {status}"
    )

    if verbose and result.failures:
        for f in result.failures[:3]:
            print(f"           {DIM}└─ {f[:60]}{RESET}")

    return result


def print_category_header(title: str):
    """Print a category header."""
    print(f"\n{BOLD}{title}{RESET}")
    print(f"{DIM}{'─' * 75}{RESET}")
    print(
        f"  {'Module':<15} {'Progress':<21}  {'CPython':>7}   {'μcharm':>7}  {'Parity'}  Status"
    )
    print(f"{DIM}{'─' * 75}{RESET}")


def run_all_tests(
    test_dir: Path, runtime_path: str, verbose: bool = False
) -> list[ModuleResult]:
    """Run all compatibility tests."""
    mpy_path = runtime_path
    results = []

    # Test stdlib modules - these need CPython compatibility comparison
    print_category_header("CPython Standard Library Compatibility")
    for module in STDLIB_MODULES:
        result = test_module(module, "stdlib", test_dir, mpy_path, verbose)
        results.append(result)

    # Note: ucharm-specific modules (ansi, charm, input, term, args, path) are
    # our own libraries - no CPython equivalent exists, so no comparison needed.
    # They are accepted as-is.

    return results


def print_summary(results: list[ModuleResult]):
    """Print test summary."""
    baseline = [r for r in results if r.cpython_total > 0]
    total_cpython = sum(r.cpython_total for r in baseline)
    total_ucharm_passed = sum(r.ucharm_compared_passed for r in baseline)
    total_ucharm_failed = sum(r.ucharm_failed for r in baseline)
    total_skipped = sum(r.ucharm_skipped for r in baseline)
    no_baseline = sum(
        1 for r in results if r.cpython_total == 0 and r.category == "stdlib"
    )

    overall_parity = (
        (total_ucharm_passed / total_cpython * 100) if total_cpython > 0 else 0
    )

    passed_modules = sum(
        1 for r in results if r.parity_percent >= 100 and r.cpython_total > 0
    )
    partial_modules = sum(1 for r in results if 0 < r.parity_percent < 100)
    failed_modules = sum(
        1 for r in results if r.parity_percent == 0 and r.cpython_total > 0
    )
    # Only count stdlib modules as "missing" - ucharm modules don't need CPython tests
    missing_modules = sum(1 for r in results if r.error and r.category == "stdlib")

    print()
    print(f"{BOLD}{'═' * 75}{RESET}")
    print()
    print(f"  {BOLD}Targeted Modules Compatibility{RESET}")
    print()

    # Big progress bar
    bar = progress_bar(total_ucharm_passed, total_cpython, 50)
    if overall_parity >= 90:
        bar_color = GREEN
    elif overall_parity >= 70:
        bar_color = YELLOW
    else:
        bar_color = RED

    print(f"  {bar_color}{bar}{RESET}")
    print(
        f"  {BOLD}{total_ucharm_passed:,}{RESET} / {total_cpython:,} tests passing ({format_percent(overall_parity)})"
    )
    print()

    # Module breakdown for targeted
    print(
        f"  {BOLD}Targeted Modules ({len(STDLIB_MODULES)} of {len(CPYTHON_STDLIB_ALL)} CPython stdlib){RESET}"
    )
    print(f"  {GREEN}✓ {passed_modules} full compatibility{RESET}")
    if partial_modules:
        print(f"  {YELLOW}○ {partial_modules} partial compatibility{RESET}")
    if failed_modules:
        print(f"  {RED}✗ {failed_modules} failing{RESET}")
    if missing_modules:
        print(f"  {DIM}? {missing_modules} missing tests{RESET}")
    if no_baseline:
        print(f"  {DIM}({no_baseline} modules not in this CPython version){RESET}")

    if total_skipped:
        print(f"\n  {DIM}({total_skipped} tests skipped - missing dependencies){RESET}")

    # Full CPython stdlib coverage
    print()
    print(f"  {BOLD}Full CPython Stdlib Coverage{RESET}")
    stdlib_coverage = (len(STDLIB_MODULES) / len(CPYTHON_STDLIB_ALL)) * 100
    not_started = len(CPYTHON_STDLIB_ALL) - len(STDLIB_MODULES)
    print(
        f"  {DIM}Modules targeted: {len(STDLIB_MODULES)}/{len(CPYTHON_STDLIB_ALL)} ({stdlib_coverage:.1f}%){RESET}"
    )
    print(f"  {DIM}Not yet started: {not_started} modules{RESET}")

    print()


def generate_report(results: list[ModuleResult], output_path: Path):
    """Generate markdown compatibility report."""
    baseline = [r for r in results if r.cpython_total > 0]
    total_cpython = sum(r.cpython_total for r in baseline)
    total_ucharm_passed = sum(r.ucharm_compared_passed for r in baseline)
    overall_parity = (
        (total_ucharm_passed / total_cpython * 100) if total_cpython > 0 else 0
    )

    passed_modules = sum(
        1 for r in results if r.parity_percent >= 100 and r.cpython_total > 0
    )
    partial_modules = sum(1 for r in results if 0 < r.parity_percent < 100)
    no_baseline = sum(
        1 for r in results if r.cpython_total == 0 and r.category == "stdlib"
    )

    stdlib_coverage = (len(STDLIB_MODULES) / len(CPYTHON_STDLIB_ALL)) * 100
    not_started_modules = set(CPYTHON_STDLIB_ALL.keys()) - set(STDLIB_MODULES)

    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    lines = [
        "# μcharm Compatibility Report",
        "",
        f"Generated: {timestamp}",
        "",
        "## Summary",
        "",
        "### Targeted Modules",
        "",
        f"- **Tests passing**: {total_ucharm_passed:,}/{total_cpython:,} ({overall_parity:.1f}%)",
        f"- **Modules at 100%**: {passed_modules}/{len(STDLIB_MODULES)}",
        f"- **Modules partial**: {partial_modules}/{len(STDLIB_MODULES)}",
        (
            f"- **No baseline (host CPython)**: {no_baseline}/{len(STDLIB_MODULES)}"
            if no_baseline
            else ""
        ),
        "",
        "### CPython Stdlib Coverage",
        "",
        f"- **Modules targeted**: {len(STDLIB_MODULES)}/{len(CPYTHON_STDLIB_ALL)} ({stdlib_coverage:.1f}%)",
        f"- **Not yet started**: {len(not_started_modules)} modules",
        "",
        "## Targeted Module Status",
        "",
        "| Module | Category | CPython | μcharm | Parity | Notes |",
        "|--------|----------|---------|--------|--------|-------|",
    ]

    for r in sorted(results, key=lambda x: (-x.parity_percent, x.name)):
        if r.error:
            notes = f"⚠️ {r.error}"
        elif r.parity_percent >= 100:
            notes = "✅ Full"
        elif r.ucharm_skipped > 0:
            notes = f"{r.ucharm_skipped} skipped"
        elif r.ucharm_failed > 0:
            notes = f"{r.ucharm_failed} failing"
        else:
            notes = ""

        cpython_str = (
            f"{r.cpython_passed}/{r.cpython_total}" if r.cpython_total > 0 else "-"
        )
        ucharm_str = (
            f"{r.ucharm_compared_passed}/{r.cpython_total}"
            if r.cpython_total > 0
            else "-"
        )
        parity_str = f"{r.parity_percent:.0f}%" if r.cpython_total > 0 else "-"

        lines.append(
            f"| {r.name} | {r.category} | {cpython_str} | {ucharm_str} | {parity_str} | {notes} |"
        )

    # Failures section
    failures_exist = any(r.failures for r in results)
    if failures_exist:
        lines.extend(
            [
                "",
                "## Failed Tests",
                "",
            ]
        )

        for r in results:
            if r.failures:
                lines.append(f"### {r.name}")
                lines.append("")
                for f in r.failures:
                    lines.append(f"- `{f}`")
                lines.append("")

    # Skipped section
    skipped_exist = any(r.ucharm_skipped > 0 for r in results)
    if skipped_exist:
        lines.extend(
            [
                "",
                "## Skipped Tests",
                "",
                "These tests require features not available in pocketpy-ucharm:",
                "",
            ]
        )

        for r in results:
            if r.ucharm_skipped > 0:
                lines.append(f"### {r.name}")
                lines.append("")
                lines.append(f"- {r.ucharm_skipped} tests skipped")
                if r.skipped_reasons:
                    for reason in r.skipped_reasons:
                        lines.append(f"  - {reason}")
                lines.append("")

    # Missing tests section - only show stdlib modules that are missing tests
    # (ucharm modules don't need CPython comparison)
    missing = [r for r in results if r.error and r.category == "stdlib"]
    if missing:
        lines.extend(
            [
                "",
                "## Missing Test Files",
                "",
            ]
        )
        for r in missing:
            lines.append(f"- **{r.name}**: {r.error}")

    # Not yet started modules section - grouped by category
    if not_started_modules:
        lines.extend(
            [
                "",
                "## Not Yet Started Modules",
                "",
                f"The following {len(not_started_modules)} CPython stdlib modules are not yet targeted:",
                "",
            ]
        )

        # Group modules by category based on their description patterns
        categories = {
            "Text Processing": [
                "string",
                "difflib",
                "unicodedata",
                "stringprep",
                "readline",
                "rlcompleter",
            ],
            "Binary Data": ["struct", "codecs"],
            "Data Types": [
                "zoneinfo",
                "calendar",
                "collections.abc",
                "bisect",
                "weakref",
                "types",
                "pprint",
                "reprlib",
                "enum",
                "graphlib",
            ],
            "Numeric and Mathematical": ["numbers", "cmath", "decimal", "fractions"],
            "Functional Programming": ["operator"],
            "File and Directory Access": [
                "pathlib",
                "os.path",
                "fileinput",
                "stat",
                "filecmp",
                "linecache",
            ],
            "Data Persistence": [
                "pickle",
                "copyreg",
                "shelve",
                "marshal",
                "dbm",
                "sqlite3",
            ],
            "Data Compression": ["zlib", "gzip", "bz2", "lzma", "zipfile", "tarfile"],
            "File Formats": ["configparser", "tomllib", "toml", "netrc", "plistlib"],
            "Cryptographic": ["hmac", "secrets"],
            "OS Services": [
                "argparse",
                "getopt",
                "logging.config",
                "logging.handlers",
                "getpass",
                "curses",
                "curses.textpad",
                "curses.ascii",
                "curses.panel",
                "platform",
                "ctypes",
            ],
            "Concurrent Execution": [
                "threading",
                "multiprocessing",
                "multiprocessing.shared_memory",
                "concurrent.futures",
                "sched",
                "queue",
                "contextvars",
            ],
            "Networking": ["asyncio", "socket", "ssl", "select", "selectors", "mmap"],
            "Internet Data Handling": ["email", "mailbox", "mimetypes", "quopri"],
            "HTML/XML": [
                "html",
                "html.parser",
                "html.entities",
                "xml.etree.ElementTree",
                "xml.dom",
                "xml.dom.minidom",
                "xml.sax",
            ],
            "Internet Protocols": [
                "urllib",
                "urllib.request",
                "urllib.parse",
                "http",
                "http.client",
                "http.server",
                "http.cookies",
                "ftplib",
                "poplib",
                "imaplib",
                "smtplib",
                "uuid",
                "socketserver",
                "ipaddress",
            ],
            "Development Tools": [
                "pydoc",
                "doctest",
                "unittest",
                "unittest.mock",
                "test",
            ],
            "Debugging and Profiling": [
                "bdb",
                "faulthandler",
                "pdb",
                "timeit",
                "trace",
                "tracemalloc",
            ],
            "Runtime Services": [
                "sys",
                "sysconfig",
                "builtins",
                "__main__",
                "warnings",
                "dataclasses",
                "contextlib",
                "abc",
                "atexit",
                "traceback",
                "__future__",
                "gc",
                "inspect",
                "site",
            ],
            "Custom Python Interpreters": ["code", "codeop"],
            "Importing": [
                "zipimport",
                "pkgutil",
                "modulefinder",
                "runpy",
                "importlib",
                "importlib.resources",
                "importlib.metadata",
            ],
        }

        for cat_name, cat_modules in categories.items():
            # Find modules in this category that are not started
            not_started_in_cat = [m for m in cat_modules if m in not_started_modules]
            if not_started_in_cat:
                lines.append(f"### {cat_name}")
                lines.append("")
                for mod in sorted(not_started_in_cat):
                    desc = CPYTHON_STDLIB_ALL.get(mod, "")
                    lines.append(f"- `{mod}` - {desc}")
                lines.append("")

    # Notes
    lines.extend(
        [
            "",
            "## Notes",
            "",
            "- Tests are adapted from CPython's test suite",
            "- Some tests require features not available in PocketPy (threading, gc introspection)",
            "- μcharm-specific modules (ansi, charm, input, term, args) have custom tests",
            "- Report generated by `python3 tests/compat_runner.py --report`",
        ]
    )

    output_path.write_text("\n".join(lines))
    print(f"  {DIM}Report saved to: {output_path}{RESET}")


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="ucharm CPython compatibility test runner"
    )
    parser.add_argument("--module", "-m", help="Test only this module")
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Show failure details"
    )
    parser.add_argument(
        "--report", "-r", action="store_true", help="Generate markdown report"
    )
    parser.add_argument(
        "--output",
        "-o",
        default="tests/compat_report_pocketpy.md",
        help="Report output path",
    )
    parser.add_argument(
        "--runtime",
        help="Path to runtime binary (defaults to pocketpy-ucharm)",
    )
    parser.add_argument(
        "--ci",
        action="store_true",
        help="CI mode: always exit 0, just report results",
    )
    args = parser.parse_args()

    script_dir = Path(__file__).parent
    test_dir = script_dir / "cpython"

    print_header()

    # Check pocketpy-ucharm exists
    mpy_path = args.runtime or get_runtime_path()
    try:
        mpy_path = str(Path(mpy_path).expanduser().resolve())
    except Exception:
        # If it's not a filesystem path, treat it as a PATH lookup.
        pass
    try:
        # Validate runtime with a tiny temp script (works across all builds).
        import tempfile

        with tempfile.TemporaryDirectory() as tmpdir:
            smoke = Path(tmpdir) / "smoke.py"
            smoke.write_text("print('ok')\n")
            subprocess.run([mpy_path, str(smoke)], capture_output=True, timeout=5)
    except Exception as e:
        print(f"{RED}Error: pocketpy-ucharm not found at {mpy_path}{RESET}")
        print(f"{DIM}Build it with: cargo build --release -p ucharm-runtime{RESET}")
        sys.exit(1)

    # Run tests
    if args.module:
        # Determine category for single module
        if args.module in STDLIB_MODULES:
            category = "stdlib"
        elif args.module in UCHARM_MODULES:
            category = "ucharm"
        elif args.module in SKIP_MODULES:
            print(f"{YELLOW}Module '{args.module}' is in skip list (not tested){RESET}")
            sys.exit(0)
        else:
            category = "stdlib"  # Default to stdlib for unknown modules
        results = [test_module(args.module, category, test_dir, mpy_path, args.verbose)]
    else:
        results = run_all_tests(test_dir, mpy_path, args.verbose)

    # Print summary
    print_summary(results)

    # Generate report
    if args.report:
        output_path = Path(args.output)
        generate_report(results, output_path)

    # Exit code
    if args.ci:
        # In CI mode, always exit 0 - we're tracking progress, not gating on 100%
        sys.exit(0)
    else:
        total_failed = sum(r.ucharm_failed for r in results)
        sys.exit(1 if total_failed > 0 else 0)


if __name__ == "__main__":
    main()
