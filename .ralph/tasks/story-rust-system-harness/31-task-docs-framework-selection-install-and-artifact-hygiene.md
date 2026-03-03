---
## Task: Install mdBook docs framework and enforce artifact git hygiene <status>not_started</status> <passes>false</passes> <passing>false</passing>

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
- [ ] Full exhaustive checklist completed with concrete module requirements: mdBook scaffold added, dev server and static build commands documented and validated, rendered static output directory confirmed from real build logs, and mdBook choice recorded as a fixed requirement (no comparison research)
- [ ] Full exhaustive checklist completed for git hygiene: `.gitignore` updated only after observing produced artifacts, includes all generated dependency/build output (for chosen framework), `git add -n`/staging checks demonstrate artifacts are ignored, if artifacts were previously staged/tracked then they are removed from index and cleaned from branch history before final commit
- [ ] No generated docs artifacts committed (`node_modules`, build output folders, caches) and verification evidence captured in task notes
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2 — Skeptically Verified)

Notes:
- Subagent parallelism target (16+) could not be met due to runtime agent thread cap (max 6). Compensated by assigning multiple review dimensions per subagent + manual repo inspection.

### Phase 0 — Preflight (keep repo clean and reproducible)
- [ ] Create evidence folder: `.ralph/evidence/task-31-mdbook/`
- [ ] Record baseline repo state for later comparison (repo is expected to be “dirty” from `.ralph/` bookkeeping):
  - [ ] `git status --porcelain > .ralph/evidence/task-31-mdbook/baseline-status.txt`
  - [ ] `git diff --name-only > .ralph/evidence/task-31-mdbook/baseline-changed-files.txt`
- [ ] Record environment facts for later debugging (OS, arch, rust/cargo versions):
  - [ ] Capture: `uname -a`, `rustc -V`, `cargo -V`, `git rev-parse HEAD`
- [ ] Fail fast if generated mdBook artifacts are already tracked (policy violation must be corrected before proceeding):
  - [ ] `git ls-files docs/book docs/.mdbook > .ralph/evidence/task-31-mdbook/preflight-tracked-generated.txt`
  - [ ] If the file is non-empty: `git rm -r --cached docs/book docs/.mdbook` (only the paths that exist), and record the action in task notes.

### Phase 1 — Install mdBook (in-repo, reproducible)
**Goal:** Avoid “works on my machine” by providing a single, repeatable installer that does not commit artifacts.

- [ ] Add a new installer script: `tools/install-mdbook.sh`
  - [ ] Mirror conventions from `tools/install-etcd.sh` (download a pinned upstream release asset; install into `.tools/`):
    - [ ] Install into `.tools/mdbook/bin/mdbook` (already ignored by `.gitignore` via `.tools/`)
    - [ ] Refuse unsupported OS/arch with a clear error message
    - [ ] Preflight required commands: `curl`, `tar`, `mktemp`, and a SHA256 tool (`sha256sum` preferred; allow `shasum -a 256` fallback)
    - [ ] Use a temp directory + cleanup trap; only copy the single expected `mdbook` binary from the extracted archive
    - [ ] Verify SHA256 checksum against a value committed inside the script (fail closed: no checksum => no install)
    - [ ] Atomic install (write to temp path, then `mv` into final bin path)
    - [ ] Print installed mdBook version at the end and assert it matches the pinned version
  - [ ] Pin mdBook to an explicit version tag (never “latest”) and record:
    - [ ] computed download URL
    - [ ] expected checksum
    - [ ] actual checksum
    - [ ] resulting `mdbook --version`
  - [ ] Fallback policy (only if needed; still pinned):
    - [ ] If the pinned binary asset is not available for this platform (or libc mismatch is detected), fallback to: `cargo install mdbook --locked --root .tools/mdbook --version <PINNED_VERSION>`
    - [ ] Log that the fallback path was used and why
  - [ ] Capture installer output in `.ralph/evidence/task-31-mdbook/install-mdbook.log` (ensure `pipefail` so failures are not masked by `tee`)

- [ ] Add `Makefile` + `makefile` docs helper targets (keep both files identical):
  - [ ] `docs-build`: runs mdBook static build (fails with guidance if mdBook missing)
  - [ ] `docs-serve`: runs mdBook dev server (bind to localhost; explicit port)
  - [ ] `docs-clean`: removes generated build artifacts (only what mdBook generates)
  - [ ] `docs-hygiene`: fails if generated artifacts are tracked or stageable (enforces the “no generated artifacts in git” policy)
  - [ ] Use an overridable variable: `MDBOOK ?= .tools/mdbook/bin/mdbook`
  - [ ] After editing, require `diff -u Makefile makefile` to be empty

### Phase 2 — Bootstrap the docs tree (mdBook scaffold + minimal IA)
**Goal:** Create a stable `docs/` layout that unblocks task 32/33 without writing architecture content yet.

- [ ] Create a new mdBook project rooted at `docs/`:
  - [ ] Run `mdbook init --help` once and record the effective command line in task notes (avoid stale flag assumptions)
  - [ ] Run `mdbook init docs` non-interactively:
    - [ ] Use `--force` and `--title`
    - [ ] Use `--ignore none` (avoid creating `docs/.gitignore`; keep ignore policy centralized in repo root `.gitignore`)
  - [ ] Ensure outputs land under `docs/` (do not create a top-level `book/`)

- [ ] Set initial information architecture to match the upcoming docs tasks:
  - [ ] Edit `docs/src/SUMMARY.md` to include placeholders like:
    - [ ] Introduction / Scope
    - [ ] Architecture overview (placeholder)
    - [ ] Components (placeholder sections)
    - [ ] Operations / Running / Testing (placeholder)
    - [ ] Glossary (placeholder)
  - [ ] Keep page content minimal and “no code dumps” aligned with task 32 expectations.

- [ ] Add a short operator-facing “how to build/serve docs” page:
  - [ ] Either `docs/src/building.md` (in-book) or `docs/README.md` (out-of-book), but keep a single source of truth.
  - [ ] Document:
    - [ ] `tools/install-mdbook.sh`
    - [ ] `make docs-build`
    - [ ] `make docs-serve`

### Phase 3 — Prove dev preview + static build end-to-end (and observe artifacts)
**Goal:** Run the framework for real, capture logs, then ignore exactly what is produced.

- [ ] Build static output and capture logs:
  - [ ] Run `make docs-build`
  - [ ] Record:
    - [ ] Exact output directory path created (expected: `docs/book/`)
    - [ ] Any cache/state directory created (do not assume one exists; only ignore if observed)
  - [ ] Capture logs in `.ralph/evidence/task-31-mdbook/docs-build.log`

- [ ] Prove dev server starts (local preview):
  - [ ] Start server in background with explicit host/port (example: `127.0.0.1:3000`)
  - [ ] Verify with `curl -f http://127.0.0.1:3000/` (or equivalent)
  - [ ] Stop server and confirm it shuts down cleanly
  - [ ] Capture logs in `.ralph/evidence/task-31-mdbook/docs-serve.log`

- [ ] Snapshot produced artifacts for hygiene work:
  - [ ] `find docs -maxdepth 2 -type d -print` after the build
  - [ ] Save output to `.ralph/evidence/task-31-mdbook/docs-tree-after-build.txt`

### Phase 4 — Enforce strict git artifact hygiene (based on observed output)
**Goal:** Ensure mdBook outputs are never committed and cannot be staged accidentally.

- [ ] Update `.gitignore` only after Phase 3 observation:
  - [ ] Add ignore entries for the *actual* generated build output directory (expected: `/docs/book/`)
  - [ ] Add ignore entries for any *observed* cache directory (only if actually present; do not assume `/docs/.mdbook/`)
  - [ ] If Node tooling is added later (not expected for pure mdBook), also ignore `node_modules/` at that time.

- [ ] Prove ignore behavior (include evidence in task notes):
  - [ ] `git check-ignore -v <each-generated-path>` for at least one file under each generated directory
  - [ ] `git add -n docs/` to show generated files are not staged
  - [ ] `git status --porcelain` remains clean (or only shows intentional, tracked edits)
  - [ ] Run `make docs-hygiene` to enforce: no tracked/stageable generated docs output
  - [ ] Capture in `.ralph/evidence/task-31-mdbook/git-hygiene.log`

- [ ] Ensure no generated artifacts are tracked:
  - [ ] `git ls-files docs/book` (and any other observed generated dir) must return empty
  - [ ] If anything is tracked/staged accidentally:
    - [ ] `git rm -r --cached <dir>`
    - [ ] If it was committed in this branch, rewrite/drop that commit before finishing task 31

- [ ] Remove generated dirs from working tree before final commit (optional but recommended):
  - [ ] `rm -rf <generated-dirs>` (they should reappear on next build anyway)

### Phase 5 — Run all required quality gates (no skips)
**Goal:** Do not mark task complete until all gates pass exactly as required.

- [ ] Ensure real-binary prerequisites exist for tests that require them:
  - [ ] If `.tools/etcd/bin/etcd` missing: run `tools/install-etcd.sh` and re-run gate
  - [ ] If postgres16 tools missing: run `tools/install-postgres16.sh` (requires AlmaLinux/RHEL-like + `sudo dnf`) and re-run gate

- [ ] Run gates with deterministic/anti-flake settings and capture logs (exit status is the source of truth; grep markers are supplemental evidence only):
  - [ ] Run docs build first (keep it separate from `make check` semantics):
    - [ ] `set -o pipefail; make docs-build |& tee .ralph/evidence/task-31-mdbook/make-docs-build.log`
  - [ ] `cargo clean` once (reduces intermittent mount/link flakes)
  - [ ] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make check |& tee .ralph/evidence/task-31-mdbook/make-check.log`
  - [ ] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test |& tee .ralph/evidence/task-31-mdbook/make-test.log`
  - [ ] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test-bdd |& tee .ralph/evidence/task-31-mdbook/make-test-bdd.log`
  - [ ] `set -o pipefail; CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make lint |& tee .ralph/evidence/task-31-mdbook/make-lint.log`
  - [ ] For `make test` + `make lint` logs, grep for markers (supplemental evidence only):
    - [ ] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-31-mdbook/make-test.log`
    - [ ] `rg -n \"congratulations|evaluation failed\" .ralph/evidence/task-31-mdbook/make-lint.log`

### Phase 6 — Finish task (only after all gates are green)
- [ ] Update this task file:
  - [ ] Tick all acceptance checkboxes
  - [ ] Set `<status>completed</status>`, `<passes>true</passes>`, and `<passing>true</passing>`
  - [ ] Add a short “evidence” note pointing at `.ralph/evidence/task-31-mdbook/`
- [ ] Run `/bin/bash .ralph/task_switch.sh` (task handoff protocol)
- [ ] `git add -A` (including `.ralph` updates) and confirm no generated artifacts are staged
- [ ] Commit message format:
  - [ ] `task finished 31-task-docs-framework-selection-install-and-artifact-hygiene: <summary + evidence>`
- [ ] `git push`
- [ ] Append any learnings/surprises to `AGENTS.md`

NOW EXECUTE
</execution_plan>
