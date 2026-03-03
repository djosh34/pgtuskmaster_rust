#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

ALLOWED_LANGS_REGEX='^(mermaid|bash|console|toml|text)$'

scan_file() {
    local file_path="$1"
    awk -v file_path="${file_path}" -v allowed="${ALLOWED_LANGS_REGEX}" '
        function trim(s) {
            sub(/^[[:space:]]+/, "", s)
            sub(/[[:space:]]+$/, "", s)
            return s
        }

        BEGIN {
            in_block = 0
            block_lang = ""
            start_line = 0
            errors = 0
        }

        /^```/ {
            if (in_block == 0) {
                block_lang = trim(substr($0, 4))
                start_line = NR
                if (block_lang == "") {
                    printf("%s:%d: unlabeled fenced block is not allowed (add an explicit language)\n", file_path, NR) > "/dev/stderr"
                    errors = 1
                } else if (block_lang !~ allowed) {
                    printf("%s:%d: fenced block language `%s` is not allowed (allowed: mermaid, bash, console, toml, text)\n", file_path, NR, block_lang) > "/dev/stderr"
                    errors = 1
                }
                in_block = 1
            } else {
                in_block = 0
                block_lang = ""
                start_line = 0
            }
            next
        }

        END {
            if (in_block == 1) {
                printf("%s:%d: fenced block starting here is not closed\n", file_path, start_line) > "/dev/stderr"
                errors = 1
            }
            exit errors
        }
    ' "${file_path}"
}

main() {
    local -a roots=(
        "${REPO_ROOT}/docs/src/architecture"
        "${REPO_ROOT}/docs/src/concepts"
        "${REPO_ROOT}/docs/src/interfaces"
        "${REPO_ROOT}/docs/src/testing"
    )

    local -a files=()
    local root
    for root in "${roots[@]}"; do
        if [[ -d "${root}" ]]; then
            while IFS= read -r -d '' f; do
                files+=("${f}")
            done < <(find "${root}" -type f -name '*.md' -print0)
        fi
    done

    if [[ "${#files[@]}" -eq 0 ]]; then
        echo "no architecture-oriented docs files found to scan" >&2
        exit 1
    fi

    local failed=0
    local file_path
    for file_path in "${files[@]}"; do
        if ! scan_file "${file_path}"; then
            failed=1
        fi
    done

    if [[ "${failed}" -ne 0 ]]; then
        exit 1
    fi
}

main "$@"

