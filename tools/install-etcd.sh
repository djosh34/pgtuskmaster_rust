#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

VERSION="${1:-v3.6.8}"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${ARCH}" in
    x86_64)
        ETCD_ARCH="amd64"
        ;;
    aarch64 | arm64)
        ETCD_ARCH="arm64"
        ;;
    *)
        echo "unsupported architecture: ${ARCH}" >&2
        exit 1
        ;;
esac

if [[ "${OS}" != "linux" ]]; then
    echo "unsupported OS: ${OS} (expected linux)" >&2
    exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
    echo "missing required command: curl" >&2
    exit 1
fi

TOOLS_DIR="${REPO_ROOT}/.tools"
DOWNLOAD_DIR="${TOOLS_DIR}/downloads"
ETCD_DIR="${TOOLS_DIR}/etcd"
BIN_DIR="${ETCD_DIR}/bin"

mkdir -p "${DOWNLOAD_DIR}" "${BIN_DIR}"

ARCHIVE_NAME="etcd-${VERSION}-${OS}-${ETCD_ARCH}.tar.gz"
RELEASE_DIR="etcd-${VERSION}-${OS}-${ETCD_ARCH}"
ARCHIVE_PATH="${DOWNLOAD_DIR}/${ARCHIVE_NAME}"
DOWNLOAD_URL="https://github.com/etcd-io/etcd/releases/download/${VERSION}/${ARCHIVE_NAME}"

echo "downloading ${DOWNLOAD_URL}"
curl -fL "${DOWNLOAD_URL}" -o "${ARCHIVE_PATH}"

echo "extracting ${ARCHIVE_NAME}"
tar -xzf "${ARCHIVE_PATH}" -C "${DOWNLOAD_DIR}"

cp "${DOWNLOAD_DIR}/${RELEASE_DIR}/etcd" "${BIN_DIR}/etcd"
chmod +x "${BIN_DIR}/etcd"

echo "installed: ${BIN_DIR}/etcd"
"${BIN_DIR}/etcd" --version | head -n 1
