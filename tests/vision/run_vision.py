import argparse
import statistics
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = Path(__file__).resolve().parent / "scripts"
BENCH_DIR = Path(__file__).resolve().parent / "bench"

TEST_SCRIPTS = [
    "t_argparse.py",
    "t_os_sys_time.py",
    "t_pathlib_glob_fnmatch.py",
    "t_subprocess.py",
    "t_json_csv.py",
    "t_logging.py",
    "t_datetime.py",
    "t_textwrap.py",
    "t_tempfile_shutil.py",
    "t_re.py",
    "t_hashlib.py",
    "t_struct.py",
    "t_random.py",
    "t_typing.py",
    "t_itertools_heapq_functools_statistics.py",
    "t_signal.py",
    "t_io.py",
    "t_errno.py",
    "t_template.py",
    "t_toml.py",
    "t_fetch_https.py",
    "t_sqlite3.py",
]

BENCH_SCRIPTS = [
    "empty.py",
    "import_sys.py",
    "import_json.py",
]


def run_cmd(cmd, timeout):
    proc = subprocess.run(
        cmd,
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return proc.returncode, proc.stdout, proc.stderr


def run_tests(runtime, timeout):
    results = {}
    failures = {}
    for name in TEST_SCRIPTS:
        path = SCRIPTS_DIR / name
        code, out, err = run_cmd([runtime, str(path)], timeout)
        out_lines = [line for line in out.strip().splitlines() if line]
        last = out_lines[-1] if out_lines else ""
        ok = last.startswith("ok") and code == 0
        results[name] = ok
        if not ok:
            msg = last or (
                err.strip().splitlines()[-1] if err.strip() else "unknown failure"
            )
            failures[name] = msg
    return results, failures


def benchmark(runtime, runs, warmup, timeout):
    timings = {}
    failures = {}
    for name in BENCH_SCRIPTS:
        path = BENCH_DIR / name
        for _ in range(warmup):
            run_cmd([runtime, str(path)], timeout)
        samples = []
        for _ in range(runs):
            start = time.perf_counter()
            code, _, _ = run_cmd([runtime, str(path)], timeout)
            samples.append((time.perf_counter() - start, code == 0))
        ok_samples = [t for t, ok in samples if ok]
        if not ok_samples:
            failures[name] = "all runs failed"
            continue
        timings[name] = {
            "avg_ms": statistics.mean(ok_samples) * 1000,
            "med_ms": statistics.median(ok_samples) * 1000,
            "p90_ms": statistics.quantiles(ok_samples, n=10)[8] * 1000,
            "runs": len(ok_samples),
        }
        if len(ok_samples) != len(samples):
            failures[name] = f"{len(samples) - len(ok_samples)} runs failed"
    return timings, failures


def file_size(path):
    try:
        return Path(path).stat().st_size
    except FileNotFoundError:
        return 0


def make_report(results, failures, bench, bench_failures, size, out_path):
    total_tests = len(TEST_SCRIPTS)
    passed = sum(1 for ok in results.values() if ok)

    lines = [
        "# PocketPy Vision Tests",
        "",
        "## Runtime Info",
        "",
        f"- **Size**: {size} bytes ({size / 1024:.1f} KB)",
        f"- **Tests passed**: {passed}/{total_tests}",
        "",
        "## Vision Tests",
        "",
        "| Test | Status |",
        "| --- | --- |",
    ]
    for test in TEST_SCRIPTS:
        status = "pass" if results.get(test) else "fail"
        lines.append(f"| {test} | {status} |")

    if failures:
        lines.extend(["", "## Test Failures", ""])
        for test, msg in failures.items():
            lines.append(f"- {test}: {msg}")

    lines.extend(
        [
            "",
            "## Startup Benchmarks",
            "",
            "| Script | Avg (ms) | Median (ms) | P90 (ms) | Runs |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for script, stats in bench.items():
        lines.append(
            f"| {script} | {stats['avg_ms']:.2f} | {stats['med_ms']:.2f} | {stats['p90_ms']:.2f} | {stats['runs']} |"
        )

    if bench_failures:
        lines.extend(["", "## Benchmark Failures", ""])
        for test, msg in bench_failures.items():
            lines.append(f"- {test}: {msg}")

    Path(out_path).write_text("\n".join(lines))


def main():
    parser = argparse.ArgumentParser(description="PocketPy vision test runner")
    parser.add_argument("--timeout", type=int, default=10)
    parser.add_argument("--runs", type=int, default=20)
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument(
        "--runtime",
        default=str(ROOT / "target" / "release" / "pocketpy-ucharm"),
    )
    parser.add_argument(
        "--report", default=str(Path(__file__).parent / "vision_report.md")
    )
    args = parser.parse_args()

    runtime = args.runtime

    results, failures = run_tests(runtime, args.timeout)
    bench, bench_failures = benchmark(runtime, args.runs, args.warmup, args.timeout)
    size = file_size(runtime)

    make_report(results, failures, bench, bench_failures, size, args.report)
    print(f"Report written to {args.report}")
    print(
        f"Tests passed: {sum(1 for ok in results.values() if ok)}/{len(TEST_SCRIPTS)}"
    )


if __name__ == "__main__":
    main()
