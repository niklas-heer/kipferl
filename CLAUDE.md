# Kipferl Contributor Guide

Kipferl runs Python-style CLI applications on PocketPy and ships them as small,
standalone executables. The production host, CLI, universal loader, and native
module surface are implemented in stable Rust.

## Architecture

```
Python application
       │
       ▼
PocketPy VM (vendored C, patched and compiled by Cargo)
       │
       ▼
Rust native modules and TUI primitives
       │
       ▼
Rust CLI + MCHARM01 universal loader
```

The Cargo workspace contains:

- `crates/pocketpy-sys`: the narrow generated PocketPy C FFI.
- `crates/kipferl-runtime`: the PocketPy host and native Python modules.
- `crates/kipferl-format`: the frozen `MCHARM01` trailer format.
- `crates/kipferl-loader`: extraction, cache, and execution of universal apps.
- `crates/kipferl-cli`: `new`, `init`, `run`, `build`, and `test`.

The production repository no longer contains the archived Zig implementation.
Use the final Zig tag `v0.5.0` and the migration history when archaeology is
needed; do not reintroduce Zig build paths into the working tree.

## Commands

```bash
just setup                  # Check Cargo and build the release workspace
just check                  # Stub drift, rustfmt, strict Clippy, and tests
just compat                 # Full 1,669-check compatibility report
just demo                   # Run the example through the public Rust CLI
just build-app app.py app   # Build a standalone universal executable
```

The direct equivalents are:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
python3 tests/compat_runner.py --runtime target/release/pocketpy-kipferl --report
```

The pinned toolchain is defined by `rust-toolchain.toml`. Production code must
remain compatible with that stable toolchain.

## Implementation policy

- Implement native functionality in Rust. Prefer the standard library and
  feature-minimal dependencies; justify every new crate against maintenance,
  supply-chain, startup, and binary-size costs.
- Preserve the Python API, exact output bytes, errors, target names, release
  artifact names, and `MCHARM01` format unless a product change is approved.
- Keep unsafe code inside the smallest practical PocketPy FFI boundary. A
  borrowed `PyValue` must not survive an allocating VM call unless it is rooted
  in a VM-owned register or container.
- Use RAII for VM, terminal, file, process, and userdata cleanup. Validate
  lengths and integer conversions before crossing the C boundary.
- Bound data read from files, subprocesses, networks, archives, and databases.
  Existing limits are compatibility and safety contracts, not suggestions.
- Add permanent Rust unit or integration coverage for every new behavior and
  run the relevant CPython fixture through the compatibility runner.

## PocketPy updates

PocketPy is vendored in `pocketpy/vendor/`. The tracked patchset lives under
`pocketpy/patches/` and is verified with:

```bash
python3 scripts/verify-pocketpy-patches.py --check-upstream
```

Regenerate FFI declarations with `just bindings`; never hand-edit generated
bindings without updating their source or generator.

## Python stubs

The hand-authored `stubs/*.pyi` files are the canonical editor API. The CLI
embeds every file through `crates/kipferl-cli/src/generated_stubs.rs`; do not add
a second handwritten list. Run `just stubs` after adding or removing a stub and
commit the generated manifest. `just stubs-check` and CI validate stub syntax
and reject manifest drift.

## Release assets

The public executables are `kipferl`, `pocketpy-kipferl`, and `kipferl-loader`.
The CLI embeds matching Rust runtime and loader assets from
`crates/kipferl-cli/assets/` for macOS ARM64, macOS x86_64, Linux ARM64 musl,
and Linux x86_64 musl. CI publishes fresh component assets for review; the
tagged release workflow rebuilds all four components, injects them into each
CLI build, creates checksums, and updates Homebrew only for stable tags.

Do not restore Zig setup or `cargo-zigbuild` to the normal CI or release path.
See `RUST_MIGRATION.md` and issue #13 for the recorded decisions and gates.
