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

echo "building shared HA cucumber image ${image_ref}" >&2
docker build \
  --file "${repo_root}/tests/docker/Dockerfile" \
  --label "${cucumber_test_label}" \
  --tag "${image_ref}" \
  "${repo_root}"

mapfile -t cucumber_images < <(
  docker image ls \
    --filter "label=${cucumber_test_label}" \
    --format '{{.Repository}}:{{.Tag}}'
)

for image in "${cucumber_images[@]}"; do
  if [[ -z "${image}" || "${image}" == "<none>:<none>" || "${image}" == "${image_ref}" ]]; then
    continue
  fi
  echo "removing older HA cucumber image ${image}" >&2
  docker image rm "${image}"
done

printf 'PGTM_CUCUMBER_TEST_RUN_ID=%s\n' "${image_run_id}" >> "${NEXTEST_ENV}"
printf 'PGTM_CUCUMBER_TEST_IMAGE=%s\n' "${image_ref}" >> "${NEXTEST_ENV}"
