#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

COMMON_SH="${SCRIPT_DIR}/install-common.sh"
if [[ ! -f "${COMMON_SH}" ]]; then
    echo "missing required helper: ${COMMON_SH}" >&2
    exit 1
fi
# shellcheck source=/dev/null
source "${COMMON_SH}"

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

curl_path="$(require_cmd_path curl)"
tar_path="$(require_cmd_path tar)"
sha256sum_path="$(require_cmd_path sha256sum)"
mktemp_path="$(require_cmd_path mktemp)"
cp_path="$(require_cmd_path cp)"
chmod_path="$(require_cmd_path chmod)"
mv_path="$(require_cmd_path mv)"
mkdir_path="$(require_cmd_path mkdir)"
awk_path="$(require_cmd_path awk)"
stat_path="$(require_cmd_path stat)"
rm_path="$(require_cmd_path rm)"
readlink_path="$(require_cmd_path readlink)"
flock_path="$(require_cmd_path flock)"
python3_path="$(require_cmd_path python3)"
date_path="$(require_cmd_path date)"

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
    if ! actual="$(sha256_file "${sha256sum_path}" "${awk_path}" "${file_path}")"; then
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

"${mkdir_path}" -p "${DOWNLOAD_DIR}" "${BIN_DIR}"

ARCHIVE_NAME="etcd-${VERSION}-${OS}-${ETCD_ARCH}.tar.gz"
RELEASE_DIR="etcd-${VERSION}-${OS}-${ETCD_ARCH}"
ARCHIVE_PATH="${DOWNLOAD_DIR}/${ARCHIVE_NAME}"
DOWNLOAD_URL="https://github.com/etcd-io/etcd/releases/download/${VERSION}/${ARCHIVE_NAME}"

if [[ -f "${ARCHIVE_PATH}" ]]; then
    echo "using cached archive ${ARCHIVE_PATH}"
    verify_sha256 "${ARCHIVE_PATH}" "${expected_sha256}"
else
    tmp_archive="$("${mktemp_path}" "${ARCHIVE_PATH}.tmp.XXXXXX")"
    echo "downloading ${DOWNLOAD_URL}"
    "${curl_path}" \
        --fail \
        --location \
        --show-error \
        --retry 3 \
        --retry-delay 1 \
        --proto '=https' \
        --tlsv1.2 \
        "${DOWNLOAD_URL}" \
        -o "${tmp_archive}"
    verify_sha256 "${tmp_archive}" "${expected_sha256}"
    "${mv_path}" -f "${tmp_archive}" "${ARCHIVE_PATH}"
fi

tmp_dir="$("${mktemp_path}" -d)"
cleanup() {
    "${rm_path}" -rf "${tmp_dir}"
}
trap cleanup EXIT

echo "extracting ${ARCHIVE_NAME}"
"${tar_path}" -xzf "${ARCHIVE_PATH}" -C "${tmp_dir}"

src_bin="${tmp_dir}/${RELEASE_DIR}/etcd"
if [[ ! -f "${src_bin}" ]]; then
    echo "expected extracted etcd binary missing: ${src_bin}" >&2
    exit 1
fi

install_copy_atomic "${cp_path}" "${chmod_path}" "${mv_path}" "${mkdir_path}" "${src_bin}" "${BIN_DIR}/etcd"

echo "installed: ${BIN_DIR}/etcd"
expected_version="${VERSION#v}"
observed_version="$("${BIN_DIR}/etcd" --version | { IFS= read -r line; printf '%s\n' "${line}"; })"
echo "${observed_version}"
if [[ "${observed_version}" != *"etcd Version: ${expected_version}"* ]]; then
    echo "unexpected etcd version string: ${observed_version} (expected etcd Version: ${expected_version})" >&2
    exit 1
fi

attestation_path="${TOOLS_DIR}/real-binaries-attestation.json"
rel_path=".tools/etcd/bin/etcd"
abs_path="${REPO_ROOT}/${rel_path}"
entry_json="$(attestation_entry_json "${sha256sum_path}" "${awk_path}" "${stat_path}" "${date_path}" "${python3_path}" "${readlink_path}" "etcd" "${rel_path}" "${abs_path}")"
update_attestation_manifest "${flock_path}" "${python3_path}" "${attestation_path}" "${entry_json}" "tools/install-etcd.sh"
"${chmod_path}" 0644 "${attestation_path}"

echo "updated attestation: ${attestation_path}"
