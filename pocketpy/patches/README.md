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
