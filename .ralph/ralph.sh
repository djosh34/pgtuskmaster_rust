#!/bin/bash

# ralph.sh — starts ralph-worker.sh as a systemd transient service.
# Ctrl+C stops the service (systemd kills entire cgroup — no orphans ever).
# Usage: /bin/bash ./ralph.sh <opencode|claude|codex>

set -euo pipefail

export MODE="${1:-claude}"
#MODE="opencode"

SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
WORK_DIR="$(dirname "$SCRIPT_DIR")"
UNIT_NAME="ralph-pgtuskmaster"
EMAIL_WATCHER_UNIT="ralph-pgtuskmaster-progress-watcher.path"

# Show current ralph services status
echo ""
echo "=== Ralph Services Status ==="
if systemctl --user is-active "$UNIT_NAME.service" &>/dev/null; then
  echo "  [ACTIVE]   $UNIT_NAME.service (worker)"
else
  echo "  [inactive] $UNIT_NAME.service (worker)"
fi
echo ""

# Disable old watcher model; progress_append.sh sends email directly now.
if systemctl --user is-enabled "$EMAIL_WATCHER_UNIT" &>/dev/null; then
  systemctl --user disable "$EMAIL_WATCHER_UNIT" >/dev/null 2>&1 || true
fi
if systemctl --user is-active "$EMAIL_WATCHER_UNIT" &>/dev/null; then
  echo "Stopping legacy email watcher (direct email now handled by progress_append.sh)..."
  systemctl --user stop "$EMAIL_WATCHER_UNIT" >/dev/null 2>&1 || true
fi

# If already running, attach to it. Otherwise start fresh.
if systemctl --user is-active "$UNIT_NAME.service" &>/dev/null; then
  echo ""
  echo "============================================="
  echo "  ATTACHING TO RUNNING ONE"
  echo "============================================="
  echo ""
  echo "  Ctrl+C   = STOP ralph (kills the service)"
  echo "  Ctrl+\\   = DETACH (leaves ralph running in background)"
  echo ""
else
  START_TIME=$(date '+%Y-%m-%d %H:%M:%S')

  systemd-run --user \
    --unit="$UNIT_NAME" \
    --collect \
    --working-directory="$WORK_DIR" \
    --setenv=PATH="$HOME/.local/bin:$HOME/go/bin:$HOME/.bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
    --setenv=HOME="$HOME" \
    --property=TimeoutStopSec=10 \
    --property=KillMode=control-group \
    /bin/bash "$SCRIPT_DIR/ralph-worker.sh" "$MODE"

  echo ""
  echo "ralph started. Following output..."
  echo ""
  echo "  Ctrl+C   = STOP ralph (kills the service)"
  echo "  Ctrl+\\   = DETACH (leaves ralph running in background)"
  echo ""
fi

SINCE="${START_TIME:-today}"

JPID=""

# Ctrl+C (INT/TERM): stop the systemd service (kills entire cgroup), kill journalctl, exit
cleanup_stop() {
  echo ""
  echo "Stopping ralph..."
  systemctl --user stop "$UNIT_NAME" 2>/dev/null || true
  [[ -n "$JPID" ]] && kill "$JPID" 2>/dev/null || true
  exit 0
}
trap cleanup_stop INT TERM

# Ctrl+\ (QUIT): detach from logs but leave ralph running in background
cleanup_detach() {
  echo ""
  echo ""
  echo "Detached. ralph continues running in background."
  echo "  Re-run:   /bin/bash $SCRIPT_DIR/ralph.sh   to reattach"
  echo "  Stop:     systemctl --user stop $UNIT_NAME"
  echo "  Status:   systemctl --user status $UNIT_NAME"
  [[ -n "$JPID" ]] && kill "$JPID" 2>/dev/null || true
  exit 0
}
trap cleanup_detach QUIT

journalctl --user -u "$UNIT_NAME" -f --output=cat --since="$SINCE" &
JPID=$!

# Wait in a loop — 'wait' without args is reliably interrupted by signals in bash
while kill -0 "$JPID" 2>/dev/null; do
  wait 2>/dev/null || true
done
