## Task: Migrate harnesses, tests, and docs to the authoritative managed-conf model <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer,02-task-make-pgtm-postgresql-conf-the-only-startup-config-entrypoint,03-task-take-full-ownership-of-replica-recovery-signal-and-auto-conf-state</blocked_by>

<description>
**Goal:** Remove the remaining raw `postgresql.conf` mutation patterns in tests and documentation, and update all proof surfaces to match the authoritative `pgtm.postgresql.conf` design.
The higher-order goal is to prevent the old mental model from surviving in harnesses, assertions, helper APIs, or docs after the runtime changes land.

**Scope:**
- Replace test helpers that append raw lines to `postgresql.conf` with helpers that exercise the authoritative managed-config path or clearly isolated direct-Postgres test setup when that is explicitly the thing being tested.
- Update real-binary assertions to check `SHOW config_file;`, `SHOW hba_file;`, and `SHOW ident_file;` against pgtuskmaster-managed paths.
- Update docs so they describe one authoritative pgtuskmaster-owned config file plus managed HBA/ident/TLS artifact paths.
- Make TLS documentation explicit:
- production cert/key/CA material is user-supplied via pgtuskmaster config;
- pgtuskmaster materializes managed copies under `PGDATA`;
- test harnesses may generate throwaway certs for tests only.

**Context from research:**
- `src/test_harness/pg16.rs` still exposes raw `append_postgresql_conf_lines(...)`.
- `src/logging/postgres_ingest.rs` test code still appends directly to `postgresql.conf`.
- Existing HA e2e checks already verify `SHOW hba_file;` and `SHOW ident_file;`, and should be extended to the authoritative config-file assertion.
- Existing docs still reflect the older startup-flag and backup-era config model in several places.

**Expected outcome:**
- Tests and docs reinforce the same ownership boundary the runtime enforces.
- There is no remaining ambiguous “sometimes append raw postgresql.conf, sometimes use pgtuskmaster ownership” story in active repo surfaces.

</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Raw `postgresql.conf` mutation helpers are removed or narrowly isolated only where a test explicitly targets vanilla Postgres behavior rather than pgtuskmaster ownership
- [ ] Real-binary tests assert `SHOW config_file;` points at `PGDATA/pgtm.postgresql.conf`
- [ ] Docs describe the authoritative `pgtm.postgresql.conf` model, managed HBA/ident files, managed TLS file materialization, and the user-supplied-vs-test-generated TLS distinction
- [ ] No active docs or helper comments suggest that generic `-c key=value` startup injection remains the normal configuration path
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [ ] `src/test_harness/pg16.rs`
  - [ ] Remove or narrow `append_postgresql_conf_lines(...)`.
  - [ ] Replace generic raw-conf append flows with managed-config-oriented helpers where the test is about pgtuskmaster-managed startup.
- [ ] `src/logging/postgres_ingest.rs`
  - [ ] Remove or isolate raw `postgresql.conf` append logic in tests.
  - [ ] Keep direct-Postgres setup only where the test explicitly targets ingest behavior independent of pgtuskmaster startup ownership.
- [ ] `src/test_harness/ha_e2e/startup.rs`
  - [ ] Add or update `SHOW config_file;` assertions.
  - [ ] Keep `SHOW hba_file;` and `SHOW ident_file;` assertions aligned with managed-file paths.
- [ ] `docs/src/operator/configuration.md`
  - [ ] Describe `pgtm.postgresql.conf` as the authoritative managed config file.
  - [ ] Remove legacy wording that normal startup depends on generic `-c` GUC injection.
- [ ] `docs/src/operator/troubleshooting.md`
  - [ ] Update operational checks to include `SHOW config_file;` and managed-file expectations.
- [ ] `docs/src/contributors/ha-pipeline.md`
  - [ ] Update contributor-facing startup explanations to describe managed-config materialization and authoritative config ownership.
- [ ] Any additional doc pages discovered by repo search for `config_file`, `postgresql.conf`, `pg_hba.conf`, `pg_ident.conf`, `ssl_cert_file`, `ssl_key_file`, `postgresql.auto.conf`, `pg_basebackup -R`, or generic `-c key=value`
  - [ ] Update or delete stale wording so no active doc contradicts the new model.
