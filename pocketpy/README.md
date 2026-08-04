# PocketPy Vendor

This directory contains the patched PocketPy C runtime embedded by μcharm’s
Rust host.

## Contents

- `vendor/pocketpy.c` and `vendor/pocketpy.h`: the vendored interpreter source.
- `POCKETPY_VERSION`: the tracked upstream revision.
- `patches/`: μcharm’s ordered patch manifest and patch files.

Cargo compiles the interpreter through `crates/pocketpy-sys/build.rs`. The
production host, module bindings, CLI, and universal loader are Rust; there is
no separate PocketPy build command.

## Updating PocketPy

Check or update the vendored revision with:

```bash
./scripts/check-pocketpy-version.sh
./scripts/check-pocketpy-version.sh --update
```

After an update, reapply and verify the tracked patchset:

```bash
./scripts/apply-pocketpy-patches.sh
python3 scripts/verify-pocketpy-patches.py --check-upstream
cargo test --workspace
```

Regenerate the checked-in Rust FFI declarations when the public C API changes:

```bash
just bindings
```
