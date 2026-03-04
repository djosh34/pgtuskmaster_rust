#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

DEFAULT_ALLOWED_LANGS_REGEX='^(mermaid|bash|console|toml|text)$'
CONTRIB_ALLOWED_LANGS_REGEX='^(mermaid|bash|console|toml|text|rust)$'

docs_src_root() {
    local book_toml="${REPO_ROOT}/docs/book.toml"
    if [[ ! -f "${book_toml}" ]]; then
        echo "${REPO_ROOT}/docs/src"
        return 0
    fi

    local src
    src="$(
        awk '
            BEGIN { in_book = 0; src = "" }
            /^\[book\][[:space:]]*$/ { in_book = 1; next }
            /^\[[^]]+\][[:space:]]*$/ { in_book = 0 }
            in_book == 1 && $0 ~ /^[[:space:]]*src[[:space:]]*=/ {
                line = $0
                sub(/^[[:space:]]*src[[:space:]]*=[[:space:]]*/, "", line)
                gsub(/"/, "", line)
                sub(/[[:space:]]+#.*/, "", line)
                src = line
            }
            END { if (src != "") print src }
        ' "${book_toml}"
    )"

    if [[ -n "${src}" ]]; then
        echo "${REPO_ROOT}/docs/${src}"
        return 0
    fi

    echo "${REPO_ROOT}/docs/src"
}

scan_file() {
    local file_path="$1"
    local rel_path="$2"
    local allowed_regex="$3"
    local html_policy="$4"

    awk -v file_path="${file_path}" -v rel_path="${rel_path}" -v allowed="${allowed_regex}" -v html_policy="${html_policy}" '
        function trim(s) {
            sub(/^[[:space:]]+/, "", s)
            sub(/[[:space:]]+$/, "", s)
            return s
        }

        function normalize_markdown_prefix(raw,    s) {
            s = raw

            # CommonMark: up to 3 spaces indentation for fences.
            sub(/^[[:space:]]{0,3}/, "", s)

            # Also treat blockquote-prefixed fences as fences (policy-bypass hardening).
            # Strip repeated > prefixes with optional following whitespace.
            while (match(s, /^>[[:space:]]*/)) {
                sub(/^>[[:space:]]*/, "", s)
                sub(/^[[:space:]]{0,3}/, "", s)
            }

            return s
        }

        function is_fence_line(line,    c, i, n) {
            if (line == "") return 0
            c = substr(line, 1, 1)
            if (c != "`" && c != "~") return 0
            n = 0
            for (i = 1; i <= length(line); i++) {
                if (substr(line, i, 1) != c) break
                n++
            }
            if (n < 3) return 0
            return 1
        }

        function fence_char(line) {
            return substr(line, 1, 1)
        }

        function fence_len(line,    c, i, n) {
            c = substr(line, 1, 1)
            n = 0
            for (i = 1; i <= length(line); i++) {
                if (substr(line, i, 1) != c) break
                n++
            }
            return n
        }

        function first_token(info,    s, a, n) {
            s = trim(info)
            if (s == "") return ""
            n = split(s, a, /[[:space:]]+/)
            if (n < 1) return ""
            return a[1]
        }

        BEGIN {
            in_block = 0
            block_char = ""
            block_len = 0
            block_lang = ""
            start_line = 0
            errors = 0
            in_pre = 0
        }

        {
            line = normalize_markdown_prefix($0)

            if (html_policy == "strict") {
                if (in_pre == 0 && line ~ /<pre([[:space:]]|>|$)/) {
                    in_pre = 1
                }
                if (in_pre == 1 && line ~ /<code([[:space:]]|>|$)/) {
                    printf("%s:%d: HTML <pre><code> blocks are not allowed (use fenced blocks with explicit language)\n", file_path, NR) > "/dev/stderr"
                    errors = 1
                }
                if (in_pre == 1 && line ~ /<\/pre>/) {
                    in_pre = 0
                }
            }

            if (is_fence_line(line)) {
                c = fence_char(line)
                n = fence_len(line)
                rest = substr(line, n + 1)

                if (in_block == 0) {
                    block_char = c
                    block_len = n
                    block_lang = first_token(rest)
                    start_line = NR
                    if (block_lang == "") {
                        printf("%s:%d: unlabeled fenced block is not allowed (add an explicit language)\n", file_path, NR) > "/dev/stderr"
                        errors = 1
                    } else if (block_lang !~ allowed) {
                        printf("%s:%d: fenced block language `%s` is not allowed by policy for %s\n", file_path, NR, block_lang, rel_path) > "/dev/stderr"
                        errors = 1
                    }
                    in_block = 1
                } else {
                    if (c == block_char && n >= block_len && trim(rest) == "") {
                        in_block = 0
                        block_char = ""
                        block_len = 0
                        block_lang = ""
                        start_line = 0
                    }
                }
                next
            }
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
    local src_root
    src_root="$(docs_src_root)"
    if [[ ! -d "${src_root}" ]]; then
        echo "docs source root not found: ${src_root}" >&2
        exit 1
    fi

    local -a files=()
    while IFS= read -r -d '' f; do
        files+=("${f}")
    done < <(find "${src_root}" -type f -name '*.md' -print0 | LC_ALL=C sort -z)

    if [[ "${#files[@]}" -eq 0 ]]; then
        echo "no docs markdown files found to scan under: ${src_root}" >&2
        exit 1
    fi

    local failed=0
    local file_path
    for file_path in "${files[@]}"; do
        rel_path="${file_path#${src_root}/}"
        allowed="${DEFAULT_ALLOWED_LANGS_REGEX}"
        html_policy="strict"
        if [[ "${rel_path}" == contributors/* ]]; then
            allowed="${CONTRIB_ALLOWED_LANGS_REGEX}"
            html_policy="allow"
        fi

        if ! scan_file "${file_path}" "${rel_path}" "${allowed}" "${html_policy}"; then
            failed=1
        fi
    done

    if [[ "${failed}" -ne 0 ]]; then
        exit 1
    fi
}

main "$@"
