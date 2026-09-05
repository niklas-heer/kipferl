#!/usr/bin/env python3
"""Apply identical executable budgets to CI and release artifacts (decimal bytes)."""
import argparse
from pathlib import Path

LIMITS = {'runtime': 5_750_000, 'core': 2_500_000, 'loader': 1_000_000, 'cli': 9_250_000}


def check(paths):
    for kind, path in paths.items():
        size = path.stat().st_size
        limit = LIMITS[kind]
        print(f'{kind}: {path}: {size:,} bytes (limit: {limit:,})')
        if not 0 < size < limit:
            raise ValueError(f'{kind} size {size:,} is outside its release budget')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for kind in LIMITS:
        parser.add_argument('--' + kind, type=Path, required=True)
    check(vars(parser.parse_args()))


if __name__ == '__main__':
    main()
