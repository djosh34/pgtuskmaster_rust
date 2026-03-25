#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

PINNED_VERSION="0.14.0"

require_cmd() {
    local cmd_name="$1"
    if ! command -v "${cmd_name}" >/dev/null 2>&1; then
        echo "missing required command: ${cmd_name}" >&2
        exit 1
    fi
}

require_cmd cargo
require_cmd mkdir
require_cmd chmod
require_cmd mv

TOOLS_DIR="${REPO_ROOT}/.tools"
INSTALL_DIR="${TOOLS_DIR}/mdbook"
BIN_DIR="${INSTALL_DIR}/bin"
TARGET_BIN="${BIN_DIR}/mdbook-mermaid"

mkdir -p "${BIN_DIR}"

if [[ -x "${TARGET_BIN}" ]]; then
    INSTALLED_VERSION="$("${TARGET_BIN}" --version | awk '{print $2}')"
    if [[ "${INSTALLED_VERSION}" == "${PINNED_VERSION}" ]]; then
        echo "already installed: ${TARGET_BIN}"
        echo "$("${TARGET_BIN}" --version)"
        exit 0
    fi
fi

TMP_BIN="${TARGET_BIN}.tmp.$$"

echo "installing mdbook-mermaid ${PINNED_VERSION} into ${BIN_DIR}"
cargo install mdbook-mermaid \
    --version "${PINNED_VERSION}" \
    --locked \
    --root "${INSTALL_DIR}" \
    --force

if [[ ! -x "${TARGET_BIN}" ]]; then
    echo "expected mdbook-mermaid at ${TARGET_BIN} but it was not found" >&2
    exit 1
fi

mv -f "${TARGET_BIN}" "${TMP_BIN}"
chmod +x "${TMP_BIN}"
mv -f "${TMP_BIN}" "${TARGET_BIN}"

echo "installed: ${TARGET_BIN}"
echo "$("${TARGET_BIN}" --version)"
