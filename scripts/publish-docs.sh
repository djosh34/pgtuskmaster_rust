#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_SOURCE="$REPO_ROOT/docs/book"
DOCS_TARGET="$REPO_ROOT/../pgtuskmaster-docs"
DOCS_TARGET_DIST="$DOCS_TARGET/dist"
DOCS_REPO="djosh34/pgtuskmaster-docs"
MDBOOK_BIN="$REPO_ROOT/.tools/mdbook/bin/mdbook"
MDBOOK_MERMAID_BIN="$REPO_ROOT/.tools/mdbook/bin/mdbook-mermaid"

# --- Bootstrap: create ../pgtuskmaster-docs if it doesn't exist ---
if [[ ! -d "$DOCS_TARGET/.git" ]]; then
  echo "Setting up ../pgtuskmaster-docs for the first time..."
  mkdir -p "$DOCS_TARGET"
  git -C "$DOCS_TARGET" init
  git -C "$DOCS_TARGET" checkout -b main
  git -C "$DOCS_TARGET" remote add origin "https://github.com/$DOCS_REPO.git"
fi

# --- Tooling: ensure mdBook + mdbook-mermaid are installed ---
if [[ ! -x "$MDBOOK_BIN" ]]; then
  echo "mdBook not found; installing pinned version..."
  "$REPO_ROOT/tools/install-mdbook.sh"
fi

if [[ ! -x "$MDBOOK_MERMAID_BIN" ]]; then
  echo "mdbook-mermaid not found; installing pinned version..."
  "$REPO_ROOT/tools/install-mdbook-mermaid.sh"
fi

# --- Build docs ---
PATH="$REPO_ROOT/.tools/mdbook/bin:$PATH" "$MDBOOK_BIN" build "$REPO_ROOT/docs"

# --- Sanity checks ---
if [[ ! -f "$DOCS_TARGET/.env.local" ]]; then
  echo "../pgtuskmaster-docs/.env.local not found. Create it with DOCS_GITHUB_TOKEN=ghp_..."
  exit 1
fi

if [[ ! -d "$DOCS_SOURCE" ]]; then
  echo "mdBook output not found at docs/book. Build failed or output path changed."
  exit 1
fi

# Token lives in the docs repo itself
# shellcheck source=/dev/null
source "$DOCS_TARGET/.env.local"

if [[ -z "${DOCS_GITHUB_TOKEN:-}" ]]; then
  echo "DOCS_GITHUB_TOKEN not set in ../pgtuskmaster-docs/.env.local"
  exit 1
fi

REMOTE="https://x-access-token:${DOCS_GITHUB_TOKEN}@github.com/$DOCS_REPO.git"

# Always keep remote URL in sync with current token
git -C "$DOCS_TARGET" remote set-url origin "$REMOTE"

# Pull latest if there are any remote commits
git -C "$DOCS_TARGET" fetch origin main 2>/dev/null && \
  git -C "$DOCS_TARGET" merge --ff-only origin/main 2>/dev/null || true

# Create wrangler config once if missing
if [[ ! -f "$DOCS_TARGET/wrangler.jsonc" ]]; then
  cat > "$DOCS_TARGET/wrangler.jsonc" << 'WRANGLER_EOF'
{
  "name": "pgtuskmaster-docs",
  "compatibility_date": "2026-02-22",
  "assets": {
    "directory": "./dist"
  }
}
WRANGLER_EOF
fi

# Wipe dist and replace with fresh mdBook output
rm -rf "$DOCS_TARGET_DIST"
mkdir -p "$DOCS_TARGET_DIST"
cp -r "$DOCS_SOURCE"/. "$DOCS_TARGET_DIST"/

# Stage everything
git -C "$DOCS_TARGET" add -A

if git -C "$DOCS_TARGET" diff --cached --quiet; then
  echo "No new changes to commit; skipping push."
  exit 0
else
  SHORT_SHA="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
  git -C "$DOCS_TARGET" commit -m "docs: sync mdbook from pgtuskmaster-rust $SHORT_SHA"
fi

# Push only when this run created a docs sync commit
git -C "$DOCS_TARGET" push origin main

echo "Docs published to github.com/$DOCS_REPO (dist from mdBook)"
