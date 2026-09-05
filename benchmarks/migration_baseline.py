#!/usr/bin/env python3
"""Compare host runtime size and warm startup during the Rust migration."""

from __future__ import annotations

import argparse
import platform
import random
import statistics
import subprocess
import time
from pathlib import Path


def parse_candidate(value: str) -> tuple[str, Path]:
    label, separator, path = value.partition("=")
    if not separator or not label or not path:
        raise argparse.ArgumentTypeError("candidates must use LABEL=PATH")
    return label, Path(path)


def measure(
    candidates: list[tuple[str, Path]],
    arguments: list[str],
    runs: int,
    warmups: int,
    seed: int,
) -> dict[str, tuple[int, float, float]]:
    commands = {
        label: [str(binary), *arguments] for label, binary in candidates
    }
    for command in commands.values():
        for _ in range(warmups):
            subprocess.run(command, check=True, capture_output=True)

    samples: dict[str, list[float]] = {label: [] for label, _ in candidates}
    order = [label for label, _ in candidates]
    generator = random.Random(seed)
    for _ in range(runs):
        generator.shuffle(order)
        for label in order:
            started = time.perf_counter_ns()
            subprocess.run(commands[label], check=True, capture_output=True)
            samples[label].append((time.perf_counter_ns() - started) / 1_000_000)

    results = {}
    for label, binary in candidates:
        values = sorted(samples[label])
        p95_index = min(len(values) - 1, int(len(values) * 0.95))
        results[label] = (
            binary.stat().st_size,
            statistics.median(values),
            values[p95_index],
        )
    return results


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--zig",
        type=Path,
        default=Path("target/migration-baseline/pocketpy-kipferl"),
    )
    parser.add_argument(
        "--rust",
        type=Path,
        default=Path("target/release/pocketpy-kipferl"),
    )
    parser.add_argument(
        "--candidate",
        action="append",
        type=parse_candidate,
        default=[],
        metavar="LABEL=PATH",
        help="measure named candidates in randomized round-robin order",
    )
    parser.add_argument("--runs", type=int, default=50)
    parser.add_argument("--warmups", type=int, default=5)
    parser.add_argument("--seed", type=int, default=42)
    workload = parser.add_mutually_exclusive_group()
    workload.add_argument(
        "--code",
        default="pass",
        help="Python code passed with -c (default: pass)",
    )
    workload.add_argument(
        "--script",
        type=Path,
        help="Python script used as the common workload",
    )
    args = parser.parse_args()

    if args.runs < 1:
        parser.error("--runs must be positive")
    if args.warmups < 0:
        parser.error("--warmups cannot be negative")

    candidates = args.candidate or [
        ("Zig baseline", args.zig),
        ("Rust spine", args.rust),
    ]
    labels = [label for label, _ in candidates]
    if len(labels) != len(set(labels)):
        parser.error("candidate labels must be unique")
    for _, binary in candidates:
        if not binary.is_file():
            parser.error(f"missing runtime: {binary}")
    if args.script is not None and not args.script.is_file():
        parser.error(f"missing workload: {args.script}")

    arguments = [str(args.script)] if args.script is not None else ["-c", args.code]
    results = measure(candidates, arguments, args.runs, args.warmups, args.seed)

    print(f"Host: {platform.platform()}")
    print(
        f"Runs: {args.runs} per candidate after {args.warmups} warmups "
        f"(randomized round-robin, seed {args.seed})"
    )
    print()
    metric = "startup" if args.script is None else "wall time"
    print(f"| Runtime | Bytes | Median {metric} | p95 {metric} |")
    print("| --- | ---: | ---: | ---: |")
    for name, _ in candidates:
        size, median_ms, p95_ms = results[name]
        print(f"| {name} | {size:,} | {median_ms:.3f} ms | {p95_ms:.3f} ms |")


if __name__ == "__main__":
    main()
