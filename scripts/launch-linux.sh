#!/usr/bin/env bash
set -euo pipefail
app_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if [[ ${1:-} == --compatibility ]]; then
  export WEBKIT_DISABLE_DMABUF_RENDERER=1
  shift
fi
exec "$app_dir/runtime/bin/cyoa-manager" "$@"
