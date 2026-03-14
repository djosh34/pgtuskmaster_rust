#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${NEXTEST_ENV:-}" ]]; then
  echo "NEXTEST_ENV is required for nextest setup scripts" >&2
  exit 1
fi

if [[ -z "${NEXTEST_RUN_ID:-}" ]]; then
  echo "NEXTEST_RUN_ID is required for nextest setup scripts" >&2
  exit 1
fi

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly image_repository="pgtm-cucumber-test"
readonly image_run_id="nextest-${NEXTEST_RUN_ID}"
readonly image_ref="${image_repository}:${image_run_id}"
readonly cucumber_test_label="io.pgtuskmaster.cucumber-test=true"

prune_unused_ha_networks() {
  mapfile -t docker_networks < <(docker network ls --format '{{.Name}}')

  for network in "${docker_networks[@]}"; do
    if [[ "${network}" != ha-* ]]; then
      continue
    fi

    if ! container_count="$(
      docker network inspect "${network}" --format '{{ len .Containers }}' 2>/dev/null
    )"; then
      continue
    fi

    if [[ "${container_count}" != "0" ]]; then
      continue
    fi

    echo "removing unused HA network ${network}" >&2
    if ! rm_output="$(docker network rm "${network}" 2>&1)"; then
      if [[ "${rm_output}" == *"No such network"* || "${rm_output}" == *"has active endpoints"* ]]; then
        printf '%s\n' "${rm_output}" >&2
        continue
      fi
      printf '%s\n' "${rm_output}" >&2
      exit 1
    fi
    printf '%s\n' "${rm_output}" >&2
  done
}

prune_unused_ha_networks

echo "building shared HA cucumber image ${image_ref}" >&2
docker build \
  --file "${repo_root}/docker/Dockerfile" \
  --target ha-test \
  --label "${cucumber_test_label}" \
  --tag "${image_ref}" \
  "${repo_root}"

printf 'PGTM_CUCUMBER_TEST_RUN_ID=%s\n' "${image_run_id}" >> "${NEXTEST_ENV}"
printf 'PGTM_CUCUMBER_TEST_IMAGE=%s\n' "${image_ref}" >> "${NEXTEST_ENV}"
