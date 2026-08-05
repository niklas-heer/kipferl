# Changelog

All notable user-facing changes are documented here.

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
- Preserves the `MCHARM01` standalone application format.
- Accepts legacy `from ucharm ...` source and environment variables, publishes
  temporary `ucharm-*` assets, and installs a deprecated `ucharm` command alias
  for the 0.6 transition.

[0.6.0]: https://github.com/niklas-heer/kipferl/compare/v0.5.0...v0.6.0
