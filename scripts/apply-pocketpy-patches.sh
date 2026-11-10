#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PATCH_DIR="$PROJECT_ROOT/pocketpy/patches"

if ! command -v git >/dev/null 2>&1; then
  echo "error: git is required (used for git apply)" >&2
  exit 2
fi

shopt -s nullglob
patches=("$PATCH_DIR"/*.patch)
if [ ${#patches[@]} -eq 0 ]; then
  echo "error: no patch files found in $PATCH_DIR" >&2
  exit 2
fi

for p in "${patches[@]}"; do
  if git apply --check "$p" >/dev/null 2>&1; then
    echo "apply: $(basename "$p")"
    git apply "$p"
  elif git apply --reverse --check "$p" >/dev/null 2>&1; then
    echo "skip:  $(basename "$p") (already applied)"
  else
    echo "error: cannot apply patch cleanly: $p" >&2
    echo "hint: update PocketPy may require patch refresh" >&2
    exit 1
  fi
done

python3 "$PROJECT_ROOT/scripts/verify-pocketpy-patches.py"
