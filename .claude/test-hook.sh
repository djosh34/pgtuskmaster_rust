#!/bin/bash
# Test script for pretooluse-hook.sh
# This simulates how Claude Code calls the hook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_SCRIPT="$SCRIPT_DIR/pretooluse-hook.sh"
SAMPLE_TRANSCRIPT="$SCRIPT_DIR/sample_transcript.jsonl"

# Create sample hook input JSON (mimics what Claude Code sends)
HOOK_INPUT=$(jq -n \
  --arg transcript_path "$SAMPLE_TRANSCRIPT" \
  --arg tool_name "Bash" \
  '{
    tool_name: $tool_name,
    tool_input: {
      command: "ls -la"
    },
    transcript_path: $transcript_path
  }')

echo "=== Testing PreToolUse Hook ==="
echo ""
echo "Input JSON being sent to hook:"
echo "$HOOK_INPUT" | jq .
echo ""
echo "=== Hook Output ==="
echo ""

# Run the hook and capture output
OUTPUT=$(echo "$HOOK_INPUT" | "$HOOK_SCRIPT")
EXIT_CODE=$?

echo "$OUTPUT"
echo ""
echo "=== Parsed Output ==="
echo "$OUTPUT" | jq . 2>/dev/null || echo "(Output is not valid JSON)"
echo ""
echo "Exit code: $EXIT_CODE"
echo ""

# Validate the output
if echo "$OUTPUT" | jq -e '.hookSpecificOutput.additionalContext' > /dev/null 2>&1; then
  echo "✓ Hook correctly outputs additionalContext field"
  CONTEXT=$(echo "$OUTPUT" | jq -r '.hookSpecificOutput.additionalContext')
  echo "  Context: $CONTEXT"
else
  echo "✗ Hook output missing additionalContext field"
fi
