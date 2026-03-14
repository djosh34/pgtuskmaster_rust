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

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

if [[ "${OS}" != "linux" ]]; then
    echo "unsupported OS: ${OS} (expected linux)" >&2
    exit 1
fi

if [[ ! -f /etc/os-release ]]; then
    echo "missing /etc/os-release; cannot validate distro" >&2
    exit 1
fi

os_id=""
os_id_like=""
while IFS="=" read -r k v; do
    case "${k}" in
        ID)
            os_id="${v%\"}"
            os_id="${os_id#\"}"
            ;;
        ID_LIKE)
            os_id_like="${v%\"}"
            os_id_like="${os_id_like#\"}"
            ;;
        *)
            ;;
    esac
done < /etc/os-release
if [[ "${os_id}" != "almalinux" && "${os_id_like}" != *"rhel"* ]]; then
    echo "this installer is for AlmaLinux/RHEL-like systems; found ID=${os_id:-unknown} ID_LIKE=${os_id_like:-unknown}" >&2
    exit 1
fi

sudo_path="$(require_cmd_path sudo)"
dnf_path="$(require_cmd_path dnf)"
mkdir_path="$(require_cmd_path mkdir)"
ln_path="$(require_cmd_path ln)"
readlink_path="$(require_cmd_path readlink)"
cp_path="$(require_cmd_path cp)"
chmod_path="$(require_cmd_path chmod)"
mv_path="$(require_cmd_path mv)"
rm_path="$(require_cmd_path rm)"
awk_path="$(require_cmd_path awk)"
sha256sum_path="$(require_cmd_path sha256sum)"
stat_path="$(require_cmd_path stat)"
flock_path="$(require_cmd_path flock)"
python3_path="$(require_cmd_path python3)"
date_path="$(require_cmd_path date)"
rpm_path="$(require_cmd_path rpm)"

TOOLS_DIR="${REPO_ROOT}/.tools"
INSTALL_DIR="${TOOLS_DIR}/postgres16"
BIN_DIR="${INSTALL_DIR}/bin"

"${mkdir_path}" -p "${BIN_DIR}"

echo "resetting/enabling postgres module stream 16"
"${sudo_path}" "${dnf_path}" -y module reset postgresql
"${sudo_path}" "${dnf_path}" -y module enable postgresql:16

echo "installing postgres16 packages via dnf"
"${sudo_path}" "${dnf_path}" -y install postgresql postgresql-server

assert_pg16() {
    local bin_path="$1"
    local version_out=""
    if ! version_out="$("${bin_path}" --version 2>&1)"; then
        echo "failed to run ${bin_path} --version: ${version_out}" >&2
        exit 1
    fi
    if [[ "${version_out}" != *"PostgreSQL) 16."* ]]; then
        echo "unexpected postgres major version for ${bin_path}: ${version_out}" >&2
        exit 1
    fi
}

resolve_bin() {
    local bin_name="$1"
    local candidates=(
        "/usr/pgsql-16/bin/${bin_name}"
        "/usr/lib/postgresql/16/bin/${bin_name}"
        "/usr/bin/${bin_name}"
    )
    for candidate in "${candidates[@]}"; do
        if [[ -x "${candidate}" ]]; then
            echo "${candidate}"
            return 0
        fi
    done

    echo "unable to locate installed binary in trusted locations: ${bin_name}" >&2
    exit 1
}

echo "verifying installed RPMs (fail-closed)"
"${sudo_path}" "${rpm_path}" -V postgresql postgresql-server

attestation_path="${TOOLS_DIR}/real-binaries-attestation.json"

for bin in postgres pg_ctl pg_rewind initdb psql pg_basebackup; do
    source_path="$(resolve_bin "${bin}")"
    assert_pg16 "${source_path}"
    dest_path="${BIN_DIR}/${bin}"

    source_real="$("${readlink_path}" -f -- "${source_path}")"
    "${ln_path}" -sf "${source_real}" "${dest_path}"
    dest_real="$("${readlink_path}" -f "${dest_path}")"
    echo "linked ${bin}: source=${source_real} dest=${dest_real}"

    rel_path=".tools/postgres16/bin/${bin}"
    abs_path="${REPO_ROOT}/${rel_path}"
    entry_json="$(attestation_entry_json "${sha256sum_path}" "${awk_path}" "${stat_path}" "${date_path}" "${python3_path}" "${readlink_path}" "${bin}" "${rel_path}" "${abs_path}")"
    update_attestation_manifest "${flock_path}" "${python3_path}" "${attestation_path}" "${entry_json}" "tools/install-postgres16.sh"
done

echo "linked postgres binaries in ${BIN_DIR}"
"${BIN_DIR}/postgres" --version

"${chmod_path}" 0644 "${attestation_path}"
echo "updated attestation: ${attestation_path}"
