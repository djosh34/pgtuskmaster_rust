---
## Task: Enforce real-binary test execution (PG16 + etcd3) via explicit gate + CI prerequisites <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Ensure “real-system” tests actually exercise real PostgreSQL 16 and etcd3 binaries in at least one deterministic gate (CI and/or developer opt-in), instead of silently reporting a pass via early-return skips.

**Scope:**
- Add an explicit enforcement mode (env var and/or `make` target) that:
  - fails fast when required binaries are missing, and
  - runs the real-binary tests when binaries are present.
- Centralize the “binary presence” and “required vs optional” policy in one place to avoid copy/paste checks across tests.
- Document prerequisites for running real-binary tests locally and in CI.

**Context from research:**
- This repository currently writes “real” tests to early-return when binaries are missing, which means `make test` can pass without executing PG16/etcd3.
- In the current environment for task `05b` (2026-03-02), `/usr/lib/postgresql/16/bin/*` and `/usr/bin/etcd` are not present, so real-binary paths are not exercised.
- A port-reservation lifetime bug was fixed during `05b` so that these tests can now bind ports correctly *when* binaries are installed.

**Expected outcome:**
- There is a deterministic gate that guarantees the real-binary tests either:
  - run against real binaries and pass, or
  - fail with a clear “missing prerequisites” error.
- Default developer flow remains convenient (tests may still skip unless the enforcement mode is enabled).

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Add a single, shared helper for “require or skip when missing binaries” (suggested location: `src/test_harness/mod.rs` or a new `src/test_harness/requirements.rs`).
- [x] Update all real-binary tests to use that helper rather than bespoke `Path::exists()` early-returns:
- [x] `src/test_harness/pg16.rs` (spawn test)
- [x] `src/test_harness/etcd3.rs` (spawn test)
- [x] `src/pginfo/worker.rs` (primary + replica integration tests)
- [x] `src/process/worker.rs` (real job tests)
- [x] Add an explicit enforcement switch (env var) with clear naming and docs (example: `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`).
- [x] Add a `Makefile` target that enables enforcement (example: `make test`), and document when to use it.
- [x] Document local prerequisites (package names / expected install paths) in `RUST_SYSTEM_HARNESS_PLAN.md` or a dedicated doc.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Implementation Plan (Phase 1 Draft)

### 1) Baseline and touchpoint audit
- Reconfirm all current real-binary entry points and policy call sites:
- `src/test_harness/binaries.rs` (current fail-fast helpers),
- `src/test_harness/pg16.rs` and `src/test_harness/etcd3.rs` tests,
- `src/pginfo/worker.rs` real PG tests,
- `src/process/worker.rs` real process-job tests,
- `src/ha/e2e_multi_node.rs` real multi-node path resolvers (currently bypass shared helper),
- plus signature-fallout callers (`src/dcs/etcd_store.rs`) to avoid compile regressions if helper API evolves.

### 2) Introduce one shared “require or skip unless enforced” policy helper
- Implement centralized policy in `src/test_harness/binaries.rs` (avoid extra module churn unless needed) that owns:
- env switch parsing for `PGTUSKMASTER_REQUIRE_REAL_BINARIES`,
- deterministic decision: missing binary => `Err(...)` when enforcement is on, otherwise `Ok(None)` for test skip path,
- consistent skip/fail message text (include missing path and env var name),
- wrappers for PG16 single binaries, PG16 process binary bundle, and etcd binary resolution.

### 3) Convert real-binary tests to use shared policy
- Refactor the listed real-binary tests to consume the shared helper and remove bespoke presence checks:
- `src/test_harness/pg16.rs` test: gate `postgres` + `initdb` through helper and short-circuit with `Ok(())` only via helper-driven optional mode.
- `src/test_harness/etcd3.rs` test: same pattern for etcd binary.
- `src/pginfo/worker.rs` tests: gate `postgres`/`initdb`/`pg_basebackup` with helper; ensure both primary and replica tests use identical policy.
- `src/process/worker.rs` real-job tests: make `pg16_binaries()` use the shared helper policy and allow skip-only-through-helper for all real job tests.
- `src/ha/e2e_multi_node.rs`: replace local `.tools/*` path checks with shared helper wrappers so e2e honors the same enforce/optional behavior.
- Keep all existing success/failure assertions unchanged once binaries are available.

### 4) Make deterministic enforcement gate explicit
- Add `make test` target in `Makefile` that sets `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1` and runs the real-binary suites explicitly (harness spawn tests, pginfo real tests, process real tests, and HA multi-node e2e), so the gate is deterministic and auditable.
- Ensure target naming/docs explicitly communicate fail-fast behavior when binaries are missing.
- Keep default `make test` behavior convenient (skip allowed only through shared helper).

### 5) Document prerequisites and usage
- Update `RUST_SYSTEM_HARNESS_PLAN.md` with:
- required local binaries (PG16 toolchain + etcd),
- expected repository paths (`.tools/postgres16/bin/*`, `.tools/etcd/bin/etcd`) and system-package mapping,
- commands for optional/default flow (`make test`) vs strict enforced flow (`make test`).

### 6) Verification and evidence capture
- Run gates in sequence and retain logs under `.ralph/evidence/10a-enforce-real-binaries/`:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- additionally run `make test` to prove enforced mode behavior.
- For acceptance grep requirements, capture explicit grep artifacts for pass/fail phrases (`congratulations` / `evaluation failed`) even if absent in native output.

### 7) Task file + finish protocol (post-implementation)
- Tick acceptance boxes with concrete evidence references.
- Set `<passes>true</passes>` only after all required gates pass.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all touched files (including `.ralph` updates) using:
- `task finished 10a-task-enforce-real-binary-tests-and-ci-prereqs: <summary + evidence + challenges>`.
- Append cross-task learnings to `AGENTS.md` if new, durable lessons emerge.

NOW EXECUTE
