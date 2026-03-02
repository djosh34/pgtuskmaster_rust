---
## Task: Enforce real-binary test execution (PG16 + etcd3) via explicit gate + CI prerequisites <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Ensure “real-system” tests actually exercise real PostgreSQL 16 and etcd3 binaries in at least one deterministic gate (CI and/or developer opt-in), instead of silently passing via early-return skips.

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
- [ ] Add a single, shared helper for “require or skip when missing binaries” (suggested location: `src/test_harness/mod.rs` or a new `src/test_harness/requirements.rs`).
- [ ] Update all real-binary tests to use that helper rather than bespoke `Path::exists()` early-returns:
- [ ] `src/test_harness/pg16.rs` (spawn test)
- [ ] `src/test_harness/etcd3.rs` (spawn test)
- [ ] `src/pginfo/worker.rs` (primary + replica integration tests)
- [ ] `src/process/worker.rs` (real job tests)
- [ ] Add an explicit enforcement switch (env var) with clear naming and docs (example: `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`).
- [ ] Add a `Makefile` target that enables enforcement (example: `make test-real`), and document when to use it.
- [ ] Document local prerequisites (package names / expected install paths) in `RUST_SYSTEM_HARNESS_PLAN.md` or a dedicated doc.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>

