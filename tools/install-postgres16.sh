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

for cmd in sudo dnf ln mkdir; do
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

resolve_bin() {
    local bin_name="$1"
    local candidate=""

    if command -v "${bin_name}" >/dev/null 2>&1; then
        candidate="$(command -v "${bin_name}")"
    fi
    if [[ -z "${candidate}" && -x "/usr/pgsql-16/bin/${bin_name}" ]]; then
        candidate="/usr/pgsql-16/bin/${bin_name}"
    fi
    if [[ -z "${candidate}" && -x "/usr/lib/postgresql/16/bin/${bin_name}" ]]; then
        candidate="/usr/lib/postgresql/16/bin/${bin_name}"
    fi
    if [[ -z "${candidate}" && -x "/usr/bin/${bin_name}" ]]; then
        candidate="/usr/bin/${bin_name}"
    fi

    if [[ -z "${candidate}" ]]; then
        echo "unable to locate installed binary: ${bin_name}" >&2
        exit 1
    fi

    echo "${candidate}"
}

for bin in postgres pg_ctl pg_rewind initdb psql pg_basebackup; do
    source_path="$(resolve_bin "${bin}")"
    ln -sf "${source_path}" "${BIN_DIR}/${bin}"
done

echo "linked postgres binaries in ${BIN_DIR}"
"${BIN_DIR}/postgres" --version
