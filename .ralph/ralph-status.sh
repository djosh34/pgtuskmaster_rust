#!/bin/bash

# ralph-status.sh — show status of all ralph systemd services
# Usage: .ralph/ralph-status.sh

echo ""
echo "=== Ralph Systemd Services ==="
echo ""

# Email watcher
if systemctl --user is-active "ralph-pgtuskmaster-progress-watcher.path" &>/dev/null; then
  echo "  [ACTIVE]   ralph-pgtuskmaster-progress-watcher.path (email on progress change)"
else
  echo "  [inactive] ralph-pgtuskmaster-progress-watcher.path (email on progress change)"
fi

# Main worker
if systemctl --user is-active "ralph-pgtuskmaster.service" &>/dev/null; then
  echo "  [ACTIVE]   ralph-pgtuskmaster.service (main worker loop)"
else
  echo "  [inactive] ralph-pgtuskmaster.service (main worker loop)"
fi

echo ""
echo "Commands:"
echo "  .ralph/ralph-email-watcher.sh start   - Start email watcher"
echo "  .ralph/ralph-email-watcher.sh stop    - Stop email watcher"
echo "  .ralph/ralph.sh                       - Start/attach to worker"
echo "  systemctl --user stop ralph-pgtuskmaster           - Stop worker"
echo ""
