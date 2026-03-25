#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cargo_path=""
if [[ -x "${HOME}/.cargo/bin/cargo" ]]; then
    cargo_path="${HOME}/.cargo/bin/cargo"
else
    cargo_path="$(command -v cargo 2>/dev/null || true)"
fi

if [[ -z "${cargo_path}" || ! -x "${cargo_path}" ]]; then
    echo "missing cargo binary; expected ${HOME}/.cargo/bin/cargo or cargo on PATH" >&2
    exit 1
fi

echo "installing cargo-nextest with cargo install --locked"
"${cargo_path}" install --locked cargo-nextest

echo "installed cargo-nextest:"
"${cargo_path}" nextest --version
