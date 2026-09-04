# Development workflow

Run these commands from the repository root. `mise.toml` and `mise.lock` are the
shared setup for Rust, Python, Node, Bun, and the diagnostic tools. The Rust pin
also appears in `rust-toolchain.toml`; `mise run doctor` checks that they agree.
Use mise 2026.9.1 or newer and a working native C compiler: Xcode Command Line
Tools on macOS, or `build-essential` on Debian/Ubuntu. PocketPy and SQLite still
compile C code.

```console
mise trust
mise install --locked rust python node bun cargo:bacon aqua:nextest-rs/nextest/cargo-nextest watchexec
mise run setup
mise run doctor
mise tasks ls
```

Trust the configuration after reviewing it. `setup` verifies the tools and C
compiler, fetches the locked Cargo dependencies, installs the website's locked
Bun dependencies, and builds the release workspace. Bacon is compiled from its
pinned upstream source and lockfile, so its first installation takes longer than
a prebuilt tool. Nix, devenv, Make, just, and pnpm are not required.

Use `mise exec -- <command>` for an individual command that needs the pinned
tools. For example, `mise exec -- python3 --version` verifies which CPython will
supply the compatibility baseline. Shell activation is optional for these
explicit commands.

## Choose the feedback loop

| Command | What it checks or runs |
| --- | --- |
| `mise run bacon` | Interactive compiler feedback across the workspace and vendored C inputs |
| `mise run watch` | Queues a debounced check, full Rust test run, and debug build after source changes |
| `mise run check` | Tool pins, generated stubs, Python helper tests, Rust formatting, all-target compilation, strict full/core Clippy, full/core nextest suites, and doctests |
| `mise run lint-audit` | Every review lint in full/core profiles, plus the source locations and reasons for explicit exceptions |
| `mise run test` | `check`, release build, CPython compatibility, vision scenarios, recipes, and website checks |
| `mise run bench` | Statistical loader benchmarks; run separately from correctness checks |
| `mise run website-dev` | Website development server |
| `mise run seek` | Optional pinned crate, feature, and MSRV explorer, installed on demand |

Inside Bacon, `c` selects full-workspace Clippy, `t` full tests, `r` core Clippy,
`Shift-R` core tests, and `d` doctests. Compiler locations are exported to
`target/bacon-locations` for editor integration. The watch loop runs tests and
builds, but does not execute an example application. It does not replace the
full/core Clippy gates in `check`.

For a focused test, retain the relevant feature profile:

```console
mise run test-rust
mise run test-core
mise run test-doc
mise exec -- cargo nextest run --locked -p kipferl-runtime --test runtime_safety
```

Nextest runs tests in separate processes and never retries a failure. It keeps
running after failures and shows captured failure output immediately and at the
end. Tests are marked slow after 30 seconds and terminated after four such
periods, with a 10-second grace period. Descendants retaining a test's output
pipes for more than one second make the test fail. This detects leaked
processes; it does not detect all memory leaks. Rust doctests run separately
through Cargo because nextest does not execute them.

## Read the diagnostics

`mise run test-ci` runs the full and core nextest profiles sequentially and then
doctests. The reports are `target/nextest/ci/junit.xml` and
`target/nextest/ci-core/junit.xml`. CI retains these as the `rust-test-reports`
artifact even when a preceding check fails; a failure before a suite starts can
leave its report absent. Set `RUST_BACKTRACE=1` when investigating a Rust panic.

`mise run lint-audit` writes `target/lint-audit/report.md`, `diagnostics.json`,
`exceptions.json`, and full/core JSON compiler output. It fails on outstanding
findings, compiler errors, or exception-policy violations. An empty findings
list does not mean the code has no explicit lint exceptions: the exception
inventory records each reviewed invariant and its source location. CI uploads
the directory as `clippy-restriction-audit`. See the
[Rust review](rust-review.md) for the policy and fixes it uncovered.

## Know which runtime you tested

`mise run build` builds the workspace from source. Compatibility and vision
tasks explicitly execute `target/release/pocketpy-kipferl`, so they exercise the
fresh runtime. The CLI instead embeds the host's checked-in full/core runtime
assets when it is compiled. Rebuilding the CLI does not replace those assets
with the freshly compiled runtime. `kipferl run` and packaged applications can
therefore test a different runtime revision from the raw-runtime suites.

Use a fresh raw-runtime smoke test when changing native modules:

```console
mise run build-runtime
target/release/pocketpy-kipferl -c 'import math; print(math.sqrt(9))'
mise run compat
mise run vision
mise run recipes
```

Recipes check both raw-runtime execution and isolated standalone packaging.
The release pipeline must refresh embedded assets and validate the produced
packages before claiming that source changes ship in them. Native foreign-target
execution, Linux static-link checks, and the Linux AddressSanitizer/
UndefinedBehaviorSanitizer job require their CI environments; a local macOS pass
does not establish those results. Compatibility totals can vary with the host
CPython baseline, so record its version alongside counts and skipped checks.

## Optional maintenance tools

`mise run bindings` requires separately installed `bindgen-cli` and libclang
when changing PocketPy's public C API. Demo recording requires VHS. Neither is
needed for routine development. See [PocketPy maintenance](../pocketpy/README.md)
for patch verification and [benchmarking](../benchmarks/README.md) for repeatable
performance measurements. Git hooks are not installed automatically; run the
checks before opening a pull request.

## Maintain editor stubs

The 27 canonical `stubs/*.pyi` files describe Kipferl's actual module surface.
Check arguments, return types, and exported names against the runtime, including
PocketPy's embedded Python definitions. Do not copy a complete CPython API into a
stub when the runtime implements only a subset. `mise run stubs-check` validates
syntax, registration, and the generated manifest; CLI tests verify exported bytes.
These checks do not establish compatibility with every external type checker.
