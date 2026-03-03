---
## Task: Install mdBook docs framework and enforce artifact git hygiene <status>completed</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Use mdBook for this Rust project, install it, prove it renders a static HTML site correctly, and lock down strict git artifact hygiene before any docs commits.

**Scope:**
- No framework research or comparison is required for this task.
- The framework choice is fixed: mdBook must be used.
- Install and bootstrap mdBook in-repo with a clear docs project structure.
- Prove the docs site renders and builds static output end-to-end (local dev preview plus production/static build output).
- Identify actual generated artifacts/folders by running the framework (do not guess), then update `.gitignore` accordingly.
- Enforce “no generated artifacts in git” policy:
  - `node_modules` must be ignored if Node tooling is used.
  - Built outputs (`book/`, `.mdbook/`, or whatever is truly produced) must be ignored based on observed output.
  - Verify ignored behavior with `git add` checks before commit.
  - If generated artifacts were staged/tracked accidentally, remove from index/history in this branch before final commit.

**Context from research:**
- User decision is final: mdBook is required for this task.
- Do not spend time on framework selection research in this task.
- This task should establish the mdBook platform and repository hygiene guardrails so following docs tasks can focus purely on content quality.

**Expected outcome:**
- mdBook is installed and working, static HTML output is verified, and artifact ignore rules are proven effective with clean git status/add behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible, but skip framework research/selection.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: mdBook scaffold added, dev server and static build commands documented and validated, rendered static output directory confirmed from real build logs, and mdBook choice recorded as a fixed requirement (no comparison research)
- [x] Full exhaustive checklist completed for git hygiene: `.gitignore` updated only after observing produced artifacts, includes all generated dependency/build output (for chosen framework), `git add -n`/staging checks demonstrate artifacts are ignored, if artifacts were previously staged/tracked then they are removed from index and cleaned from branch history before final commit
- [x] No generated docs artifacts committed (`node_modules`, build output folders, caches) and verification evidence captured in task notes
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2 — Skeptically Verified)

Notes:
- Subagent parallelism target (16+) could not be met due to runtime agent thread cap (max 6). Compensated by assigning multiple review dimensions per subagent + manual repo inspection.

### Phase 0 — Preflight (keep repo clean and reproducible)
- [x] Create evidence folder: `.ralph/evidence/task-31-mdbook/`
- [x] Record baseline repo state for later comparison (repo is expected to be “dirty” from `.ralph/` bookkeeping):
  - [x] `git status --porcelain > .ralph/evidence/task-31-mdbook/baseline-status.txt`
  - [x] `git diff --name-only > .ralph/evidence/task-31-mdbook/baseline-changed-files.txt`
- [x] Record environment facts for later debugging (OS, arch, rust/cargo versions):
  - [x] Capture: `uname -a`, `rustc -V`, `cargo -V`, `git rev-parse HEAD`
- [x] Fail fast if generated mdBook artifacts are already tracked (policy violation must be corrected before proceeding):
  - [x] `git ls-files docs/book docs/.mdbook > .ralph/evidence/task-31-mdbook/preflight-tracked-generated.txt`
  - [x] If the file is non-empty: `git rm -r --cached docs/book docs/.mdbook` (not needed; preflight was empty)

### Phase 1 — Install mdBook (in-repo, reproducible)
**Goal:** Avoid “works on my machine” by providing a single, repeatable installer that does not commit artifacts.

- [x] Add a new installer script: `tools/install-mdbook.sh`
  - [x] Mirror conventions from `tools/install-etcd.sh` (download a pinned upstream release asset; install into `.tools/`):
    - [x] Install into `.tools/mdbook/bin/mdbook` (already ignored by `.gitignore` via `.tools/`)
    - [x] Refuse unsupported OS/arch with a clear error message
    - [x] Preflight required commands: `curl`, `tar`, `mktemp`, and a SHA256 tool (`sha256sum` preferred; allow `shasum -a 256` fallback)
    - [x] Use a temp directory + cleanup trap; only copy the single expected `mdbook` binary from the extracted archive
    - [x] Verify SHA256 checksum against a value committed inside the script (fail closed: no checksum => no install)
    - [x] Atomic install (write to temp path, then `mv` into final bin path)
    - [x] Print installed mdBook version at the end and assert it matches the pinned version
  - [x] Pin mdBook to an explicit version tag (never “latest”) and record:
    - [x] computed download URL
    - [x] expected checksum
    - [x] actual checksum
    - [x] resulting `mdbook --version`
  - [x] Fallback policy (only if needed; still pinned):
    - [x] If the pinned binary asset is not available for this platform (or libc mismatch is detected), fallback to: `cargo install mdbook --locked --root .tools/mdbook --version <PINNED_VERSION>` (not needed; upstream release asset worked)
    - [x] Log that the fallback path was used and why (not needed)
  - [x] Capture installer output in `.ralph/evidence/task-31-mdbook/install-mdbook.log` (ensure `pipefail` so failures are not masked by `tee`)

- [x] Add `Makefile` docs helper targets:
  - [x] `docs-build`: runs mdBook static build (fails with guidance if mdBook missing)
  - [x] `docs-serve`: runs mdBook dev server (bind to localhost; explicit port)
  - [x] `docs-hygiene`: fails if generated artifacts are tracked (enforces the “no generated artifacts in git” policy)
  - [x] Use a repo-local mdBook path: `MDBOOK := .tools/mdbook/bin/mdbook`

### Phase 2 — Bootstrap the docs tree (mdBook scaffold + minimal IA)
**Goal:** Create a stable `docs/` layout that unblocks task 32/33 without writing architecture content yet.

  - [x] Run `mdbook init --help` once and record the effective command line in task notes (avoid stale flag assumptions)
  - [x] Run `mdbook init docs` non-interactively:
    - [x] Use `--force` and `--title`
    - [x] Use `--ignore none` (avoid creating `docs/.gitignore`; keep ignore policy centralized in repo root `.gitignore`)
  - [x] Ensure outputs land under `docs/` (do not create a top-level `book/`)

- [x] Set initial information architecture to match the upcoming docs tasks:
  - [x] Edit `docs/src/SUMMARY.md` to include placeholders like:
    - [x] Introduction / Scope
    - [x] Architecture overview (placeholder)
    - [x] Components (placeholder sections)
    - [x] Operations / Running / Testing (placeholder)
    - [x] Glossary (placeholder)
  - [x] Keep page content minimal and “no code dumps” aligned with task 32 expectations.

- [x] Add a short operator-facing “how to build/serve docs” page:
  - [x] Implemented as `docs/src/operations.md` (single source of truth, in-book)
  - [x] Document:
    - [x] `tools/install-mdbook.sh`
    - [x] `make docs-build`
    - [x] `make docs-serve`

### Phase 3 — Prove dev preview + static build end-to-end (and observe artifacts)
**Goal:** Run the framework for real, capture logs, then ignore exactly what is produced.

- [x] Build static output and capture logs:
  - [x] Run `make docs-build`
  - [x] Record:
    - [x] Exact output directory path created (expected: `docs/book/`)
    - [x] Any cache/state directory created (do not assume one exists; only ignore if observed) (none observed)
  - [x] Capture logs in `.ralph/evidence/task-31-mdbook/docs-build.log`

- [x] Prove dev server starts (local preview):
  - [x] Start server in background with explicit host/port (example: `127.0.0.1:3000`)
  - [x] Verify with `curl -f http://127.0.0.1:3000/` (or equivalent)
  - [x] Stop server and confirm it shuts down cleanly
  - [x] Capture logs in `.ralph/evidence/task-31-mdbook/docs-serve.log`

- [x] Snapshot produced artifacts for hygiene work:
  - [x] `find docs -maxdepth 2 -type d -print` after the build
  - [x] Save output to `.ralph/evidence/task-31-mdbook/docs-tree-after-build.txt`

### Phase 4 — Enforce strict git artifact hygiene (based on observed output)
**Goal:** Ensure mdBook outputs are never committed and cannot be staged accidentally.

- [x] Update `.gitignore` only after Phase 3 observation:
  - [x] Add ignore entries for the *actual* generated build output directory (expected: `/docs/book/`)
  - [x] Add ignore entries for any *observed* cache directory (only if actually present; do not assume `/docs/.mdbook/`) (none observed)
  - [x] If Node tooling is added later (not expected for pure mdBook), also ignore `node_modules/` at that time. (not applicable here)

- [x] Prove ignore behavior (include evidence in task notes):
  - [x] `git check-ignore -v <each-generated-path>` for at least one file under each generated directory
  - [x] `git add -n docs/` to show generated files are not staged
  - [x] `git status --porcelain` remains clean (or only shows intentional, tracked edits)
  - [x] Run `make docs-hygiene` to enforce: no tracked/stageable generated docs output
  - [x] Capture in `.ralph/evidence/task-31-mdbook/git-hygiene.log`

- [x] Ensure no generated artifacts are tracked:
  - [x] `git ls-files docs/book` (and any other observed generated dir) must return empty
  - [ ] If anything is tracked/staged accidentally:
    - [x] `git rm -r --cached <dir>` (not needed)
    - [x] If it was committed in this branch, rewrite/drop that commit before finishing task 31 (not needed)

- [ ] Remove generated dirs from working tree before final commit (optional but recommended):
  - [ ] `rm -rf <generated-dirs>` (optional; not performed — `rm -rf` is blocked by policy in this session)

### Phase 5 — Run all required quality gates (no skips)
**Goal:** Do not mark task complete until all gates pass exactly as required.

- [x] Ensure real-binary prerequisites exist for tests that require them:
  - [x] If `.tools/etcd/bin/etcd` missing: run `tools/install-etcd.sh` and re-run gate (not needed; present)
  - [x] If postgres16 tools missing: run `tools/install-postgres16.sh` (requires AlmaLinux/RHEL-like + `sudo dnf`) and re-run gate (not needed; present)

- [x] Run gates with deterministic/anti-flake settings and capture logs (exit status is the source of truth; grep markers are supplemental evidence only):
  - [x] Run docs build first (keep it separate from `make check` semantics):
    - [x] `set -o pipefail; make docs-build |& tee .ralph/evidence/task-31-mdbook/make-docs-build.log`
  - [x] `cargo clean` once (reduces intermittent mount/link flakes)
  - [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make check |& tee .ralph/evidence/task-31-mdbook/make-check.log`
  - [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test |& tee .ralph/evidence/task-31-mdbook/make-test.log`
  - [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test |& tee .ralph/evidence/task-31-mdbook/make-test.log`
  - [x] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make lint |& tee .ralph/evidence/task-31-mdbook/make-lint.log`
  - [x] For `make test` + `make lint` logs, grep for markers (supplemental evidence only):
    - [x] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-31-mdbook/make-test.log` (no matches; captured in `make-test-markers.txt`)
    - [x] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-31-mdbook/make-lint.log` (no matches; captured in `make-lint-markers.txt`)

### Phase 6 — Finish task (only after all gates are green)
- [x] Update this task file:
  - [x] Tick all acceptance checkboxes
  - [x] Set `<status>completed</status>`, `<passes>true</passes>`, and `<passing>true</passing>`
  - [x] Add a short “evidence” note pointing at `.ralph/evidence/task-31-mdbook/`
- [x] Run `/bin/bash .ralph/task_switch.sh` (task handoff protocol)
- [x] `git add -A` (including `.ralph` updates) and confirm no generated artifacts are staged
- [x] Commit message format:
  - [x] `task finished 31-task-docs-framework-selection-install-and-artifact-hygiene: <summary + evidence>`
- [x] `git push`
- [x] Append any learnings/surprises to `AGENTS.md`

Evidence (logs + commands): `.ralph/evidence/task-31-mdbook/`
- installer: `install-mdbook.log`
- docs init + help: `mdbook-init.log`, `mdbook-init-help.txt`
- docs build + serve: `docs-build.log`, `docs-serve.log`, `docs-tree-after-build.txt`
- git hygiene proof: `git-hygiene.log`
- gates: `make-docs-build.log`, `make-check.log`, `make-test.log`, `make-lint.log`

COMPLETED
</execution_plan>
