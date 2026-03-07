## Task: Migrate harnesses, tests, and docs to the authoritative managed-conf model <status>done</status> <passes>true</passes> <priority>high</priority>

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
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Raw `postgresql.conf` mutation helpers are removed or narrowly isolated only where a test explicitly targets vanilla Postgres behavior rather than pgtuskmaster ownership
- [x] Real-binary tests assert `SHOW config_file;` points at `PGDATA/pgtm.postgresql.conf`
- [x] Docs describe the authoritative `pgtm.postgresql.conf` model, managed HBA/ident files, managed TLS file materialization, and the user-supplied-vs-test-generated TLS distinction
- [x] No active docs or helper comments suggest that generic `-c key=value` startup injection remains the normal configuration path
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [x] `src/test_harness/pg16.rs`
  - [x] Remove or narrow `append_postgresql_conf_lines(...)`.
  - [x] Replace generic raw-conf append flows with managed-config-oriented helpers where the test is about pgtuskmaster-managed startup.
- [x] `src/logging/postgres_ingest.rs`
  - [x] Remove or isolate raw `postgresql.conf` append logic in tests.
  - [x] Keep direct-Postgres setup only where the test explicitly targets ingest behavior independent of pgtuskmaster startup ownership.
- [x] `src/pginfo/worker.rs`
  - [x] Review replica/standby real-binary tests that still append to `postgresql.conf` after `pg_basebackup`.
  - [x] Keep any remaining raw config writes only as an explicitly named vanilla-Postgres replication exception, or convert them to the narrowed helper if that is clearer.
- [x] `src/test_harness/ha_e2e/startup.rs`
  - [x] Verify the existing `SHOW config_file;`, `SHOW hba_file;`, and `SHOW ident_file;` assertions still match managed-file paths after the harness changes.
  - [x] Edit only if failures or assertion messaging show drift.
- [x] `docs/src/operator/configuration.md`
  - [x] Describe `pgtm.postgresql.conf` as the authoritative managed config file.
  - [x] Remove legacy wording that normal startup depends on generic `-c` GUC injection.
- [x] `docs/src/operator/troubleshooting.md`
  - [x] Update operational checks to include `SHOW config_file;` and managed-file expectations.
- [x] `docs/src/contributors/ha-pipeline.md`
  - [x] Update contributor-facing startup explanations to describe managed-config materialization and authoritative config ownership.
- [x] Any additional doc pages discovered by repo search for `config_file`, `postgresql.conf`, `pg_hba.conf`, `pg_ident.conf`, `ssl_cert_file`, `ssl_key_file`, `postgresql.auto.conf`, `pg_basebackup -R`, or generic `-c key=value`
  - [x] Update or delete stale wording so no active doc contradicts the new model.

## Plan (authoritative managed-conf migration for harnesses, tests, and docs)

### Research summary and current repo state

- `src/test_harness/pg16.rs` still exposes a generic raw-config startup path:
  - `spawn_pg16(spec)` delegates to `spawn_pg16_with_conf_lines(spec, &[])`
  - `spawn_pg16_with_conf_lines(...)` appends arbitrary lines into `PGDATA/postgresql.conf`
  - `append_postgresql_conf_lines(...)` is the concrete raw mutation helper that keeps the old mental model alive
- Repo search only found one active caller of the raw helper API outside the harness itself:
  - `src/logging/postgres_ingest.rs` real-binary test `ingests_jsonlog_and_stderr_files_from_real_postgres()`
- Repo search also found another active raw `postgresql.conf` mutation in a real-binary test that the first-pass plan missed:
  - `src/pginfo/worker.rs` appends `primary_conninfo` to a cloned replica data dir after `pg_basebackup`
  - that scenario is testing vanilla PostgreSQL standby polling behavior rather than pgtuskmaster-managed startup, so it can remain only if execution makes the exception explicit instead of leaving it as an accidental generic pattern
- One acceptance item is already satisfied in current code:
  - `src/test_harness/ha_e2e/startup.rs` already asserts `SHOW hba_file;`, `SHOW ident_file;`, and `SHOW config_file;` against `PGDATA/pgtm.pg_hba.conf`, `PGDATA/pgtm.pg_ident.conf`, and `PGDATA/pgtm.postgresql.conf`
- Several managed-config docs are already aligned:
  - `docs/src/operator/configuration.md`
  - `docs/src/contributors/ha-pipeline.md`
  - `docs/src/lifecycle/bootstrap.md`
  - `docs/src/lifecycle/recovery.md`
- The main doc gap found during this planning pass is `docs/src/operator/troubleshooting.md`, which still gives generic symptom checks but does not tell operators to verify `SHOW config_file;`, `SHOW hba_file;`, or `SHOW ident_file;` against the managed paths.
- `docs/src/contributors/harness-internals.md` still describes the Postgres harness as a generic raw process launcher and should be updated so contributors do not reintroduce raw `postgresql.conf` mutation as the normal harness pattern.
- `docs/book/` contains generated artifacts. Do not hand-edit generated HTML/JSON; update source docs under `docs/src/` and regenerate tracked outputs only through the repo’s normal docs/build flow used by the verification gates.

### Constraints and execution choices

- This is a greenfield repo with no backward-compatibility requirement. The execution pass should remove or rename the generic raw-conf API instead of preserving it as a convenience wrapper.
- The harness must distinguish two cases explicitly:
  - pgtuskmaster-managed startup tests should use managed-config-oriented helpers only
  - vanilla-Postgres tests may still need direct Postgres setup, but that path must be obviously named and narrowly scoped so it is not confused with the authoritative pgtuskmaster startup model
- `src/logging/postgres_ingest.rs` is the only currently observed direct caller, and its scenario is about PostgreSQL log ingestion rather than pgtuskmaster-managed startup. That test can keep direct Postgres setup, but the API must make that intent explicit.
- `src/pginfo/worker.rs` contains a second direct-Postgres replication setup path after `pg_basebackup`; the execution pass must account for it when removing or renaming the generic helper so the repo still has a coherent story for explicit vanilla-Postgres exceptions.
- The later `NOW EXECUTE` pass should not spend time rediscovering whether `SHOW config_file` assertions or several doc updates are missing; those are already present and only need verification that they still satisfy acceptance after the remaining edits.
- The required user gates do not build the mdBook output. Since docs are in scope, the execution pass should run `make docs-build` and `make docs-hygiene` in addition to the required gates so broken or stale doc source changes do not slip through.

### Detailed execution plan

1. Refactor the low-level Postgres harness API in `src/test_harness/pg16.rs`.
- Remove the generic names `spawn_pg16_with_conf_lines(...)` and `append_postgresql_conf_lines(...)`.
- Keep `spawn_pg16(spec)` as the default path for plain unmanaged Postgres startup with no extra config mutation.
- Introduce a replacement API only if the ingest test still needs it after review, and give it a deliberately narrow name that makes the ownership boundary explicit, for example a helper framed as vanilla Postgres test setup rather than generic harness configuration.
- If the replacement helper remains, keep its contract minimal:
  - it may write direct Postgres config only for tests that explicitly target raw Postgres behavior
  - it must not be described as the normal startup path for pgtuskmaster-managed tests
  - its implementation must keep full error propagation and must not swallow filesystem write failures
- Update module-local tests in `src/test_harness/pg16.rs` if needed so the harness API surface reflects the new ownership boundary.

2. Rewrite the known direct-Postgres exception tests against the renamed/narrowed harness path.
- `src/logging/postgres_ingest.rs`
  - replace the current `spawn_pg16_with_conf_lines(spec, &conf_lines)` call with the narrowed helper from step 1, or an even more explicit local setup flow if that is clearer
- Preserve the test’s real purpose:
  - enable `logging_collector`
  - emit `jsonlog` and `stderr`
  - point `log_directory` and `log_filename` at the namespace log dir
  - verify ingestion sees both JSON and stderr records
- Add a short local comment if needed to explain why this is one of the allowed direct-Postgres exceptions: the test is validating ingest behavior independent of pgtuskmaster-managed startup ownership.
- `src/pginfo/worker.rs`
  - review whether the standby-polling test should use the same narrowed helper or stay as a fully local direct `postgresql.conf` edit after `pg_basebackup`
  - if it stays local, make the exception explicit in naming/commentary so the repo no longer implies raw config mutation is generic harness policy
- Confirm there are no other crate callers of the removed generic raw-conf helper after the refactor.

3. Re-verify the HA startup proof surface in `src/test_harness/ha_e2e/startup.rs`.
- During execution, treat this as verification-first, not expected edit scope.
- Confirm the existing `SHOW config_file;`, `SHOW hba_file;`, and `SHOW ident_file;` assertions still cover the acceptance requirement after any supporting helper changes.
- If the assertion block needs only small wording cleanup or stronger failure messages, keep that change minimal.

4. Update operator docs in `docs/src/operator/troubleshooting.md`.
- Add a concrete startup/config ownership troubleshooting probe that tells operators to run:
  - `SHOW config_file;`
  - `SHOW hba_file;`
  - `SHOW ident_file;`
- State the expected managed paths explicitly:
  - `PGDATA/pgtm.postgresql.conf`
  - `PGDATA/pgtm.pg_hba.conf`
  - `PGDATA/pgtm.pg_ident.conf`
- Tie the troubleshooting guidance back to the current ownership contract:
  - generic `postgresql.conf` mutation is not the normal pgtuskmaster path
  - managed file mismatches indicate startup/materialization drift or out-of-band operator interference
- Keep the operator guidance symptom-first rather than turning this page into a contributor design doc.

5. Update contributor docs so they reinforce the same boundary.
- `docs/src/contributors/harness-internals.md`
  - add that the pg16 harness is for real-binary process control, but pgtuskmaster-managed tests should prefer managed-config-oriented runtime/harness flows instead of appending to `postgresql.conf`
  - if a direct Postgres setup helper remains, describe it as an explicit exception for tests that truly target vanilla Postgres behavior
- `docs/src/contributors/ha-pipeline.md`
  - re-check that the startup/process-dispatch wording still matches the final code after the harness API cleanup
  - only edit if execution reveals wording drift; the planning pass found the core managed-config language already present
- `docs/src/operator/configuration.md`
  - re-check TLS wording during execution and strengthen it only if needed so the distinction stays explicit:
    - production certificate, key, and CA inputs are operator supplied through pgtuskmaster config
    - pgtuskmaster materializes managed copies under `PGDATA`
    - tests may generate throwaway certs only for harness scenarios

6. Run one final repo search for stale wording and stray raw-config call sites after the code/doc edits land.
- Search at minimum for:
  - `spawn_pg16_with_conf_lines`
  - `append_postgresql_conf_lines`
  - `postgresql.conf`
  - `config_file`
  - `pg_hba.conf`
  - `pg_ident.conf`
  - `ssl_cert_file`
  - `ssl_key_file`
  - `pg_basebackup -R`
  - generic `-c key=value` startup phrasing in active docs
- Include code surfaces in the review, not just docs, so any leftover generic raw-config helper use is caught before gate runs.
- Treat generated `docs/book/` files as build outputs: refresh them through the normal docs generation path if the repo tracks them, but do not use them as the source of truth.
- Remove or update any newly discovered active doc wording that still suggests generic raw `postgresql.conf` mutation or generic `-c` injection is the normal configuration path.

7. Verify docs explicitly before the required user gates.
- Run `make docs-build`.
- Run `make docs-hygiene`.

8. Execute the required verification gates only after the implementation and doc updates are complete.
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Fix every failure rather than weakening coverage, skipping tests, or leaving stale generated docs behind.

9. Finish the Ralph completion sequence only after all gates pass.
- Set `<passes>true</passes>` in this task file.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes, including `.ralph` artifacts, with message:
  - `task finished 04-task-migrate-harness-tests-and-docs-to-the-authoritative-managed-conf-model: ...`
- Include evidence for the verification gates and any non-trivial implementation challenge in the commit message body.
- Push with `git push`.
- Stop immediately after the push.

NOW EXECUTE
