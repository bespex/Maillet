#!/usr/bin/env bash
# Build Maillet as it is in the working tree and start it.
#
#   ./run.sh [ARGS...]            debug build (quick to build)
#   ./run.sh --release [ARGS...]  optimized build, as packaged
#
# The frontend in dist/ is embedded into the binary; no dev server is needed.
# ARGS go to maillet.

set -euo pipefail

repo="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

profile=()
case "${1:-}" in
  -h | --help)
    sed -n '2,8s/^# \{0,1\}//p' "$0"
    exit 0
    ;;
  --release)
    profile=(--release)
    shift
    ;;
esac

command -v cargo >/dev/null || { echo "run.sh: cargo not found" >&2; exit 1; }

# A second Maillet watches the same accounts and cannot take the port of the local API
if pids="$(pgrep -x maillet)"; then
  echo "run.sh: Maillet is already running (PID ${pids//$'\n'/, }) - quit it first for a clean test." >&2
fi

cd "$repo/src-tauri"
exec cargo run "${profile[@]}" -- "$@"
