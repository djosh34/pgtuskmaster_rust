#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

AGENT_NAME="$1"
if [[ -z "$AGENT_NAME" ]]; then
  echo "Error: agent name required as first argument" >&2
  exit 1
fi

CONTENT=$(cat)

ITERATION=$("$SCRIPT_DIR/task_get_iteration.sh")
if [[ -z "$ITERATION" ]]; then
  echo "Error: could not determine current iteration" >&2
  exit 1
fi

TIMESTAMP=$(date +%s%3N)

mkdir -p "$SCRIPT_DIR/progress"

jq -n --arg time "$TIMESTAMP" --arg agent "$AGENT_NAME" --arg content "$CONTENT" \
  '{time: $time, agent: $agent, content: $content}' -c \
  >> "$SCRIPT_DIR/progress/${ITERATION}.jsonl"
