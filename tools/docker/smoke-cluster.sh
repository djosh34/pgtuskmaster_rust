#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tools/docker/common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd curl
require_cmd docker

readonly COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.cluster.yml"
readonly TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/pgtuskmaster-docker-cluster.XXXXXX")"
readonly ENV_FILE="${TEMP_ROOT}/smoke-cluster.env"
readonly PROJECT_NAME="pgtuskmaster-smoke-cluster-$$"

cleanup() {
  local exit_status=$?
  local cleanup_output=""

  if [[ -f "${ENV_FILE}" ]]; then
    if ! cleanup_output="$(compose_down_with_diagnostics "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}")"; then
      printf 'failed to tear down smoke project %s\n' "${PROJECT_NAME}" >&2
      if [[ -n "${cleanup_output}" ]]; then
        printf '%s\n' "${cleanup_output}" >&2
      fi
    fi
  fi

  rm -rf "${TEMP_ROOT}"
  return "${exit_status}"
}
trap cleanup EXIT

write_smoke_env_file "cluster" "${ENV_FILE}" "${TEMP_ROOT}"
readonly PSQL_PASSWORD="$(cat "${TEMP_ROOT}/secrets/postgres-superuser.password")"

log "building and starting the cluster stack"
docker compose \
  --project-name "${PROJECT_NAME}" \
  --env-file "${ENV_FILE}" \
  -f "${COMPOSE_FILE}" \
  up -d --build

for node_name in "${CLUSTER_NODE_NAMES[@]}"; do
  wait_for_http_ok "$(cluster_ha_state_url "${ENV_FILE}" "${node_name}")" "cluster ${node_name} /ha/state" 180
  wait_for_http_ok "$(cluster_debug_url "${ENV_FILE}" "${node_name}")" "cluster ${node_name} /debug/verbose" 180
  wait_for_tcp_port "127.0.0.1" "$(cluster_pg_port_from_env "${ENV_FILE}" "${node_name}")" "cluster ${node_name} published PostgreSQL" 180
  wait_for_ha_member_count "$(cluster_ha_state_url "${ENV_FILE}" "${node_name}")" 3 180
  wait_for_sql_ready "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "${node_name}" "${PSQL_PASSWORD}" 180
done
  wait_for_cluster_replication_roles "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "${PSQL_PASSWORD}" 180
check_etcd_health "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" >/dev/null

log "cluster smoke passed"
