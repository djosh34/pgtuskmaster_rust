#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Read stdin into a variable
INPUT=$(cat)

# Get the transcript path from the hook input
TRANSCRIPT_PATH=$(echo "$INPUT" | jq -r '.transcript_path')

# Get the last line of the transcript and calculate total input tokens
LAST_LINE=$(tail -n 1 "$TRANSCRIPT_PATH")
TOTAL_TOKENS=$(echo "$LAST_LINE" | jq '(.message.usage.input_tokens // 0) + (.message.usage.cache_creation_input_tokens // 0) + (.message.usage.cache_read_input_tokens // 0)')

# Get the max count from config file
MAX_COUNT_FILE="$SCRIPT_DIR/max_count.txt"
PROMPT_FILE="$SCRIPT_DIR/max_count_prompt.txt"

# Build the additional context message
ADDITIONAL_CONTEXT=""

if [ -f "$MAX_COUNT_FILE" ]; then
    MAX_COUNT=$(cat "$MAX_COUNT_FILE")


#     Uncomment to only show when over token limit:
     if [ "$TOTAL_TOKENS" -gt "$MAX_COUNT" ] 2>/dev/null; then
        if [ -f "$PROMPT_FILE" ]; then
            PROMPT_CONTENT=$(cat "$PROMPT_FILE")
            ADDITIONAL_CONTEXT="$PROMPT_CONTENT"
        fi
     fi
fi

# Output JSON with additionalContext (only if we have content)
if [ -n "$ADDITIONAL_CONTEXT" ]; then
    jq -n --arg context "$ADDITIONAL_CONTEXT" '{
        hookSpecificOutput: {
            hookEventName: "PreToolUse",
            additionalContext: $context
        }
    }'

    exit 2
fi
