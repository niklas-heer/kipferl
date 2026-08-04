# ucharm development commands
# Run `just` to see all available commands

# Default: show help
default:
    @just --list

# Check the Rust migration workspace
rust-check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Build the Rust-hosted PocketPy runtime
rust-build:
    cargo build --release -p ucharm-runtime

# Run code through the Rust-hosted PocketPy runtime
rust-run code:
    cargo run -p ucharm-runtime --bin pocketpy-ucharm-rs -- -c {{ quote(code) }}

# Regenerate the checked-in PocketPy FFI declarations
rust-bindings:
    ./scripts/generate-rust-bindings.sh

# Compare the host Zig runtime and Rust migration spine
rust-baseline runs="50":
    python3 benchmarks/migration_baseline.py --runs {{ runs }}

# Build the CLI in release mode
build:
    cp VERSION cli/src/VERSION
    cd cli && zig build -Doptimize=ReleaseSmall

# Build the CLI in debug mode
build-debug:
    cp VERSION cli/src/VERSION
    cd cli && zig build

# Run all tests
test: test-unit test-e2e

# Run unit tests
test-unit:
    cd cli && zig build test

# Run end-to-end tests
test-e2e:
    cd cli && ./test_e2e.sh

# Run a Python script with ucharm
run script:
    ./cli/zig-out/bin/ucharm run {{ script }}

# Build a universal binary from a Python script
build-app script output="app":
    ./cli/zig-out/bin/ucharm build {{ script }} -o {{ output }} --mode universal

# Build PocketPy runtime with native modules
build-pocketpy:
    cd pocketpy && zig build -Doptimize=ReleaseSmall

# Run the demo
demo:
    ./cli/zig-out/bin/ucharm run examples/demo.py

# Run the full feature demo
demo-full:
    ./cli/zig-out/bin/ucharm run examples/simple_cli.py

# Clean build artifacts
clean:
    rm -rf cli/zig-out cli/.zig-cache
    rm -rf pocketpy/zig-out pocketpy/.zig-cache

# Format Zig code
fmt:
    cd cli && zig fmt src/

# Check Zig code formatting
fmt-check:
    cd cli && zig fmt --check src/

# Create a new release (interactive) - built with ucharm!

# Interactive release
release:
    ./cli/zig-out/bin/ucharm run scripts/release.py

# Show binary size breakdown
size:
    @echo "CLI binary:"
    @ls -lh cli/zig-out/bin/ucharm 2>/dev/null || echo "  Not built yet. Run: just build"
    @echo ""
    @echo "PocketPy runtime:"
    @ls -lh pocketpy/zig-out/bin/pocketpy-ucharm 2>/dev/null || echo "  Not built yet. Run: just build-pocketpy"

# Run benchmarks
bench:
    @echo "Running native module benchmarks..."
    cd cli && zig build test 2>&1 | grep -E "(benchmark|ns|ms)"

# Install locally (symlink to ~/.local/bin)
install:
    @mkdir -p ~/.local/bin
    @ln -sf $(pwd)/cli/zig-out/bin/ucharm ~/.local/bin/ucharm
    @echo "Installed ucharm to ~/.local/bin/ucharm"
    @echo "Make sure ~/.local/bin is in your PATH"

# Uninstall local installation
uninstall:
    @rm -f ~/.local/bin/ucharm
    @echo "Removed ucharm from ~/.local/bin"

# Setup development environment
setup:
    @echo "Checking dependencies..."
    @which zig > /dev/null || (echo "Error: zig not found. Install from https://ziglang.org" && exit 1)
    @echo "Building PocketPy runtime..."
    @just build-pocketpy
    @echo "Building CLI..."
    @just build
    @echo ""
    @echo "Setup complete! Try: just demo"

# Watch for changes and rebuild (requires watchexec)
watch:
    watchexec -w cli/src -e zig -- just build

# Generate Homebrew formula (after release)
homebrew version:
    ./scripts/update-homebrew.sh {{ version }}
