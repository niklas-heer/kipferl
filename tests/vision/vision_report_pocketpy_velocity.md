# Vision A/B

> Historical note: this captured an early PocketPy vs MicroPython comparison.
> Kipferl now uses PocketPy only; the current Vision suite is tracked in `tests/vision/vision_report.md`.

## Runtime Sizes

| Runtime | Size (bytes) |
| --- | --- |
| pocketpy | 608976 |
| micropython | 668104 |

## Vision Tests

| Test | PocketPy | MicroPython |
| --- | --- | --- |
| t_argparse.py | pass | fail |
| t_os_sys_time.py | pass | pass |
| t_pathlib_glob_fnmatch.py | pass | fail |
| t_subprocess.py | pass | fail |
| t_json_csv.py | pass | fail |
| t_logging.py | pass | fail |
| t_datetime.py | pass | fail |
| t_textwrap.py | pass | fail |
| t_tempfile_shutil.py | pass | fail |
| t_re.py | pass | pass |
| t_hashlib.py | pass | fail |
| t_struct.py | pass | pass |
| t_random.py | pass | pass |
| t_typing.py | pass | fail |
| t_itertools_heapq_functools_statistics.py | pass | fail |
| t_signal.py | pass | fail |
| t_io.py | pass | pass |
| t_errno.py | pass | pass |

## Test Summary

- PocketPy (historical snapshot) passes: 18/18
- MicroPython (historical snapshot) passes: 6/18


## Test Failures

### pocketpy
- none

### micropython
- t_argparse.py: fail: TypeError unexpected keyword argument 'prog'
- t_pathlib_glob_fnmatch.py: fail: ImportError no module named 'fnmatch'
- t_subprocess.py: fail: ImportError no module named 'subprocess'
- t_json_csv.py: fail: ImportError no module named 'csv'
- t_logging.py: fail: ImportError no module named 'logging'
- t_datetime.py: fail: ImportError no module named 'datetime'
- t_textwrap.py: fail: ImportError no module named 'textwrap'
- t_tempfile_shutil.py: fail: ImportError no module named 'shutil'
- t_hashlib.py: fail: AttributeError 'sha256' object has no attribute 'hexdigest'
- t_typing.py: fail: ImportError no module named 'typing'
- t_itertools_heapq_functools_statistics.py: fail: ImportError no module named 'functools'
- t_signal.py: fail: ImportError no module named 'signal'

## Startup Benchmarks

| Script | Runtime | Avg (ms) | Median (ms) | P90 (ms) | Runs |
| --- | --- | --- | --- | --- | --- |
| empty.py | pocketpy | 2.24 | 2.19 | 2.46 | 50 |
| import_sys.py | pocketpy | 2.19 | 2.16 | 2.38 | 50 |
| import_json.py | pocketpy | 2.23 | 2.17 | 2.46 | 50 |
| empty.py | micropython | 1.77 | 1.73 | 1.93 | 50 |
| import_sys.py | micropython | 1.78 | 1.76 | 1.96 | 50 |
| import_json.py | micropython | 1.80 | 1.77 | 1.96 | 50 |

## Benchmark Failures

### pocketpy
- none

### micropython
- none
