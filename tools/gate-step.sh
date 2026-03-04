#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: gate-step.sh --gate <name> --step <name> --run-id <id> --evidence-dir <path> \
  --timeout-bin <path> --timeout-secs <n> --kill-after-secs <n> -- <command...>
EOF
}

die() {
  local msg="$1"
  printf 'gate-step: %s\n' "$msg" >&2
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

main() {
  local gate=""
  local step=""
  local run_id=""
  local evidence_dir=""
  local timeout_bin=""
  local timeout_secs=""
  local kill_after_secs=""

  while [[ "$#" -gt 0 ]]; do
    case "$1" in
      --gate) gate="${2:-}"; shift 2 ;;
      --step) step="${2:-}"; shift 2 ;;
      --run-id) run_id="${2:-}"; shift 2 ;;
      --evidence-dir) evidence_dir="${2:-}"; shift 2 ;;
      --timeout-bin) timeout_bin="${2:-}"; shift 2 ;;
      --timeout-secs) timeout_secs="${2:-}"; shift 2 ;;
      --kill-after-secs) kill_after_secs="${2:-}"; shift 2 ;;
      --) shift; break ;;
      -h|--help) usage; exit 0 ;;
      *) usage; die "unknown argument: $1" ;;
    esac
  done

  if [[ -z "$gate" || -z "$step" || -z "$run_id" || -z "$evidence_dir" || -z "$timeout_bin" || -z "$timeout_secs" || -z "$kill_after_secs" ]]; then
    usage
    die "missing required flags"
  fi
  if [[ "$#" -eq 0 ]]; then
    usage
    die "missing command after --"
  fi
  if [[ ! -x "$timeout_bin" ]]; then
    die "timeout binary is not executable: $timeout_bin"
  fi
  if [[ ! "$timeout_secs" =~ ^[0-9]+$ ]] || [[ ! "$kill_after_secs" =~ ^[0-9]+$ ]]; then
    die "timeout values must be integers: timeout_secs=$timeout_secs kill_after_secs=$kill_after_secs"
  fi
  if [[ "$timeout_secs" -le 0 ]]; then
    die "timeout_secs must be > 0"
  fi

  local gate_dir="${evidence_dir}/${gate}"
  local steps_dir="${gate_dir}/steps"
  mkdir -p "$steps_dir"

  local steps_jsonl="${gate_dir}/steps.jsonl"
  local step_index="1"
  if [[ -f "$steps_jsonl" ]]; then
    local lines
    lines="$(wc -l < "$steps_jsonl" | tr -d ' ')"
    if [[ ! "$lines" =~ ^[0-9]+$ ]]; then
      die "failed to count lines in: $steps_jsonl"
    fi
    step_index="$((lines + 1))"
  fi

  local step_slug
  step_slug="$(printf '%s' "$step" | tr -cs 'A-Za-z0-9._-' '_' | sed -E 's/^_+|_+$//g')"
  if [[ -z "$step_slug" ]]; then
    step_slug="step"
  fi

  local log_path
  log_path="$(printf '%s/%02d-%s.log' "$steps_dir" "$step_index" "$step_slug")"

  local start_utc
  local start_ms
  start_utc="$(utc_rfc3339)"
  start_ms="$(epoch_ms)"

  local -a argv=("$@")

  printf '== gate %s step %s (run %s) ==\n' "$gate" "$step" "$run_id" >&2
  printf '== timeout %ss (kill-after %ss) ==\n' "$timeout_secs" "$kill_after_secs" >&2

  local cmd_exit_code=0
  local tee_exit_code=0

  set +e
  "$timeout_bin" --kill-after="${kill_after_secs}s" "${timeout_secs}s" "${argv[@]}" 2>&1 | tee "$log_path"
  cmd_exit_code="${PIPESTATUS[0]:-0}"
  tee_exit_code="${PIPESTATUS[1]:-0}"
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

  local exit_code="$cmd_exit_code"
  local timed_out="false"
  if [[ "$cmd_exit_code" -eq 124 || "$cmd_exit_code" -eq 137 ]]; then
    timed_out="true"
  fi

  if [[ "$tee_exit_code" -ne 0 ]]; then
    printf 'gate-step: tee failed with exit %s writing %s\n' "$tee_exit_code" "$log_path" >&2
    exit_code="$tee_exit_code"
    timed_out="false"
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
    printf '"exit_code":%s,' "$(json_escape "$exit_code")"
    printf '"cmd_exit_code":%s,' "$(json_escape "$cmd_exit_code")"
    printf '"tee_exit_code":%s,' "$(json_escape "$tee_exit_code")"
    printf '"timed_out":%s,' "$(json_escape "$timed_out")"
    printf '"timeout_secs":%s,' "$(json_escape "$timeout_secs")"
    printf '"kill_after_secs":%s,' "$(json_escape "$kill_after_secs")"
    printf '"log_path":"%s"' "$(json_escape "$log_path")"
    printf '}'
  )"

  printf '%s\n' "$record" >> "$steps_jsonl"

  if [[ "$timed_out" == "true" ]]; then
    printf 'gate-step: TIMEOUT after %ss (kill-after %ss): gate=%s step=%s\n' "$timeout_secs" "$kill_after_secs" "$gate" "$step" >&2
  fi

  if [[ "$exit_code" -ne 0 ]]; then
    exit "$exit_code"
  fi
}

main "$@"
