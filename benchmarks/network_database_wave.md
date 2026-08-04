# Rust Network and Database Wave

Recorded on 2026-08-04 on macOS 26.5.1, Apple Silicon. Startup results use 400
measured launches after 5 warmups via `benchmarks/migration_baseline.py`.

## Accepted implementation

| Artifact | ARM64 bytes | x86_64 bytes | ARM64 median | ARM64 p95 |
| --- | ---: | ---: | ---: | ---: |
| Rust runtime with `http.client` + `sqlite3` | 1,840,336 | 1,971,304 | 6.986 ms | 8.024 ms |
| Legacy Zig runtime baseline | 2,313,264 | — | 4.332 ms | 5.007 ms |

The four-target release build recorded these stripped Rust runtime sizes:

| Target | Bytes | Enforced ceiling |
| --- | ---: | ---: |
| `aarch64-apple-darwin` | 1,840,336 | 2,000,000 |
| `x86_64-apple-darwin` | 1,971,304 | 2,000,000 |
| `aarch64-unknown-linux-musl` | 2,378,848 | 2,500,000 |
| `x86_64-unknown-linux-musl` | 2,449,232 | 2,500,000 |

The original Linux ceiling was an explicit cutover exception for the fully
static musl artifact with bundled SQLite. The post-cutover optimization review
relaxes the aspirational 2 MB target: 2.5 MB is the practical runtime goal and
3 MB is the Linux regression ceiling. This keeps performance and the standalone
deployment model ahead of arbitrary byte shaving; the history remains in
[issue #41](https://github.com/ucharmdev/ucharm/issues/41).

- `http.client` uses `std::net` and adds no runtime dependency. It supports the
  compatibility API, plain HTTP/1.1, bounded 8 MiB responses, content length,
  chunked transfer decoding, case-insensitive header lookup, request bodies,
  headers, and timeouts. HTTPS/TLS remains explicit future work.
- `sqlite3` uses [`rusqlite`](https://github.com/rusqlite/rusqlite) 0.40.1 with
  vendored SQLite 3.53.2 statically linked
  into the runtime. Default crate features are disabled and CLI-irrelevant
  SQLite extensions and legacy APIs are compiled out through
  `LIBSQLITE3_FLAGS` in `.cargo/config.toml`.
- The result is one deployable executable with no system SQLite dependency.
  PocketPy and SQLite are still vendored C sources; eliminating all C/libc
  boundaries remains a longer-term architectural goal, not a claim of this
  release.

## Rejected pure-Rust spike

[`Turso`](https://github.com/tursodatabase/turso) 0.6.1 was tested with default
features disabled. It passed the focused
SQLite compatibility and file-backed join scenarios, but it added 211 packages
to `Cargo.lock` and produced a 5,420,336-byte optimized ARM64 runtime. That is
2.7 times the 2 MB runtime gate and materially increased build and process
startup cost. Turso also documents the engine as beta, so μcharm rejected it
for this cutover despite the attractive pure-Rust implementation.

Reconsider Turso only after re-running the same compatibility, dependency,
binary-size, startup, persistence, and four-target tests. Do not weaken the
artifact gate merely to replace a statically linked C component.

## Verification

- Full compatibility: 1,668/1,668 available checks (100%).
- Targeted modules: 51 fully compatible, 0 partial, 1 unavailable host baseline.
- HTTP: real loopback request/response plus content-length and chunked parser
  tests.
- SQLite: in-memory and file-backed databases, joins, positional binding,
  integers, floats, text, blobs, nulls, `fetchone`, `fetchall`, and close paths.
- Release builds and smoke tests: native ARM64 and x86_64 macOS locally, plus
  ARM64 and x86_64 static Linux in CI. macOS stays below the 2.5 MB practical
  runtime target; Linux stays below the 3 MB regression ceiling.
