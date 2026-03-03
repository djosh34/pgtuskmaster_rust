#!/bin/bash

# ralph-status.sh — show status of all ralph systemd services
# Usage: .ralph/ralph-status.sh

set -euo pipefail

echo ""
echo "=== Ralph Systemd Services ==="
echo ""

# Main worker
if systemctl --user is-active "ralph-pgtuskmaster.service" &>/dev/null; then
  echo "  [ACTIVE]   ralph-pgtuskmaster.service (main worker loop)"
else
  echo "  [inactive] ralph-pgtuskmaster.service (main worker loop)"
fi

EVENT_WATCH_UNIT="ralph-event-watch.service"
if systemctl --user list-unit-files "$EVENT_WATCH_UNIT" >/dev/null 2>&1; then
  if systemctl --user is-active "$EVENT_WATCH_UNIT" &>/dev/null; then
    echo "  [ACTIVE]   $EVENT_WATCH_UNIT (compatibility watcher)"
  else
    echo "  [inactive] $EVENT_WATCH_UNIT (compatibility watcher)"
  fi

  EVENT_EXEC_PATH="$(systemctl --user cat "$EVENT_WATCH_UNIT" 2>/dev/null | sed -n 's/^ExecStart=\/bin\/bash[[:space:]]\+\([^[:space:]]\+\).*$/\1/p' | head -n 1)"
  if [[ -z "$EVENT_EXEC_PATH" ]]; then
    echo "  [warn]     $EVENT_WATCH_UNIT ExecStart target could not be parsed"
  elif [[ ! -f "$EVENT_EXEC_PATH" ]]; then
    echo "  [warn]     $EVENT_WATCH_UNIT ExecStart target missing: $EVENT_EXEC_PATH"
  fi
fi

echo ""
echo "Email updates are sent directly by .ralph/progress_append.sh"
echo ""
echo "Commands:"
echo "  .ralph/ralph.sh                       - Start/attach to worker"
echo "  systemctl --user stop ralph-pgtuskmaster           - Stop worker"
echo ""
