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
readonly subnet_manifest_dir="${repo_root}/tests/ha/runs/_shared"
readonly subnet_manifest_path="${subnet_manifest_dir}/ha-subnets-${NEXTEST_RUN_ID}.json"

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

generate_ha_subnet_manifest() {
  mkdir -p "${subnet_manifest_dir}"

  python3 - "${repo_root}" "${subnet_manifest_path}" <<'PY'
import ipaddress
import json
import subprocess
import sys
from pathlib import Path

repo_root = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
features_root = repo_root / "tests" / "ha" / "features"


def run_text(args):
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


def existing_docker_networks():
    network_ids = run_text(["docker", "network", "ls", "-q"]).split()
    if not network_ids:
        return []
    inspected = json.loads(run_text(["docker", "network", "inspect", *network_ids]))
    subnets = []
    for network in inspected:
        for config in network.get("IPAM", {}).get("Config") or []:
            subnet = config.get("Subnet")
            if subnet:
                subnets.append(ipaddress.ip_network(subnet, strict=False))
    return subnets


def host_routes():
    try:
        route_output = run_text(["ip", "-4", "route", "show"])
    except (FileNotFoundError, subprocess.CalledProcessError):
        return []
    routes = []
    for line in route_output.splitlines():
        if not line:
            continue
        target = line.split()[0]
        if target == "default":
            continue
        try:
            routes.append(ipaddress.ip_network(target, strict=False))
        except ValueError:
            continue
    return routes


feature_names = sorted(
    directory.name
    for directory in features_root.iterdir()
    if directory.is_dir() and (directory / f"{directory.name}.feature").is_file()
)
if not feature_names:
    raise SystemExit("no HA feature directories found")

occupied = existing_docker_networks() + host_routes()
selected = []

candidate_parents = [
    ipaddress.ip_network("10.240.0.0/16"),
    ipaddress.ip_network("10.241.0.0/16"),
    ipaddress.ip_network("10.242.0.0/16"),
    ipaddress.ip_network("10.243.0.0/16"),
]

for parent in candidate_parents:
    for candidate in parent.subnets(new_prefix=28):
        if any(candidate.overlaps(used) for used in occupied):
            continue
        if any(candidate.overlaps(used) for used in selected):
            continue
        selected.append(candidate)
        if len(selected) == len(feature_names):
            break
    if len(selected) == len(feature_names):
        break

if len(selected) != len(feature_names):
    raise SystemExit(
        f"unable to allocate {len(feature_names)} HA /28 subnets without overlap; allocated {len(selected)}"
    )

manifest = {
    "feature_subnets": {
        feature_name: str(subnet)
        for feature_name, subnet in zip(feature_names, selected)
    }
}
manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
print(
    f"generated HA subnet manifest {manifest_path} with {len(feature_names)} /28 networks",
    file=sys.stderr,
)
PY
}

prune_unused_ha_networks
generate_ha_subnet_manifest

echo "building shared HA cucumber image ${image_ref}" >&2
docker build \
  --file "${repo_root}/docker/Dockerfile" \
  --target ha-test \
  --label "${cucumber_test_label}" \
  --tag "${image_ref}" \
  "${repo_root}"

printf 'PGTM_CUCUMBER_TEST_RUN_ID=%s\n' "${image_run_id}" >> "${NEXTEST_ENV}"
printf 'PGTM_CUCUMBER_TEST_IMAGE=%s\n' "${image_ref}" >> "${NEXTEST_ENV}"
printf 'PGTM_HA_SUBNET_MANIFEST=%s\n' "${subnet_manifest_path}" >> "${NEXTEST_ENV}"
