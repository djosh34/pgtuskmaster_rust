#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

readonly -a TARGET_FILES=(
  "src/api/worker.rs"
  "src/process/worker.rs"
  "src/runtime/node.rs"
  "src/ha/worker.rs"
  "src/dcs/worker.rs"
  "src/pginfo/worker.rs"
  "src/logging/postgres_ingest.rs"
)

first_cfg_test_line() {
  local file_path="$1"
  local line
  line="$(rg -n "^[[:space:]]*#\\[cfg\\(test\\)\\]" "${file_path}" | head -n 1 | cut -d: -f2 || true)"
  if [[ -z "${line}" ]]; then
    echo 2147483647
    return 0
  fi
  echo "${line}"
}

scan_pattern() {
  local file_path="$1"
  local max_line="$2"
  local pattern="$3"
  local label="$4"

  local matches
  matches="$(rg -n "${pattern}" "${file_path}" || true)"
  if [[ -z "${matches}" ]]; then
    return 0
  fi

  local filtered
  filtered="$(
    echo "${matches}" | awk -F: -v max="${max_line}" '($2 + 0) < (max + 0) { print }'
  )"

  if [[ -n "${filtered}" ]]; then
    echo "silent-error lint failed (${label}) in ${file_path} (excluding #[cfg(test)] blocks):" >&2
    echo "${filtered}" >&2
    return 1
  fi
  return 0
}

main() {
  local failed=0
  local rel
  for rel in "${TARGET_FILES[@]}"; do
    local path="${REPO_ROOT}/${rel}"
    if [[ ! -f "${path}" ]]; then
      echo "missing target file: ${rel}" >&2
      exit 1
    fi

    local max_line
    max_line="$(first_cfg_test_line "${path}")"

    if ! scan_pattern "${path}" "${max_line}" '^[[:space:]]*let[[:space:]]+_[[:space:]]*=' "let _ =" ; then
      failed=1
    fi
    if ! scan_pattern "${path}" "${max_line}" '\\.ok\\(\\)' ".ok()" ; then
      failed=1
    fi
    if ! scan_pattern "${path}" "${max_line}" 'filter_map\\([[:space:]]*Result::ok[[:space:]]*\\)' "filter_map(Result::ok)" ; then
      failed=1
    fi
    if ! scan_pattern "${path}" "${max_line}" 'filter_map\\([[:space:]]*\\|[^|]+\\|[^)]*\\.ok\\(\\)[^)]*\\)' "filter_map(|..| ..ok())" ; then
      failed=1
    fi
  done

  if [[ "${failed}" -ne 0 ]]; then
    exit 1
  fi
}

main "$@"

