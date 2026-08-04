# μcharm development commands
# Run `just` to see all available commands.

default:
    @just --list

# Format, lint, and test the Rust workspace
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Build the public CLI, runtime, and loader in release mode
build:
    cargo build --release --workspace

# Build the workspace with debug information
build-debug:
    cargo build --workspace

# Run the full CPython compatibility report against the Rust runtime
compat: build
    python3 tests/compat_runner.py --runtime target/release/pocketpy-ucharm --report

# Run all local release-cutover checks
test: check compat

# Run only the CLI end-to-end integration tests
test-e2e:
    cargo test -p ucharm-cli --test cli

# Run a Python script through the public CLI
run script: build
    target/release/ucharm run {{ script }}

# Build a universal binary from a Python script
build-app script output="app": build
    target/release/ucharm build {{ script }} -o {{ output }} --mode universal

# Build only the PocketPy runtime
build-runtime:
    cargo build --release -p ucharm-runtime

# Run code directly through the Rust-hosted PocketPy runtime
runtime code:
    cargo run -p ucharm-runtime --bin pocketpy-ucharm -- -c {{ quote(code) }}

# Run the CLI without a release build
cli *args:
    cargo run -p ucharm-cli --bin ucharm -- {{ args }}

# Run the example demo
demo: build
    target/release/ucharm run examples/demo.py

# Run the full feature demo
demo-full: build
    target/release/ucharm run examples/simple_cli.py

# Regenerate the checked-in PocketPy FFI declarations
bindings:
    ./scripts/generate-rust-bindings.sh

# Verify the PocketPy vendor patches against upstream
check-pocketpy:
    python3 scripts/verify-pocketpy-patches.py --check-upstream

# Run the broader vision suite
vision: build
    python3 tests/vision/run_vision.py --runtime target/release/pocketpy-ucharm

# Remove Cargo build artifacts
clean:
    cargo clean

# Format Rust code
fmt:
    cargo fmt --all

# Check Rust formatting
fmt-check:
    cargo fmt --all --check

# Create a new release interactively
release: build
    target/release/ucharm run scripts/release.py

# Show the public binary sizes
size: build
    @ls -lh target/release/ucharm target/release/pocketpy-ucharm target/release/ucharm-loader

# Install the release CLI locally
install: build
    @mkdir -p ~/.local/bin
    @ln -sf "$(pwd)/target/release/ucharm" ~/.local/bin/ucharm
    @echo "Installed ucharm to ~/.local/bin/ucharm"

# Remove the local CLI symlink
uninstall:
    @rm -f ~/.local/bin/ucharm
    @echo "Removed ucharm from ~/.local/bin"

# Check the Rust toolchain and build the project
setup:
    @command -v cargo >/dev/null || (echo "Error: cargo not found. Install Rust with rustup." && exit 1)
    @just build
    @echo "Setup complete! Try: just demo"

# Rebuild when Rust sources change (requires watchexec)
watch:
    watchexec -w crates -e rs,toml -- just build

# Generate the Homebrew formula after a release
homebrew version:
    ./scripts/update-homebrew.sh {{ version }}
