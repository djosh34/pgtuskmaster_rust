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

export PATH="${HOME}/.cargo/bin:${PATH}"
export INSTALL_ALLOW_PATH_PREFIXES="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${HOME}/.cargo/bin"

strace_path="$(require_cmd_path strace)"
cargo_path="$(require_cmd_path cargo)"
mkdir_path="$(require_cmd_path mkdir)"
grep_path="$(require_cmd_path grep)"
date_path="$(require_cmd_path date)"

test_name="${1:-ha::e2e_multi_node::e2e_no_quorum_enters_failsafe_strict_all_nodes}"

run_id="$("${date_path}" -u +%Y%m%dT%H%M%SZ)-$$"
evidence_dir="${REPO_ROOT}/.ralph/evidence/bug-real-binary-provenance-enforcement-gaps/execve/${run_id}"
"${mkdir_path}" -p "${evidence_dir}"

echo "execve trace evidence dir: ${evidence_dir}"

echo "prebuilding test binaries (avoid tracing compilation)"
env CARGO_INCREMENTAL=0 "${cargo_path}" test --all-targets --no-run

trace_prefix="${evidence_dir}/strace"
echo "running ${test_name} under strace (execve/execveat)"
"${strace_path}" -ff -e trace=execve,execveat -s 256 -o "${trace_prefix}" \
    env CARGO_INCREMENTAL=0 "${cargo_path}" test --all-targets "${test_name}" -- --exact

expected_etcd="${REPO_ROOT}/.tools/etcd/bin/etcd"
expected_postgres="${REPO_ROOT}/.tools/postgres16/bin/postgres"
expected_initdb="${REPO_ROOT}/.tools/postgres16/bin/initdb"

echo "checking execve evidence"
if ! "${grep_path}" -R -F "execve(\"${expected_etcd}\"" "${evidence_dir}" >/dev/null 2>&1; then
    die "expected execve of ${expected_etcd} was not observed in ${evidence_dir}"
fi
if ! "${grep_path}" -R -F "execve(\"${expected_postgres}\"" "${evidence_dir}" >/dev/null 2>&1; then
    die "expected execve of ${expected_postgres} was not observed in ${evidence_dir}"
fi
if ! "${grep_path}" -R -F "execve(\"${expected_initdb}\"" "${evidence_dir}" >/dev/null 2>&1; then
    die "expected execve of ${expected_initdb} was not observed in ${evidence_dir}"
fi

echo "execve trace evidence passed"
