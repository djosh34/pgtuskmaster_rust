#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

if [[ "${OS}" != "linux" ]]; then
    echo "unsupported OS: ${OS} (expected linux)" >&2
    exit 1
fi

if [[ ! -f /etc/os-release ]]; then
    echo "missing /etc/os-release; cannot validate distro" >&2
    exit 1
fi

# shellcheck source=/dev/null
source /etc/os-release
if [[ "${ID:-}" != "almalinux" && "${ID_LIKE:-}" != *"rhel"* ]]; then
    echo "this installer is for AlmaLinux/RHEL-like systems; found ID=${ID:-unknown}" >&2
    exit 1
fi

for cmd in sudo dnf ln mkdir readlink; do
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        echo "missing required command: ${cmd}" >&2
        exit 1
    fi
done

TOOLS_DIR="${REPO_ROOT}/.tools"
INSTALL_DIR="${TOOLS_DIR}/postgres16"
BIN_DIR="${INSTALL_DIR}/bin"

mkdir -p "${BIN_DIR}"

echo "resetting/enabling postgres module stream 16"
sudo dnf -y module reset postgresql
sudo dnf -y module enable postgresql:16

echo "installing postgres16 packages via dnf"
sudo dnf -y install postgresql postgresql-server

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

for bin in postgres pg_ctl pg_rewind initdb psql pg_basebackup; do
    source_path="$(resolve_bin "${bin}")"
    assert_pg16 "${source_path}"
    source_real="$(readlink -f "${source_path}")"
    dest_path="${BIN_DIR}/${bin}"
    ln -sf "${source_path}" "${dest_path}"
    dest_real="$(readlink -f "${dest_path}")"
    echo "linked ${bin}: source=${source_real} dest=${dest_real}"
done

echo "linked postgres binaries in ${BIN_DIR}"
"${BIN_DIR}/postgres" --version
