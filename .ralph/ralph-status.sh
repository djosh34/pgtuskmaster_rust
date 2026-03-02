#!/bin/bash

# ralph-status.sh — show status of all ralph systemd services
# Usage: .ralph/ralph-status.sh

echo ""
echo "=== Ralph Systemd Services ==="
echo ""

# Main worker
if systemctl --user is-active "ralph-pgtuskmaster.service" &>/dev/null; then
  echo "  [ACTIVE]   ralph-pgtuskmaster.service (main worker loop)"
else
  echo "  [inactive] ralph-pgtuskmaster.service (main worker loop)"
fi

echo ""
echo "Email updates are sent directly by .ralph/progress_append.sh"
echo ""
echo "Commands:"
echo "  .ralph/ralph.sh                       - Start/attach to worker"
echo "  systemctl --user stop ralph-pgtuskmaster           - Stop worker"
echo ""
