#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tools/docker/common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd docker

readonly ENV_FILE="${REPO_ROOT}/.env.docker.example"
readonly SINGLE_COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.single.yml"
readonly CLUSTER_COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.cluster.yml"
readonly COMPOSE_ROOT="$(cd "$(dirname "${SINGLE_COMPOSE_FILE}")" && pwd)"

if [[ ! -f "${ENV_FILE}" ]]; then
  printf 'missing env example: %s\n' "${ENV_FILE}" >&2
  exit 1
fi

read_env_value() {
  local key="$1"
  local file="$2"
  local value=""

  value="$(awk -F= -v wanted="${key}" '$1 == wanted {print substr($0, index($0, "=") + 1)}' "${file}")"
  if [[ -z "${value}" ]]; then
    printf 'missing required env var %s in %s\n' "${key}" "${file}" >&2
    return 1
  fi
  printf '%s' "${value}"
}

resolve_compose_relative_path() {
  local raw="$1"
  local resolved=""

  if [[ "${raw}" = /* ]]; then
    resolved="${raw}"
  else
    resolved="${COMPOSE_ROOT}/${raw}"
  fi

  printf '%s' "${resolved}"
}

require_non_empty_file() {
  local label="$1"
  local path="$2"

  if [[ ! -f "${path}" ]]; then
    printf '%s points at missing file: %s\n' "${label}" "${path}" >&2
    return 1
  fi
  if [[ ! -s "${path}" ]]; then
    printf '%s points at empty file: %s\n' "${label}" "${path}" >&2
    return 1
  fi
}

log "validating env secret file paths"
superuser_secret="$(read_env_value "PGTM_SECRET_SUPERUSER_FILE" "${ENV_FILE}")"
replicator_secret="$(read_env_value "PGTM_SECRET_REPLICATOR_FILE" "${ENV_FILE}")"
rewinder_secret="$(read_env_value "PGTM_SECRET_REWINDER_FILE" "${ENV_FILE}")"

require_non_empty_file "PGTM_SECRET_SUPERUSER_FILE" "$(resolve_compose_relative_path "${superuser_secret}")"
require_non_empty_file "PGTM_SECRET_REPLICATOR_FILE" "$(resolve_compose_relative_path "${replicator_secret}")"
require_non_empty_file "PGTM_SECRET_REWINDER_FILE" "$(resolve_compose_relative_path "${rewinder_secret}")"

log "validating single-node Compose config"
docker compose --env-file "${ENV_FILE}" -f "${SINGLE_COMPOSE_FILE}" config >/dev/null

log "validating cluster Compose config"
docker compose --env-file "${ENV_FILE}" -f "${CLUSTER_COMPOSE_FILE}" config >/dev/null

log "compose config validation passed"
