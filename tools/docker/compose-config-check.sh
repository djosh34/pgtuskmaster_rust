#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=tools/docker/common.sh
source "${SCRIPT_DIR}/common.sh"

require_cmd docker

readonly ENV_FILE="${REPO_ROOT}/.env.docker.example"
readonly SINGLE_COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.single.yml"
readonly CLUSTER_COMPOSE_FILE="${REPO_ROOT}/docker/compose/docker-compose.cluster.yml"

if [[ ! -f "${ENV_FILE}" ]]; then
  printf 'missing env example: %s\n' "${ENV_FILE}" >&2
  exit 1
fi

log "validating single-node Compose config"
docker compose --env-file "${ENV_FILE}" -f "${SINGLE_COMPOSE_FILE}" config >/dev/null

log "validating cluster Compose config"
docker compose --env-file "${ENV_FILE}" -f "${CLUSTER_COMPOSE_FILE}" config >/dev/null

log "compose config validation passed"
