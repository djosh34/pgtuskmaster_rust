#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tools/docker/common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd curl
require_cmd docker
require_cmd python3

readonly COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.cluster.yml"
readonly DEFAULT_ENV_FILE="${REPO_ROOT}/.env.docker"
readonly FALLBACK_ENV_FILE="${REPO_ROOT}/.env.docker.example"
readonly DEFAULT_PROJECT_NAME="pgtuskmaster-cluster"

usage() {
  cat <<'EOF'
Usage:
  tools/docker/cluster.sh <up|status|down> [--env-file PATH] [--project-name NAME]

Commands:
  up       Build and start the persistent local HA cluster, wait for readiness, then print endpoints and topology.
  status   Inspect the running cluster and print endpoints and topology without recreating it.
  down     Tear down the persistent local HA cluster and remove its volumes.

Options:
  --env-file PATH      Environment file to use. Defaults to .env.docker when present, otherwise .env.docker.example.
  --project-name NAME  Docker Compose project name. Defaults to pgtuskmaster-cluster.
  --help               Show this help text.
EOF
}

resolve_default_env_file() {
  if [[ -f "${DEFAULT_ENV_FILE}" ]]; then
    printf '%s\n' "${DEFAULT_ENV_FILE}"
    return 0
  fi
  if [[ -f "${FALLBACK_ENV_FILE}" ]]; then
    printf '%s\n' "${FALLBACK_ENV_FILE}"
    return 0
  fi
  printf 'missing both %s and %s\n' "${DEFAULT_ENV_FILE}" "${FALLBACK_ENV_FILE}" >&2
  exit 1
}

wait_for_cluster_readiness() {
  local env_file="$1"
  local project_name="$2"
  local node_name
  for node_name in "${CLUSTER_NODE_NAMES[@]}"; do
    wait_for_http_ok "$(cluster_ha_state_url "${env_file}" "${node_name}")" "${node_name} /ha/state" 180
    wait_for_http_ok "$(cluster_debug_url "${env_file}" "${node_name}")" "${node_name} /debug/verbose" 180
    wait_for_tcp_port "127.0.0.1" "$(cluster_pg_port_from_env "${env_file}" "${node_name}")" "${node_name} published PostgreSQL" 180
    wait_for_ha_member_count "$(cluster_ha_state_url "${env_file}" "${node_name}")" 3 180
    wait_for_sql_ready "${COMPOSE_FILE}" "${env_file}" "${project_name}" "${node_name}" "<unused>" 180
  done
  wait_for_cluster_replication_roles "${COMPOSE_FILE}" "${env_file}" "${project_name}" 180
  check_etcd_health "${COMPOSE_FILE}" "${env_file}" "${project_name}" >/dev/null
}

ensure_cluster_services_running() {
  local env_file="$1"
  local project_name="$2"
  local running_services
  running_services="$(compose_running_service_names "${COMPOSE_FILE}" "${env_file}" "${project_name}")"
  if [[ -z "${running_services}" ]]; then
    printf 'cluster stack is not running for project %s; start it with tools/docker/cluster.sh up\n' "${project_name}" >&2
    exit 1
  fi

  local expected_service
  for expected_service in etcd "${CLUSTER_NODE_NAMES[@]}"; do
    if ! grep -Fxq "${expected_service}" <<<"${running_services}"; then
      printf 'cluster stack for project %s is incomplete; missing running service %s\n' "${project_name}" "${expected_service}" >&2
      exit 1
    fi
  done
}

run_up() {
  local env_file="$1"
  local project_name="$2"
  log "cluster command: up"
  log "compose project: ${project_name}"
  log "compose file: ${COMPOSE_FILE}"
  log "env file: ${env_file}"
  log "first run may take a while because Docker needs to build the local image"

  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${COMPOSE_FILE}" \
    up -d --build

  wait_for_cluster_readiness "${env_file}" "${project_name}"
  printf '\n'
  print_cluster_summary "${env_file}" "${project_name}" "${COMPOSE_FILE}"
}

run_status() {
  local env_file="$1"
  local project_name="$2"
  log "cluster command: status"
  log "compose project: ${project_name}"
  log "compose file: ${COMPOSE_FILE}"
  log "env file: ${env_file}"

  ensure_cluster_services_running "${env_file}" "${project_name}"
  wait_for_cluster_readiness "${env_file}" "${project_name}"
  printf '\n'
  print_cluster_summary "${env_file}" "${project_name}" "${COMPOSE_FILE}"
}

run_down() {
  local env_file="$1"
  local project_name="$2"
  log "cluster command: down"
  log "compose project: ${project_name}"
  log "compose file: ${COMPOSE_FILE}"
  log "env file: ${env_file}"
  compose_down "${COMPOSE_FILE}" "${env_file}" "${project_name}"
}

main() {
  if [[ "$#" -eq 0 ]]; then
    usage >&2
    exit 1
  fi

  local command="$1"
  shift
  local env_file=""
  local project_name="${DEFAULT_PROJECT_NAME}"

  while [[ "$#" -gt 0 ]]; do
    case "$1" in
      --env-file)
        if [[ "$#" -lt 2 ]]; then
          printf 'missing value for --env-file\n' >&2
          exit 1
        fi
        env_file="$2"
        shift 2
        ;;
      --project-name)
        if [[ "$#" -lt 2 ]]; then
          printf 'missing value for --project-name\n' >&2
          exit 1
        fi
        project_name="$2"
        shift 2
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        printf 'unsupported argument: %s\n' "$1" >&2
        usage >&2
        exit 1
        ;;
    esac
  done

  if [[ -z "${env_file}" ]]; then
    env_file="$(resolve_default_env_file)"
  fi

  require_file "${COMPOSE_FILE}"
  require_file "${env_file}"

  case "${command}" in
    up)
      run_up "${env_file}" "${project_name}"
      ;;
    status)
      run_status "${env_file}" "${project_name}"
      ;;
    down)
      run_down "${env_file}" "${project_name}"
      ;;
    --help|-h|help)
      usage
      ;;
    *)
      printf 'unsupported command: %s\n' "${command}" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
