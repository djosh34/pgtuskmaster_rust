#!/bin/bash
# Send crash notification email
# Usage: .ralph/email_crash.sh <exit_code> <last_lines> [backoff_status]

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXIT_CODE="${1:-unknown}"
LAST_LINES="${2:-<no output captured>}"
BACKOFF_STATUS="${3:-}"

# Read email addresses
SEND_FROM=$(cat ~/send_from 2>/dev/null) || { echo "Error: ~/send_from not found"; exit 1; }
SEND_TO=$(cat ~/send_to 2>/dev/null) || { echo "Error: ~/send_to not found"; exit 1; }

# Get iteration number
ITERATION_NUMBER=$("$SCRIPT_DIR/email_get_iteration.sh" 2>/dev/null) || true

# Get current task name
CURRENT_TASK_NAME=""
if [[ -f "$SCRIPT_DIR/current_task.txt" ]]; then
    CURRENT_TASK_PATH=$(cat "$SCRIPT_DIR/current_task.txt")
    if [[ -n "$CURRENT_TASK_PATH" && -f "$CURRENT_TASK_PATH" ]]; then
        CURRENT_TASK_NAME=$(basename "$CURRENT_TASK_PATH" .md)
    fi
fi

MODE="${MODE:-claude}"

SUBJECT="[$ITERATION_NUMBER] CRASH: $MODE exited $EXIT_CODE during ${CURRENT_TASK_NAME:-(no task)}"

RETRY_LINE=""
if [[ -n "$BACKOFF_STATUS" ]]; then
    RETRY_LINE="Retry: $BACKOFF_STATUS
"
fi

BODY="$MODE crashed with exit code $EXIT_CODE
Task: ${CURRENT_TASK_NAME:-(no task)}
${RETRY_LINE}Worker will retry (backoff — never gives up).

--- Last output ---
$LAST_LINES"

EMAIL_CONTENT="From: Ralph Bot <$SEND_FROM>
To: $SEND_TO
Subject: $SUBJECT
Content-Type: text/plain; charset=utf-8

$BODY"

echo "Sending crash email: $SUBJECT"
echo "$EMAIL_CONTENT" | msmtp "$SEND_TO"
