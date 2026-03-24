#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
cd "$repo_root"

base_ref="$(tr -d '\n' < "$SCRIPT_DIR/git_diff_lines.txt")"

if [ -z "$base_ref" ]; then
  echo "error: ${SCRIPT_DIR}/git_diff_lines.txt is empty" >&2
  exit 1
fi

if ! git rev-parse --verify --quiet "$base_ref^{commit}" >/dev/null; then
  echo "error: unknown revision: $base_ref" >&2
  exit 1
fi

read -r added removed < <(
  git diff --numstat "$base_ref" -- src tests |
    awk '
      $1 == "-" || $2 == "-" { next }
      { added += $1; removed += $2 }
      END { printf "%d %d\n", added + 0, removed + 0 }
    '
)

net=$(( ${added:-0} - ${removed:-0} ))

printf '+%s -%s diff: %+d\n' "${added:-0}" "${removed:-0}" "$net"
