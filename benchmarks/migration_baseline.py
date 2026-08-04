#!/usr/bin/env python3
"""Compare host runtime size and warm startup during the Rust migration."""

from __future__ import annotations

import argparse
import platform
import statistics
import subprocess
import time
from pathlib import Path


def measure(binary: Path, runs: int) -> tuple[int, float, float]:
    command = [str(binary), "-c", "pass"]
    for _ in range(5):
        subprocess.run(command, check=True, capture_output=True)

    samples: list[float] = []
    for _ in range(runs):
        started = time.perf_counter_ns()
        subprocess.run(command, check=True, capture_output=True)
        samples.append((time.perf_counter_ns() - started) / 1_000_000)

    samples.sort()
    p95_index = min(len(samples) - 1, int(len(samples) * 0.95))
    return binary.stat().st_size, statistics.median(samples), samples[p95_index]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--zig",
        type=Path,
        default=Path("pocketpy/zig-out/bin/pocketpy-ucharm"),
    )
    parser.add_argument(
        "--rust",
        type=Path,
        default=Path("target/release/pocketpy-ucharm-rs"),
    )
    parser.add_argument("--runs", type=int, default=50)
    args = parser.parse_args()

    if args.runs < 1:
        parser.error("--runs must be positive")

    print(f"Host: {platform.platform()}")
    print(f"Runs: {args.runs} after 5 warmups")
    print()
    print("| Runtime | Bytes | Median startup | p95 startup |")
    print("| --- | ---: | ---: | ---: |")
    for name, binary in (("Zig baseline", args.zig), ("Rust spine", args.rust)):
        if not binary.is_file():
            raise SystemExit(f"missing runtime: {binary}")
        size, median_ms, p95_ms = measure(binary, args.runs)
        print(f"| {name} | {size:,} | {median_ms:.3f} ms | {p95_ms:.3f} ms |")


if __name__ == "__main__":
    main()
