#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

RUST_LOG=error bindgen "$project_root/pocketpy/vendor/pocketpy.h" \
  --allowlist-function 'py_(initialize|finalize|exec|printexc|sys_setargv|newmodule|bindfunc|retval|newint)' \
  --allowlist-type 'py_CompileMode' \
  --use-core \
  --no-layout-tests \
  --output "$project_root/crates/pocketpy-sys/src/bindings.rs" \
  -- \
  -std=c11 \
  -I"$project_root/pocketpy/vendor"
