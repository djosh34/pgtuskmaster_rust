#!/bin/bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: .ralph/git_diff_lines_since.sh <hash-or-hash-snippet>

Print the total added and removed lines between the given commit-ish and the
current working tree. Files under .ralph are ignored.
EOF
}

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

base_ref="$1"

if ! git rev-parse --verify --quiet "$base_ref^{commit}" >/dev/null; then
  echo "error: unknown revision: $base_ref" >&2
  exit 1
fi

read -r added removed < <(
  git diff --numstat "$base_ref" -- . ':(exclude).ralph' |
    awk '
      $1 == "-" || $2 == "-" { next }
      { added += $1; removed += $2 }
      END { printf "%d %d\n", added + 0, removed + 0 }
    '
)

net=$(( ${added:-0} - ${removed:-0} ))

printf 'since %s: +%s -%s diff: %+d\n' "$base_ref" "${added:-0}" "${removed:-0}" "$net"
