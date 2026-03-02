#!/bin/bash

# ralph-list.sh — list running ralph systemd services with uptime.

FOUND=0
while read -r unit; do
  [[ -z "$unit" ]] && continue
  FOUND=1
  started=$(systemctl --user show "$unit" --property=ActiveEnterTimestamp --value)
  elapsed=$(systemctl --user show "$unit" --property=ActiveEnterTimestampMonotonic --value)
  now_mono=$(cut -d' ' -f1 /proc/uptime)
  seconds=$(awk "BEGIN {printf \"%d\", $now_mono - $elapsed/1000000}")
  duration=$(printf '%dd %02dh %02dm' $((seconds/86400)) $((seconds%86400/3600)) $((seconds%3600/60)))
  echo "$unit  active since $started  ($duration)"
done < <(systemctl --user list-units --type=service --state=running 'ralph*' --plain --no-legend | awk '{print $1}')

if [[ $FOUND -eq 0 ]]; then
  echo "No running ralph processes."
fi
