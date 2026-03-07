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
check_etcd_health "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" >/dev/null

log "cluster smoke passed"
