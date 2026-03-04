---
## Task: Build exhaustive fixture-driven pgBackRest test matrix across normal and edge restores <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Create a comprehensive real-binary test matrix that generates diverse backup fixtures and validates backup + restore + takeover behavior end-to-end with deterministic timing and strong diagnostics.

**Scope:**
- Add fixture generation tooling (Rust harness and/or scripts) that creates real pgBackRest backups for many cluster states.
- Add restore validation scenarios for both pgtuskmaster-origin backups and external/non-pgtuskmaster-origin backups.
- Validate config takeover semantics at recovery start (before postgres can accept traffic).
- Validate cluster-wide convergence after forced restore takeover endpoint execution.
- Keep each individual test runtime bounded and deterministic, while achieving broad total matrix coverage.

**Context from research:**
- Hot standby startup can fail when restored config carries incompatible values (e.g. `max_connections`/related settings lower than primary expectations): https://www.postgresql.org/docs/16/hot-standby.html
- WAL archiving/recovery flow requires correct restore/archive command behavior and clear recovery signal/config handling: https://www.postgresql.org/docs/16/continuous-archiving.html
- pgBackRest restore emits structured logs and supports recovery option overrides useful for scenario construction: https://pgbackrest.org/command.html
- Existing harness already supports real PostgreSQL/etcd e2e and should be extended rather than bypassed (`src/test_harness/*`, `src/ha/e2e_*`).

**Expected outcome:**
- Test matrix catches backup/restore regressions early, including subtle config and WAL issues that only appear in real binaries.
- Restore takeover endpoint is validated under realistic failure and success conditions.
- Failures produce actionable diagnostics (captured postgres + pgBackRest + command logs) without requiring manual reproduction.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add fixture-generation harness modules/scripts (expected under `src/test_harness/` and/or `scripts/`) with:
- [ ] deterministic fixture naming and reproducible input data
- [ ] reusable helper for creating base backups + WAL activity + metadata manifests
- [ ] no hidden state outside explicit fixture directories
- [ ] Add pgBackRest-capable harness helpers, e.g. `src/test_harness/pgbackrest.rs`:
- [ ] stanza/repo setup helpers
- [ ] backup creation helpers for full/incremental/differential modes
- [ ] restore execution helpers with managed-config takeover hooks
- [ ] structured log collection helpers for pgBackRest and postgres output
- [ ] Add exhaustive scenario matrix tests including at minimum:
- [ ] normal full backup + restore success
- [ ] differential/incremental restore success
- [ ] backup containing wrong/incompatible postgres config (`max_connections`/WAL-related) recovered successfully via managed config takeover
- [ ] backup from non-pgtuskmaster source cluster restored and adopted into pgtuskmaster-managed cluster
- [ ] missing/corrupt WAL segment restore failure with explicit diagnostics
- [ ] restore endpoint takeover success with follower convergence via rewind/basebackup
- [ ] concurrent restore request rejection/single-flight guarantee
- [ ] restore failure rollback/safe-state behavior and operator-visible error reason
- [ ] Add timing and flake controls for each real scenario:
- [ ] per-operation timeout and full-scenario timeout
- [ ] deterministic teardown guaranteeing no leftover postgres/etcd/pgBackRest processes
- [ ] bounded polling loops with actionable timeout messages
- [ ] Add matrix partitioning so each test is individually short while total suite coverage is broad:
- [ ] split scenarios into targeted tests rather than monolithic mega-test
- [ ] classify only genuinely long scenarios into `make test-long` with exact-match naming updates
- [ ] Add docs for fixture matrix and how to regenerate fixtures locally in `docs/src/operator/` or `docs/src/development/`
- [ ] No scenario is skipped or made optional due to missing real binaries; required binaries must be installed via tooling and validated by harness
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
