# Changelog

All notable user-facing changes are documented here.

## [0.9.0] - 2026-11-12

### Added

- `try` statements accept `else` and `finally` clauses. `finally` runs on
  normal completion, on exceptions, and when `break`, `continue` or `return`
  leaves the block; a `return`, `break` or `continue` inside `finally`
  replaces a pending return as in CPython. Each way out of a `try` statement
  compiles its `finally` clause again, so nesting `try`/`finally` inside
  `finally` clauses more than ten levels deep raises `SyntaxError`.
- Generator expressions, lazily evaluated as in CPython: `sum(x * x for x in xs)`,
  `any(...)`, `"".join(...)` and `next((x for x in xs if ...), None)`.
  Comprehensions accept several `for` and `if` clauses
  (`[(a, b) for a in xs for b in ys if a != b]`).
- The format specification mini-language for numbers and strings, except the
  `z` option:
  thousands separators (`f'{n:,}'`, `'{:_}'`, `f'{price:,.2f}'`), `b`/`o`/`x`/`X`
  with `#` prefixes, `e`/`g`/`%` types, `+` and space signs, `=` alignment and
  any fill character and any precision. Zero padding now follows the sign
  (`f'{-3.5:08.3f}'` is `-003.500`). New `format()` builtin, `__format__` support and the `!s`
  conversion in f-strings.
- `%`-formatting supports flags, width, precision, `%(key)s` mappings and every
  conversion type (`'%5.1f'`, `'%-10s'`, `'%05d'`, `'%x'`), and `str.format`
  supports named, attribute and index fields, conversions and nested
  specifications (`'{name}'.format(name=x)`, `'{:>{w}}'`); `str.format_map()`
  is new.
- `dict()` and `dict.update()` accept mappings, any iterable of pairs and keyword
  arguments (`dict(zip(keys, values))`, `dict(a=1)`, `dict(defaultdict)`), and
  dicts gain `setdefault()`, `dict.fromkeys()` and the `|` and `|=` operators.
- Augmented assignment calls `__iadd__`, `__ior__` and the other in-place
  methods first, as in CPython: `items += more`, `items *= n`, `d |= other`
  and `s |= other` change a list, dict or set in place (and `*=` an
  `array.array`), so a function that receives it changes the caller's object.
- `typing.Optional`, `Union`, `Literal` and the other special forms can be
  subscripted outside annotations (`MaybeInt = Optional[int]`), and `typing`
  provides `NamedTuple`, `TypedDict` and `NewType`.
- Any iterable can be unpacked (`a, b = map(int, line.split())`), the starred
  target may appear anywhere (`first, *middle, last = items`), and `for`
  targets nest (`for i, (key, value) in enumerate(pairs)`).
- `raise NewError(...) from err` sets `__cause__`, and exceptions accept several
  arguments (`ValueError('a', 1).args`).
- `print()` accepts `file=` (`print(message, file=sys.stderr)`), and `sep` and
  `end` accept `None`.
- `with a() as x, b() as y:`, keyword-only (`*`) and positional-only (`/`)
  parameters, `*args` after parameters with defaults, and digit separators in
  literals and `int()` (`1_000_000`, `int('1_000')`, `int(' 42 ')`). Arguments
  bind as in CPython: positional parameters accept keywords (`f(1, b=2)`),
  keyword-only parameters reject positional arguments, a missing one raises
  `TypeError` at the call, and positional-only parameters reject keywords.
  `inspect.signature()` shows the `/` and `*` markers.
- `ArithmeticError`, the base class of `OverflowError` and `ZeroDivisionError`.
- Everyday builtins and methods: `pow(base, exp, mod)`, `sum(..., start)`,
  `min`/`max` with `default=`, `zip(..., strict=True)`, `str.split` and
  `str.rsplit` with `maxsplit` (and `sep=None`), `str.replace` with a count,
  `str.removeprefix`/`removesuffix`/`partition`/`rpartition`/`title`/
  `capitalize`/`swapcase`/`casefold`/`center`/`expandtabs`, set comparisons
  (`<=`, `<`, `>=`, `>`) and in-place updates, `set.pop`, `float.is_integer`,
  `int.to_bytes`/`from_bytes`/`bit_count` and `bytes.hex`.

### Changed

- `repr()` of a string containing `'` but no `"` uses double quotes, as in
  CPython.
- `round()` rounds halves to even as CPython does: `round(2.5)` is `2` and
  `round(2.675, 2)` is `2.67` (correctly rounded from the binary value);
  negative `ndigits` work for ints and floats.
- **BREAKING: integer overflow raises `OverflowError` instead of wrapping.**
  Kipferl's integers are 64-bit; CPython's are unbounded. `2**63`, `2**62 * 2`,
  `int('9223372036854775808')`, `1 << 64`, `True + sys.maxsize` and
  `math.factorial(21)` used to return silently wrong numbers and now raise the
  new builtin `OverflowError`. `int()`, `round()` and
  `math.floor`/`ceil`/`trunc` raise `OverflowError` instead of `ValueError` for
  infinities and floats beyond 64 bits. Programs that relied on wrapping, such
  as hash functions that multiply freely, must keep every intermediate result
  within 64 bits, for example by multiplying 32-bit halves: masking afterwards
  does not help, because the operation that overflows raises first
  (`(sys.maxsize * 2) & sys.maxsize` raises).
  `sys.maxsize` is the largest int, 2**63 - 1. Write it, and the smallest
  int, as `sys.maxsize` and `-sys.maxsize - 1`: `2**63 - 1` and `-2**63`
  raise `OverflowError` because `2**63` is computed first. Integer literals
  beyond 64 bits, such as `0xFFFFFFFFFFFFFFFF`, are a `SyntaxError` ("int
  literal is too large"), and `int()` of such text raises `OverflowError`.
  `(-sys.maxsize - 1) // 2`, `range()` near the limits, `int.bit_length()` and
  `math.gcd()` of the smallest int, and `pow(base, exp, mod)` and `round(n, -k)`
  whose intermediate results exceed 64 bits are computed exactly.
- Argument errors use CPython's messages: `f() missing 1 required positional
  argument: 'x'` and `f() takes from 1 to 2 positional arguments but 3 were
  given` replace `f() takes 1 positional arguments but 0 were given` and
  `too many arguments (f)`.

### Fixed

- `items.extend(items)` and `items += items` on a list that has to grow read
  the list after growing had freed it, so the copy could contain garbage or
  crash the process.
- A bare `raise` re-raises the exception being handled from any block inside
  the handler or a `finally` clause (`except E: with lock: raise`), and
  outside a handler raises `RuntimeError: No active exception to reraise`
  when it runs, as in CPython; both were a `SyntaxError`. A function called
  from a handler or `finally` clause (`def reraiser(): raise`) re-raises the
  exception its callers are handling.
- A function or module whose jumps exceed PocketPy's 16-bit offsets, such as
  an `if` around tens of thousands of statements, raises `SyntaxError` ("code
  is too large to compile") instead of aborting the process.
- `json.dumps` indents when combined with `sort_keys=True` or `separators`,
  accepts a string or zero `indent`, and quotes non-string keys as CPython does.
- `{**a, **b}` and `{**a, 'key': value}` merge dicts instead of crashing the
  process, and `[*a, *b]`, `(*a,)` and `{*a}` unpack instead of storing
  internal `star_wrapper` objects.
- `None` is hashable, so `{None: ...}`, `set([None])` and grouping by an
  optional key work.
- A class that defines `__eq__` without `__ne__` no longer fails with `TypeError`;
  `!=` negates `__eq__` as in CPython.
- `print()` and `sys.stdout.write()` output keep program order when piped.
- Custom exceptions can store attributes (`self.code = code` in `__init__`)
  and take keyword arguments; both raised `TypeError`.
- An exception raised while evaluating an `except` clause, such as a misspelled
  exception name, propagates instead of hanging the program.
- `break`, `continue` and `return` inside a `with` block now call the context
  manager's `__exit__`; previously `with open(path) as f: return f.read()`
  left the file open.
- `__exit__` receives `(exc_type, exc_value, traceback)` as in CPython. A
  context manager written as `def __exit__(self, exc_type, exc, tb)` failed
  with `TypeError`, and `contextlib.suppress` did not suppress anything.
  `unittest`'s `assertRaises` context now also exposes `.exception`.

## [0.8.0] - 2026-11-11

### Changed

- Update the embedded PocketPy interpreter from 2.1.3 to 2.2.0, with Kipferl's
  patches ported to its new module system. Visible differences:
  - Floats print with full round-trip precision, as in CPython:
    `0.1 + 0.2` prints `0.30000000000000004` instead of `0.3`, and
    `json.dumps` and f-strings follow. Subnormals such as `5e-324` print with
    15 significant digits.
  - `math.floor`, `math.ceil` and `math.trunc` return `int` instead of `float`;
    NaN, infinities and values beyond 64-bit integers raise `ValueError`.
    `math.isinf`, `math.isnan` and `math.isfinite` return `bool`.
  - `abs(-0.0)` is `0.0`; `math.log(x, base)` rounds like CPython.
  - New: `%`-formatting of tuples (`"%s %r" % (a, b)`), `bool` arithmetic,
    `str.splitlines`, iteration over `bytes`, `bin()`, `dict.popitem()`,
    signed `str.zfill`, `inspect.signature`, `typing.override`,
    `typing.TypeGuard` and `typing.TypeIs`. Modules print as `<module 'name'>`.
  - PocketPy's undocumented `linalg` and `colorcvt` modules are gone upstream;
    its new `stdc` raw-memory module and `gc.track`/`gc.untrack` are not
    exposed, because they let scripts corrupt memory, and neither is
    `gc.setup_debug_callback`, which aborts the process when its callback
    raises.
- Startup and the bundled benchmarks are unchanged or slightly faster; the full
  runtime grows by about 66 KB and the core runtime by about 83 KB.
- **Tracebacks moved to stderr.** An uncaught exception now prints its
  traceback on stderr, as CPython does, instead of stdout; the exit status is
  still 1 and stdout holds only what the program printed. This applies to
  `kipferl run`, `kipferl dev`, `kipferl test`, built applications, and both
  runtime profiles. Scripts or CI jobs that read tracebacks from stdout must
  read stderr instead, for example with `2>&1`.
- Tracebacks no longer show Kipferl's startup frame
  (`exec(__kipferl_code)` in a temporary file) or the loader frames of bundled
  local modules. The remaining frames, their file names, and their line
  numbers are unchanged.
- Kipferl's own CLI output is styled only on a terminal. Piped or redirected
  output, such as `kipferl --version > file`, no longer contains escape codes.
  `NO_COLOR` set to a non-empty value disables styling; `CLICOLOR_FORCE`
  (except `0`) or `FORCE_COLOR` (except `0` and `false`) enable it for pipes;
  `NO_COLOR` wins over both, and `TERM=dumb` counts as no terminal.
- `tui` and `input` add no colours or styles of their own when `NO_COLOR` is
  set to a non-empty value, and `tui.style()` then returns its text unchanged.
  Without `NO_COLOR` their output is unchanged, including in pipes. `ansi`
  always returns the requested codes. `input` previously also treated an
  empty `NO_COLOR` as set.
- The `kipferl new` starters (`cli`, `api`, `interactive`) print their errors
  on stderr instead of stdout and still exit with status 2 (1 for request
  failures). Existing projects keep their generated code.

### Security

- Scripts could corrupt memory through PocketPy: `'a' * (2**32 + 1)` and
  `array2d.array2d(65536, 65537, f)` wrote past their allocations, and
  `pkpy.TValue` built objects from raw addresses. Oversized repetitions and
  grids now raise `ValueError`, and `pkpy.TValue` is no longer available.
  These also affect 0.7.x.
- Scripts could also forge objects or read arbitrary memory through types that
  stayed reachable through `object.__subclasses__()`: the TValue types behind
  `pkpy.TValue` and the static `read`/`write` methods of `stdc`'s memory types.
  Neither exists any more.
- A C method or constructor called through its class trusted the type of its
  first argument, so `dict.clear([1])`, `dict.__new__(list)`,
  `bytearray.__len__([])` or a class that borrowed another type's method
  (`__getitem__ = list.__getitem__`) read or wrote one object as another. They
  now raise `TypeError`, as in CPython.
- `pickle.loads` trusted its input: crafted bytes could forge objects at any
  address, read past the data or overflow the interpreter stack. Malformed
  data now raises `ValueError`; pickles made by `pickle.dumps` load as before.
  As in CPython, a pickle can still call functions, so load only trusted data.
- An object whose size does not fit PocketPy's 32-bit allocation size, such as
  a string of 2 GiB, raises `ValueError` where a script requests it directly
  and otherwise stops the process instead of writing past a short allocation.
  Long lists no longer overflow the byte offsets used to extend or sort them.

### Fixed

- `int(x)` and `round(x)` raise `ValueError` for NaN, infinities and floats
  beyond 64-bit integers, like `math.floor`, instead of returning an arbitrary
  value. `round(x, n)` returns `x` when `n` exceeds its precision instead of
  looping `n` times. `x << n` and `x >> n` raise `ValueError` for negative `n`
  and are 0 or -1 for `n` of 64 or more; 64-bit integer overflow keeps
  wrapping, now by definition rather than by accident of the C compiler.
- Assigning a function's `__doc__`, as `functools.wraps` does, leaked the
  previous docstring.
- A garbage collection while caught exceptions were still referenced could
  read uninitialized memory and crash, seen on Linux. The cause is in
  PocketPy's traceback recording and also affects 0.7.x.
- `input.prompt`, `password`, `select`, `multiselect` and `confirm` finish when
  their input ends instead of polling a closed stdin at full CPU forever. This
  affects scripts run without a terminal (CI, cron, `ssh` without `-t`) and
  injected test keys that run out. Selections and confirmation cancel, as
  Escape does. `prompt` and `password` accept text typed before the end as the
  final line, as with `printf 'alice' | app`, and cancel when nothing was
  typed; Ctrl-D in a terminal now does the same.

## [0.7.4] - 2026-11-10

### Security

- Update the `zip` crate to 9.x, which fixes archive parsing advisories in the
  `zipfile` module and wheel extraction, and `rustls` to 0.23.46 for HTTPS.
  `zipfile` member names are now decoded as UTF-8 whenever they are valid
  UTF-8 and as CP437 otherwise, whatever the entry's UTF-8 flag says. Before,
  the flag decided, as in CPython. Non-ASCII names stored without the flag
  differ when their bytes also form valid UTF-8, mislabelled or genuine CP437.

## [0.7.3] - 2026-11-10

### Added

- A workflow compatibility guide with Verified, Limited, Unsupported, and
  Untested ratings, tested scopes, and explicit platform coverage. Assess all
  44 compilation-complete candidates through real installation; separate the
  15 typing distributions and five dependency bundles from runtime libraries.
- Reviewed package scenarios with CPython controls, locked dependency evidence,
  offline restoration, and detached standalone execution. The released 0.7.2
  macOS ARM64 binary passes the scoped tzdata and trove-classifiers data workflows.
  Add their exact tested artifacts to the source CLI catalog.
- A ranked support roadmap for 100 application-oriented packages, with useful
  workflows, concrete acceptance tests, current blockers, and explainable scores.
- `kipferl deps verified` lists the tested package versions and behavior scopes
  matching the CLI's embedded runtime and platform; `--json` exports their evidence.
- A source-built Nix flake: `nix profile add github:niklas-heer/kipferl`. It
  follows `main`, so it can be ahead of the latest release.

### Fixed

- Skip transitive test/docs requirements that cannot activate without extras,
  consistently during dependency resolution and offline lock validation. Keep
  active or ambiguous conditions rejected; preserve original wheel metadata.
- Accept Python null-prefixed and three-digit octal escapes in native regular
  expressions across search, findall, substitution, and splitting. Numeric
  backreferences, short nonzero class escapes, and surrogate patterns remain
  unsupported. This clears a real character-class blocker without promoting
  packages whose remaining patterns still fail.

## [0.7.2] - 2026-09-05

### Fixed

- Reaudit all 1,000 pinned popular projects on the current runtime and update
  the CLI's embedded full report and exact syntax-blocker catalog.
- Bind each release CLI to a fresh audit of its own native runtime and target.
  Validate complete coverage, metadata pins, hashes, and derived exports before
  packaging, and verify the embedded report in published-binary smoke tests.
- Publish per-platform audit JSON, CSV, and syntax catalogs with checksums.
- Refresh the documentation and website with the rerun and comparison: the
  candidate retains 44 compilation-complete distributions, 24 with Python
  source, and no changed first blockers. These remain behaviorally unverified.

### Development

- Check in the original 999 available package metadata pins and explicitly
  record the missing ddtrace pin. Reuse verified downloads, never old compiler
  results, when producing release audits.

## [0.7.1] - 2026-09-05

### Fixed

- Replace the old line-art project banner with the canonical Kipferl banner.
- Complete the previously announced rename transition: use `kipferl` imports,
  commands, download names, and `KIPFERL_*` environment variables. Retire the
  deprecated command/download aliases and test-key variable.
- Check branding in source, filenames, and the obsolete line-art logo in CI.
- Refresh the website and all demo formats using the published 0.7.1 CLI.

### Upgrade notes

- Update old project imports and shell configuration to the canonical names
  before upgrading. The v1 universal executable bytes remain compatible;
  existing standalone applications continue to run.
- Runtime-bound dependency locks need refreshing after the runtime changes.
  Re-run `kipferl add` for declared requirements, review the lock, and test.

## [0.7.0] - 2026-09-05

### Added

- Native dotted imports with parent initialization, root/alias binding, circular
  imports, failure retries, and preserved globals from failed attempts.
- Exact per-platform release catalogs and package smoke reports verify online
  installation, locked offline restoration, and detached standalone execution.
- Trailing commas in parenthesized imports and function/lambda parameters,
  and adjacent plain strings/bytes, including default values.
- A reproducible top-1,000 package audit, ranked CLI/website views, and a
  before/after comparison. The first language patches increase source-bearing
  compilation-complete candidates from 12 to 20; dotted imports raise this to
  24 (44 total including zero-source distributions). These remain unverified
  and tied to the historical audit runtime, separately from release evidence.

- PyPI pure-Python wheel dependencies with `kipferl add`, exact artifact/runtime
  locks, offline `sync --locked`, compatibility catalog and installed-file checks.
  The initial catalog contains ten distribution versions with scoped test or
  blocker evidence. Unknown artifacts require explicit acceptance; native
  extensions, source builds, extras, and environment markers are unsupported.
- Installed package imports, resources, and license metadata flow through run,
  test, and standalone builds; new editor configurations include package paths.
- Portable builds include local Python modules and packages, follow their
  dependencies when selecting a runtime, and support explicit `--asset` files
  and directories. Build-time checks catch unsupported imports and syntax.
- `kipferl new --template cli|api|interactive` creates a runnable project with
  editor stubs, a README, tests, and a `kipferl.json` configuration.
- Project defaults let `run`, `dev`, `build`, and `test` work without repeating
  paths, including from nested project directories.
- Ordinary `test_*.py` project tests and Bash, Zsh, and Fish completions.
- Four executable recipes for CSV summaries, HTTP API clients, repository
  summaries, and report generation, checked against their documentation in CI.

### Changed

- Dynamic `__import__("http.client")` returns root `http`, following Python.
  Use a child-module alias or a nonempty positional fromlist when needed.
- Dependency locks identify exact runtime bytes and target. After an upgrade,
  rerun `kipferl add` for declared requirements, review the lock, and test the app.

### Fixed

- Package and bundle syntax checks use nonexecuting module compilation.
  Generated wrappers preserve module globals and original traceback filenames.

- Prevent callback-driven heap and iterator mutations from using invalid values;
  preserve comparator exceptions and snapshot containers before deepcopy hooks.
- Handle Unicode regex replacement escapes, oversized byte arrays, and negative
  iterator ranges without panicking. Reject finite values overflowing struct f32.
- Avoid overflowing median/progress calculations and long-version logo padding.
  Validate terminal dimensions and cap rule output at one million columns.

- Reject malformed non-ASCII download checksums without panicking. Report corrupt
  embedded runtimes and missing build assets with contextual errors.
- Reject extreme HTTP timeouts without aborting across FFI. Correct frexp/ldexp
  at float boundaries, atanh endpoint/NaN handling, and negative fractional times.
- Return structured VM metadata errors for C ABI length overflow. Remove
  unchecked format/loader slicing and move cache buffers off small worker stacks.

- Runtime errors identify original Python files and line numbers; application
  arguments and `sys.exit()` statuses survive development and packaged runs.
- Validate complete cached runtime and script contents, repairing same-size
  corruption and stale standalone payloads beyond the legacy hash sample.
- Build outputs are written atomically, preserving the previous artifact on
  failure and replacing output symlinks without changing their targets.
- Enforce the script size limit while reading, handle multiline legacy imports,
  and quote generated project names safely in Python source.
- Correct in-memory buffer seeking, line hints, closed-state errors, integer
  arguments, and empty/sparse writes; avoid quadratic work when appending.
- Preserve captured subprocess output after `wait()`, honor `Popen(text=True)`,
  and return the actual negative signal number for terminated processes.
- Compatibility checks now reject crashed processes, missing fixtures, and
  invalid runtimes. Vision reports retain timeout and benchmark failures,
  support single-sample runs, and include the supported TOML fixture.

### Development

- Add pinned Bacon, nextest, watchexec, and optional cargo-seek through mise;
  configure failure navigation, zero retries, slow/leaked-process detection,
  and CI JUnit reports. Add Criterion 0.8.2 as a loader-only development dependency.
- Enforce every requested Clippy restriction, pedantic, and nursery across the
  workspace. Require a clean full/core audit and publish all narrowly justified
  exceptions with their reasons; reject blanket groups and unexplained exceptions.

- Include Rust Analyzer and standard-library sources in the pinned setup;
  explicitly check all Rust features and targets in the shared CI task.
- Reject debug/placeholder macros and unchecked time subtraction in workspace
  Clippy. Forbid unsafe code in the format crate and document error contracts
  with executable API examples.
- Replace the justfile with mise tasks, exact development tool pins, a
  multiplatform tool lockfile, and shared local/CI setup for Rust and the website.

- Add regression coverage across CLI, loader, runtime, and test runners,
  including 56 additional I/O and subprocess compatibility checks.
- Correct CPython fixture inputs and skip Kipferl-only extensions on the host,
  so compatibility baselines complete on the CI Python version.
- Run development and release tooling tests in both `mise run check` and CI.
- Document focused runtime checks and the prebuilt CLI asset workflow.

## [0.6.0] - 2026-08-05

### Added

- Profile-based tree shaking selects a small core runtime or the complete
  runtime from static imports without requiring a user Rust toolchain.
- `--full-runtime` provides an explicit conservative override for universal
  builds.
- `kipferl dev` watches project and extra paths with native filesystem events,
  debounces editor bursts, and restores terminal state between restarts.
- Ratatui-backed `input.select` and `input.multiselect` provide responsive,
  accessible interaction while preserving terminal scrollback.
- Maintained Rust parsers provide YAML 1.2, TOML, and KDL 2.0 alongside the
  existing JSON, XML, CSV, and INI/CFG APIs.
- HTTPS, SQLite, archive, cryptographic, filesystem, process, and terminal
  capabilities are available through the curated runtime.
- macOS ARM64/x86_64 and static-musl Linux ARM64/x86_64 release assets include
  adjacent SHA-256 checksums.

### Changed

- Renamed the project, repository, binary, packages, and public site from
  μcharm/ucharm to Kipferl.
- Reimplemented the production CLI, universal loader, PocketPy host, and native
  modules in stable Rust; PocketPy remains the embedded C runtime.
- Replaced handwritten HTTP and archive internals with feature-minimal,
  maintained Rust libraries while preserving the Python-facing API.
- Updated the README, website, command documentation, templates, examples,
  benchmarks, and migration retrospective for the Rust architecture.

### Performance

- The four tree-shaken core runtimes are 1,130,352–1,349,904 bytes, 72.2–76.6%
  smaller than their full-runtime counterparts.
- A minimal Apple Silicon standalone app is 1,450,837 bytes, 69.9% smaller than
  the 4,817,925-byte full-runtime build.
- The measured Apple Silicon core app starts in 7.679 ms median and 8.433 ms
  p95 over 100 runs.

### Compatibility

- Passes 1,669/1,669 available compatibility checks, with 51 fully compatible
  targeted modules and one host-unavailable TOML baseline.
- Preserves the Kipferl v1 standalone application format.
- Accepts legacy `from ucharm ...` source and environment variables, publishes
  temporary `ucharm-*` assets, and installs a deprecated `ucharm` command alias
  for the 0.6 transition.

[0.7.0]: https://github.com/niklas-heer/kipferl/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/niklas-heer/kipferl/compare/v0.5.0...v0.6.0

[0.7.1]: https://github.com/niklas-heer/kipferl/compare/v0.7.0...v0.7.1

[0.7.2]: https://github.com/niklas-heer/kipferl/compare/v0.7.1...v0.7.2

[0.7.3]: https://github.com/niklas-heer/kipferl/compare/v0.7.2...v0.7.3

[0.7.4]: https://github.com/niklas-heer/kipferl/compare/v0.7.3...v0.7.4

[0.8.0]: https://github.com/niklas-heer/kipferl/compare/v0.7.4...v0.8.0

[0.9.0]: https://github.com/niklas-heer/kipferl/compare/v0.8.0...v0.9.0
