#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

RUST_LOG=error bindgen "$project_root/pocketpy/vendor/pocketpy.h" \
  --allowlist-function 'py_(initialize|finalize|exec|printexc|sys_setargv|newmodule|getmodule|bind|bindfunc|retval|newint|newbool|newnone|newstrn|newtuple|tuple_len|tuple_getitem|tuple_setitem|newlist|list_len|list_getitem|list_append|newdict|dict_getitem|dict_setitem|dict_apply|name|getdict|setdict|pushtmp|pop|tpobject|isidentical|istype|toint|tofloat|tostrn|tobool|exception)' \
  --allowlist-type 'py_(CompileMode|PredefinedType|TValue|Name)' \
  --use-core \
  --no-layout-tests \
  --output "$project_root/crates/pocketpy-sys/src/bindings.rs" \
  -- \
  -std=c11 \
  -DPK_IS_PUBLIC_INCLUDE \
  -I"$project_root/pocketpy/vendor"
