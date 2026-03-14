#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_PATH="${SCRIPT_DIR}/opencode.json"
MODEL="openrouter/moonshotai/kimi-k2-thinking"
FORMAT="${OPENCODE_FORMAT:-default}"
RUN_DIR="${OPENCODE_DIR:-}"
ARGS=(run --model "${MODEL}" --format "${FORMAT}")


if [[ -n "${RUN_DIR}" ]]; then
  ARGS+=(--dir "${RUN_DIR}")
fi

OPENCODE_CONFIG="${CONFIG_PATH}" exec opencode "${ARGS[@]}"