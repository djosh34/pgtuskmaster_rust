#!/bin/bash

TASKS_DIR=".ralph/tasks"
CURRENT_OUTPUT_FILE=".ralph/current_tasks.md"
DONE_OUTPUT_FILE=".ralph/current_tasks_done.md"
PREVIEW_LINES=5

render_preview() {
    local task_file="$1"
    head -n "$PREVIEW_LINES" "$task_file"
}

# Build entire content in variables
current_content="# Current Tasks Summary

Generated: $(date)
"
done_content="# Done Tasks Summary

Generated: $(date)
"

# Find all task markdown files
first_current=true
first_done=true
for task_file in "$TASKS_DIR"/*/*.md; do
    if [[ -f "$task_file" ]]; then
        passes_tag="$(grep -m1 -o '<passes>[^<]*</passes>' "$task_file" || true)"
        passes_value="$(echo "$passes_tag" | sed -e 's#<passes>##' -e 's#</passes>##')"

        if [[ "$passes_value" == "true" ]]; then
            if [[ "$first_done" == true ]]; then
                first_done=false
            else
                done_content+="
==============
"
            fi
            done_content+="
# Task \`$task_file\`

\`\`\`
$(render_preview "$task_file")
\`\`\`
"
        else
            if [[ "$first_current" == true ]]; then
                first_current=false
            else
                current_content+="
==============
"
            fi
            current_content+="
# Task \`$task_file\`

\`\`\`
$(render_preview "$task_file")
\`\`\`
"
        fi
    fi
done

# Write files in one operation each
echo "$current_content" > "$CURRENT_OUTPUT_FILE"
echo "$done_content" > "$DONE_OUTPUT_FILE"

echo "Updated $CURRENT_OUTPUT_FILE"
echo "Updated $DONE_OUTPUT_FILE"
