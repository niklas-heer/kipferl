## PocketPy vendor patchset (Kipferl)

PocketPy is vendored as an amalgamated `pocketpy/vendor/pocketpy.c` + `pocketpy/vendor/pocketpy.h`.
For Kipferl CPython-compatibility, we maintain a small patchset that must be re-applied after updating PocketPy.

### Apply

From repo root:

```bash
./scripts/apply-pocketpy-patches.sh
```

The script is idempotent: it applies missing patches and skips patches that are already applied.

### Verify

```bash
python3 scripts/verify-pocketpy-patches.py
python3 scripts/verify-pocketpy-patches.py --check-upstream
```

Verification checks the manifest and `kipferl patch:` anchors in
`pocketpy/vendor/pocketpy.c`. With `--check-upstream`, it downloads pristine
upstream `pocketpy.c` for the recorded `pocketpy/POCKETPY_VERSION`, replays the
ordered patches, and compares the result byte for byte with the vendored file.

### Package syntax compatibility patches

`0005-trailing-commas.patch` accepts trailing commas in parenthesized
`from ... import (...)` lists and function/lambda parameters, including
`*args` and `**kwargs`. Empty import lists, double commas, unparenthesized
trailing import commas, and parameters after `**kwargs` remain errors.
PocketPy 2.2 accepts a trailing comma in `def` parameter lists and drops
newlines inside brackets itself, so the patch only adds the lambda terminator
and the parenthesized from-import case.

`0006-adjacent-literals.patch` combines adjacent plain strings and bytes at
compile time, including default values. It preserves Unicode and embedded NUL
bytes, keeps string and bytes types separate, and ignores physical newlines
and comments only inside brackets. Default literal parentheses preserve the
distinction between grouping, an empty tuple, and a tuple with a trailing comma.
The existing four-element default-tuple limit remains. Adjacent combinations
involving f-strings remain unsupported; ordinary f-strings are unchanged.

Behavior and malformed-input regressions live in
`crates/kipferl-runtime/tests/language_literals.rs`, with CPython as the
comparison oracle. The adjacent-literal parser uses a single growing buffer
for each run and keeps token ownership intact during both successful and
failed compilation.

### Dotted import semantics

`0007-dotted-imports.patch` implements `import package.child` (bind the root),
`import package.child as alias` (bind the child attribute, with a cached-module
fallback), and parent initialization for `from package.child import name`.
Packages initialize before their children; successful children attach to their
parent, and repeated imports preserve deliberate parent-attribute changes.
Relative from-imports use the current module's package metadata. Native dotted
modules such as `os.path`, `urllib.parse`, and `xml.etree.ElementTree` keep their
registered identities. A regular package takes precedence over a same-named
`.py` file, reload keeps using the package initializer, and a plain module
cannot acquire filesystem children.

The dedicated `IMPORT_FROM` opcode keeps missing-submodule fallback separate
from ordinary attribute access, including circular imports. Modules expose full
`__name__`, `__package__`, `__file__`, and package `__path__` metadata. Failed
imports remove only the failed cache entry and preserve successful side-effect
imports. Module references have stable storage per import attempt; escaped
functions and classes keep the original globals across retries and collection.
Cache keys are interned so failed-module collection cannot invalidate them.

Python's `__import__` returns the root by default, or the leaf with a nonempty
positional fromlist. Package fromlists and `__all__` can load children; list
mutation during import does not leave stale C pointers. Nonzero `__import__`
levels remain explicitly unsupported: use a relative from-import statement.
`sys.modules` reflects loaded and loading modules and removes failed attempts;
arbitrary edits to that dictionary do not control the native import cache.
Namespace packages, custom import finders, and custom `__path__` search locations
are outside this patch.

Regression tests in `crates/kipferl-runtime/tests/dotted_imports.rs` construct
isolated local package trees and compare their behavior with CPython, including
failure retries, circular imports, shadowing, reexports, and retained objects.

PocketPy 2.2 rewrote the module system and now provides parts of this itself:
implicit parent imports, root binding for `import a.b`, full `__name__`, and a
`__package__` used for relative imports. The patch keeps that structure and adds
what upstream still lacks: the package flag (upstream gives a top-level module
its own name as `__package__`), parent attributes for children, failure
eviction with stable module identity, `IMPORT_FROM`, `sys.modules`, `__file__`
and `__path__`, package-before-module precedence, and `__import__` fromlists.
Upstream's ordinary `getattr` fallback that imported `module.name` on any
missing attribute is removed. Kipferl imports source files only: the `.pyc`
probes that 2.2 added are left out, so an import performs at most two file
lookups and never executes serialized bytecode found on disk.

### Math accuracy

PocketPy 2.2 replaced the C library's math with its own `dmath` routines
everywhere: `sqrt(x)` is `pow(x, 0.5)`, `pow` is `exp(y * log(x))`, `exp` and
`log` use short polynomial and spline approximations, `floor`/`ceil`/`trunc`
cast through `int64_t`, and large-argument trigonometric reduction is disabled.
`math.sqrt(9)`, `9 ** 0.5` and `math.pow(9, 0.5)` returned `2.9999999999999996`,
and `1e300 // 1.0` was undefined. `0008-libm-math.patch` keeps `dmath` only
when `PK_ENABLE_DETERMINISM` is set and otherwise delegates every approximated
function to the platform libm, as PocketPy 2.1 and CPython do. Exact helpers
(`fmod`, `modf`, `copysign`, `fabs`, the classification functions) stay shared.
It also computes `math.log(x, base)` as `log(x) / log(base)` like CPython, and
`math.floor`, `math.ceil` and `math.trunc`, which now return `int`, raise
`ValueError` for NaN, infinities and values outside 64-bit integers instead of
performing an undefined conversion (PocketPy has no `OverflowError`).
`int(x)` and `round(x)` reject the same values with the same messages; they
cast to `int64_t` directly in upstream, also in 2.1.3. `round(x, n)` returns
`x` unchanged when `x * 10**n` does not fit an `int64`, which covers NaN,
infinities, huge `x` and huge `n`; it previously looped `n` times.
`crates/kipferl-runtime/tests/float_math.rs` compares results bit for bit with
CPython.

### Memory-unsafe script APIs

PocketPy 2.2 adds two ways for a script to corrupt memory, and
`0009-unsafe-script-apis.patch` removes both so memory-unsafe operations stay
behind Kipferl's Rust FFI boundary:

- A `stdc` module with `malloc`, `free`, `memcpy` and raw reads and writes by
  integer address. Its types are predefined, so the VM still creates them, but
  the module is removed from the import cache. A local `stdc.py` still imports.
- `gc.track` and `gc.untrack`. An untracked container is kept but its children
  are not marked, so the next collection frees objects it still references and
  later reads see reused memory. CPython has neither function; `gc.is_tracked`
  remains.

The same patch also removes:

- `pkpy.TValue`, present since 2.1. Its constructors built a value of any type
  from a raw integer, so `pkpy.TValue[int].__new__(list, address)` gave a list
  whose object pointer was that address. Unpublishing the name was not enough:
  the concrete `TValue_int`, `TValue_float`, `TValue_vec2` and `TValue_vec2i`
  types stayed reachable through `object.__subclasses__()`, so the patch never
  creates them. The rest of `pkpy` stays.
- The methods of `stdc`'s memory types (`Int.read(address, offset)`, `write`,
  `array`, unchecked indexing). The types are still created, empty, because
  their indices are predefined, and they too are reachable through
  `__subclasses__()`; the module's functions are not bound either.
- `gc.setup_debug_callback`, new in 2.2. It runs a Python callback inside every
  collection and aborts the process when the callback raises, so a script
  error ended the process instead of raising.

### Traceback frames and the collector

Each caught exception records up to seven traceback frames. PocketPy fills
their `locals` and `globals` only while its debugger runs, yet the collector
marks both fields for every exception, so a collection read uninitialized
memory. This crashed Linux builds (SIGSEGV) and fails UBSan's bool check; it is
present in 2.1.3 as well. `0010-exception-frame-gc.patch` sets both to nil when
the debugger is off. `crates/kipferl-runtime/tests/ffi_stress.rs`, which the
sanitizer CI job runs, keeps hundreds of caught exceptions alive across
collections.

### Integer bounds

PocketPy's integers are C `int64_t`, and several operations reached
undefined behaviour from a script. These are present in 2.1.3 as well.
`0011-integer-bounds.patch` fixes the ones that corrupt memory or whose
result depends on the compiler:

- `str * n` sized the result with a C `int`, so `'a' * (2**32 + 1)` allocated
  one byte and kept writing past it. A result longer than
  `INT32_MAX` bytes raises `ValueError("repeated string is too long")`.
- `array2d(cols, rows)` multiplied its dimensions as `int`, so
  `array2d(65536, 65537, f)` allocated 65,536 cells and wrote past them. The
  cells are the object's slots, and the object's size in bytes, header
  included, is an `int` as well: `array2d(178956971, 1)` passed a check on the
  element count alone. Every array2d is now created through one check of its
  byte size, so `array2d()`, `fromlist`, the results of `map` and similar
  methods on views, and unpickling raise `ValueError` for dimensions that do
  not fit; views reject a `width * height` beyond `INT32_MAX`.
- The allocator itself (`ManagedHeap__gcnew`, behind `py_newobject`, strings,
  bytes and tuples) computes the object size in 64 bits and stops the process
  when it does not fit an `int` or the allocation fails, instead of wrapping
  to a short allocation. `py_newstrn` and `py_newbytes` stop on a negative
  length, which only a wrapped size computation produces. `str * n` raises
  `ValueError` before reaching that limit.
- Growing a vector past `INT32_MAX` elements stops the process instead of
  wrapping its capacity, and extending a vector or sorting a list computes byte
  offsets in `size_t`, which an `int` product overflowed from about 90 million
  list items.
- `<<` and `>>` used the count directly: negative counts raise
  `ValueError("negative shift count")` as in CPython, and counts of 64 or more
  give `0` (or `-1` for a negative operand shifted right).

- An empty `bytes` slice passed a null buffer to `memcpy`, which C leaves
  undefined even for zero bytes; GCC's UndefinedBehaviorSanitizer reports it.

Overflowing `+`, `-`, `*`, `**`, negation, `abs`, integer parsing and slice
arithmetic wraps in two's complement, which every build already did in
practice. `crates/pocketpy-sys/build.rs` compiles PocketPy with `-fwrapv` so
that this is defined rather than an accident of the optimizer, and the
sanitizers stop reporting it. Python code that needs integers beyond 64 bits
does not get them; that is unchanged.

### Receiver types of C methods

PocketPy's C methods read their first argument's userdata without checking
its type, relying on attribute lookup to pass an instance. Calling them through
the class bypassed that: `dict.clear([1])` treated a list as a dict,
`dict.__new__(list)` or `random.Random.__new__(list)` built a list with another
type's payload, and a class could borrow another type's C magic method
(`__getitem__ = list.__getitem__`), which the interpreter's fast paths then
called directly. Kipferl's own `py_bind` methods, such as `bytearray`'s, had
the same assumption. `0012-native-receiver-types.patch` records the owning type
when `py_bindmethod`, `py_bindmagic`, `py_bindproperty` or `py_bind` (for a
first parameter named `self`, or `cls` of `__new__`) binds a C function to a
type, and every call path checks it: the first argument must be an instance of
that type, or for `__new__` a subclass of it. A mismatch raises `TypeError`, as
in CPython. Instances and subclasses behave as before.

### Pickle input

`pickle.loads` trusted its input completely: reads were not bounded by the
data, memo indices and stack depths were not checked, a raw value (`PKL_TVALUE`
and array2d cells) could carry any type and pointer, and an object record could
name a native type. `0013-pickle-loads-validation.patch` bounds every read and
checks every operand. Raw values are accepted only for plain data types (`int`,
`float`, `bool`, `None`, `NotImplemented`, `...`, the vector types and
`color32`), object records only for Python classes, and a value of the wrong
kind, a short buffer or a too-deep stack raises `ValueError("invalid pickle
data")`. Pickles written by `pickle.dumps` load unchanged. Loading still calls
the functions a pickle names, as in CPython, so it is only for trusted data.

### try, finally and with cleanup

PocketPy 2.2 rejects `finally` ("finally clause is not supported yet") and
`else` after `except`. `0014-try-else-finally.patch` compiles both. The `else`
clause runs after the try body completes without an exception, outside the
handlers, so its own exceptions skip them. A `finally` clause protects the whole
statement, `except` and `else` included: an exception that leaves the statement
runs the clause and is re-raised, unless the clause itself raises, returns,
breaks or continues. A `try` may have `finally` without `except`.

`break`, `continue` and `return` that leave a `finally`-protected block run its
clause, innermost first, before transferring control. The compiler locates the
clause's tokens before compiling the body and compiles the clause again at each
such exit, as well as once for normal completion and once for the exception
path, so no new opcode or runtime state is needed. A `return` value stays on the
stack while the clauses run. The same unwinding calls `__exit__` for every `with`
block that `break`, `continue` or `return` leaves; upstream only discarded the
manager, so `with open(...) as f: return f.read()` never closed the file.
A `return` that leaves no `finally` or `with` block compiles exactly as before.

Not covered: a generator that is never exhausted does not run its pending
`finally` clauses (PocketPy generators have no `close()`), and an exception
raised inside a `finally` clause does not record the original as `__context__`.

`0015-with-exit-arguments.patch` calls `__exit__` with the CPython arguments:
`(None, None, None)` on normal exit and `(type(exc), exc, None)` when an
exception leaves the block (PocketPy has no traceback objects). Upstream
called it with no arguments, so a context manager written for CPython,
`def __exit__(self, exc_type, exc, tb)`, raised `TypeError`, and Kipferl's
earlier `0003` exception path passed only the exception. `contextlib.suppress`
now receives the exception type it checks.

### Generator expressions and nested comprehensions

PocketPy 2.2 has no generator expressions, and its comprehensions accept a
single `for` clause with at most one `if`. `0016-generator-expressions.patch`
compiles `(<expr> for ...)`, and a generator expression that is a call's sole
argument (`sum(x for x in xs)`), the way CPython does: into a nested
generator function, `<genexpr>`, called with the iterator of the first
iterable. That iterable is evaluated, and `iter()` applied, in the enclosing
scope, so `(y for y in 5)` raises `TypeError` at once and a class body can use
its own names there; everything else runs lazily in the function's scope, so
its loop variables do not leak. List, set and dict comprehensions with more
than one `for` clause compile to a nested function the same way and return
the built container; single-clause comprehensions keep PocketPy's faster
inline code (and its leaking loop variable). Clauses may carry several `if`
conditions.

The compiler decides by scanning the bracket's tokens for a `for` at its own
level before the first comma. Like other nested functions in PocketPy, the
generator sees the enclosing function's locals as they were when it was
created. `yield` inside a generator expression and `:=` binding in the
enclosing scope are not supported.
