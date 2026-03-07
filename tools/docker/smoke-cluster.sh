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
  compose_down "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" >/dev/null 2>&1 || true
  rm -rf "${TEMP_ROOT}"
}
trap cleanup EXIT

write_smoke_env_file "cluster" "${ENV_FILE}" "${TEMP_ROOT}"
readonly NODE_A_API_PORT="$(grep '^PGTM_CLUSTER_NODE_A_API_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly NODE_A_PG_PORT="$(grep '^PGTM_CLUSTER_NODE_A_PG_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly NODE_B_API_PORT="$(grep '^PGTM_CLUSTER_NODE_B_API_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly NODE_B_PG_PORT="$(grep '^PGTM_CLUSTER_NODE_B_PG_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly NODE_C_API_PORT="$(grep '^PGTM_CLUSTER_NODE_C_API_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly NODE_C_PG_PORT="$(grep '^PGTM_CLUSTER_NODE_C_PG_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly PSQL_PASSWORD="$(cat "${TEMP_ROOT}/secrets/postgres-superuser.password")"

query_pg_is_in_recovery() {
  local service_name="$1"
  docker compose \
    --project-name "${PROJECT_NAME}" \
    --env-file "${ENV_FILE}" \
    -f "${COMPOSE_FILE}" \
    exec -T "${service_name}" \
    /bin/bash -lc "/usr/lib/postgresql/16/bin/psql -h /var/lib/pgtuskmaster/socket -U postgres -d postgres -Atqc 'select pg_is_in_recovery()'" \
    2>/dev/null
}

wait_for_cluster_replication_roles() {
  local timeout_secs="$1"
  local deadline
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    local a b c
    if ! a="$(query_pg_is_in_recovery node-a)"; then
      sleep 1
      continue
    fi
    if ! b="$(query_pg_is_in_recovery node-b)"; then
      sleep 1
      continue
    fi
    if ! c="$(query_pg_is_in_recovery node-c)"; then
      sleep 1
      continue
    fi

    a="$(echo "${a}" | tr -d '\r' | tail -n 1)"
    b="$(echo "${b}" | tr -d '\r' | tail -n 1)"
    c="$(echo "${c}" | tr -d '\r' | tail -n 1)"

    if [[ "${a}" != "t" && "${a}" != "f" ]]; then
      sleep 1
      continue
    fi
    if [[ "${b}" != "t" && "${b}" != "f" ]]; then
      sleep 1
      continue
    fi
    if [[ "${c}" != "t" && "${c}" != "f" ]]; then
      sleep 1
      continue
    fi

    local primaries=0
    local replicas=0
    for value in "${a}" "${b}" "${c}"; do
      if [[ "${value}" == "f" ]]; then
        primaries=$((primaries + 1))
      else
        replicas=$((replicas + 1))
      fi
    done

    if [[ "${primaries}" -eq 1 && "${replicas}" -eq 2 ]]; then
      return 0
    fi

    sleep 1
  done

  printf 'timed out waiting for cluster replication roles: expected 1 primary + 2 replicas (pg_is_in_recovery)\n' >&2
  return 1
}

log "building and starting the cluster stack"
docker compose \
  --project-name "${PROJECT_NAME}" \
  --env-file "${ENV_FILE}" \
  -f "${COMPOSE_FILE}" \
  up -d --build

wait_for_http_ok "http://127.0.0.1:${NODE_A_API_PORT}/ha/state" "cluster node-a /ha/state" 180
wait_for_http_ok "http://127.0.0.1:${NODE_B_API_PORT}/ha/state" "cluster node-b /ha/state" 180
wait_for_http_ok "http://127.0.0.1:${NODE_C_API_PORT}/ha/state" "cluster node-c /ha/state" 180
wait_for_http_ok "http://127.0.0.1:${NODE_A_API_PORT}/debug/verbose" "cluster node-a /debug/verbose" 180
wait_for_http_ok "http://127.0.0.1:${NODE_B_API_PORT}/debug/verbose" "cluster node-b /debug/verbose" 180
wait_for_http_ok "http://127.0.0.1:${NODE_C_API_PORT}/debug/verbose" "cluster node-c /debug/verbose" 180
wait_for_tcp_port "127.0.0.1" "${NODE_A_PG_PORT}" "cluster node-a published PostgreSQL" 180
wait_for_tcp_port "127.0.0.1" "${NODE_B_PG_PORT}" "cluster node-b published PostgreSQL" 180
wait_for_tcp_port "127.0.0.1" "${NODE_C_PG_PORT}" "cluster node-c published PostgreSQL" 180
wait_for_ha_member_count "http://127.0.0.1:${NODE_A_API_PORT}/ha/state" 3 180
wait_for_ha_member_count "http://127.0.0.1:${NODE_B_API_PORT}/ha/state" 3 180
wait_for_ha_member_count "http://127.0.0.1:${NODE_C_API_PORT}/ha/state" 3 180
wait_for_sql_ready "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "node-a" "${PSQL_PASSWORD}" 180
wait_for_sql_ready "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "node-b" "${PSQL_PASSWORD}" 180
wait_for_sql_ready "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "node-c" "${PSQL_PASSWORD}" 180
wait_for_cluster_replication_roles 180
check_etcd_health "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" >/dev/null

log "cluster smoke passed"
