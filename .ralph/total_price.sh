#!/bin/bash


set -euo pipefail

SOURCE_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(cd -P "$( dirname "$SOURCE_PATH" )" >/dev/null 2>&1 && pwd)"
ARCHIVE_DIR="$SCRIPT_DIR/../../gleam_archive"



cat $ARCHIVE_DIR/*.json | jq -r '.part | .tokens | select(. != null) | .' | jq -s '{ sums: { input: map(.input) | add, output: map(.output) | add, cache_read: map(.cache.read) | add } } | { token_counts: .sums, costs_usd: { input: (.sums.input / 1000000 * 0.60), cached_input: (.sums.cache_read / 1000000 * 0.10), output: (.sums.output / 1000000 * 3.00) } } | .costs_usd += { total: (.costs_usd | add) } '
