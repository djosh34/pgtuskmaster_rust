#!/bin/bash

# ralph-email-watcher.sh — manage the email watcher service independently
# Usage:
#   ralph-email-watcher.sh start   - Start the email watcher
#   ralph-email-watcher.sh stop    - Stop the email watcher
#   ralph-email-watcher.sh status  - Show status of the email watcher
#   ralph-email-watcher.sh         - Toggle (start if off, stop if on)

set -euo pipefail

SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
WORK_DIR="$(dirname "$SCRIPT_DIR")"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

PATH_UNIT="ralph-pgtuskmaster-progress-watcher.path"
SERVICE_UNIT="ralph-pgtuskmaster-progress-email.service"

# Ensure systemd directory exists
mkdir -p "$SYSTEMD_USER_DIR"
mkdir -p "$SCRIPT_DIR/progress"

# Create/update unit files
setup_units() {
  cat > "$SYSTEMD_USER_DIR/$PATH_UNIT" <<EOF
[Unit]
Description=Watch progress directory for changes

[Path]
PathModified=$SCRIPT_DIR/progress
Unit=$SERVICE_UNIT

[Install]
WantedBy=default.target
EOF

  cat > "$SYSTEMD_USER_DIR/$SERVICE_UNIT" <<EOF
[Unit]
Description=Send update email when progress is updated

[Service]
Type=oneshot
WorkingDirectory=$WORK_DIR
ExecStart=/bin/bash $SCRIPT_DIR/email.sh
EOF

  systemctl --user daemon-reload
}

is_running() {
  systemctl --user is-active "$PATH_UNIT" &>/dev/null
}

start_watcher() {
  setup_units
  if is_running; then
    echo "WARNING: Email watcher was already running"
    return 1
  else
    systemctl --user enable --now "$PATH_UNIT"
    echo "Email watcher started"
    return 0
  fi
}

stop_watcher() {
  if is_running; then
    systemctl --user stop "$PATH_UNIT"
    systemctl --user disable "$PATH_UNIT" 2>/dev/null || true
    echo "Email watcher stopped"
  else
    echo "Email watcher was not running"
  fi
}

show_status() {
  echo ""
  echo "=== Ralph Systemd Services ==="
  echo ""

  # Check email watcher
  if is_running; then
    echo "  [ACTIVE]   $PATH_UNIT"
  else
    echo "  [inactive] $PATH_UNIT"
  fi

  # Check main ralph service
  if systemctl --user is-active "ralph-pgtuskmaster.service" &>/dev/null; then
    echo "  [ACTIVE]   ralph-pgtuskmaster.service (worker)"
  else
    echo "  [inactive] ralph-pgtuskmaster.service (worker)"
  fi

  echo ""
}

case "${1:-}" in
  start)
    start_watcher
    ;;
  stop)
    stop_watcher
    ;;
  status)
    show_status
    ;;
  "")
    # Toggle mode
    if is_running; then
      stop_watcher
    else
      start_watcher
    fi
    ;;
  *)
    echo "Usage: $0 [start|stop|status]"
    echo "  start  - Start the email watcher"
    echo "  stop   - Stop the email watcher"
    echo "  status - Show status of ralph services"
    echo "  (none) - Toggle watcher on/off"
    exit 1
    ;;
esac
