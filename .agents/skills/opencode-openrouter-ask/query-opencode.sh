#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_PATH="${SCRIPT_DIR}/opencode.json"
MODEL="${OPENCODE_MODEL:-openrouter/kimi-k2-thinking}"
FORMAT="${OPENCODE_FORMAT:-default}"
THINKING_FLAG="${OPENCODE_THINKING:-}"
ATTACH_URL="${OPENCODE_ATTACH:-}"
RUN_DIR="${OPENCODE_DIR:-}"
PROMPT="$(cat)"

if [[ -z "${PROMPT}" ]]; then
  printf 'query-opencode.sh: expected prompt on stdin\n' >&2
  exit 1
fi

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

ARGS+=("${PROMPT}")

OPENCODE_CONFIG="${CONFIG_PATH}" exec opencode "${ARGS[@]}"
