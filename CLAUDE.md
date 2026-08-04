# μcharm Contributor Guide

μcharm runs Python-style CLI applications on PocketPy and ships them as small,
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
- `crates/ucharm-runtime`: the PocketPy host and native Python modules.
- `crates/ucharm-format`: the frozen `MCHARM01` trailer format.
- `crates/ucharm-loader`: extraction, cache, and execution of universal apps.
- `crates/ucharm-cli`: `new`, `init`, `run`, `build`, and `test`.

The historical `cli/`, `loader/`, `pocketpy/*.zig`, and `runtime/**/*.zig`
sources are retained temporarily for migration archaeology. They are not part
of the normal build, CI, packaging, or release path. Do not add production
features to them.

## Commands

```bash
just setup                  # Check Cargo and build the release workspace
just check                  # rustfmt, strict Clippy, and workspace tests
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
python3 tests/compat_runner.py --runtime target/release/pocketpy-ucharm --report
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

## Release assets

The public executables are `ucharm`, `pocketpy-ucharm`, and `ucharm-loader`.
The CLI embeds matching Rust runtime and loader assets for macOS ARM64, macOS
x86_64, Linux ARM64 musl, and Linux x86_64 musl. CI publishes fresh component
assets for review; the tagged release workflow rebuilds all four components,
injects them into each CLI build, creates checksums, and updates Homebrew.

Do not restore Zig setup or `cargo-zigbuild` to the normal CI or release path.
See `RUST_MIGRATION.md` and issue #13 for the recorded decisions and gates.
