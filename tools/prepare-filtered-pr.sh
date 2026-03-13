#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

readonly UPSTREAM_REF="${FILTERED_PR_UPSTREAM_REF:-origin/master}"
readonly SOURCE_REF="${FILTERED_PR_SOURCE_REF:-HEAD}"
readonly FORCE_RECREATE="${FILTERED_PR_FORCE:-0}"

TEMP_WORKTREE=""
CLEANUP_BASE_BRANCH=""
CLEANUP_HEAD_BRANCH=""
SUCCESS="false"

ALLOWED_PATHS=(
    "src"
    "docker"
    "tests"
)

die() {
    echo "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: ./tools/prepare-filtered-pr.sh <name>

Creates two local branches for a PR whose diff is limited to the allowlisted
paths declared near the top of this script.

Branch names:
  <name>-base
  <name>-head

Defaults:
  source ref   FILTERED_PR_SOURCE_REF=HEAD
  upstream ref FILTERED_PR_UPSTREAM_REF=origin/master

Optional:
  FILTERED_PR_FORCE=1 recreates existing local <name>-base / <name>-head

Result:
  Open the PR from <name>-head into <name>-base.
EOF
}

warn_if_worktree_dirty() {
    local status_output=""
    status_output="$(git status --short)"
    if [[ -n "${status_output}" ]]; then
        echo "warning: working tree is dirty; this tool uses ${SOURCE_REF} only and ignores uncommitted changes" >&2
    fi
}

validate_branch_name() {
    local branch_name="$1"
    git check-ref-format --branch "${branch_name}" >/dev/null 2>&1 || \
        die "invalid branch name: ${branch_name}"
}

branch_exists() {
    local branch_name="$1"
    git show-ref --verify --quiet "refs/heads/${branch_name}"
}

reset_branch_if_needed() {
    local branch_name="$1"

    if ! branch_exists "${branch_name}"; then
        return
    fi

    if [[ "${FORCE_RECREATE}" != "1" ]]; then
        die "branch already exists: ${branch_name} (set FILTERED_PR_FORCE=1 to recreate it)"
    fi

    git branch -D "${branch_name}" >/dev/null
}

verify_allowlisted_diff() {
    local base_branch="$1"
    local head_branch="$2"
    local diff_path=""

    while IFS= read -r diff_path; do
        [[ -z "${diff_path}" ]] && continue

        local allowed="false"
        local path_prefix=""
        for path_prefix in "${ALLOWED_PATHS[@]}"; do
            if [[ "${diff_path}" == "${path_prefix}/"* ]] || [[ "${diff_path}" == "${path_prefix}" ]]; then
                allowed="true"
                break
            fi
        done

        if [[ "${allowed}" != "true" ]]; then
            die "verification failed: ${base_branch}...${head_branch} still includes non-allowlisted path ${diff_path}"
        fi
    done < <(git diff --name-only "${base_branch}...${head_branch}")
}

cleanup() {
    if [[ -n "${TEMP_WORKTREE}" ]]; then
        git worktree remove --force "${TEMP_WORKTREE}" >/dev/null 2>&1 || true
        rm -rf -- "${TEMP_WORKTREE}" >/dev/null 2>&1 || true
    fi

    if [[ "${SUCCESS}" == "true" ]]; then
        return
    fi

    if [[ -n "${CLEANUP_HEAD_BRANCH}" ]] && branch_exists "${CLEANUP_HEAD_BRANCH}"; then
        git branch -D "${CLEANUP_HEAD_BRANCH}" >/dev/null 2>&1 || true
    fi
    if [[ -n "${CLEANUP_BASE_BRANCH}" ]] && branch_exists "${CLEANUP_BASE_BRANCH}"; then
        git branch -D "${CLEANUP_BASE_BRANCH}" >/dev/null 2>&1 || true
    fi
}

trap cleanup EXIT

main() {
    if [[ "$#" -ne 1 ]]; then
        usage >&2
        exit 2
    fi

    cd -- "${REPO_ROOT}"

    git rev-parse --show-toplevel >/dev/null 2>&1 || die "not inside a git repository"
    warn_if_worktree_dirty

    local name="$1"
    local base_branch="${name}-base"
    local head_branch="${name}-head"
    CLEANUP_BASE_BRANCH="${base_branch}"
    CLEANUP_HEAD_BRANCH="${head_branch}"
    validate_branch_name "${base_branch}"
    validate_branch_name "${head_branch}"

    git rev-parse --verify "${UPSTREAM_REF}^{commit}" >/dev/null 2>&1 || \
        die "upstream ref not found: ${UPSTREAM_REF}"
    git rev-parse --verify "${SOURCE_REF}^{commit}" >/dev/null 2>&1 || \
        die "source ref not found: ${SOURCE_REF}"

    local source_commit=""
    source_commit="$(git rev-parse "${SOURCE_REF}^{commit}")"
    local merge_base=""
    merge_base="$(git merge-base "${UPSTREAM_REF}" "${source_commit}")"

    reset_branch_if_needed "${base_branch}"
    reset_branch_if_needed "${head_branch}"

    TEMP_WORKTREE="$(mktemp -d "${TMPDIR:-/tmp}/prepare-filtered-pr.XXXXXX")"

    git worktree add --detach "${TEMP_WORKTREE}" "${merge_base}" >/dev/null

    local disallowed_patch="${TEMP_WORKTREE}/disallowed.patch"
    local allowed_patch="${TEMP_WORKTREE}/allowed.patch"
    local exclude_specs=(".")
    local path_prefix=""
    for path_prefix in "${ALLOWED_PATHS[@]}"; do
        exclude_specs+=(":(exclude)${path_prefix}")
    done

    (
        cd -- "${TEMP_WORKTREE}"

        git checkout -B "${base_branch}" "${merge_base}" >/dev/null
        git diff --binary "${merge_base}" "${source_commit}" -- "${exclude_specs[@]}" > "${disallowed_patch}"
        if [[ -s "${disallowed_patch}" ]]; then
            git apply --index "${disallowed_patch}"
            git -c core.hooksPath=/dev/null commit -m "Seed ${base_branch} with shared non-PR changes" >/dev/null
        fi

        git checkout -B "${head_branch}" >/dev/null
        git diff --binary "${merge_base}" "${source_commit}" -- "${ALLOWED_PATHS[@]}" > "${allowed_patch}"
        if [[ -s "${allowed_patch}" ]]; then
            git apply --index "${allowed_patch}"
            git -c core.hooksPath=/dev/null commit -m "Add allowlisted changes for ${name}" >/dev/null
        fi
    )

    verify_allowlisted_diff "${base_branch}" "${head_branch}"
    SUCCESS="true"

    echo "Created filtered PR branches:"
    echo "  base: ${base_branch}"
    echo "  head: ${head_branch}"
    echo "  upstream: ${UPSTREAM_REF}"
    echo "  source: ${SOURCE_REF} (${source_commit})"
    echo "  merge-base: ${merge_base}"
    echo
    echo "Open the PR from ${head_branch} into ${base_branch}."
    echo "Verify with: git diff --name-only ${base_branch}...${head_branch}"
}

main "$@"
