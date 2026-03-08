#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tools/docker/common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd curl
require_cmd docker

readonly COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.single.yml"
readonly TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/pgtuskmaster-docker-single.XXXXXX")"
readonly ENV_FILE="${TEMP_ROOT}/smoke-single.env"
readonly PROJECT_NAME="pgtuskmaster-smoke-single-$$"

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

write_smoke_env_file "single" "${ENV_FILE}" "${TEMP_ROOT}"
readonly API_PORT="$(grep '^PGTM_SINGLE_API_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly PG_PORT="$(grep '^PGTM_SINGLE_PG_PORT=' "${ENV_FILE}" | cut -d= -f2)"
readonly PSQL_PASSWORD="$(cat "${TEMP_ROOT}/secrets/postgres-superuser.password")"

log "building and starting the single-node stack"
docker compose \
  --project-name "${PROJECT_NAME}" \
  --env-file "${ENV_FILE}" \
  -f "${COMPOSE_FILE}" \
  up -d --build

wait_for_http_ok "http://127.0.0.1:${API_PORT}/ha/state" "single-node /ha/state" 120
wait_for_http_ok "http://127.0.0.1:${API_PORT}/debug/verbose" "single-node /debug/verbose" 120
wait_for_tcp_port "127.0.0.1" "${PG_PORT}" "single-node published PostgreSQL" 120
wait_for_sql_ready "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" "node-a" "${PSQL_PASSWORD}" 120
check_etcd_health "${COMPOSE_FILE}" "${ENV_FILE}" "${PROJECT_NAME}" >/dev/null

log "single-node smoke passed"
