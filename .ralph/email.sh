#!/bin/bash
# Email script for task updates
# Usage: .ralph/email.sh [finish]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_DIR="$SCRIPT_DIR"

# Check for finish argument
FINISH_MODE=false
if [[ "$1" == "finish" ]]; then
    FINISH_MODE=true
fi

# Always run update_task_summary.sh first
if [[ -x "$RALPH_DIR/update_task_summary.sh" ]]; then
    "$RALPH_DIR/update_task_summary.sh"
fi

# Get iteration number
ITERATION_NUMBER=$("$RALPH_DIR/email_get_iteration.sh" 2>/dev/null) || true

# Function to get grouped task list
get_grouped_task_list() {
    local result=""
    local all_tasks=""
    if [[ -f "$RALPH_DIR/current_tasks.md" ]]; then
        all_tasks+=$(grep '^##' "$RALPH_DIR/current_tasks.md" || true)
    fi
    if [[ -f "$RALPH_DIR/current_tasks_done.md" ]]; then
        if [[ -n "$all_tasks" ]]; then
            all_tasks+=$'\n'
        fi
        all_tasks+=$(grep '^##' "$RALPH_DIR/current_tasks_done.md" || true)
    fi
    if [[ -n "$all_tasks" ]]; then
        local tasks_false=""
        local tasks_true=""
        local tasks_meta=""

        while IFS= read -r line; do
            [[ -z "$line" ]] && continue
            if [[ "$line" =~ \<passes\>([^<]+)\</passes\> ]]; then
                local passes_val="${BASH_REMATCH[1]}"
                local task_name=$(echo "$line" | sed 's/^## Task: //; s/ <.*//')
                # Extract all tags except passes to display
                local tags=""
                local remaining="$line"
                while [[ "$remaining" =~ \<([a-z_]+)\>([^<]+)\</([a-z_]+)\> ]]; do
                    local tag_name="${BASH_REMATCH[1]}"
                    local tag_val="${BASH_REMATCH[2]}"
                    if [[ "$tag_name" != "passes" ]]; then
                        tags+=" ${tag_name}=${tag_val}"
                    fi
                    remaining="${remaining#*</${BASH_REMATCH[3]}>}"
                done
                [[ -n "$tags" ]] && tags=" |$tags"
                case "$passes_val" in
                    false)
                        tasks_false+="[FAILS] $task_name$tags"$'\n'
                        ;;
                    true)
                        tasks_true+="[PASS]  $task_name$tags"$'\n'
                        ;;
                    meta-task)
                        tasks_meta+="[META]  $task_name$tags"$'\n'
                        ;;
                esac
            fi
        done <<< "$all_tasks"

        if [[ -n "$tasks_false" ]]; then
            result+="--- Failing ---"$'\n'"$tasks_false"$'\n'
        fi
        if [[ -n "$tasks_true" ]]; then
            result+="--- Passes ---"$'\n'"$tasks_true"$'\n'
        fi
        if [[ -n "$tasks_meta" ]]; then
            result+="--- Meta Tasks ---"$'\n'"$tasks_meta"
        fi
    fi
    echo "$result"
}

# Read email addresses
SEND_FROM=$(cat ~/send_from 2>/dev/null) || { echo "Error: ~/send_from not found"; exit 1; }
SEND_TO=$(cat ~/send_to 2>/dev/null) || { echo "Error: ~/send_to not found"; exit 1; }

# Get current task path and name
CURRENT_TASK_PATH=""
CURRENT_TASK_NAME=""
CURRENT_TASK_CONTENT=""

if [[ -f "$RALPH_DIR/current_task.txt" ]]; then
    CURRENT_TASK_PATH=$(cat "$RALPH_DIR/current_task.txt")
    if [[ -n "$CURRENT_TASK_PATH" && -f "$CURRENT_TASK_PATH" ]]; then
        CURRENT_TASK_NAME=$(basename "$CURRENT_TASK_PATH" .md)
        CURRENT_TASK_CONTENT=$(cat "$CURRENT_TASK_PATH")
    fi
fi

# Get task list grouped by passes value
TASK_LIST=$(get_grouped_task_list)

# Count pass/fail tasks
COUNT_PASS=0
COUNT_FAIL=0
COUNT_META=0
if [[ -f "$RALPH_DIR/current_tasks.md" || -f "$RALPH_DIR/current_tasks_done.md" ]]; then
    count_input=""
    if [[ -f "$RALPH_DIR/current_tasks.md" ]]; then
        count_input+=$(grep '^##' "$RALPH_DIR/current_tasks.md" || true)
    fi
    if [[ -f "$RALPH_DIR/current_tasks_done.md" ]]; then
        if [[ -n "$count_input" ]]; then
            count_input+=$'\n'
        fi
        count_input+=$(grep '^##' "$RALPH_DIR/current_tasks_done.md" || true)
    fi
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        if [[ "$line" =~ \<passes\>([^<]+)\</passes\> ]]; then
            case "${BASH_REMATCH[1]}" in
                false) ((COUNT_FAIL++)) || true ;;
                true)  ((COUNT_PASS++)) || true ;;
                meta-task) ((COUNT_META++)) || true ;;
            esac
        fi
    done <<< "$count_input"
fi
COUNT_TOTAL=$((COUNT_PASS + COUNT_FAIL))
TASK_COUNTS="${COUNT_PASS}/${COUNT_TOTAL}"

# Get progress content
PROGRESS=$("$SCRIPT_DIR/progress_read.sh" EMAIL 2>/dev/null) || true

# Build gauges
TASK_PATH_VAL="${CURRENT_TASK_PATH:-(not set)}"
TASK_NAME_VAL="${CURRENT_TASK_NAME:-(not set)}"
LINE_DIFF_VAL=$("$SCRIPT_DIR/git_diff_lines_since.sh" 2>/dev/null) || true
PROGRESS_VAL="not found"
[[ -n "$PROGRESS" ]] && PROGRESS_VAL="exists (see below)"

GAUGES="task_passes:                 $TASK_COUNTS
task_name:                      $TASK_NAME_VAL
progress:                         $PROGRESS_VAL
task_file:                         $TASK_PATH_VAL
line-diffs:                         $LINE_DIFF_VAL"

# Build email
# Choose subject prefix
if [[ "$FINISH_MODE" == true ]]; then
    SUBJECT_ACTION="Finished"
else
    SUBJECT_ACTION=""
fi
SUBJECT="[$ITERATION_NUMBER] $TASK_COUNTS $SUBJECT_ACTION: $CURRENT_TASK_NAME $LINE_DIFF_VAL"

TASK_ITERATION=$("$RALPH_DIR/task_get_iteration.sh" 2>/dev/null) || true

# Build finish section from current task
FINISH_SECTION=""
if [[ "$FINISH_MODE" == true && -n "$CURRENT_TASK_NAME" ]]; then
    FINISH_SECTION="--- Finished Task ---
[$TASK_ITERATION] $CURRENT_TASK_NAME

"
fi

BODY=$(cat <<EOF
$GAUGES

${FINISH_SECTION}--- Progress $TASK_ITERATION.jsonl ---
$PROGRESS

--- Task List ---
$TASK_LIST

--- Task ---
$CURRENT_TASK_CONTENT
EOF
)

# Construct and send email
EMAIL_CONTENT=$(cat <<EOF
From: Ralph Bot <$SEND_FROM>
To: $SEND_TO
Subject: $SUBJECT
Content-Type: text/plain; charset=utf-8

$BODY
EOF
)

echo "Sending email: $SUBJECT"
echo "$EMAIL_CONTENT" | msmtp "$SEND_TO"

if [[ $? -eq 0 ]]; then
    echo "Email sent successfully"
else
    echo "Failed to send email"
    exit 1
fi
