#!/usr/bin/env bash

set -euo pipefail

readonly WIPE_FILE="/var/lib/pgtuskmaster/faults/wipe-data-on-start"
readonly DATA_DIR="/var/lib/postgresql/data"

for entry in \
    "/var/lib/pgtuskmaster/faults/clear-block-pg-basebackup-on-start:/var/lib/pgtuskmaster/faults/block-pg-basebackup" \
    "/var/lib/pgtuskmaster/faults/clear-fail-pg-rewind-on-start:/var/lib/pgtuskmaster/faults/fail-pg-rewind" \
    "/var/lib/pgtuskmaster/faults/clear-fail-postgres-start-on-start:/var/lib/pgtuskmaster/faults/fail-postgres-start"
do
    clear_marker="${entry%%:*}"
    blocker_marker="${entry#*:}"
    if [[ -f "${clear_marker}" ]]; then
        rm -f "${blocker_marker}" "${clear_marker}"
    fi
done

if [[ -f "${WIPE_FILE}" ]]; then
    printf 'entrypoint wiping data directory because flag exists: %s\n' "${WIPE_FILE}" >&2
    find "${DATA_DIR}" -mindepth 1 -maxdepth 1 -exec rm -rf {} +
    rm -f "${WIPE_FILE}"
fi

exec /usr/local/bin/pgtuskmaster --config /etc/pgtuskmaster/runtime.toml
