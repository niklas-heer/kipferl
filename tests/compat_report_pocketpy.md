# μcharm Compatibility Report

Generated: 2026-08-04 18:40:11

## Summary

### Targeted Modules

- **Tests passing**: 1,185/1,668 (71.0%)
- **Modules at 100%**: 30/52
- **Modules partial**: 6/52
- **No baseline (host CPython)**: 1/52

### CPython Stdlib Coverage

- **Modules targeted**: 52/160 (32.5%)
- **Not yet started**: 109 modules

## Targeted Module Status

| Module | Category | CPython | μcharm | Parity | Notes |
|--------|----------|---------|--------|--------|-------|
| array | stdlib | 69/69 | 69/69 | 100% | ✅ Full |
| base64 | stdlib | 18/18 | 18/18 | 100% | ✅ Full |
| binascii | stdlib | 55/55 | 55/55 | 100% | ✅ Full |
| bisect | stdlib | 58/58 | 58/58 | 100% | ✅ Full |
| collections | stdlib | 49/49 | 49/49 | 100% | ✅ Full |
| copy | stdlib | 33/33 | 33/33 | 100% | ✅ Full |
| csv | stdlib | 24/24 | 24/24 | 100% | ✅ Full |
| dataclasses | stdlib | 8/8 | 8/8 | 100% | ✅ Full |
| datetime | stdlib | 21/21 | 21/21 | 100% | ✅ Full |
| enum | stdlib | 13/13 | 13/13 | 100% | ✅ Full |
| errno | stdlib | 38/38 | 38/38 | 100% | ✅ Full |
| fnmatch | stdlib | 55/55 | 55/55 | 100% | ✅ Full |
| functools | stdlib | 40/40 | 40/40 | 100% | ✅ Full |
| gzip | stdlib | 6/6 | 6/6 | 100% | ✅ Full |
| hashlib | stdlib | 29/29 | 29/29 | 100% | ✅ Full |
| heapq | stdlib | 42/42 | 42/42 | 100% | ✅ Full |
| hmac | stdlib | 4/4 | 4/4 | 100% | ✅ Full |
| io | stdlib | 53/53 | 53/53 | 100% | ✅ Full |
| itertools | stdlib | 33/33 | 33/33 | 100% | ✅ Full |
| json | stdlib | 70/70 | 70/70 | 100% | ✅ Full |
| operator | stdlib | 114/115 | 115/115 | 100% | ✅ Full |
| random | stdlib | 46/46 | 46/46 | 100% | ✅ Full |
| secrets | stdlib | 8/8 | 8/8 | 100% | ✅ Full |
| statistics | stdlib | 28/28 | 28/28 | 100% | ✅ Full |
| struct | stdlib | 68/68 | 68/68 | 100% | ✅ Full |
| tarfile | stdlib | 7/7 | 7/7 | 100% | ✅ Full |
| textwrap | stdlib | 24/24 | 24/24 | 100% | ✅ Full |
| toml | stdlib | - | - | - | ✅ Full |
| typing | stdlib | 43/43 | 43/43 | 100% | ✅ Full |
| uuid | stdlib | 18/18 | 18/18 | 100% | ✅ Full |
| zipfile | stdlib | 7/7 | 7/7 | 100% | ✅ Full |
| math | stdlib | 82/82 | 73/82 | 89% | 9 skipped |
| time | stdlib | 42/42 | 22/42 | 52% | 6 skipped |
| os | stdlib | 45/45 | 3/45 | 7% |  |
| sys | stdlib | 58/58 | 3/58 | 5% | 1 failing |
| urllib_parse | stdlib | 24/24 | 1/24 | 4% | 1 skipped |
| configparser | stdlib | 26/26 | 1/26 | 4% | 1 skipped |
| argparse | stdlib | 26/26 | 0/26 | 0% | 1 skipped |
| contextlib | stdlib | 10/10 | 0/10 | 0% | 1 failing |
| glob | stdlib | 3/3 | 0/3 | 0% | 1 failing |
| http.client | stdlib | 8/8 | 0/8 | 0% |  |
| logging | stdlib | 39/39 | 0/39 | 0% | 1 failing |
| pathlib | stdlib | 40/40 | 0/40 | 0% | 1 skipped |
| re | stdlib | 79/79 | 0/79 | 0% | 1 failing |
| shutil | stdlib | 6/6 | 0/6 | 0% | 1 failing |
| signal | stdlib | 15/15 | 0/15 | 0% | 1 failing |
| sqlite3 | stdlib | 2/2 | 0/2 | 0% |  |
| subprocess | stdlib | 19/19 | 0/19 | 0% | 1 failing |
| tempfile | stdlib | 9/9 | 0/9 | 0% | 1 failing |
| tomllib | stdlib | 0/1 | 0/1 | 0% |  |
| unittest | stdlib | 40/40 | 0/40 | 0% | 1 skipped |
| xml.etree.ElementTree | stdlib | 12/12 | 0/12 | 0% |  |

## Failed Tests

### argparse

- `error: Python execution failed
`

### contextlib

- `error: Python execution failed
`

### re

- `error: Python execution failed
`

### sys

- `FAIL: version_info exists`

### glob

- `error: Python execution failed
`

### logging

- `error: Python execution failed
`

### pathlib

- `error: Python execution failed
`

### shutil

- `error: Python execution failed
`

### signal

- `error: Python execution failed
`

### subprocess

- `error: Python execution failed
`

### tempfile

- `error: Python execution failed
`

### unittest

- `error: Python execution failed
`


## Skipped Tests

These tests require features not available in pocketpy-ucharm:

### argparse

- 1 tests skipped

### bisect

- 5 tests skipped

### collections

- 4 tests skipped

### configparser

- 1 tests skipped

### enum

- 8 tests skipped

### json

- 1 tests skipped

### math

- 9 tests skipped

### time

- 6 tests skipped

### urllib_parse

- 1 tests skipped

### pathlib

- 1 tests skipped

### toml

- 1 tests skipped

### unittest

- 1 tests skipped


## Not Yet Started Modules

The following 109 CPython stdlib modules are not yet targeted:

### Text Processing

- `difflib` - Helpers for computing deltas
- `readline` - GNU readline interface
- `rlcompleter` - Completion function for readline
- `string` - Common string operations
- `stringprep` - Internet string preparation
- `unicodedata` - Unicode database

### Binary Data

- `codecs` - Codec registry and base classes

### Data Types

- `calendar` - Calendar-related functions
- `collections.abc` - Abstract base classes for containers
- `graphlib` - Topological sorting
- `pprint` - Pretty-print data structures
- `reprlib` - Alternate repr() implementation
- `types` - Dynamic type creation
- `weakref` - Weak references
- `zoneinfo` - IANA time zone support

### Numeric and Mathematical

- `cmath` - Math for complex numbers
- `decimal` - Decimal fixed point arithmetic
- `fractions` - Rational numbers
- `numbers` - Numeric abstract base classes

### File and Directory Access

- `filecmp` - File and directory comparisons
- `fileinput` - Iterate over lines from input
- `linecache` - Random access to text lines
- `os.path` - Common pathname manipulations
- `stat` - Interpreting stat() results

### Data Persistence

- `copyreg` - Register pickle support functions
- `dbm` - Interfaces to Unix databases
- `marshal` - Internal Python object serialization
- `pickle` - Python object serialization
- `shelve` - Python object persistence

### Data Compression

- `bz2` - Support for bzip2 compression
- `lzma` - Compression using LZMA algorithm
- `zlib` - Compression compatible with gzip

### File Formats

- `netrc` - netrc file processing
- `plistlib` - Generate and parse Apple plist files

### OS Services

- `ctypes` - Foreign function library
- `curses` - Terminal handling for character-cell
- `curses.ascii` - ASCII character utilities
- `curses.panel` - Panel stack extension for curses
- `curses.textpad` - Text input widget for curses
- `getopt` - C-style parser for command line
- `getpass` - Portable password input
- `logging.config` - Logging configuration
- `logging.handlers` - Logging handlers
- `platform` - Access to platform's identifying data

### Concurrent Execution

- `concurrent.futures` - Launching parallel tasks
- `contextvars` - Context variables
- `multiprocessing` - Process-based parallelism
- `multiprocessing.shared_memory` - Shared memory
- `queue` - Synchronized queue class
- `sched` - Event scheduler
- `threading` - Thread-based parallelism

### Networking

- `asyncio` - Asynchronous I/O
- `mmap` - Memory-mapped file support
- `select` - Waiting for I/O completion
- `selectors` - High-level I/O multiplexing
- `socket` - Low-level networking interface
- `ssl` - TLS/SSL wrapper for sockets

### Internet Data Handling

- `email` - Email and MIME handling
- `mailbox` - Manipulate mailboxes
- `mimetypes` - Map filenames to MIME types
- `quopri` - Encode and decode MIME quoted-printable

### HTML/XML

- `html` - HyperText Markup Language support
- `html.entities` - HTML entity definitions
- `html.parser` - Simple HTML and XHTML parser
- `xml.dom` - Document Object Model API
- `xml.dom.minidom` - Minimal DOM implementation
- `xml.sax` - SAX2 parser support

### Internet Protocols

- `ftplib` - FTP protocol client
- `http` - HTTP modules
- `http.cookies` - HTTP cookie handling
- `http.server` - HTTP servers
- `imaplib` - IMAP4 protocol client
- `ipaddress` - IPv4/IPv6 manipulation
- `poplib` - POP3 protocol client
- `smtplib` - SMTP protocol client
- `socketserver` - Framework for network servers
- `urllib` - URL handling modules
- `urllib.parse` - Parse URLs into components
- `urllib.request` - URL opening library

### Development Tools

- `doctest` - Test interactive Python examples
- `pydoc` - Documentation generator
- `test` - Regression test package
- `unittest.mock` - Mock object library

### Debugging and Profiling

- `bdb` - Debugger framework
- `faulthandler` - Dump Python tracebacks
- `pdb` - Python debugger
- `timeit` - Measure execution time
- `trace` - Trace Python statement execution
- `tracemalloc` - Trace memory allocations

### Runtime Services

- `__future__` - Future statement definitions
- `__main__` - Top-level code environment
- `abc` - Abstract base classes
- `atexit` - Exit handlers
- `builtins` - Built-in objects
- `gc` - Garbage collector interface
- `inspect` - Inspect live objects
- `site` - Site-specific configuration hook
- `sysconfig` - Python's configuration info
- `traceback` - Print or retrieve a traceback
- `warnings` - Warning control

### Custom Python Interpreters

- `code` - Interpreter base classes
- `codeop` - Compile Python code

### Importing

- `importlib` - Import machinery
- `importlib.metadata` - Package metadata
- `importlib.resources` - Package resources
- `modulefinder` - Find modules used by a script
- `pkgutil` - Package extension utilities
- `runpy` - Locate and run Python modules
- `zipimport` - Import modules from ZIP archives


## Notes

- Tests are adapted from CPython's test suite
- Some tests require features not available in PocketPy (threading, gc introspection)
- μcharm-specific modules (ansi, charm, input, term, args) have custom tests
- Report generated by `python3 tests/compat_runner.py --report`