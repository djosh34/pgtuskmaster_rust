#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

require_cmd() {
    local cmd_name="$1"
    if ! command -v "${cmd_name}" >/dev/null 2>&1; then
        echo "missing required command: ${cmd_name}" >&2
        exit 1
    fi
}

require_cmd npm

cd "${SCRIPT_DIR}"
npm ci --ignore-scripts --no-audit --no-fund
