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
Rust CLI + Kipferl v1 universal loader
```

The Cargo workspace contains:

- `crates/pocketpy-sys`: the narrow generated PocketPy C FFI.
- `crates/kipferl-runtime`: the PocketPy host and native Python modules.
- `crates/kipferl-format`: the frozen Kipferl v1 universal trailer format.
  Preserve its original wire bytes for existing application compatibility.
- `crates/kipferl-loader`: extraction, cache, and execution of universal apps.
- `crates/kipferl-cli`: `new`, `init`, `run`, `dev`, `build`, `test`, and completions.

The production repository no longer contains the archived Zig implementation.
Use the final Zig tag `v0.5.0` and the migration history when archaeology is
needed; do not reintroduce Zig build paths into the working tree.

## Commands

```bash
mise run setup                  # Prepare pinned tools, Rust workspace, and website
mise run check                  # Stub drift, rustfmt, strict Clippy, and tests
mise run lint-audit             # Zero findings plus reviewed exception inventory
mise run compat                 # Compatibility against the pinned Python baseline
mise run demo                   # Run the example through the public Rust CLI
mise run build-app app.py app   # Build a standalone universal executable
```

The direct equivalents are:

```bash
cargo fmt --all --check
cargo check --locked --workspace --all-targets --all-features
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo build --release --workspace
python3 tests/compat_runner.py --runtime target/release/pocketpy-kipferl --report
```

Run `mise install --locked rust python node bun cargo:bacon aqua:nextest-rs/nextest/cargo-nextest watchexec` before setup on a fresh clone.
The pinned toolchain is defined by `rust-toolchain.toml` and mirrored in
`mise.toml`; `mise run doctor` rejects drift. Production code must
remain compatible with that stable toolchain.

See [docs/development.md](docs/development.md) for the complete workflow and
[docs/rust-review.md](docs/rust-review.md) for review decisions and evidence.
Current source additions are unreleased until the tag workflow rebuilds and
publishes matching embedded assets; keep user documentation explicit about this.

## Implementation policy

- Implement native functionality in Rust. Prefer the standard library and
  feature-minimal dependencies; justify every new crate against maintenance,
  supply-chain, startup, and binary-size costs.
- Preserve the Python API, exact output bytes, errors, target names, release
  artifact names, and Kipferl v1 universal format unless a product change is approved.
- Keep unsafe code inside the smallest practical PocketPy FFI boundary. A
  borrowed `Value` must not survive an allocating VM call unless it is rooted
  in a VM-owned register or container.
- Use RAII for VM, terminal, file, process, and userdata cleanup. Validate
  lengths and integer conversions before crossing the C boundary.
- Bound data read from files, subprocesses, networks, archives, and databases.
  Existing limits are compatibility and safety contracts, not suggestions.
- Add permanent Rust unit or integration coverage for every new behavior and
  run the relevant CPython fixture through the compatibility runner.
- Workspace Clippy denies pedantic, nursery, and every restriction in
  `docs/rust-review.md`, alongside undocumented unsafe blocks. Fix findings or
  use a narrowly scoped `#[expect(..., reason = "concrete invariant")]`.
  Never suppress a Clippy group. The audit inventories all exceptions and
  rejects missing reasons; generated FFI declarations preserve upstream names.
- Use nextest (`mise run test-rust` and `test-core`) for process-isolated tests;
  keep Cargo doctests separate. `mise run lint-audit` records every remaining
  restriction diagnostic and fails on outstanding findings; run it with `check`.
- `mise run bacon` provides compiler/Clippy/test feedback, `mise run watch`
  serializes checks/tests/builds, and `mise run bench` measures loader regressions.
- Rust Analyzer and `rust-src` are installed with the pinned toolchain. Use
  `mise run test-doc` to verify public API examples independently.

## PocketPy updates

PocketPy is vendored in `pocketpy/vendor/`. The tracked patchset lives under
`pocketpy/patches/` and is verified with:

```bash
python3 scripts/verify-pocketpy-patches.py --check-upstream
```

Regenerate FFI declarations with `mise run bindings`; never hand-edit generated
bindings without updating their source or generator.

## Python stubs

The hand-authored `stubs/*.pyi` files are the canonical editor API. The CLI
embeds every file through `crates/kipferl-cli/src/generated_stubs.rs`; do not add
a second handwritten list. Run `mise run stubs` after adding or removing a stub and
commit the generated manifest. `mise run stubs-check` and CI validate stub syntax
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
