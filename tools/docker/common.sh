#!/usr/bin/env bash
set -euo pipefail

readonly TOOL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${TOOL_DIR}/../.." && pwd)"
readonly DEFAULT_ETCD_IMAGE="quay.io/coreos/etcd:v3.5.21"
readonly DEFAULT_PGTUSKMASTER_IMAGE="pgtuskmaster:local"
readonly CLUSTER_NODE_NAMES=("node-a" "node-b" "node-c")

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

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    printf 'missing required file: %s\n' "${path}" >&2
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
  local analysis
  local last_reason="no response received yet"
  local last_detail=""
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    if ! response="$(curl --silent --show-error --fail "${url}" 2>&1)"; then
      last_reason="HTTP request failed"
      last_detail="${response}"
      sleep 1
      continue
    fi

    if [[ -z "${response}" ]]; then
      last_reason="endpoint returned an empty body"
      last_detail=""
      sleep 1
      continue
    fi

    if analysis="$(python3 - "${expected_count}" "${response}" <<'PY'
import json
import sys

expected = int(sys.argv[1])
body = sys.argv[2]

try:
    payload = json.loads(body)
except json.JSONDecodeError as exc:
    print(f"invalid JSON: {exc.msg} at line {exc.lineno} column {exc.colno}")
    sys.exit(10)

if not isinstance(payload, dict):
    print(f"unexpected JSON type: {type(payload).__name__}")
    sys.exit(11)

member_count = payload.get("member_count")
phase = payload.get("ha_phase")
decision = payload.get("ha_decision")

if isinstance(member_count, int) and member_count >= expected and phase and decision:
    sys.exit(0)

summary = []
summary.append(f"member_count={member_count!r}")
summary.append(f"ha_phase={phase!r}")
summary.append(f"ha_decision={decision!r}")
leader_id = payload.get("leader_id")
if leader_id is not None:
    summary.append(f"leader_id={leader_id!r}")
print(", ".join(summary))
sys.exit(11)
PY
    )"; then
      return 0
    fi

    case "$?" in
      10)
        last_reason="endpoint returned malformed JSON"
        last_detail="${analysis}"
        ;;
      11)
        last_reason="cluster has not converged yet"
        last_detail="${analysis}"
        ;;
      *)
        last_reason="readiness parser failed unexpectedly"
        last_detail="${analysis}"
        ;;
    esac
    sleep 1
  done
  printf 'timed out waiting for %s members at %s; last observed reason: %s\n' "${expected_count}" "${url}" "${last_reason}" >&2
  if [[ -n "${last_detail}" ]]; then
    printf 'last observed detail: %s\n' "${last_detail}" >&2
  fi
  return 1
}

compose_down_with_diagnostics() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local output
  local status
  local max_chars=4000

  if output="$(compose_down "${compose_file}" "${env_file}" "${project_name}" 2>&1)"; then
    return 0
  fi

  status=$?
  if (( ${#output} > max_chars )); then
    output="${output: -max_chars}"
  fi

  printf '%s' "${output}"
  return "${status}"
}

compose_ps_service_names() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${compose_file}" \
    ps --services
}

compose_running_service_names() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${compose_file}" \
    ps --services --status running
}

compose_container_id() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local service_name="$4"
  docker compose \
    --project-name "${project_name}" \
    --env-file "${env_file}" \
    -f "${compose_file}" \
    ps -q "${service_name}"
}

wait_for_sql_ready() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local service_name="$4"
  local password="$5"
  local timeout_secs="$6"
  local container_id=""
  local deadline
  local probe_timeout_secs=10
  local last_reason="no probe executed yet"
  deadline=$((SECONDS + timeout_secs))

  require_cmd timeout
  while (( SECONDS < deadline )); do
    container_id="$(compose_container_id "${compose_file}" "${env_file}" "${project_name}" "${service_name}" 2>/dev/null)"
    if [[ -z "${container_id}" ]]; then
      last_reason="service container is not created yet"
      sleep 1
      continue
    fi

    if timeout --kill-after=5s "${probe_timeout_secs}s" \
      docker exec \
        --env "PGPASSWORD=${password}" \
        "${container_id}" \
        /usr/lib/postgresql/16/bin/psql -w -h /var/lib/pgtuskmaster/socket -U postgres -d postgres -Atqc 'select 1' \
        >/dev/null 2>&1; then
      return 0
    fi

    case "$?" in
      124)
        last_reason="probe command timed out after ${probe_timeout_secs}s"
        ;;
      125)
        last_reason="timeout command failed to start"
        ;;
      *)
        last_reason="service rejected the SQL readiness probe"
        ;;
    esac
    sleep 1
  done
  printf 'timed out waiting for SQL readiness on service %s; last observed reason: %s\n' "${service_name}" "${last_reason}" >&2
  return 1
}

check_etcd_health() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local container_id

  container_id="$(compose_container_id "${compose_file}" "${env_file}" "${project_name}" "etcd")"
  if [[ -z "${container_id}" ]]; then
    printf 'etcd container is not running for project %s\n' "${project_name}" >&2
    return 1
  fi

  docker exec "${container_id}" \
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

read_env_value() {
  local env_file="$1"
  local key="$2"
  python3 - "${env_file}" "${key}" <<'PY'
import pathlib
import sys

env_path = pathlib.Path(sys.argv[1])
key = sys.argv[2]

for raw_line in env_path.read_text(encoding="utf-8").splitlines():
    line = raw_line.strip()
    if not line or line.startswith("#"):
        continue
    if "=" not in line:
        continue
    current_key, value = line.split("=", 1)
    if current_key == key:
        print(value)
        sys.exit(0)

sys.exit(1)
PY
}

cluster_node_env_key() {
  local node_name="$1"
  local suffix="$2"
  local normalized="${node_name//-/_}"
  normalized="${normalized^^}"
  printf 'PGTM_CLUSTER_%s_%s\n' "${normalized}" "${suffix}"
}

cluster_api_port_from_env() {
  local env_file="$1"
  local node_name="$2"
  local key
  key="$(cluster_node_env_key "${node_name}" "API_PORT")"
  read_env_value "${env_file}" "${key}"
}

cluster_pg_port_from_env() {
  local env_file="$1"
  local node_name="$2"
  local key
  key="$(cluster_node_env_key "${node_name}" "PG_PORT")"
  read_env_value "${env_file}" "${key}"
}

cluster_api_url() {
  local env_file="$1"
  local node_name="$2"
  printf 'http://127.0.0.1:%s\n' "$(cluster_api_port_from_env "${env_file}" "${node_name}")"
}

cluster_debug_url() {
  local env_file="$1"
  local node_name="$2"
  printf '%s/debug/verbose\n' "$(cluster_api_url "${env_file}" "${node_name}")"
}

cluster_ha_state_url() {
  local env_file="$1"
  local node_name="$2"
  printf '%s/ha/state\n' "$(cluster_api_url "${env_file}" "${node_name}")"
}

cluster_pg_endpoint() {
  local env_file="$1"
  local node_name="$2"
  printf '127.0.0.1:%s\n' "$(cluster_pg_port_from_env "${env_file}" "${node_name}")"
}

query_pg_is_in_recovery() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local service_name="$4"
  local password="$5"
  local container_id

  container_id="$(compose_container_id "${compose_file}" "${env_file}" "${project_name}" "${service_name}")"
  if [[ -z "${container_id}" ]]; then
    printf 'container for service %s is not running in project %s\n' "${service_name}" "${project_name}" >&2
    return 1
  fi

  docker exec \
    --env "PGPASSWORD=${password}" \
    "${container_id}" \
    /usr/lib/postgresql/16/bin/psql -w -h /var/lib/pgtuskmaster/socket -U postgres -d postgres -Atqc 'select pg_is_in_recovery()'
}

cluster_role_from_recovery_value() {
  local recovery_value="$1"
  case "${recovery_value}" in
    f)
      printf 'primary\n'
      ;;
    t)
      printf 'replica\n'
      ;;
    *)
      printf 'unknown\n'
      return 1
      ;;
  esac
}

wait_for_cluster_replication_roles() {
  local compose_file="$1"
  local env_file="$2"
  local project_name="$3"
  local password="$4"
  local timeout_secs="$5"
  local deadline
  deadline=$((SECONDS + timeout_secs))
  while (( SECONDS < deadline )); do
    local recovery_values=()
    local node_name
    for node_name in "${CLUSTER_NODE_NAMES[@]}"; do
      local raw_value
      if ! raw_value="$(query_pg_is_in_recovery "${compose_file}" "${env_file}" "${project_name}" "${node_name}" "${password}")"; then
        recovery_values=()
        break
      fi
      raw_value="$(printf '%s\n' "${raw_value}" | tr -d '\r' | tail -n 1)"
      if [[ "${raw_value}" != "t" && "${raw_value}" != "f" ]]; then
        recovery_values=()
        break
      fi
      recovery_values+=("${raw_value}")
    done

    if [[ "${#recovery_values[@]}" -eq "${#CLUSTER_NODE_NAMES[@]}" ]]; then
      local primaries=0
      local replicas=0
      local recovery_value
      for recovery_value in "${recovery_values[@]}"; do
        if [[ "${recovery_value}" == "f" ]]; then
          primaries=$((primaries + 1))
        else
          replicas=$((replicas + 1))
        fi
      done

      if [[ "${primaries}" -eq 1 && "${replicas}" -eq 2 ]]; then
        return 0
      fi
    fi

    sleep 1
  done

  printf 'timed out waiting for cluster replication roles: expected 1 primary + 2 replicas (pg_is_in_recovery)\n' >&2
  return 1
}

fetch_ha_state_json() {
  local env_file="$1"
  local node_name="$2"
  curl --silent --show-error --fail "$(cluster_ha_state_url "${env_file}" "${node_name}")"
}

ha_state_field() {
  local json_payload="$1"
  local field_name="$2"
  python3 -c '
import json
import sys

field_name = sys.argv[1]
payload = json.load(sys.stdin)
value = payload.get(field_name)
if value is None:
    print("<none>")
elif isinstance(value, bool):
    print(str(value).lower())
elif isinstance(value, (dict, list)):
    print(json.dumps(value, sort_keys=True))
else:
    print(value)
' "${field_name}" <<<"${json_payload}"
}

print_cluster_summary() {
  local env_file="$1"
  local project_name="$2"
  local compose_file="$3"
  local leader="<unknown>"
  local node_name

  for node_name in "${CLUSTER_NODE_NAMES[@]}"; do
    local node_payload
    node_payload="$(fetch_ha_state_json "${env_file}" "${node_name}")"
    leader="$(ha_state_field "${node_payload}" "leader")"
    break
  done

  printf 'Compose project: %s\n' "${project_name}"
  printf 'Compose file: %s\n' "${compose_file}"
  printf 'Env file: %s\n' "${env_file}"
  printf 'Leader: %s\n' "${leader}"
  printf '\n'
  printf '%-7s %-10s %-22s %-28s %-17s %-12s %-12s %s\n' \
    'Node' 'Role' 'API' 'Debug' 'PostgreSQL' 'Members' 'HA phase' 'HA decision'

  for node_name in "${CLUSTER_NODE_NAMES[@]}"; do
    local payload
    local api_url
    local debug_url
    local pg_endpoint
    local role
    local member_count
    local ha_phase
    local ha_decision

    payload="$(fetch_ha_state_json "${env_file}" "${node_name}")"
    api_url="$(cluster_api_url "${env_file}" "${node_name}")"
    debug_url="$(cluster_debug_url "${env_file}" "${node_name}")"
    pg_endpoint="$(cluster_pg_endpoint "${env_file}" "${node_name}")"
    ha_phase="$(ha_state_field "${payload}" "ha_phase")"
    ha_decision="$(ha_state_field "${payload}" "ha_decision")"
    member_count="$(ha_state_field "${payload}" "member_count")"
    role="${ha_phase}"
    if [[ "${role}" != "primary" && "${role}" != "replica" ]]; then
      role="unknown"
    fi

    printf '%-7s %-10s %-22s %-28s %-17s %-12s %-12s %s\n' \
      "${node_name}" \
      "${role}" \
      "${api_url}" \
      "${debug_url}" \
      "${pg_endpoint}" \
      "${member_count}" \
      "${ha_phase}" \
      "${ha_decision}"
  done
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
