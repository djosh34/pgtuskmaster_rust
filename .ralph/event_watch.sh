#!/bin/bash

# event_watch.sh — compatibility event watcher entrypoint for legacy systemd unit.
# This keeps the unit healthy while the real event pipeline lives elsewhere.

set -euo pipefail

SCRIPT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HEARTBEAT_SECONDS=300
RUNNING=1

on_shutdown() {
  RUNNING=0
  echo "[ralph-event-watch] shutdown requested at $(date -Is)"
}

trap on_shutdown TERM INT

echo "[ralph-event-watch] started at $(date -Is) repo=$REPO_ROOT"
echo "[ralph-event-watch] compatibility loop active; heartbeat every ${HEARTBEAT_SECONDS}s"

while (( RUNNING )); do
  sleep "$HEARTBEAT_SECONDS" &
  sleeper_pid=$!
  wait "$sleeper_pid" || true
  if (( RUNNING )); then
    echo "[ralph-event-watch] heartbeat $(date -Is)"
  fi
done

echo "[ralph-event-watch] exited at $(date -Is)"
