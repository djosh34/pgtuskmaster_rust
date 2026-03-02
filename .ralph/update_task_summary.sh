#!/bin/bash

TASKS_DIR=".ralph/tasks"
OUTPUT_FILE=".ralph/current_tasks.md"
PREVIEW_LINES=5

# Build entire content in a variable
content="# Current Tasks Summary

Generated: $(date)
"

# Find all task markdown files
first=true
for task_file in "$TASKS_DIR"/*/*.md; do
    if [[ -f "$task_file" ]]; then
        if [[ "$first" == true ]]; then
            first=false
        else
            content+="
---
"
        fi
        content+="
**Path:** \`$task_file\`

$(tail -n +2 "$task_file" | head -n "$PREVIEW_LINES")
"
    fi
done

# Write entire file in one operation
echo "$content" > "$OUTPUT_FILE"

echo "Updated $OUTPUT_FILE"
