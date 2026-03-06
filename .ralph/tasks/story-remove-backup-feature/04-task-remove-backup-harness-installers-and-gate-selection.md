## Task: Remove backup-specific harness, installer, and gate-selection surfaces while preserving real tests for replica cloning <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's harness and packaging residue so real-binary verification no longer provisions or expects pgBackRest, while preserving real coverage for normal Postgres and replica-clone behavior.
This cleanup is part of the same immediate removal story and should follow the code-path deletion without being deferred to a later general cleanup pass.

**Scope:**
- Remove pgBackRest requirements from the harness and provenance policy.
- Remove backup-specific HA test harness config and restore repository setup helpers.
- Remove the restore ultra-long test selection and any dedicated WAL passthrough integration tests.
- Remove all remaining backup-shaped fixture/config literals in examples and tests after the root config/process surface is deleted.

**Context from research:**
- The real-binary/provenance additions for pgBackRest came with commit `023be6f`.
- The restore ultra-long scenario and external pgBackRest repository prep live in `src/ha/e2e_multi_node.rs` and `Makefile`.
- The harness currently models `backup` in `src/test_harness/ha_e2e/config.rs`, `src/test_harness/ha_e2e/startup.rs`, and `src/test_harness/ha_e2e/mod.rs`.
- `src/test_harness/binaries.rs` currently requires pgBackRest for the shared process binary bundle even though the only remaining required bootstrap path should be `pg_basebackup`.
- Additional known test/example fallout sites carrying backup-shaped literals or assumptions include:
  - `examples/debug_ui_smoke_server.rs`
  - `tests/bdd_api_http.rs`
  - `src/logging/{mod,postgres_ingest}.rs`
  - `src/worker_contract_tests.rs`

**Expected outcome:**
- No harness code or policy file expects a pgBackRest binary.
- No test or gate selection still references backup/restore/wal passthrough flows.
- Real-binary verification still covers Postgres, etcd, and replica clone flows.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Remove `tests/wal_passthrough.rs`.
- [x] Remove restore/backup-specific helper code and scenarios from `src/ha/e2e_multi_node.rs`, including external repo preparation helpers and the restore takeover test.
- [x] Remove backup-specific config/harness types from `src/test_harness/ha_e2e/config.rs`.
- [x] Remove backup payload generation and runtime wiring from `src/test_harness/ha_e2e/startup.rs`.
- [x] Remove any now-dead backup fields or helpers from `src/test_harness/ha_e2e/mod.rs` and related harness modules.
- [x] Update `src/test_harness/binaries.rs` so shared real process binaries no longer require pgBackRest.
- [x] Update `src/test_harness/provenance.rs` if it has any pgBackRest-specific version or remediation logic.
- [x] Delete `tools/install-pgbackrest.sh`.
- [x] Remove the pgBackRest entry from `tools/real-binaries-policy.json`.
- [x] Remove the restore ultra-long test from `Makefile` and clean up any restore-only skip-list implications.
- [x] Remove or rewrite remaining known example/debug/test fallout that still serializes backup blocks or pgBackRest paths:
  - `examples/debug_ui_smoke_server.rs`
  - `tests/bdd_api_http.rs`
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/worker_contract_tests.rs`
- [x] Search tests/examples/harness code for any remaining backup-shaped fixtures discovered after compilation and remove them in this task.
- [x] Confirm by search that `.tools/pgbackrest`, `install-pgbackrest.sh`, and `pgbackrest` no longer appear in harness, tools, tests, or Makefile.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>

## Detailed Execution Plan (Draft 1, 2026-03-06)

### 1. Lock the execution boundary against current HEAD before making changes

- Treat the task description as historically useful but partially stale.
- Current research already established the following drift that execution must respect:
  - `src/ha/e2e_multi_node.rs` does not exist at current HEAD; the live multi-node HA scenario support now sits in `tests/ha/support/multi_node.rs`, with entry tests in `tests/ha_multi_node_*.rs`.
  - `tests/wal_passthrough.rs` is already absent from the tree.
  - `tools/install-pgbackrest.sh` is already absent from the tree.
  - `tools/real-binaries-policy.json` no longer carries a `pgbackrest` entry.
  - broad repository search over `src tests examples tools Makefile` found no live `pgbackrest`, `install-pgbackrest`, or `wal_passthrough` references.
- Because of that drift, the execution pass must treat several acceptance items as verification work rather than new deletion work. Do not attempt to recreate stale files or hunt for paths that no longer exist.

### 2. Product intent to preserve while executing deletions and verifications

- Keep replica clone coverage via `pg_basebackup`; this task is not a removal of replica bootstrap.
- Keep the HA real-binary harness working for:
  - PostgreSQL
  - etcd
  - replica clone flows
- Keep the current `src/test_harness/ha_e2e` startup path intact except for removing any residual backup-shaped config or wiring if any is still discovered.
- Do not remove ordinary `pg_basebackup` references from runtime configs, tests, logging fixtures, or contract tests. Those are part of the surviving replica clone path, not backup-feature residue.
- Do not broaden this task into unrelated harness redesign. If an issue is discovered that is real but outside the backup-removal boundary, create a bug with the `add-bug` skill instead of silently folding it into this task.

### 3. Files and modules to inspect or patch during `NOW EXECUTE`

- Primary verification-and-edit scope:
  - `src/test_harness/ha_e2e/config.rs`
  - `src/test_harness/ha_e2e/mod.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/test_harness/binaries.rs`
  - `src/test_harness/provenance.rs`
  - `tests/ha/support/multi_node.rs`
  - `tests/ha_multi_node_failover.rs`
  - `tests/ha_multi_node_failsafe.rs`
  - `tests/ha_multi_node_stress.rs`
  - `.config/nextest.toml`
  - `Makefile`
  - `examples/debug_ui_smoke_server.rs`
  - `tests/bdd_api_http.rs`
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/worker_contract_tests.rs`
- Supporting verification scope:
  - `tools/real-binaries-policy.json`
  - `tools/`
  - any docs that still mention pgBackRest, restore bootstrap, or WAL passthrough behavior

### 4. Execution phase A: re-verify already-removed backup surfaces and mark them complete only with evidence

- Run targeted file-existence checks and searches first:
  - verify `tests/wal_passthrough.rs` is absent
  - verify `tools/install-pgbackrest.sh` is absent
  - verify `tools/real-binaries-policy.json` contains no `pgbackrest` label or path
  - verify `src`, `tests`, `examples`, `tools`, `Makefile`, and `.config/nextest.toml` contain no `pgbackrest`, `install-pgbackrest`, or `wal_passthrough` references
- If those checks still pass during execution, tick the corresponding acceptance items instead of editing code unnecessarily.
- If any residual reference is found, remove it in the owning file and add focused test or search evidence for the deletion.

### 5. Execution phase B: audit the HA harness for backup-shaped config and runtime wiring

- Inspect `src/test_harness/ha_e2e/config.rs` and `src/test_harness/ha_e2e/mod.rs` for any backup-specific enums, structs, fields, or re-exports.
- Inspect `src/test_harness/ha_e2e/startup.rs` for:
  - backup payload generation
  - restore repository setup helpers
  - restore takeover flow
  - archive or restore command assertions
  - backup blocks serialized into generated runtime config
- Current research suggests the live startup harness only provisions ordinary process binaries plus the backup-free managed Postgres assertions. If that holds, execution should record the verification and avoid churn.
- If stale backup-specific helpers are still present, remove them and keep the harness focused on normal startup plus replica clone prerequisites.

### 6. Execution phase C: verify real-binary policy and provenance requirements remain aligned with the surviving harness

- Confirm `src/test_harness/binaries.rs` only requires the binary set needed by the current harness:
  - `postgres`
  - `pg_ctl`
  - `pg_rewind`
  - `initdb`
  - `pg_basebackup`
  - `psql`
  - `etcd`
- Confirm `src/test_harness/provenance.rs` has no pgBackRest-specific expected-version, attestation, or remediation text.
- If provenance error messages still tell users to install a removed backup binary, rewrite them to mention only the current installers.
- Keep error handling explicit and lint-clean; no `unwrap`, `expect`, or ignored errors.

### 7. Execution phase D: verify multi-node scenarios and ultra-long gate selection still match the intended coverage

- Treat `tests/ha/support/multi_node.rs` as the real replacement for the stale `src/ha/e2e_multi_node.rs` reference in the task description.
- Search that support file and the `tests/ha_multi_node_*.rs` entrypoints for any remaining restore/backup-specific scenarios or helpers.
- Inspect `.config/nextest.toml` and `Makefile` together to ensure:
  - default profile excludes only the current ultra-long HA scenarios
  - ultra-long profile includes only the current intended HA scenarios
  - there is no restore-only scenario selection or restore-specific skip-list logic left behind
- If the current ultra-long profile already contains only the live HA stress/failover/failsafe/partition scenarios, keep it as-is and treat the acceptance item as satisfied by verification.
- If a stale test name or stale selection comment remains, remove or rename it so `make test` and `make test-long` reflect the current suite accurately.

### 8. Execution phase E: clean up only truly stale fallback fixtures in examples and non-HA tests

- Revisit the task-named fallout files:
  - `examples/debug_ui_smoke_server.rs`
  - `tests/bdd_api_http.rs`
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/worker_contract_tests.rs`
- Current research shows these files still use `pg_basebackup` in sample runtime configs, which is expected and should remain.
- During execution, search those files for actual backup-feature residue, not generic replica-clone binaries:
  - `pgbackrest`
  - `backup.` config blocks
  - `restore_command`
  - `archive_command`
  - restore-specific API routes or payloads
- Only rewrite these fixtures if a true backup-shaped literal remains. Do not delete legitimate `pg_basebackup` paths.

### 9. Parallel execution split for the later `NOW EXECUTE` pass

- Worker A ownership:
  - `src/test_harness/ha_e2e/config.rs`
  - `src/test_harness/ha_e2e/mod.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - responsibility: remove or prove absence of backup-specific harness config and runtime wiring, then add or adjust focused tests only if needed
- Worker B ownership:
  - `src/test_harness/binaries.rs`
  - `src/test_harness/provenance.rs`
  - `tools/real-binaries-policy.json`
  - `.config/nextest.toml`
  - `Makefile`
  - responsibility: align binary policy and gate selection with the surviving HA and replica-clone coverage, while verifying no backup installer or restore-only selectors remain
- Main agent ownership:
  - verify stale-path drift against `tests/ha/support/multi_node.rs`
  - inspect the named fixture/example files
  - run final searches
  - update docs if needed
  - update the task checkboxes
  - run `make check`, `make test`, `make test-long`, `make lint`

### 10. Search checklist that must be run during execution

- Run and clear these searches across tracked source inputs, not generated outputs:
  - prefer `git ls-files src tests examples tools docs .config Makefile` piped into `rg` or an equivalent tracked-file search
  - explicitly exclude generated `docs/book/` output and any other untracked build artifacts so the search does not get polluted by stale rendered content
  - `pgbackrest`
  - `install-pgbackrest`
  - `wal_passthrough`
  - `restore_command`
  - `archive_command`
  - `backup =`
  - `backup.`
  - `restore takeover`
  - `restore bootstrap`
- Interpret hits carefully:
  - task files under `.ralph/tasks/` may still mention removed behavior and do not count as live product residue for this execution pass
  - generated docs output such as `docs/book/searchindex.json` does not count; only contributor-facing source docs under `docs/src/` matter for this task
  - negative assertions that prove managed Postgres config stays free of `archive_command` or `restore_command` are expected guardrails, not backup-feature residue
  - `pg_basebackup` hits are expected and should remain unless they are mislabeled as backup-feature behavior
- If an unexpected live hit remains in code, tests, tools, Makefile, or docs, remove it or rewrite it before running the full gates.

### 11. Documentation expectations

- Search tracked docs sources for `pgbackrest`, restore bootstrap, and WAL passthrough references before deciding docs are unchanged.
- Ignore generated `docs/book/` artifacts during this review; if they change later, they must come from rebuilt docs source rather than direct edits.
- If no contributor-facing or user-facing docs mention those removed surfaces, avoid gratuitous doc churn.
- If docs do still mention them, update or delete those docs in this task rather than deferring to Task 05, because the user’s completion rule requires stale docs to be removed when encountered.

### 12. Exact execution order for the later `NOW EXECUTE` pass

1. Spawn the two workers defined above.
2. Locally verify the stale file-path drift and current multi-node scenario ownership in `tests/ha/support/multi_node.rs`.
3. Integrate Worker A results.
4. Integrate Worker B results.
5. Run the full repository searches listed above and clear any remaining live backup-feature residue.
6. Inspect and update docs only if the searches show stale contributor-facing or user-facing references.
7. Tick acceptance boxes based on actual file state, verified searches, and any edits made.
8. Run the required gates in this order:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
9. Only after all gates pass:
   - set `<passes>true</passes>`
   - run `/bin/bash .ralph/task_switch.sh`
   - commit with `task finished 04-task-remove-backup-harness-installers-and-gate-selection: ...`
   - `git push`

### 13. Known plan risks to scrutinize in the required `TO BE VERIFIED` pass

- Risk: this draft may still be too optimistic that the task is mostly verification; the skeptical pass must challenge that assumption by re-checking for backup-shaped config hidden behind generic names in the HA harness.
- Risk: the task description may still require at least one real code deletion in `tests/ha/support/multi_node.rs` or `.config/nextest.toml` even though the first survey did not show obvious residue.
- Risk: docs may still contain stale backup wording outside the code paths already searched.
- The `TO BE VERIFIED` pass must alter at least one concrete part of this plan before switching to `NOW EXECUTE`.

</plan>

NOW EXECUTE
