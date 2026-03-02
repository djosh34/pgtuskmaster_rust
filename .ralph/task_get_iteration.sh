#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"


if [[ -f "$SCRIPT_DIR/task_history.txt" ]]; then
  wc -l < "$SCRIPT_DIR/task_history.txt"
else
  echo 0
fi
