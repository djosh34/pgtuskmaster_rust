#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../../.." && pwd)"
ASK_K2="${REPO_ROOT}/.agents/skills/ask-k2/ask-k2.sh"
SUMMARY_PROMPT="${SCRIPT_DIR}/summary-prompt.md"
DIATAXIS_SUMMARY="${SCRIPT_DIR}/diataxis-summary.md"
CHOOSE_PROMPT="${SCRIPT_DIR}/choose-doc-prompt.md"
WRITE_PROMPT="${SCRIPT_DIR}/write-doc-prompt.md"

usage() {
  cat <<'EOF'
Usage:
  k2-docs-loop.sh summarize-diataxis
  k2-docs-loop.sh choose-doc
  k2-docs-loop.sh prepare-draft <docs/path.md> [full-context-file ...]
EOF
}

list_docs_src() {
  (
    cd "${REPO_ROOT}"
    printf '# docs/src file listing\n\n'
    rg --files docs/src | sort
  )
}

full_docs_src() {
  (
    cd "${REPO_ROOT}"
    printf '# full docs/src file contents\n'
    while IFS= read -r path; do
      printf '\n\n===== %s =====\n' "${path}"
      cat "${path}"
    done < <(rg --files docs/src | sort)
  )
}

docs_summary_context() {
  (
    cd "${REPO_ROOT}"
    printf '# current docs summary context\n\n'
    if [[ -f docs/src/SUMMARY.md ]]; then
      printf '===== docs/src/SUMMARY.md =====\n'
      cat docs/src/SUMMARY.md
      printf '\n'
    else
      printf 'missing source support\n'
    fi
  )
}

list_src_and_tests() {
  (
    cd "${REPO_ROOT}"
    printf '# src and test file listing\n\n'
    {
      rg --files src
      if [[ -d tests ]]; then
        rg --files tests
      fi
    } | sort
  )
}

list_support_files() {
  (
    cd "${REPO_ROOT}"
    printf '# docker and docs support file listing\n\n'
    {
      if [[ -d docker ]]; then
        rg --files docker
      fi
      rg --files docs
    } | sort
  )
}

project_manifests() {
  (
    cd "${REPO_ROOT}"
    printf '# project manifests and docs config\n\n'
    for path in Cargo.toml docs/book.toml; do
      if [[ -f "${path}" ]]; then
        printf '===== %s =====\n' "${path}"
        cat "${path}"
        printf '\n\n'
      fi
    done
  )
}

raw_diataxis_corpus() {
  (
    cd "${REPO_ROOT}"
    for f in $(rg --files .agents/skills/diataxis/references -g '*.rst' | sort); do
      printf '\n===== %s =====\n' "${f}"
      cat "${f}"
    done
  )
}

summarize_diataxis() {
  (
    cd "${REPO_ROOT}"
    {
      cat "${SUMMARY_PROMPT}"
      printf '\n'
      raw_diataxis_corpus
    } | "${ASK_K2}" > "${DIATAXIS_SUMMARY}"
  )
  printf '%s\n' "${DIATAXIS_SUMMARY}"
}

choose_doc() {
  (
    cd "${REPO_ROOT}"
    {
      cat "${CHOOSE_PROMPT}"
      printf '\n\n'
      list_docs_src
      printf '\n\n'
      full_docs_src
      printf '\n\n# diataxis summary markdown\n\n'
      cat "${DIATAXIS_SUMMARY}"
      printf '\n\n'
      project_manifests
      printf '\n\n'
      list_src_and_tests
      printf '\n\n'
      list_support_files
    } | tee /tmp/k2-docs-loop-choose-doc-prompt.md | "${ASK_K2}"
  )
}

prepare_draft() {
  if [[ $# -lt 1 ]]; then
    usage
    exit 1
  fi

  local target_rel="$1"
  shift

  local target_draft="${REPO_ROOT}/docs/draft/${target_rel}"
  local tmp_prompt="${REPO_ROOT}/docs/tmp/${target_rel%.md}.prompt.md"

  mkdir -p "$(dirname "${target_draft}")" "$(dirname "${tmp_prompt}")"

  {
    cat "${WRITE_PROMPT}"
    printf '\n\n# target docs path\n\n%s\n' "${target_rel}"
    printf '\n# docs/src file listing\n\n'
    list_docs_src
    printf '\n\n'
    docs_summary_context
    printf '\n\n# diataxis summary markdown\n\n'
    cat "${DIATAXIS_SUMMARY}"
    printf '\n\n'
    project_manifests
    printf '\n\n# src and test file listing\n\n'
    list_src_and_tests
    printf '\n\n'
    list_support_files
    if [[ $# -gt 0 ]]; then
      for path in "$@"; do
        printf '\n\n===== %s =====\n' "${path}"
        cat "${REPO_ROOT}/${path}"
      done
    fi
  } | tee "${tmp_prompt}" | "${ASK_K2}" > "${target_draft}"

  printf '%s\n%s\n' "${tmp_prompt}" "${target_draft}"
}

main() {
  if [[ $# -lt 1 ]]; then
    usage
    exit 1
  fi

  case "$1" in
    summarize-diataxis)
      shift
      summarize_diataxis "$@"
      ;;
    choose-doc)
      shift
      choose_doc "$@"
      ;;
    prepare-draft)
      shift
      prepare_draft "$@"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "$@"
