#!/usr/bin/env bash
set -euo pipefail

info_output_file="$(mktemp)"
cleanup() {
  rm -f "${info_output_file}"
}
trap cleanup EXIT

if docker info >"${info_output_file}" 2>&1; then
  exit 0
fi

cat "${info_output_file}" >&2
if grep -Eqi 'permission denied.*docker API|permission denied.*docker.sock' "${info_output_file}"; then
  echo "hint: ensure this account can access /var/run/docker.sock (for example through the docker group), or point DOCKER_HOST at a reachable daemon" >&2
fi
exit 1
