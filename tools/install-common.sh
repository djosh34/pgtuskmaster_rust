#!/usr/bin/env bash
set -euo pipefail

# Keep installer command resolution stable and avoid picking up malicious/accidental PATH entries.
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

die() {
    echo "$*" >&2
    exit 1
}

require_cmd_path() {
    local name="$1"
    local resolved=""
    resolved="$(command -v "${name}" 2>/dev/null || true)"
    if [[ -z "${resolved}" ]]; then
        die "missing required command: ${name}"
    fi
    if [[ "${resolved}" != /* ]]; then
        die "resolved command path is not absolute for ${name}: ${resolved}"
    fi

    # Many common system commands (for example `awk`) are symlinks. Resolve to the final
    # target path and validate *that*, but return the symlink path so multi-call
    # binaries (for example rustup's `cargo` proxy) preserve argv[0] semantics.
    local resolved_target="${resolved}"
    if [[ -L "${resolved}" ]]; then
        resolved_target="$(readlink -f -- "${resolved}" 2>/dev/null || true)"
        if [[ -z "${resolved_target}" || "${resolved_target}" != /* ]]; then
            die "failed to resolve command symlink for ${name}: ${resolved}"
        fi
    fi

    if [[ ! -f "${resolved}" ]]; then
        die "resolved command path is not a regular file for ${name}: ${resolved}"
    fi
    if [[ ! -x "${resolved}" ]]; then
        die "resolved command path is not executable for ${name}: ${resolved}"
    fi
    if [[ "${resolved_target}" != "${resolved}" ]]; then
        if [[ ! -f "${resolved_target}" ]]; then
            die "resolved command target is not a regular file for ${name}: ${resolved_target}"
        fi
        if [[ ! -x "${resolved_target}" ]]; then
            die "resolved command target is not executable for ${name}: ${resolved_target}"
        fi
    fi

    local allow_prefixes=(
        "/usr/local/sbin"
        "/usr/local/bin"
        "/usr/sbin"
        "/usr/bin"
        "/sbin"
        "/bin"
    )
    if [[ -n "${INSTALL_ALLOW_PATH_PREFIXES:-}" ]]; then
        IFS=":" read -r -a allow_prefixes <<< "${INSTALL_ALLOW_PATH_PREFIXES}"
    fi

    local allowed="false"
    for prefix in "${allow_prefixes[@]}"; do
        if [[ "${resolved}" == "${prefix}/"* ]]; then
            allowed="true"
            break
        fi
    done
    if [[ "${allowed}" != "true" ]]; then
        die "resolved command path for ${name} is outside allowlist: ${resolved} (override with INSTALL_ALLOW_PATH_PREFIXES=/prefix1:/prefix2)"
    fi
    if [[ "${resolved_target}" != "${resolved}" ]]; then
        local allowed_target="false"
        for prefix in "${allow_prefixes[@]}"; do
            if [[ "${resolved_target}" == "${prefix}/"* ]]; then
                allowed_target="true"
                break
            fi
        done
        if [[ "${allowed_target}" != "true" ]]; then
            die "resolved command target for ${name} is outside allowlist: ${resolved_target} (override with INSTALL_ALLOW_PATH_PREFIXES=/prefix1:/prefix2)"
        fi
    fi

    printf '%s\n' "${resolved}"
}

sha256_file() {
    local sha256sum_path="$1"
    local awk_path="$2"
    local file_path="$3"

    "${sha256sum_path}" "${file_path}" | "${awk_path}" '{print $1}'
}

file_size_bytes() {
    local stat_path="$1"
    local file_path="$2"

    "${stat_path}" -Lc '%s' "${file_path}"
}

install_copy_atomic() {
    local cp_path="$1"
    local chmod_path="$2"
    local mv_path="$3"
    local mkdir_path="$4"
    local src="$5"
    local dst="$6"

    local dst_dir=""
    dst_dir="$(dirname -- "${dst}")"
    "${mkdir_path}" -p "${dst_dir}"

    local tmp="${dst}.tmp.$$"
    "${cp_path}" "${src}" "${tmp}"
    "${chmod_path}" 0755 "${tmp}"
    "${mv_path}" -f "${tmp}" "${dst}"
}

update_attestation_manifest() {
    local flock_path="$1"
    local python3_path="$2"
    local attestation_path="$3"
    local entry_json="$4"
    local generated_by="$5"

    local lock_path="${attestation_path}.lock"
    ENTRY_JSON="${entry_json}" GENERATED_BY="${generated_by}" \
        "${flock_path}" -x "${lock_path}" "${python3_path}" - "${attestation_path}" <<'PY'
import json
import os
import sys
import time

attestation_path = sys.argv[1]
entry_json = os.environ.get("ENTRY_JSON", "")
generated_by = os.environ.get("GENERATED_BY", "unknown")

try:
    entry = json.loads(entry_json)
except Exception as e:
    raise SystemExit(f"invalid ENTRY_JSON: {e}")

data = {"schema_version": 1, "generated_by": generated_by, "entries": []}
if os.path.exists(attestation_path):
    try:
        with open(attestation_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        raise SystemExit(f"failed to parse existing attestation manifest {attestation_path}: {e}")

if not isinstance(data, dict):
    raise SystemExit(f"attestation manifest {attestation_path} must be a JSON object")

entries = data.get("entries", [])
if entries is None:
    entries = []
if not isinstance(entries, list):
    raise SystemExit(f"attestation manifest {attestation_path} entries must be a JSON array")

by_path = {}
for existing in entries:
    if isinstance(existing, dict) and "path" in existing:
        by_path[str(existing["path"])] = existing

by_path[str(entry.get("path"))] = entry

merged = list(by_path.values())
merged.sort(key=lambda e: str(e.get("path", "")))

data["schema_version"] = 1
data["generated_by"] = generated_by
data["entries"] = merged

tmp_path = f"{attestation_path}.tmp.{os.getpid()}"
with open(tmp_path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, sort_keys=True)
    f.write("\n")
os.replace(tmp_path, attestation_path)
PY
}

attestation_entry_json() {
    local sha256sum_path="$1"
    local awk_path="$2"
    local stat_path="$3"
    local date_path="$4"
    local python3_path="$5"
    local readlink_path="$6"
    local label="$7"
    local rel_path="$8"
    local abs_path="$9"

    local sha=""
    sha="$(sha256_file "${sha256sum_path}" "${awk_path}" "${abs_path}")"
    local size=""
    size="$(file_size_bytes "${stat_path}" "${abs_path}")"
    local installed_at_utc=""
    installed_at_utc="$("${date_path}" -u +%Y-%m-%dT%H:%M:%SZ)"
    local resolved_path_abs=""
    resolved_path_abs="$("${readlink_path}" -f -- "${abs_path}" 2>/dev/null || true)"

    "${python3_path}" - "${label}" "${rel_path}" "${sha}" "${size}" "${installed_at_utc}" "${resolved_path_abs}" <<'PY'
import json
import sys

label = sys.argv[1]
rel_path = sys.argv[2]
sha = sys.argv[3]
size = int(sys.argv[4])
installed_at_utc = sys.argv[5]
resolved_path_abs = sys.argv[6] or None

print(
    json.dumps(
        {
            "label": label,
            "path": rel_path,
            "sha256": sha,
            "size_bytes": size,
            "installed_at_utc": installed_at_utc,
            "resolved_path_abs": resolved_path_abs,
        },
        sort_keys=True,
    )
)
PY
}
