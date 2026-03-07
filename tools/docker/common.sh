#!/usr/bin/env bash
set -euo pipefail

readonly TOOL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${TOOL_DIR}/../.." && pwd)"
readonly DEFAULT_ETCD_IMAGE="quay.io/coreos/etcd:v3.5.21"
readonly DEFAULT_PGTUSKMASTER_IMAGE="pgtuskmaster:local"

log() {
  printf '[tools/docker] %s\n' "$*"
}

require_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "${cmd}" >&2
    exit 1
  fi
}

ensure_temp_root() {
  local dir="$1"
  mkdir -p "${dir}"
}

pick_free_port() {
  python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

wait_for_http_ok() {
  local url="$1"
  local label="$2"
  local timeout_secs="$3"
  local deadline
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    if curl --silent --show-error --fail "${url}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  printf 'timed out waiting for %s at %s\n' "${label}" "${url}" >&2
  return 1
}

wait_for_tcp_port() {
  local host="$1"
  local port="$2"
  local label="$3"
  local timeout_secs="$4"
  local deadline
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    if bash -c ":</dev/tcp/${host}/${port}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  printf 'timed out waiting for %s at %s:%s\n' "${label}" "${host}" "${port}" >&2
  return 1
}

wait_for_ha_member_count() {
  local url="$1"
  local expected_count="$2"
  local timeout_secs="$3"
  local deadline
  local response
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    response="$(curl --silent --show-error --fail "${url}" 2>/dev/null || true)"
    if [[ -n "${response}" ]] && python3 -c '
import json
import sys

expected = int(sys.argv[1])
payload = json.load(sys.stdin)
member_count = payload.get("member_count")
phase = payload.get("ha_phase")
decision = payload.get("ha_decision")
if isinstance(member_count, int) and member_count >= expected and phase and decision:
    sys.exit(0)
sys.exit(1)
' "${expected_count}" <<<"${response}"
    then
      return 0
    fi
    sleep 1
  done
  printf 'timed out waiting for %s members at %s\n' "${expected_count}" "${url}" >&2
  return 1
}

wait_for_sql_ready() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local service_name="$4"
  local _password="$5"
  local timeout_secs="$6"
  local deadline
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    if docker compose \
      --project-name "${project_name}" \
      --env-file "${env_file}" \
      -f "${compose_file}" \
      exec -T "${service_name}" \
      /bin/bash -lc "/usr/lib/postgresql/16/bin/psql -h /var/lib/pgtuskmaster/socket -U postgres -d postgres -Atqc 'select 1'" \
      >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  printf 'timed out waiting for SQL readiness on service %s\n' "${service_name}" >&2
  return 1
}

check_etcd_health() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${compose_file}" \
    exec -T etcd \
    etcdctl --endpoints=http://127.0.0.1:2379 endpoint health
}

compose_down() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${compose_file}" \
    down -v --remove-orphans
}

write_smoke_env_file() {
  local mode="$1"
  local output_path="$2"
  local temp_root="$3"
  local secrets_dir="${temp_root}/secrets"
  local superuser_secret="${secrets_dir}/postgres-superuser.password"
  local replicator_secret="${secrets_dir}/replicator.password"
  local rewinder_secret="${secrets_dir}/rewinder.password"

  ensure_temp_root "${secrets_dir}"
  printf 'postgres-secret\n' >"${superuser_secret}"
  printf 'replicator-secret\n' >"${replicator_secret}"
  printf 'rewinder-secret\n' >"${rewinder_secret}"

  {
    printf 'PGTUSKMASTER_IMAGE=%s\n' "${PGTUSKMASTER_IMAGE:-${DEFAULT_PGTUSKMASTER_IMAGE}}"
    printf 'ETCD_IMAGE=%s\n' "${ETCD_IMAGE:-${DEFAULT_ETCD_IMAGE}}"
    printf 'PGTM_SECRET_SUPERUSER_FILE=%s\n' "${superuser_secret}"
    printf 'PGTM_SECRET_REPLICATOR_FILE=%s\n' "${replicator_secret}"
    printf 'PGTM_SECRET_REWINDER_FILE=%s\n' "${rewinder_secret}"
    if [[ "${mode}" == "single" ]]; then
      printf 'PGTM_SINGLE_API_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_SINGLE_PG_PORT=%s\n' "$(pick_free_port)"
    elif [[ "${mode}" == "cluster" ]]; then
      printf 'PGTM_CLUSTER_NODE_A_API_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_CLUSTER_NODE_A_PG_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_CLUSTER_NODE_B_API_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_CLUSTER_NODE_B_PG_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_CLUSTER_NODE_C_API_PORT=%s\n' "$(pick_free_port)"
      printf 'PGTM_CLUSTER_NODE_C_PG_PORT=%s\n' "$(pick_free_port)"
    else
      printf 'unsupported smoke env mode: %s\n' "${mode}" >&2
      exit 1
    fi
  } >"${output_path}"
}
