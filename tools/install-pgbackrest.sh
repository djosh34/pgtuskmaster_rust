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
chmod_path="$(require_cmd_path chmod)"
flock_path="$(require_cmd_path flock)"
python3_path="$(require_cmd_path python3)"
date_path="$(require_cmd_path date)"
sha256sum_path="$(require_cmd_path sha256sum)"
stat_path="$(require_cmd_path stat)"
awk_path="$(require_cmd_path awk)"
rpm_path="$(require_cmd_path rpm)"

TOOLS_DIR="${REPO_ROOT}/.tools"
INSTALL_DIR="${TOOLS_DIR}/pgbackrest"
BIN_DIR="${INSTALL_DIR}/bin"

"${mkdir_path}" -p "${BIN_DIR}"

echo "enabling repos required for pgBackRest (crb + epel when available)"
if "${sudo_path}" "${dnf_path}" -y install dnf-plugins-core; then
    if "${sudo_path}" "${dnf_path}" -y config-manager --set-enabled crb; then
        :
    else
        echo "warning: failed to enable crb repo (continuing; pgBackRest package may be unavailable)" >&2
    fi
else
    echo "warning: failed to install dnf-plugins-core (continuing; cannot enable crb automatically)" >&2
fi

if "${sudo_path}" "${dnf_path}" -y install epel-release; then
    if "${sudo_path}" "${dnf_path}" -y config-manager --set-enabled epel; then
        :
    else
        echo "warning: failed to enable epel repo (continuing; dependencies like libssh2 may be unavailable)" >&2
    fi
else
    echo "warning: failed to install epel-release (continuing; pgBackRest package may be unavailable)" >&2
fi

echo "installing pgbackrest package via dnf"
if ! "${sudo_path}" "${dnf_path}" -y install pgbackrest; then
    arch="$(uname -m)"
    case "${arch}" in
        x86_64|aarch64)
            ;;
        *)
            echo "unsupported arch for PGDG repo bootstrap: ${arch}" >&2
            exit 1
            ;;
    esac

    pgdg_repo_rpm="https://download.postgresql.org/pub/repos/yum/reporpms/EL-9-${arch}/pgdg-redhat-repo-latest.noarch.rpm"
    echo "pgbackrest not found in current enabled repos; bootstrapping PostgreSQL Global Development Group (PGDG) repo: ${pgdg_repo_rpm}"
    if ! "${sudo_path}" "${dnf_path}" -y install "${pgdg_repo_rpm}"; then
        echo "failed to install PGDG repo RPM; cannot install pgbackrest" >&2
        exit 1
    fi

    echo "refreshing dnf metadata after repo install"
    if ! "${sudo_path}" "${dnf_path}" -y makecache; then
        echo "warning: dnf makecache failed (continuing; install may still work)" >&2
    fi

    echo "retrying pgbackrest install via dnf (with PGDG repo enabled)"
    if ! "${sudo_path}" "${dnf_path}" -y install pgbackrest; then
        echo "failed to install pgbackrest via dnf even after enabling PGDG repo. Check repo availability and package name on this distro." >&2
        exit 1
    fi
fi

echo "verifying installed RPMs (fail-closed when possible)"
if ! "${sudo_path}" "${rpm_path}" -V pgbackrest; then
    echo "rpm verification failed for pgbackrest (rpm -V); refusing to continue" >&2
    exit 1
fi

resolve_bin() {
    local bin_name="$1"
    local candidates=(
        "/usr/bin/${bin_name}"
        "/usr/local/bin/${bin_name}"
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

assert_pgbackrest_major() {
    local bin_path="$1"
    local version_out=""
    if ! version_out="$("${bin_path}" --version 2>&1)"; then
        echo "failed to run ${bin_path} --version: ${version_out}" >&2
        exit 1
    fi
    if [[ "${version_out}" != *"pgBackRest 2."* ]]; then
        echo "unexpected pgBackRest major version for ${bin_path}: ${version_out}" >&2
        exit 1
    fi
}

source_path="$(resolve_bin "pgbackrest")"
assert_pgbackrest_major "${source_path}"

dest_path="${BIN_DIR}/pgbackrest"
source_real="$("${readlink_path}" -f -- "${source_path}" 2>/dev/null || true)"
if [[ -z "${source_real}" ]]; then
    echo "failed to resolve pgbackrest binary path: ${source_path}" >&2
    exit 1
fi
"${ln_path}" -sf "${source_real}" "${dest_path}"
dest_real="$("${readlink_path}" -f "${dest_path}")"
echo "linked pgbackrest: source=${source_real} dest=${dest_real}"

attestation_path="${TOOLS_DIR}/real-binaries-attestation.json"
rel_path=".tools/pgbackrest/bin/pgbackrest"
abs_path="${REPO_ROOT}/${rel_path}"
entry_json="$(attestation_entry_json "${sha256sum_path}" "${awk_path}" "${stat_path}" "${date_path}" "${python3_path}" "${readlink_path}" "pgbackrest" "${rel_path}" "${abs_path}")"
update_attestation_manifest "${flock_path}" "${python3_path}" "${attestation_path}" "${entry_json}" "tools/install-pgbackrest.sh"

"${chmod_path}" 0644 "${attestation_path}"
echo "updated attestation: ${attestation_path}"

echo "pgbackrest installed and linked in ${BIN_DIR}"
"${BIN_DIR}/pgbackrest" --version
