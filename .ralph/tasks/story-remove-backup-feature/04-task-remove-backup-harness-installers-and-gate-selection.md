---
## Task: Remove backup-specific harness, installer, and gate-selection surfaces while preserving real tests for replica cloning <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the backup feature's harness and packaging residue so real-binary verification no longer provisions or expects pgBackRest, while preserving real coverage for normal Postgres and replica-clone behavior.

**Scope:**
- Remove pgBackRest requirements from the harness and provenance policy.
- Remove backup-specific HA test harness config and restore repository setup helpers.
- Remove the restore ultra-long test selection and any dedicated WAL passthrough integration tests.

**Context from research:**
- The real-binary/provenance additions for pgBackRest came with commit `023be6f`.
- The restore ultra-long scenario and external pgBackRest repository prep live in `src/ha/e2e_multi_node.rs` and `Makefile`.
- The harness currently models `backup` in `src/test_harness/ha_e2e/config.rs`, `src/test_harness/ha_e2e/startup.rs`, and `src/test_harness/ha_e2e/mod.rs`.
- `src/test_harness/binaries.rs` currently requires pgBackRest for the shared process binary bundle even though the only remaining required bootstrap path should be `pg_basebackup`.

**Expected outcome:**
- No harness code or policy file expects a pgBackRest binary.
- No test or gate selection still references backup/restore/wal passthrough flows.
- Real-binary verification still covers Postgres, etcd, and replica clone flows.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove `tests/wal_passthrough.rs`.
- [ ] Remove restore/backup-specific helper code and scenarios from `src/ha/e2e_multi_node.rs`, including external repo preparation helpers and the restore takeover test.
- [ ] Remove backup-specific config/harness types from `src/test_harness/ha_e2e/config.rs`.
- [ ] Remove backup payload generation and runtime wiring from `src/test_harness/ha_e2e/startup.rs`.
- [ ] Remove any now-dead backup fields or helpers from `src/test_harness/ha_e2e/mod.rs` and related harness modules.
- [ ] Update `src/test_harness/binaries.rs` so shared real process binaries no longer require pgBackRest.
- [ ] Update `src/test_harness/provenance.rs` if it has any pgBackRest-specific version or remediation logic.
- [ ] Delete `tools/install-pgbackrest.sh`.
- [ ] Remove the pgBackRest entry from `tools/real-binaries-policy.json`.
- [ ] Remove the restore ultra-long test from `Makefile` and clean up any restore-only skip-list implications.
- [ ] Remove any example/debug/test fixture references that still serialize backup blocks or pgBackRest paths.
- [ ] Confirm by search that `.tools/pgbackrest`, `install-pgbackrest.sh`, and `pgbackrest` no longer appear in harness, tools, tests, or Makefile.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
