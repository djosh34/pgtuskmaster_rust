#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_PATH="${SCRIPT_DIR}/opencode.json"
MODEL="openrouter/moonshotai/kimi-k2-thinking"
FORMAT="${OPENCODE_FORMAT:-json}"
RUN_DIR="${OPENCODE_DIR:-}"
ARGS=(run --model "${MODEL}" --format "${FORMAT}")


if [[ -n "${RUN_DIR}" ]]; then
  ARGS+=(--dir "${RUN_DIR}")
fi

process_line() {
  printf "\n\n\n"
  printf "\n\n"
  printf "\n\n"
  printf "===================="
  printf "===================="
  printf "===================="
  printf "===================="
  echo "$1" | yq -C -p=json '.'
  printf "===================="
  printf "===================="
  printf "===================="
  printf "===================="
  printf "\n\n"
  printf "\n\n"
  printf "\n\n"

}
OPENCODE_CONFIG="${CONFIG_PATH}" exec opencode "${ARGS[@]}" | while read -r line; do
  process_line "$line"
done
