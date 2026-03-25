#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
WORK_DIR="$(dirname "$SCRIPT_DIR")"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
UNIT_NAME="ralph-worker.service"
UNIT_PATH="$SYSTEMD_USER_DIR/$UNIT_NAME"
DEFAULT_MODE="codex"
MODE="$DEFAULT_MODE"
ACTION="attach"

usage() {
  cat <<EOF
Usage: /bin/bash .ralph/ralph.sh [--start|--stop|--status] [--mode codex|claude|opencode]

No flags:
  Start the Ralph worker service if needed, then attach to its logs.

Flags:
  --start, start     Start the service only; do not attach to logs.
  --stop, stop       Stop the service.
  --status, status   Show service status.
  --mode MODE        Set the worker mode for future starts. Default: $DEFAULT_MODE
  --help, help       Show this help.
EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --start|start)
        ACTION="start"
        shift
        ;;
      --stop|stop)
        ACTION="stop"
        shift
        ;;
      --status|status)
        ACTION="status"
        shift
        ;;
      --mode)
        if [[ $# -lt 2 ]]; then
          echo "error: --mode requires a value" >&2
          exit 1
        fi
        MODE="$2"
        shift 2
        ;;
      --help|help|-h)
        ACTION="help"
        shift
        ;;
      *)
        echo "error: unknown argument: $1" >&2
        usage >&2
        exit 1
        ;;
    esac
  done
}

write_unit_file() {
  mkdir -p "$SYSTEMD_USER_DIR"

  local tmp_path
  tmp_path="$(mktemp)"

  cat > "$tmp_path" <<EOF
[Unit]
Description=Ralph worker for pgtuskmaster_rust
After=network.target

[Service]
Type=simple
WorkingDirectory=$WORK_DIR
Environment=HOME=$HOME
Environment=PATH=$HOME/.npm-global/bin:$HOME/.local/bin:$HOME/go/bin:$HOME/.bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
Environment=MODE=$MODE
ExecStart=/bin/bash $SCRIPT_DIR/ralph-worker.sh $MODE
KillMode=control-group
TimeoutStopSec=10
Restart=no

[Install]
WantedBy=default.target
EOF

  if [[ ! -f "$UNIT_PATH" ]] || ! cmp -s "$tmp_path" "$UNIT_PATH"; then
    mv "$tmp_path" "$UNIT_PATH"
    systemctl --user daemon-reload
  else
    rm -f "$tmp_path"
  fi
}

is_active() {
  systemctl --user is-active "$UNIT_NAME" >/dev/null 2>&1
}

start_service() {
  write_unit_file

  if is_active; then
    echo "ralph already running"
    return 0
  fi

  systemctl --user start "$UNIT_NAME"
  echo "ralph started"
}

ensure_service_running_for_attach() {
  if is_active; then
    echo "ralph already running"
    return 0
  fi

  start_service
}

stop_service() {
  systemctl --user stop "$UNIT_NAME" 2>/dev/null || true
  echo "ralph stopped"
}

show_status() {
  write_unit_file
  systemctl --user status "$UNIT_NAME" --no-pager || true
}

cleanup_stop() {
  echo ""
  echo "Stopping ralph..."
  systemctl --user stop "$UNIT_NAME" 2>/dev/null || true
  exit 0
}

cleanup_detach() {
  echo ""
  echo ""
  echo "Detached. ralph continues running in background."
  echo "  Re-run:   /bin/bash $SCRIPT_DIR/ralph.sh"
  echo "  Stop:     /bin/bash $SCRIPT_DIR/ralph.sh --stop"
  echo "  Status:   /bin/bash $SCRIPT_DIR/ralph.sh --status"
  exit 0
}

attach_logs() {
  echo ""
  echo "Attaching to $UNIT_NAME logs..."
  echo ""
  echo "  Ctrl+C   = STOP ralph"
  echo "  Ctrl+\\   = DETACH"
  echo ""

  trap cleanup_stop INT TERM
  trap cleanup_detach QUIT

  journalctl --user -u "$UNIT_NAME" -n 50 -f --output=cat || true
}

parse_args "$@"

case "$ACTION" in
  help)
    usage
    ;;
  status)
    show_status
    ;;
  stop)
    stop_service
    ;;
  start)
    start_service
    ;;
  attach)
    ensure_service_running_for_attach
    attach_logs
    ;;
esac
