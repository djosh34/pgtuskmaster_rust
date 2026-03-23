#!/bin/bash

set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

count_files() {
  git ls-files -- "$1" | wc -l | tr -d ' '
}

count_lines() {
  local file_count
  file_count="$(count_files "$1")"

  if [ "$file_count" -eq 0 ]; then
    printf '0\n'
    return
  fi

  git ls-files -z -- "$1" |
    xargs -0 wc -l |
    awk 'END { print $1 + 0 }'
}

src_files="$(count_files src)"
src_lines="$(count_lines src)"
tests_files="$(count_files tests)"
tests_lines="$(count_lines tests)"
total_files=$((src_files + tests_files))
total_lines=$((src_lines + tests_lines))

printf 'src/: %s lines across %s git-tracked files\n' "$src_lines" "$src_files"
printf 'tests/: %s lines across %s git-tracked files\n' "$tests_lines" "$tests_files"
printf 'total: %s lines across %s git-tracked files\n' "$total_lines" "$total_files"
