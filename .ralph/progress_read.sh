#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

MODE="$1"
if [[ -z "$MODE" ]]; then
  echo "Error: mode required as first argument (agent name, ALL, or EMAIL)" >&2
  exit 1
fi

ITERATION=$("$SCRIPT_DIR/task_get_iteration.sh")
if [[ -z "$ITERATION" ]]; then
  echo "Error: could not determine current iteration" >&2
  exit 1
fi

JSONL_FILE="$SCRIPT_DIR/progress/${ITERATION}.jsonl"

if [[ ! -f "$JSONL_FILE" ]] || [[ ! -s "$JSONL_FILE" ]]; then
  exit 0
fi

if [[ "$MODE" == "EMAIL" ]]; then
  LIMIT_BYTES="false"
else
  MAX_BYTES=$(cat "$SCRIPT_DIR/progress_max_bytes.txt")
  LIMIT_BYTES="true"
fi

TOTAL_ENTRIES=$(jq -s '
  reduce .[] as $entry (
    {};
    .counts[$entry.agent] = ((.counts[$entry.agent] // 0) + 1) |
    .entries += [$entry + {log_number: .counts[$entry.agent]}]
  ) | .entries
  | if $mode != "ALL" and $mode != "EMAIL" then
      map(select(.agent == $mode))
    else . end
  | length
' --arg mode "$MODE" "$JSONL_FILE")

PREV_OUTPUT=""
OUTPUT="



"
INCLUDED=0

while IFS= read -r json_line; do
  TIME=$(printf '%s' "$json_line" | jq -r '.time')
  LOG_NUM=$(printf '%s' "$json_line" | jq -r '.log_number')
  AGENT=$(printf '%s' "$json_line" | jq -r '.agent')
  CONTENT=$(printf '%s' "$json_line" | jq -r '.content')

  TIME_FORMATTED=$(TZ='Europe/Amsterdam' date -d "@${TIME:0:10}" '+%Y-%m-%d %H:%M ')

  ENTRY="=== ${AGENT} [${LOG_NUM}] -- ${TIME_FORMATTED} ===

${CONTENT}



"
  PREV_OUTPUT="$OUTPUT"
  OUTPUT="${OUTPUT}${ENTRY}
"
  INCLUDED=$((INCLUDED + 1))
  if [[ "$LIMIT_BYTES" == "true" ]]; then
    BYTE_COUNT=$(printf '%s' "$OUTPUT" | wc -c)
    if [[ "$BYTE_COUNT" -gt "$MAX_BYTES" ]]; then
      INCLUDED=$((INCLUDED - 1))
      TRUNCATED=$((TOTAL_ENTRIES - INCLUDED))
      printf '%s' "$PREV_OUTPUT"
      echo "=== TRUNCATED: ${TRUNCATED} older entries omitted (${INCLUDED}/${TOTAL_ENTRIES} shown, byte limit ${MAX_BYTES}) ==="
      exit 0
    fi
  fi
done < <(jq -s '
  reduce .[] as $entry (
    {};
    .counts[$entry.agent] = ((.counts[$entry.agent] // 0) + 1) |
    .entries += [$entry + {log_number: .counts[$entry.agent]}]
  ) | .entries
  | if $mode != "ALL" and $mode != "EMAIL" then
      map(select(.agent == $mode))
    else . end
  | reverse
  | .[]
' --arg mode "$MODE" "$JSONL_FILE" -c)

printf '%s' "$OUTPUT"
