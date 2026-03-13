#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Notify email script that task is finished
/bin/bash "$SCRIPT_DIR/email.sh" finish

CURRENT_TASK="NONE"

# if current task is not empty, set current_task
if [[ -f "$SCRIPT_DIR/current_task.txt" ]]; then
  CURRENT_TASK="$(cat "$SCRIPT_DIR/current_task.txt")"

  # remove all newlines from current task
  CURRENT_TASK="$(echo "$CURRENT_TASK" | tr -d '\n')"
fi

# Append current task to history and remove current task file
echo "$CURRENT_TASK" >> "$SCRIPT_DIR/task_history.txt"

rm "$SCRIPT_DIR/current_task.txt" || true
