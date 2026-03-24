#!/bin/bash

set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

tracked_existing_paths() {
  git ls-files -z -- "$1" |
    while IFS= read -r -d '' path; do
      if [ -f "$path" ]; then
        printf '%s\0' "$path"
      fi
    done
}

count_files() {
  tracked_existing_paths "$1" |
    awk 'BEGIN { RS = "\0" } END { print NR + 0 }'
}

count_lines() {
  tracked_existing_paths "$1" |
    xargs -0r wc -l |
    awk 'END { print $1 + 0 }'
}

src_files="$(count_files src)"
src_lines="$(count_lines src)"
tests_files="$(count_files tests)"
tests_lines="$(count_lines tests)"
total_files=$((src_files + tests_files))
total_lines=$((src_lines + tests_lines))

printf 'src/: %s lines across %s existing git-tracked files\n' "$src_lines" "$src_files"
printf 'tests/: %s lines across %s existing git-tracked files\n' "$tests_lines" "$tests_files"
printf 'total: %s lines across %s existing git-tracked files\n' "$total_lines" "$total_files"
