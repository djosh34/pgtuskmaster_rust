#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCHIVE_DIR="$SCRIPT_DIR/archive"

ls "$ARCHIVE_DIR"/*_iteration_*.json 2>/dev/null \
  | sed 's/.*_iteration_0*//' \
  | sed 's/\.json$//' \
  | sort -n \
  | tail -1
