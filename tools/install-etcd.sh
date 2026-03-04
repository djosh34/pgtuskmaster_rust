#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

PINNED_VERSION="v3.6.8"
VERSION="${1:-${PINNED_VERSION}}"
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

if [[ ! "${VERSION}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "invalid etcd version format: ${VERSION} (expected v<major>.<minor>.<patch>)" >&2
    exit 1
fi

for cmd in curl tar sha256sum mktemp cp chmod mv mkdir; do
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        echo "missing required command: ${cmd}" >&2
        exit 1
    fi
done

expected_sha256=""
case "${VERSION}" in
    v3.6.8)
        case "${ETCD_ARCH}" in
            amd64)
                expected_sha256="cf9cfe91a4856cb90eed9c99e6aee4b708db2c7888b88a6f116281f04b0ea693"
                ;;
            arm64)
                expected_sha256="438f56a700d17ce761510a3e63e6fa5c1d587b2dd4d7a22c179c09a649366760"
                ;;
            *)
                echo "unsupported etcd architecture: ${ETCD_ARCH}" >&2
                exit 1
                ;;
        esac
        ;;
    *)
        echo "unsupported etcd version for pinned checksum verification: ${VERSION} (pinned=${PINNED_VERSION})" >&2
        exit 1
        ;;
esac

verify_sha256() {
    local file_path="$1"
    local expected="$2"
    local actual=""
    if ! actual="$(sha256sum "${file_path}" | awk '{print $1}')"; then
        echo "sha256sum failed for ${file_path}" >&2
        exit 1
    fi
    if [[ "${actual}" != "${expected}" ]]; then
        echo "downloaded archive sha256 mismatch for ${file_path}: expected=${expected} actual=${actual}" >&2
        exit 1
    fi
}

TOOLS_DIR="${REPO_ROOT}/.tools"
DOWNLOAD_DIR="${TOOLS_DIR}/downloads"
ETCD_DIR="${TOOLS_DIR}/etcd"
BIN_DIR="${ETCD_DIR}/bin"

mkdir -p "${DOWNLOAD_DIR}" "${BIN_DIR}"

ARCHIVE_NAME="etcd-${VERSION}-${OS}-${ETCD_ARCH}.tar.gz"
RELEASE_DIR="etcd-${VERSION}-${OS}-${ETCD_ARCH}"
ARCHIVE_PATH="${DOWNLOAD_DIR}/${ARCHIVE_NAME}"
DOWNLOAD_URL="https://github.com/etcd-io/etcd/releases/download/${VERSION}/${ARCHIVE_NAME}"

if [[ -f "${ARCHIVE_PATH}" ]]; then
    echo "using cached archive ${ARCHIVE_PATH}"
    verify_sha256 "${ARCHIVE_PATH}" "${expected_sha256}"
else
    tmp_archive="${ARCHIVE_PATH}.tmp.$$"
    echo "downloading ${DOWNLOAD_URL}"
    curl -fL "${DOWNLOAD_URL}" -o "${tmp_archive}"
    verify_sha256 "${tmp_archive}" "${expected_sha256}"
    mv -f "${tmp_archive}" "${ARCHIVE_PATH}"
fi

tmp_dir="$(mktemp -d)"
cleanup() {
    rm -rf "${tmp_dir}"
}
trap cleanup EXIT

echo "extracting ${ARCHIVE_NAME}"
tar -xzf "${ARCHIVE_PATH}" -C "${tmp_dir}"

src_bin="${tmp_dir}/${RELEASE_DIR}/etcd"
if [[ ! -f "${src_bin}" ]]; then
    echo "expected extracted etcd binary missing: ${src_bin}" >&2
    exit 1
fi

dst_tmp="${BIN_DIR}/etcd.tmp.$$"
cp "${src_bin}" "${dst_tmp}"
chmod +x "${dst_tmp}"
mv -f "${dst_tmp}" "${BIN_DIR}/etcd"

echo "installed: ${BIN_DIR}/etcd"
expected_version="${VERSION#v}"
observed_version="$("${BIN_DIR}/etcd" --version | head -n 1)"
echo "${observed_version}"
if [[ "${observed_version}" != *"etcd Version: ${expected_version}"* ]]; then
    echo "unexpected etcd version string: ${observed_version} (expected etcd Version: ${expected_version})" >&2
    exit 1
fi
