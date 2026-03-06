## Task: Run full verification for secure explicit config refactor <status>done</status> <passes>true</passes>

<description>
**Goal:** Execute full validation gates after the config refactor and convert any failures into actionable bug tasks.

**Scope:**
- Run full required project gates after merging upstream tasks.
- Record evidence logs and failure signatures.
- Create bug tasks for residual regressions with exact repro details.

**Context from research:**
- This refactor touches shared config, runtime wiring, auth, and tests; broad verification is mandatory.
- Real-binary and BDD coverage are required by project policy.

**Expected outcome:**
- End-state confidence that explicit secure startup config works across compile, lint, unit/integration, and BDD flows.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Evidence logs are captured for each gate and linked from the task update
- [x] Any failing gate has corresponding bug task(s) with repro and impacted modules (N/A: no failures observed)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 1)

### 0) Constraints (project policy)
- Do not skip tests. If a gate needs a real binary, install it and rerun.
- Do not introduce any new `unwrap`, `expect`, `panic`, `todo`, `unimplemented` usage anywhere (clippy restriction lints enforce this).
- Gate truth is command exit status. Grep-based checks are evidence only.

### 1) Preflight + evidence directory
- Create evidence directory:
  - `.ralph/evidence/story-secure-explicit-node-config/06-task-full-verification-for-secure-explicit-config-refactor/`
- Quick environment sanity (record output to evidence files so later failures are explainable):
  - `df -h . /tmp` -> `disk.txt`
  - `ulimit -a` -> `ulimit.txt`
  - If disk is low and prior test artifacts exist, delete only repo-owned temp dirs:
    - `rm -rf /tmp/pgtuskmaster-*` (record before/after `ls -ld /tmp/pgtuskmaster-*` in `tmp-cleanup.txt`)
- Capture baseline metadata (in files under the evidence directory):
  - `git rev-parse HEAD` -> `git-head.txt`
  - `git status --porcelain=v1` -> `git-status.txt`
  - `rustc -V` and `cargo -V` -> `toolchain.txt`
  - `uname -a` -> `system.txt`
- Capture gate-affecting env vars (recorded, not relied upon):
  - `env | rg -n '^(CARGO|RUST|TEST_TIMEOUT|PG|ETCD|OPENSSL|SSL)_'` -> `env.txt`
- Run every gate via `bash -lc 'set -o pipefail; ... 2>&1 | tee <log>'` to preserve exit codes and capture stderr.
- Default stability hardening for this workspace mount:
  - run with `CARGO_BUILD_JOBS=1` for `make check`/`make test`/`make test-long`/`make lint`
  - keep `CARGO_INCREMENTAL=0` (default in Makefile)
  - ensure `RUST_TEST_THREADS` is not set to `1` (Makefile rejects it for `make test`); safest is to `unset RUST_TEST_THREADS` in the gate wrapper shell

### 2) Prerequisite binaries (only if required by failures)
If any test fails due to missing real binaries, install them and rerun the failing gate(s) (retain both the failing log and the post-install rerun log).
- `timeout` / `gtimeout` missing:
  - on AlmaLinux: `sudo dnf install -y coreutils` (provides `timeout`), then rerun `make test`
- PostgreSQL 16 missing:
  - run `./tools/install-postgres16.sh`
- etcd missing:
  - run `./tools/install-etcd.sh`
- mdBook / mdbook-mermaid missing (only needed for `make docs-build`, not required by this task’s gates):
  - `./tools/install-mdbook.sh`
  - `./tools/install-mdbook-mermaid.sh`

### 3) Execute the required gates (single serial pass)
Run in this order to fail fast on lint and keep `make test` warmed by prior builds:
1. `make check` -> `make-check.log`
2. `make lint` -> `make-lint.log`
3. `make test` -> `make-test.log`
4. `make test-long` -> `make-test-long.log` (wrap with an external `timeout` to prevent indefinite hangs; if it times out, treat as a failing gate and file a bug)

### 4) Evidence extraction (always, even on pass)
- Marker grep evidence (policy-only, not pass/fail):
  - `rg -n "congratulations|evaluation failed" make-test.log` -> `grep-make-test-markers.log` (or write `not found`)
  - `rg -n "congratulations|evaluation failed" make-lint.log` -> `grep-make-lint-markers.log` (or write `not found`)
- Failure extract helpers (triage, not pass/fail):
  - `rg -n "FAILED|failures|error" make-test.log` -> `make-test-failures.log` (or write `not found`)
  - `rg -n "error\\[E[0-9]{4}\\]" make-lint.log` -> `make-lint-errors.log` (or write `not found`)

### 5) Failure triage map (exhaustive “where to edit” checklist)
Only modify files if a gate fails. For each failure class below, record the failing gate + the exact log path(s), then fix in the scoped area, then rerun the smallest reproducer first, then rerun the full required gates.

- **Build/compile (`make check`)**
  - Likely areas: `Cargo.toml`, `src/`, `tests/`, `examples/`, `docs/` (doc includes in tests), feature flags.
  - Requirements: fix compilation on `--all-targets`; do not weaken lints; keep changes minimal and targeted.

- **Unit/integration (`make test`)**
  - Likely areas:
    - Config parsing/validation: `src/config/`, `src/config_v2/` (if present), `src/config.rs`
    - CLI/config UX: `src/cli/` and `tests/cli_binary.rs`
    - Auth/roles/secrets: `src/auth/`, `src/pginfo/`, `src/secrets/`
    - TLS/rustls provider wiring: `src/tls.rs`, `src/test_harness/tls.rs`
    - Real-binary harness / HA tests: `src/test_harness/`, `src/ha/`, `tests/`
    - Examples compile: `examples/`
  - Requirements: no ignored tests; no “optional” binaries; install missing tools; keep deterministic timeouts; add/adjust tests when fixing behavior regressions.

- **Ultra-long suite (`make test-long`)**
  - Likely areas: `src/ha/e2e_multi_node.rs` and harness modules it pulls in.
  - Requirements: preserve long-scenario semantics; prefer fixing root causes (timeouts, port collisions, teardown) over increasing timeouts; ensure long tests remain long-only.

- **Lint (`make lint`)**
  - Likely areas:
    - Rust: `src/`, `tests/`, `examples/`
    - Docs architecture guard: `docs/src/` (see `tools/docs-architecture-no-code-guard.sh`)
  - Requirements: do not add `allow(...)` to bypass; fix the underlying issue; never introduce unwrap/expect/panic/todo/unimplemented.

### 6) Failure -> bug task conversion (mandatory)
For each distinct failure signature (even if you also fix it in this task), create a bug task in `.ralph/tasks/bugs/` using the `$add-bug` skill:
- include: repro command(s), expected vs actual, impacted modules, and evidence log paths
- if failure looks like stale artifacts (`*.rcgu.o` missing / archive not found), run exactly one `cargo clean`, rerun the failing gate, and keep pre/post logs

### 7) Completion + bookkeeping (only after all gates pass)
- Update this task file:
  - tick all acceptance checkboxes
  - set header tags to: `<status>done</status> <passes>true</passes>`
  - append `<execution_report>` summarizing each gate + pointing to evidence files
- Run `/bin/bash .ralph/task_switch.sh`
- Commit all changes (including `.ralph/evidence/...`) with message:
  - `task finished 06-task-full-verification-for-secure-explicit-config-refactor: <summary + evidence of make check/test/test-long/lint pass + any bugs filed>`
- `git push`
- Add any new learning/surprise to `AGENTS.md`
</implementation_plan>

NOW EXECUTE

<execution_report>
Evidence directory:
- `.ralph/evidence/story-secure-explicit-node-config/06-task-full-verification-for-secure-explicit-config-refactor/`

Gates executed (all pass):
- `make check` (log: `make-check.log`)
- `make lint` (log: `make-lint.log`)
- `make test` (log: `make-test.log`)
- `make test-long` wrapped in `timeout --preserve-status --kill-after=60s 90m ...` (log: `make-test-long.log`)

Notes:
- `make test` uses the Makefile `timeout` wrapper (default 120s). It completed successfully within the limit.
- `make test-long` emits “has been running for over 60 seconds” heartbeats; this is expected and not a hang signal by itself.
</execution_report>
