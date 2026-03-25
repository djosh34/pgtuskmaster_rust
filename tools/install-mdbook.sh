#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

PINNED_VERSION="v0.4.36"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [[ "${OS}" != "linux" ]]; then
    echo "unsupported OS: ${OS} (expected linux)" >&2
    exit 1
fi

require_cmd() {
    local cmd_name="$1"
    if ! command -v "${cmd_name}" >/dev/null 2>&1; then
        echo "missing required command: ${cmd_name}" >&2
        exit 1
    fi
}

sha256_file() {
    local file_path="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "${file_path}" | awk '{print $1}'
        return 0
    fi
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "${file_path}" | awk '{print $1}'
        return 0
    fi

    echo "missing required checksum tool: sha256sum (preferred) or shasum" >&2
    exit 1
}

require_cmd curl
require_cmd tar
require_cmd mktemp
require_cmd mkdir
require_cmd chmod

TARGET_TRIPLE=""
EXPECTED_SHA256=""
case "${ARCH}" in
    x86_64)
        TARGET_TRIPLE="x86_64-unknown-linux-gnu"
        EXPECTED_SHA256="72a50f5eecefef173114b53543a968fd1e8265ef67760b9d5bb20cd7712a9511"
        ;;
    aarch64 | arm64)
        # Upstream currently publishes an aarch64 linux-musl release asset.
        TARGET_TRIPLE="aarch64-unknown-linux-musl"
        EXPECTED_SHA256="dad8195fca7bac42b91cc9f7be12509425153df6c87d947699a4b04f9a84f844"
        ;;
    *)
        echo "unsupported architecture: ${ARCH}" >&2
        exit 1
        ;;
esac

TOOLS_DIR="${REPO_ROOT}/.tools"
DOWNLOAD_DIR="${TOOLS_DIR}/downloads"
INSTALL_DIR="${TOOLS_DIR}/mdbook"
BIN_DIR="${INSTALL_DIR}/bin"

mkdir -p "${DOWNLOAD_DIR}" "${BIN_DIR}"

ARCHIVE_NAME="mdbook-${PINNED_VERSION}-${TARGET_TRIPLE}.tar.gz"
DOWNLOAD_URL="https://github.com/rust-lang/mdBook/releases/download/${PINNED_VERSION}/${ARCHIVE_NAME}"

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"
EXTRACT_DIR="${TMP_DIR}/extract"

echo "downloading ${DOWNLOAD_URL}"
curl -fL "${DOWNLOAD_URL}" -o "${ARCHIVE_PATH}"

ACTUAL_SHA256="$(sha256_file "${ARCHIVE_PATH}")"
echo "checksum sha256 expected=${EXPECTED_SHA256}"
echo "checksum sha256 actual=${ACTUAL_SHA256}"
if [[ "${ACTUAL_SHA256}" != "${EXPECTED_SHA256}" ]]; then
    echo "checksum verification failed for ${ARCHIVE_NAME}" >&2
    exit 1
fi

mkdir -p "${EXTRACT_DIR}"
echo "extracting ${ARCHIVE_NAME}"
tar -xzf "${ARCHIVE_PATH}" -C "${EXTRACT_DIR}"

if [[ ! -f "${EXTRACT_DIR}/mdbook" ]]; then
    echo "expected mdbook binary at ${EXTRACT_DIR}/mdbook but it was not found" >&2
    echo "extracted files:" >&2
    (cd "${EXTRACT_DIR}" && find . -maxdepth 2 -type f -print) >&2
    exit 1
fi

INSTALL_TMP_PATH="${BIN_DIR}/mdbook.tmp.$$"
cp "${EXTRACT_DIR}/mdbook" "${INSTALL_TMP_PATH}"
chmod +x "${INSTALL_TMP_PATH}"
mv -f "${INSTALL_TMP_PATH}" "${BIN_DIR}/mdbook"

echo "installed: ${BIN_DIR}/mdbook"
INSTALLED_VERSION="$("${BIN_DIR}/mdbook" --version | awk '{print $2}')"
if [[ "${INSTALLED_VERSION}" != "${PINNED_VERSION}" ]]; then
    echo "installed mdBook version mismatch: expected=${PINNED_VERSION} actual=${INSTALLED_VERSION}" >&2
    exit 1
fi
echo "$("${BIN_DIR}/mdbook" --version)"
