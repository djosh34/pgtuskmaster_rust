#!/usr/bin/env bash

set -euo pipefail

readonly DEFAULT_CONFIG_PATH="/etc/pgtuskmaster/runtime.toml"

die() {
    printf 'pgtuskmaster entrypoint error: %s\n' "$*" >&2
    exit 1
}

require_readable_file() {
    local label="$1"
    local path="$2"

    [[ -e "${path}" ]] || die "${label} does not exist: ${path}"
    [[ -f "${path}" ]] || die "${label} is not a regular file: ${path}"
    [[ -r "${path}" ]] || die "${label} is not readable: ${path}"
}

require_executable_file() {
    local label="$1"
    local path="$2"

    [[ -e "${path}" ]] || die "${label} does not exist: ${path}"
    [[ -f "${path}" ]] || die "${label} is not a regular file: ${path}"
    [[ -x "${path}" ]] || die "${label} is not executable: ${path}"
}

ensure_directory() {
    local label="$1"
    local path="$2"

    if [[ -e "${path}" ]]; then
        [[ -d "${path}" ]] || die "${label} exists but is not a directory: ${path}"
        [[ -w "${path}" ]] || die "${label} is not writable: ${path}"
        return
    fi

    mkdir -p "${path}" || die "failed to create ${label}: ${path}"
}

ensure_parent_directory() {
    local label="$1"
    local path="$2"
    local parent

    parent="$(dirname "${path}")"
    ensure_directory "${label} parent directory" "${parent}"
}

extract_first_key_path() {
    local key="$1"
    local config_path="$2"

    awk -v key="${key}" '
        $0 ~ "^[[:space:]]*" key "[[:space:]]*=[[:space:]]*\"" {
            line = $0
            sub(/^[^"]*"/, "", line)
            sub(/".*$/, "", line)
            print line
            exit
        }
    ' "${config_path}"
}

emit_section_paths() {
    local config_path="$1"

    awk '
        /^[[:space:]]*\[/ {
            section = $0
            gsub(/^[[:space:]]*\[/, "", section)
            gsub(/\][[:space:]]*$/, "", section)
            next
        }
        /path[[:space:]]*=[[:space:]]*"/ {
            path = $0
            sub(/^.*path[[:space:]]*=[[:space:]]*"/, "", path)
            sub(/".*$/, "", path)
            printf "%s\t%s\n", section, path
        }
    ' "${config_path}"
}

validate_runtime_contract() {
    local config_path="$1"
    local binary_name
    local binary_path
    local directory_key
    local directory_path
    local file_key
    local file_path
    local section_path
    local section
    local path

    for binary_name in postgres pg_ctl pg_rewind initdb pg_basebackup psql; do
        binary_path="$(extract_first_key_path "${binary_name}" "${config_path}")"
        [[ -n "${binary_path}" ]] || die "missing process.binaries.${binary_name} in ${config_path}"
        require_executable_file "process.binaries.${binary_name}" "${binary_path}"
    done

    for directory_key in data_dir socket_dir log_dir; do
        directory_path="$(extract_first_key_path "${directory_key}" "${config_path}")"
        if [[ -n "${directory_path}" ]]; then
            ensure_directory "${directory_key}" "${directory_path}"
        fi
    done

    for file_key in log_file pg_ctl_log_file; do
        file_path="$(extract_first_key_path "${file_key}" "${config_path}")"
        if [[ -n "${file_path}" ]]; then
            ensure_parent_directory "${file_key}" "${file_path}"
        fi
    done

    while IFS=$'\t' read -r section path; do
        [[ -n "${path}" ]] || continue

        case "${section}" in
            postgres.pg_hba|postgres.pg_ident|postgres.roles.superuser.auth.password|postgres.roles.replicator.auth.password|postgres.roles.rewinder.auth.password|postgres.tls.identity|postgres.tls.client_auth|api.security.tls.identity|api.security.tls.client_auth)
                require_readable_file "${section}.path" "${path}"
                ;;
            logging.sinks.file)
                ensure_parent_directory "${section}.path" "${path}"
                ;;
            *)
                if [[ "${path}" == /run/secrets/* ]]; then
                    require_readable_file "${section}.path" "${path}"
                fi
                ;;
        esac
    done < <(emit_section_paths "${config_path}")

    while IFS= read -r section_path; do
        [[ -n "${section_path}" ]] || continue
        require_readable_file "referenced docker secret" "${section_path}"
    done < <(
        grep -oE '/run/secrets/[^"[:space:],}]+' "${config_path}" | sort -u
    )
}

main() {
    local config_path="${PGTUSKMASTER_CONFIG:-${DEFAULT_CONFIG_PATH}}"

    umask 077

    if [[ "$#" -eq 0 ]]; then
        set -- /usr/local/bin/pgtuskmaster --config "${config_path}"
    elif [[ "$1" == "pgtuskmaster" || "$1" == "/usr/local/bin/pgtuskmaster" ]]; then
        set -- /usr/local/bin/pgtuskmaster --config "${config_path}" "${@:2}"
    else
        exec "$@"
    fi

    require_readable_file "runtime config" "${config_path}"
    validate_runtime_contract "${config_path}"

    exec "$@"
}

main "$@"
