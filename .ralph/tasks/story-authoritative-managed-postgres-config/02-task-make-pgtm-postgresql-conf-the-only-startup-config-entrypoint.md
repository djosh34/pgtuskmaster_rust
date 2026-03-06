## Task: Make `pgtm.postgresql.conf` the only startup config entrypoint and remove generic `-c` GUC injection <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer</blocked_by>

<description>
**Goal:** Make Postgres always start through `config_file=<abs path to PGDATA/pgtm.postgresql.conf>` and remove the generic startup path that injects arbitrary repeated `-c key=value` settings through `pg_ctl -o`.
This task establishes the actual runtime ownership boundary promised by the new managed-config model. The higher-order goal is to make `SHOW config_file;` point to one authoritative pgtuskmaster-owned file on every managed start path.

**Scope:**
- Change normal startup, resume, and HA-driven starts so `pgtm.postgresql.conf` is always the authoritative config file.
- Remove generic `StartPostgresSpec.extra_postgres_settings`.
- Remove generic `host`, `port`, and `socket_dir` startup flag injection as a configuration mechanism; these must come from the typed managed config and its serializer.
- Narrow the process-layer startup contract so it carries only what the process runner actually needs to start Postgres under pgtuskmaster ownership.
- Preserve `pg_ctl -l <log_file>` and other non-config operational arguments where still needed, but do not use `pg_ctl -o` as a generic config transport anymore.

**Context from research:**
- `src/process/worker.rs` currently builds `-o "-h ... -p ... -k ... -c hba_file=... -c ident_file=..."`.
- `src/runtime/node.rs` and `src/ha/process_dispatch.rs` both pass the managed settings bag into `StartPostgresSpec`.
- Normal startup still relies on default `postgresql.conf` plus startup flags, while `pgtm.postgresql.conf` only appears in the backup-era path today.
- That means pgtuskmaster does not yet have one clear active config authority file in steady-state startup.

**Expected outcome:**
- Every pgtuskmaster-managed Postgres start path uses `config_file=<abs path to pgtm.postgresql.conf>`.
- The process layer no longer transports a generic settings map for Postgres configuration.
- Postgres listen host, port, socket dir, HBA file, ident file, and TLS file paths come from the managed config file, not startup flag injection.

</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `StartPostgresSpec` no longer contains a generic config bag for arbitrary Postgres GUC injection
- [ ] Runtime startup and HA start paths always materialize and use `config_file=<abs path to pgtm.postgresql.conf>`
- [ ] Generic repeated `-c key=value` injection is removed as the main configuration path for Postgres startup
- [ ] `SHOW config_file;` real-binary tests prove the active config file is `PGDATA/pgtm.postgresql.conf`
- [ ] No start path still depends on the default `postgresql.conf` as the active parent config
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [ ] `src/process/jobs.rs`
  - [ ] Replace `StartPostgresSpec.extra_postgres_settings`.
  - [ ] Remove any config-transport fields that only existed to support startup flag injection.
  - [ ] Keep only the minimal operational fields needed once config ownership moves to `pgtm.postgresql.conf`.
- [ ] `src/process/worker.rs`
  - [ ] Remove the loop that renders repeated `-c key=value` from a generic settings map.
  - [ ] Replace it with the narrow startup contract that points Postgres at the authoritative managed config file.
  - [ ] Update command-rendering tests accordingly.
- [ ] `src/runtime/node.rs`
  - [ ] Make every managed start path materialize the authoritative managed config and pass only the new startup contract.
  - [ ] Remove reliance on default `postgresql.conf` as an active configuration parent.
- [ ] `src/ha/process_dispatch.rs`
  - [ ] Make HA-driven `StartPostgres` requests use the same authoritative managed config path as runtime startup.
  - [ ] Remove any remaining per-start config-bag plumbing.
- [ ] `src/postgres_managed.rs`
  - [ ] Ensure the authoritative managed config path is always available for managed starts.
  - [ ] Do not leave backup-era conditional ownership around `config_file`.
- [ ] `src/test_harness/ha_e2e/startup.rs`
  - [ ] Add or update real-binary assertions for `SHOW config_file;`.
  - [ ] Keep existing `SHOW hba_file;` and `SHOW ident_file;` assertions aligned with the new ownership model.
- [ ] `src/process/worker.rs` tests
  - [ ] Remove tests that assert generic `-c hba_file=...` / `-c ident_file=...` rendering.
  - [ ] Add tests that assert the narrowed startup contract and the exact `config_file` entrypoint.
