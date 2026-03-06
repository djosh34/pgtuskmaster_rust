#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: test-long-parallel.sh --gate <name> --run-id <id> --steps-jsonl <path> --steps-dir <path> \
  --timeout-bin <path> --timeout-secs <n> --kill-after-secs <n> --metadata-file <path> -- <test-name...>
EOF
}

tmp_dir=""

die() {
  local msg="$1"
  printf 'test-long-parallel: %s\n' "$msg" >&2
  exit 1
}

json_escape() {
  local s="$1"
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  s=${s//$'\n'/\\n}
  s=${s//$'\r'/\\r}
  s=${s//$'\t'/\\t}
  printf '%s' "$s"
}

json_array() {
  local -a items=("$@")
  local first=1
  printf '['
  local item
  for item in "${items[@]}"; do
    if [[ "$first" -eq 0 ]]; then
      printf ','
    fi
    first=0
    printf '"%s"' "$(json_escape "$item")"
  done
  printf ']'
}

utc_rfc3339() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

epoch_ms() {
  local out
  out="$(date +%s%3N 2>/dev/null || true)"
  if [[ "$out" =~ ^[0-9]+$ ]]; then
    printf '%s' "$out"
    return 0
  fi
  printf '%s' "$(( $(date +%s) * 1000 ))"
}

slugify() {
  local value="$1"
  value="$(printf '%s' "$value" | tr -cs 'A-Za-z0-9._-' '_')"
  value="$(printf '%s' "$value" | sed -E 's/^_+|_+$//g')"
  if [[ -z "$value" ]]; then
    value="step"
  fi
  printf '%s' "$value"
}

discover_executables() {
  local metadata_file="$1"
  local output_file="$2"
  python3 - "$metadata_file" "$output_file" <<'PY'
import json
import sys

metadata_file = sys.argv[1]
output_file = sys.argv[2]
executables = []
seen = set()

with open(metadata_file, encoding="utf-8") as handle:
    for raw_line in handle:
        line = raw_line.strip()
        if not line or not line.startswith("{"):
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("reason") != "compiler-artifact":
            continue
        executable = record.get("executable")
        target = record.get("target") or {}
        kinds = target.get("kind") or []
        if not executable or "test" not in kinds:
            continue
        if executable in seen:
            continue
        seen.add(executable)
        executables.append(executable)

with open(output_file, "w", encoding="utf-8") as handle:
    for executable in executables:
        handle.write(f"{executable}\n")
PY
}

run_one_test() {
  local gate="$1"
  local run_id="$2"
  local timeout_bin="$3"
  local timeout_secs="$4"
  local kill_after_secs="$5"
  local test_name="$6"
  local executable="$7"
  local log_path="$8"
  local record_path="$9"

  local step
  step="test_long.exec.${test_name}"

  local start_utc
  local start_ms
  start_utc="$(utc_rfc3339)"
  start_ms="$(epoch_ms)"

  local -a argv=("$executable" "$test_name" --exact --nocapture)

  printf '== gate %s step %s (run %s) ==\n' "$gate" "$step" "$run_id" >&2
  printf '== timeout %ss (kill-after %ss) ==\n' "$timeout_secs" "$kill_after_secs" >&2

  local cmd_exit_code=0
  set +e
  "$timeout_bin" --kill-after="${kill_after_secs}s" "${timeout_secs}s" \
    "$executable" "$test_name" --exact --nocapture >"$log_path" 2>&1
  cmd_exit_code="$?"
  set -e

  local end_utc
  local end_ms
  end_utc="$(utc_rfc3339)"
  end_ms="$(epoch_ms)"

  local duration_ms
  if [[ "$end_ms" -ge "$start_ms" ]]; then
    duration_ms="$((end_ms - start_ms))"
  else
    duration_ms="0"
  fi

  local timed_out="false"
  if [[ "$cmd_exit_code" -eq 124 || "$cmd_exit_code" -eq 137 ]]; then
    timed_out="true"
  fi

  local argv_json
  argv_json="$(json_array "${argv[@]}")"

  local record
  record="$(
    printf '{'
    printf '"run_id":"%s",' "$(json_escape "$run_id")"
    printf '"gate":"%s",' "$(json_escape "$gate")"
    printf '"step":"%s",' "$(json_escape "$step")"
    printf '"argv":%s,' "$argv_json"
    printf '"start_utc":"%s",' "$(json_escape "$start_utc")"
    printf '"end_utc":"%s",' "$(json_escape "$end_utc")"
    printf '"duration_ms":%s,' "$(json_escape "$duration_ms")"
    printf '"exit_code":%s,' "$(json_escape "$cmd_exit_code")"
    printf '"cmd_exit_code":%s,' "$(json_escape "$cmd_exit_code")"
    printf '"tee_exit_code":0,'
    printf '"timed_out":%s,' "$(json_escape "$timed_out")"
    printf '"timeout_secs":%s,' "$(json_escape "$timeout_secs")"
    printf '"kill_after_secs":%s,' "$(json_escape "$kill_after_secs")"
    printf '"log_path":"%s"' "$(json_escape "$log_path")"
    printf '}'
  )"
  printf '%s\n' "$record" >"$record_path"

  if [[ "$timed_out" == "true" ]]; then
    printf 'test-long-parallel: TIMEOUT after %ss (kill-after %ss): %s\n' \
      "$timeout_secs" "$kill_after_secs" "$test_name" >&2
  fi
  if [[ "$cmd_exit_code" -ne 0 ]]; then
    printf 'test-long-parallel: test failed: %s (exit %s); tail of %s follows\n' \
      "$test_name" "$cmd_exit_code" "$log_path" >&2
    tail -n 40 "$log_path" >&2 || true
  fi

  return "$cmd_exit_code"
}

main() {
  local gate=""
  local run_id=""
  local steps_jsonl=""
  local steps_dir=""
  local timeout_bin=""
  local timeout_secs=""
  local kill_after_secs=""
  local metadata_file=""

  while [[ "$#" -gt 0 ]]; do
    case "$1" in
      --gate) gate="${2:-}"; shift 2 ;;
      --run-id) run_id="${2:-}"; shift 2 ;;
      --steps-jsonl) steps_jsonl="${2:-}"; shift 2 ;;
      --steps-dir) steps_dir="${2:-}"; shift 2 ;;
      --timeout-bin) timeout_bin="${2:-}"; shift 2 ;;
      --timeout-secs) timeout_secs="${2:-}"; shift 2 ;;
      --kill-after-secs) kill_after_secs="${2:-}"; shift 2 ;;
      --metadata-file) metadata_file="${2:-}"; shift 2 ;;
      --) shift; break ;;
      -h|--help) usage; exit 0 ;;
      *) usage; die "unknown argument: $1" ;;
    esac
  done

  if [[ -z "$gate" || -z "$run_id" || -z "$steps_jsonl" || -z "$steps_dir" || -z "$timeout_bin" || -z "$timeout_secs" || -z "$kill_after_secs" || -z "$metadata_file" ]]; then
    usage
    die "missing required flags"
  fi
  if [[ "$#" -eq 0 ]]; then
    usage
    die "missing test names after --"
  fi
  if [[ ! -x "$timeout_bin" ]]; then
    die "timeout binary is not executable: $timeout_bin"
  fi
  if [[ ! -f "$metadata_file" ]]; then
    die "metadata file does not exist: $metadata_file"
  fi
  if [[ ! "$timeout_secs" =~ ^[0-9]+$ ]] || [[ ! "$kill_after_secs" =~ ^[0-9]+$ ]]; then
    die "timeout values must be integers: timeout_secs=$timeout_secs kill_after_secs=$kill_after_secs"
  fi

  mkdir -p "$steps_dir"
  touch "$steps_jsonl"

  tmp_dir="$(mktemp -d "${steps_dir}/parallel.XXXXXX")"
  trap 'if [[ -n "$tmp_dir" ]]; then rm -rf -- "$tmp_dir"; fi' EXIT

  local executable_file
  executable_file="${tmp_dir}/executables.txt"
  discover_executables "$metadata_file" "$executable_file"

  declare -A wanted_set=()
  declare -A test_to_executable=()
  local -a wanted_tests=("$@")
  local test_name
  for test_name in "${wanted_tests[@]}"; do
    if [[ -n "${wanted_set[$test_name]+x}" ]]; then
      die "duplicate requested test name: $test_name"
    fi
    wanted_set["$test_name"]=1
  done

  local executable
  while IFS= read -r executable; do
    [[ -n "$executable" ]] || continue
    while IFS= read -r list_line; do
      local_name="${list_line%%:*}"
      if [[ -z "${wanted_set[$local_name]+x}" ]]; then
        continue
      fi
      if [[ -n "${test_to_executable[$local_name]+x}" ]]; then
        die "long test mapped to multiple executables: $local_name"
      fi
      test_to_executable["$local_name"]="$executable"
    done < <("$executable" --list | awk -F': ' '$2=="test"{print $1}')
  done < "$executable_file"

  for test_name in "${wanted_tests[@]}"; do
    if [[ -z "${test_to_executable[$test_name]+x}" ]]; then
      die "failed to map long test to executable: $test_name"
    fi
  done

  local start_index
  local existing_lines
  existing_lines="$(wc -l < "$steps_jsonl" | tr -d ' ')"
  if [[ ! "$existing_lines" =~ ^[0-9]+$ ]]; then
    die "failed to count existing step records in: $steps_jsonl"
  fi
  start_index="$((existing_lines + 1))"

  declare -A pid_by_test=()
  declare -A record_by_test=()
  declare -A log_by_test=()
  local test_index=0
  for test_name in "${wanted_tests[@]}"; do
    local index
    index="$((start_index + test_index))"
    test_index="$((test_index + 1))"

    local step_slug
    step_slug="$(slugify "test_long.exec.${test_name}")"

    local log_path
    log_path="$(printf '%s/%02d-%s.log' "$steps_dir" "$index" "$step_slug")"
    local record_path
    record_path="$(printf '%s/%02d-%s.json' "$tmp_dir" "$index" "$step_slug")"

    record_by_test["$test_name"]="$record_path"
    log_by_test["$test_name"]="$log_path"

    run_one_test \
      "$gate" \
      "$run_id" \
      "$timeout_bin" \
      "$timeout_secs" \
      "$kill_after_secs" \
      "$test_name" \
      "${test_to_executable[$test_name]}" \
      "$log_path" \
      "$record_path" &
    pid_by_test["$test_name"]="$!"
  done

  local overall_exit=0
  for test_name in "${wanted_tests[@]}"; do
    if ! wait "${pid_by_test[$test_name]}"; then
      overall_exit=1
    fi
  done

  for test_name in "${wanted_tests[@]}"; do
    if [[ ! -f "${record_by_test[$test_name]}" ]]; then
      die "missing step record for test: $test_name"
    fi
    cat "${record_by_test[$test_name]}" >> "$steps_jsonl"
  done

  if [[ "$overall_exit" -ne 0 ]]; then
    exit "$overall_exit"
  fi
}

main "$@"
