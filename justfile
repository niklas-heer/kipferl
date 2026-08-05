# Kipferl development commands
# Run `just` to see all available commands.

default:
    @just --list

# Format, lint, and test the Rust workspace
check:
    python3 scripts/generate_stubs.py --check
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy -p kipferl-runtime --no-default-features --all-targets -- -D warnings
    cargo test --workspace
    cargo test -p kipferl-runtime --no-default-features

# Build the public CLI, runtime, and loader in release mode
build:
    cargo build --release --workspace

# Build the workspace with debug information
build-debug:
    cargo build --workspace

# Run the full CPython compatibility report against the Rust runtime
compat: build
    python3 tests/compat_runner.py --runtime target/release/pocketpy-kipferl --report

# Run all local release-cutover checks
test: check compat vision

# Run only the CLI end-to-end integration tests
test-e2e:
    cargo test -p kipferl-cli --test cli

# Run a Python script through the public CLI
run script: build
    target/release/kipferl run {{ script }}

# Build a universal binary from a Python script
build-app script output="app": build
    target/release/kipferl build {{ script }} -o {{ output }} --mode universal

# Build only the PocketPy runtime
build-runtime:
    cargo build --release -p kipferl-runtime

# Run code directly through the Rust-hosted PocketPy runtime
runtime code:
    cargo run -p kipferl-runtime --bin pocketpy-kipferl -- -c {{ quote(code) }}

# Run the CLI without a release build
cli *args:
    cargo run -p kipferl-cli --bin kipferl -- {{ args }}

# Run the example demo
demo: build
    target/release/kipferl run examples/demo.py

# Run the full feature demo
demo-full: build
    target/release/kipferl run examples/simple_cli.py

# Regenerate the branded README and website demo recording
demo-gif: build
    @command -v vhs >/dev/null || (echo "Error: vhs not found. Install it with Homebrew." && exit 1)
    VHS_NO_SANDBOX=1 vhs demo.tape
    cp demo.gif website/public/demo.gif
    @echo "Updated demo.gif and website/public/demo.gif"

# Regenerate the checked-in PocketPy FFI declarations
bindings:
    ./scripts/generate-rust-bindings.sh

# Regenerate the Rust manifest that embeds every canonical root stub
stubs:
    python3 scripts/generate_stubs.py

# Verify stub syntax and fail if the generated Rust manifest drifted
stubs-check:
    python3 scripts/generate_stubs.py --check

# Verify the PocketPy vendor patches against upstream
check-pocketpy:
    python3 scripts/verify-pocketpy-patches.py --check-upstream

# Run the broader vision suite
vision: build
    python3 tests/vision/run_vision.py --runtime target/release/pocketpy-kipferl

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
    target/release/kipferl run scripts/release.py

# Show the public binary sizes
size: build
    @ls -lh target/release/kipferl target/release/pocketpy-kipferl target/release/kipferl-loader

# Install the release CLI locally
install: build
    @mkdir -p ~/.local/bin
    @ln -sf "$(pwd)/target/release/kipferl" ~/.local/bin/kipferl
    @echo "Installed kipferl to ~/.local/bin/kipferl"

# Remove the local CLI symlink
uninstall:
    @rm -f ~/.local/bin/kipferl
    @echo "Removed kipferl from ~/.local/bin"

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
