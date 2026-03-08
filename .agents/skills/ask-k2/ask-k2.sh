#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_PATH="${SCRIPT_DIR}/opencode.json"
MODEL="${OPENCODE_MODEL:-openrouter/kimi-k2-thinking}"
FORMAT="${OPENCODE_FORMAT:-default}"
THINKING_FLAG="${OPENCODE_THINKING:-}"
ATTACH_URL="${OPENCODE_ATTACH:-}"
RUN_DIR="${OPENCODE_DIR:-}"
ARGS=(run --model "${MODEL}" --format "${FORMAT}")

if [[ -n "${THINKING_FLAG}" ]]; then
  ARGS+=(--thinking)
fi

if [[ -n "${ATTACH_URL}" ]]; then
  ARGS+=(--attach "${ATTACH_URL}")
fi

if [[ -n "${RUN_DIR}" ]]; then
  ARGS+=(--dir "${RUN_DIR}")
fi

OPENCODE_CONFIG="${CONFIG_PATH}" exec opencode "${ARGS[@]}"
