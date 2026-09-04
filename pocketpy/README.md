# PocketPy Vendor

This directory contains the patched PocketPy C runtime embedded by Kipferl's
Rust host.

## Contents

- `vendor/pocketpy.c` and `vendor/pocketpy.h`: the vendored interpreter source.
- `POCKETPY_VERSION`: the tracked upstream revision.
- `patches/`: Kipferl's ordered patch manifest and patch files.

Cargo compiles the interpreter through `crates/pocketpy-sys/build.rs`. The
production host, module bindings, CLI, and universal loader are Rust. Use the
shared [mise development setup](../docs/development.md); there is no separate
PocketPy build command.

## Updating PocketPy

From the repository root, check whether a newer upstream release exists:

```console
./scripts/check-pocketpy-version.sh
```

A newer release makes this check exit with status 1 and print update guidance.
When intentionally updating the vendor sources:

```console
./scripts/check-pocketpy-version.sh --update
```

The updater downloads the new sources, updates `POCKETPY_VERSION`, and applies
the tracked patches automatically. Review the resulting vendor and version
diff. If upstream changes conflict with a patch, resolve and review the patch
and its manifest entry before verification. The updater is not a transactional
operation: a patch failure can leave partially updated files that need repair.
Do not blindly apply the patchset a second time after a successful update.

Verify both the local patchset and reconstruction from the recorded upstream
revision, then run the complete local checks:

```console
mise run check-pocketpy
mise run lint-audit
mise run test
```

`check-pocketpy` needs network access to verify the upstream source. `test`
includes full/core Rust suites, doctests, compatibility against the pinned
CPython, vision scenarios, recipe packaging, and the website. These checks are
complementary: the compatibility task executes the fresh raw runtime, while
packaged applications use the CLI's embedded runtime assets. Refreshing those
release assets and validating each target remains part of release preparation.
CI additionally exercises PocketPy callbacks and lifecycle under Linux address
and undefined-behavior sanitizers.

Regenerate the checked-in Rust FFI declarations when the public C API changes:

```console
mise run bindings
```

This optional maintenance task requires `bindgen-cli` and libclang. Review the
generated declaration changes and rerun the checks after regeneration; neither
tool is needed for ordinary builds using the checked-in bindings.
