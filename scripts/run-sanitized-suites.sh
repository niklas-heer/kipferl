#!/bin/bash
# Build the runtime with AddressSanitizer and UndefinedBehaviorSanitizer, then
# run the compatibility suite, the Vision scenarios, the documentation recipes
# and the examples against it. Any sanitizer report fails the run.
#
# Linux uses GCC's libasan/libubsan (as the FFI sanitizer CI job does); macOS
# uses the Xcode clang runtime. Instrumented output goes to target/sanitize.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export CARGO_TARGET_DIR="$ROOT/target/sanitize"
export CARGO_PROFILE_RELEASE_LTO=false
export CARGO_PROFILE_RELEASE_STRIP=none
export CFLAGS="-fsanitize=address,undefined,float-cast-overflow -fno-omit-frame-pointer -g"
host="$(rustc -vV | sed -n 's/^host: //p')"
case "$(uname -s)" in
  Linux)
    export RUSTFLAGS="-C link-arg=-fsanitize=address -C link-arg=-fsanitize=undefined -C link-arg=-lasan -C link-arg=-lubsan"
    preload="LD_PRELOAD=$(gcc -print-file-name=libasan.so)"
    leaks="detect_leaks=1"
    ;;
  Darwin)
    runtime_dir="$(clang -print-resource-dir)/lib/darwin"
    export RUSTFLAGS="-C link-arg=$runtime_dir/libclang_rt.asan_osx_dynamic.dylib -C link-arg=-Wl,-rpath,$runtime_dir"
    preload=""
    # LeakSanitizer is unavailable on macOS.
    leaks="detect_leaks=0"
    ;;
  *)
    echo "error: sanitized suites support Linux and macOS" >&2
    exit 2
    ;;
esac

# An explicit target keeps instrumentation out of build scripts and proc macros.
cargo build --locked --release --target "$host" -p kipferl-runtime --bin pocketpy-kipferl
binary="$CARGO_TARGET_DIR/$host/release/pocketpy-kipferl"

reports="$CARGO_TARGET_DIR/reports"
rm -rf "$reports"
mkdir -p "$reports"
# Only the runtime is instrumented: the wrapper confines the preload and the
# sanitizer settings to it, so the CPython baseline and harnesses run normally.
wrapper="$CARGO_TARGET_DIR/pocketpy-kipferl-sanitized"
cat >"$wrapper" <<EOF
#!/bin/sh
export ASAN_OPTIONS="$leaks:halt_on_error=1:strict_string_checks=1:detect_stack_use_after_return=0:log_path=$reports/asan"
export LSAN_OPTIONS="exitcode=23"
export UBSAN_OPTIONS="halt_on_error=1:print_stacktrace=1:log_path=$reports/ubsan"
exec env $preload "$binary" "\$@"
EOF
chmod +x "$wrapper"

status=0
if ! python3 tests/compat_runner.py --runtime "$wrapper"; then
  status=1
  # The runner reports counts only; show how each failing test file ended.
  for test_file in tests/cpython/test_*.py; do
    code=0
    output="$(cd tests/cpython && timeout 120 "$wrapper" "$(basename "$test_file")" 2>&1)" || code=$?
    if [ "$code" -ne 0 ]; then
      echo "== $test_file exited with $code" >&2
      printf '%s\n' "$output" | tail -n 40 >&2
    fi
  done
fi
python3 tests/vision/run_vision.py --runtime "$wrapper" || status=1
python3 scripts/check_recipes.py --runtime "$wrapper" || status=1
# Examples run without a terminal; interactive ones cancel at end of input.
for example in examples/*.py; do
  for arguments in "" "--help"; do
    code=0
    (cd examples && timeout 60 "$wrapper" "$(basename "$example")" $arguments </dev/null >/dev/null 2>&1) || code=$?
    if [ "$code" -eq 124 ]; then
      echo "error: $example $arguments did not finish" >&2
      status=1
    fi
  done
done

if compgen -G "$reports/*" >/dev/null; then
  echo "error: sanitizer reports:" >&2
  cat "$reports"/* >&2
  status=1
fi
exit "$status"
