## Task: Introduce a typed authoritative managed Postgres config model and serializer <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>

<description>
**Goal:** Replace the current stringly startup-setting map with one authoritative typed Rust model for `PGDATA/pgtm.postgresql.conf`, plus explicit typed ownership of the config fragments that remain separate files.
This task is the foundation for making pgtuskmaster the sole determinant of effective Postgres configuration. The higher-order goal is to ensure that all Postgres GUC decisions come from pgtuskmaster runtime config and DCS-derived runtime state, not from implicit Postgres-owned files or ad hoc `-c key=value` injection.

**Scope:**
- Introduce one dedicated managed-config domain model for the authoritative `pgtm.postgresql.conf` contents.
- Keep `pgtm.pg_hba.conf` and `pgtm.pg_ident.conf` as separate managed files referenced from the authoritative managed config.
- Add one explicit config surface for opaque user-requested extra GUCs that pgtuskmaster does not semantically interpret, but still owns and serializes. Do not allow any other unmanaged config path.
- Define exact typed handling for owned settings such as `listen_addresses`, `port`, `unix_socket_directories`, `hba_file`, `ident_file`, `ssl`, `ssl_cert_file`, `ssl_key_file`, `ssl_ca_file`, `hot_standby`, `primary_conninfo`, and `primary_slot_name`.
- Make the serializer deterministic and single-owner. It must be the only code path that writes `pgtm.postgresql.conf`.
- Be explicit about TLS ownership:
- production certificate/key/CA material is user-supplied through pgtuskmaster config;
- pgtuskmaster materializes managed copies under `PGDATA` and points Postgres at those managed paths;
- only test harnesses may generate throwaway certs for tests.

**Context from research:**
- `src/postgres_managed.rs` currently materializes HBA/ident/TLS files, but its output contract is still `ManagedPostgresConfig { extra_settings: BTreeMap<String, String> }`.
- `src/process/jobs.rs` and `src/process/worker.rs` still carry and render `StartPostgresSpec.extra_postgres_settings`.
- `src/runtime/node.rs` and `src/ha/process_dispatch.rs` still treat managed Postgres config as a bag of startup flags rather than an authoritative config artifact.
- `src/pginfo/state.rs` already has typed observed config fields, but there is not yet a typed authoritative startup config model.
- The backup-removal story must complete first so this task does not preserve or migrate pgBackRest-era `archive_command` / `restore_command` behavior into the new ownership model.

**Expected outcome:**
- A single typed Rust model exists for authoritative `pgtm.postgresql.conf`.
- A single serializer emits deterministic `.conf` contents from that model.
- Opaque extra GUCs are still fully owned by pgtuskmaster via explicit config and deterministic serialization, not by startup flag injection.
- TLS material ownership is unambiguous: user-supplied in production, test-generated only in harnesses, always materialized into managed files before Postgres start.

</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Add one authoritative managed-config domain module and use it as the only serializer for `pgtm.postgresql.conf`
- [x] Replace `ManagedPostgresConfig.extra_settings: BTreeMap<String, String>` with typed managed-config output; no new generic stringly startup setting bag may remain
- [x] Add one explicit config field for pgtuskmaster-owned opaque extra GUCs and validate GUC names deterministically
- [x] Serializer output is deterministic and covered by direct unit tests for ordering, escaping, booleans, enums, and opaque extra GUC rendering
- [x] Task text and code comments make TLS ownership explicit: production inputs are user-supplied, pgtuskmaster only materializes managed copies, and test-generated certs are test-only
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [x] `src/postgres_managed.rs`
  - [x] Stop exposing a generic `extra_settings` bag as the main managed-config product.
  - [x] Introduce or call one typed authoritative managed-config builder.
  - [x] Keep managed file materialization for `pgtm.pg_hba.conf`, `pgtm.pg_ident.conf`, `pgtm.server.crt`, `pgtm.server.key`, and optional `pgtm.ca.crt`.
  - [x] Make TLS-file comments and errors explicit that production contents come from user config, not generated by pgtuskmaster.
- [x] `src/config/schema.rs`
  - [x] Add one explicit config surface for pgtuskmaster-owned opaque extra GUCs.
  - [x] Do not introduce a second parallel config surface for unmanaged startup flags.
- [x] `src/config/parser.rs`
  - [x] Validate opaque extra GUC names and values sufficiently to prevent malformed serialization or option injection.
  - [x] Keep validation errors field-specific and actionable.
- [x] `src/config/defaults.rs`
  - [x] Do not synthesize security-sensitive managed Postgres settings here.
  - [x] Only add defaults if they are safe and explicitly part of the managed-config contract.
- [x] `src/config/mod.rs`
  - [x] Re-export the new typed managed-config-facing fields cleanly if required.
- [x] `src/lib.rs`
  - [x] Export any new managed-config module needed by the rest of the crate.
- [x] New module(s), likely under `src/postgres_managed/` or a new dedicated managed-config module
  - [x] Define the typed authoritative config model.
  - [x] Define deterministic serialization to `.conf`.
  - [x] Keep this code free of backup-specific concepts.
- [x] Direct unit tests in the managed-config domain
  - [x] Assert stable rendering order.
  - [x] Assert correct quoting/escaping for string GUC values.
  - [x] Assert booleans and enums serialize correctly.
  - [x] Assert opaque extra GUCs are merged deterministically and cannot override forbidden owned keys unless that override policy is intentionally and explicitly designed.

## Detailed Execution Plan (Draft 1, 2026-03-06)

### 1. Current-head facts this execution must respect

- All listed blocker tasks are already complete with `<passes>true</passes>`, so execution can target the authoritative-config migration directly.
- Current HEAD still has three incompatible surfaces that this task must remove together:
  - `src/postgres_managed.rs` writes `PGDATA/pgtm.postgresql.conf`, but only as a header-only artifact plus TLS/HBA/ident side files.
  - `src/process/jobs.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, and `src/ha/process_dispatch.rs` still start PostgreSQL through `pg_ctl -o ...` with `host`, `port`, `socket_dir`, and a generic `extra_postgres_settings: BTreeMap<String, String>`.
  - replica cloning still uses `pg_basebackup -R` in `src/process/worker.rs`, which lets PostgreSQL own `primary_conninfo`/recovery state through `postgresql.auto.conf` and `standby.signal`.
- Because the task goal is “pgtuskmaster is the sole determinant of effective PostgreSQL configuration,” execution must remove both the stringly startup bag and the `pg_basebackup -R` recovery-conf side effect. Replacing only one of them would leave authority split across pgtuskmaster and Postgres-owned files.

### 2. Architecture decision locked for execution

- `PGDATA/pgtm.postgresql.conf` becomes the live authoritative PostgreSQL config file.
- PostgreSQL startup should use one explicit `config_file=<abs path to PGDATA/pgtm.postgresql.conf>` override and stop passing owned GUCs through ad hoc `-h`, `-p`, `-k`, or generic `-c key=value` startup injection.
- `listen_addresses`, `port`, `unix_socket_directories`, `hba_file`, `ident_file`, `ssl`, `ssl_cert_file`, `ssl_key_file`, `ssl_ca_file`, `hot_standby`, `primary_conninfo`, `primary_slot_name`, and user-supplied opaque extra GUCs must be represented as typed fields in one Rust model, then serialized deterministically by one serializer.
- Replica follow state must no longer come from `pg_basebackup -R`. Execution must instead make pgtuskmaster own both:
  - the typed replica GUCs in `pgtm.postgresql.conf`
  - the presence or absence of `PGDATA/standby.signal`
- On primary starts, execution must remove any stale managed replica-follow artifacts (`standby.signal`, and if present stale `primary_conninfo`/`primary_slot_name` lines by regenerating the full authoritative config).
- Opaque extra GUCs are allowed only via one explicit runtime-config field and must not override pgtuskmaster-owned keys.

### 3. Typed model and serializer design to implement

- Add one dedicated domain module for authoritative PostgreSQL config. Use a new crate module instead of growing the current `BTreeMap` API.
- Preferred shape for the new domain:
  - one top-level typed config struct for `pgtm.postgresql.conf`
  - one enum or typed sub-struct that distinguishes startup role:
    - primary/local standalone start
    - replica follow-leader start with typed upstream conninfo and optional slot name
  - one explicit collection for opaque user-owned extra GUCs, stored deterministically
- Serializer contract:
  - emits the managed header plus fully rendered GUC assignments
  - uses a fixed owned-setting order, then deterministic ordering for opaque extra GUCs
  - renders booleans as `on`/`off`
  - renders paths and free-form strings with PostgreSQL-conf-safe quoting/escaping
  - omits optional settings cleanly when absent
  - rejects extra GUC keys that collide with owned keys
- Typed handling policy to preserve in execution unless skeptical review proves it wrong:
  - `listen_addresses` = `cfg.postgres.listen_host`
  - `port` = `cfg.postgres.listen_port`
  - `unix_socket_directories` = `cfg.postgres.socket_dir`
  - `hba_file` / `ident_file` = managed file paths under `PGDATA`
  - `ssl` and TLS file paths derive only from `cfg.postgres.tls`
  - `hot_standby` is `on` for replica starts and omitted or explicitly `off` for primary starts only if needed for determinism
  - `primary_conninfo` is rendered from typed `PgConnInfo` when following a leader
  - `primary_slot_name` remains typed `Option<String>` and is omitted when no slot policy exists

### 4. File-by-file execution checklist

- [x] `src/postgres_managed.rs`
  - [x] Replace `ManagedPostgresConfig.extra_settings: BTreeMap<String, String>` with a typed managed-start product, likely a struct that includes:
    - authoritative `postgresql_conf_path`
    - managed HBA/ident/TLS file paths
    - whether `standby.signal` must exist
  - [x] Stop writing a header-only `pgtm.postgresql.conf`; instead build the full typed model and serialize it here through the new domain module.
  - [x] Keep managed file materialization for `pgtm.pg_hba.conf`, `pgtm.pg_ident.conf`, `pgtm.server.crt`, `pgtm.server.key`, and optional `pgtm.ca.crt`.
  - [x] Add explicit production-TLS wording in comments/errors: pgtuskmaster only copies user-supplied cert/key/CA material into managed files; it does not generate production credentials.
  - [x] Add managed ownership for `standby.signal`: create it for replica starts, remove it for primary starts.
- [x] New authoritative-config domain module
  - [x] Create a dedicated module such as `src/postgres_managed_conf.rs` or a `src/postgres_managed/` submodule owned only by this serializer domain.
  - [x] Define the typed config model and deterministic serializer there.
  - [x] Keep it free of backup-era concepts and free of generic stringly bags.
- [x] `src/config/schema.rs`
  - [x] Add one explicit config field for pgtuskmaster-owned opaque extra GUCs under `[postgres]`.
  - [x] Keep this as the only user surface for unmanaged-but-owned GUCs; do not reintroduce startup flag bags elsewhere.
  - [x] Reify the field as a deterministic type, preferably a `BTreeMap<String, String>` or a new typed wrapper around one.
- [x] `src/config/parser.rs`
  - [x] Normalize the new extra-GUC field into `RuntimeConfig`.
  - [x] Validate names and values so serialization cannot be malformed:
    - key must be non-empty and restricted to a safe PostgreSQL-GUC identifier character set
    - key must not collide with owned settings such as `config_file`, `hba_file`, `ident_file`, `listen_addresses`, `port`, `unix_socket_directories`, `ssl`, `ssl_cert_file`, `ssl_key_file`, `ssl_ca_file`, `hot_standby`, `primary_conninfo`, `primary_slot_name`
    - value must reject control characters that would break line-oriented `.conf` rendering
  - [x] Keep validation errors field-specific and actionable (`postgres.extra_gucs.<key>` or equivalent).
- [x] `src/config/defaults.rs`
  - [x] Do not add hidden defaults for owned PostgreSQL security or replication settings.
  - [x] Only add a safe empty default for the new extra-GUC field if the schema shape requires it.
- [x] `src/config/mod.rs`
  - [x] Re-export the new config field/wrapper cleanly if the rest of the crate needs it.
- [x] `src/lib.rs`
  - [x] Export the new authoritative-config module.
- [x] `src/process/jobs.rs`
  - [x] Replace `StartPostgresSpec.host`, `port`, `socket_dir`, and `extra_postgres_settings` with a typed start input that points at a fully prepared managed config artifact.
  - [x] Add a typed start-intent payload if the managed-config builder still needs to know whether the start is primary or replica.
- [x] `src/process/worker.rs`
  - [x] Change `build_command(...)` so `StartPostgres` only passes the managed `config_file` override required to boot from `pgtm.postgresql.conf`, plus `-l`/timeout handling.
  - [x] Remove generic `validate_postgres_setting(...)` usage from the start path once the string bag is gone.
  - [x] Remove `-R` from the basebackup command so PostgreSQL no longer writes recovery config on pgtuskmaster’s behalf.
  - [x] Keep password/env handling and timeout behavior intact.
- [x] `src/runtime/node.rs`
  - [x] Replace the payload-less `StartupAction::StartPostgres` with a typed action or equivalent helper input so startup can distinguish primary starts from replica starts.
  - [x] Thread that typed start intent from `plan_startup` through `build_startup_actions(...)` into `run_start_job(...)`; do not re-derive role at the final start step after planning already selected it.
  - [x] `StartupMode::CloneReplica { source, .. }` must pass typed follow-leader info into managed-config materialization for the later start step.
  - [x] `StartupMode::ResumeExisting` must no longer rely on legacy PostgreSQL-owned recovery config. Execution must make the resume path choose an explicit managed start intent, and verification must challenge whether DCS-derived role selection is sufficient or whether a small persisted managed marker is needed.
- [x] `src/ha/actions.rs`, `src/ha/lower.rs`, `src/ha/process_dispatch.rs`
  - [x] Ensure HA-driven starts have the same typed start intent available as runtime startup.
  - [x] Do not rely on `FollowLeader` being a separate no-op action while `StartPostgres` lacks the upstream identity needed to render `primary_conninfo`.
  - [x] Fix the missing data flow at the action/effect shape level rather than by reordering apply buckets: carry optional leader-follow context on `PostgresEffect::Start` / `HaAction::StartPostgres`, or otherwise refactor lowering so the start request itself receives all data needed to build replica config deterministically.
- [x] `src/ha/source_conn.rs`
  - [x] Reuse or extend the existing typed `PgConnInfo` construction for replica `primary_conninfo` rendering instead of inventing a string serializer in a second place.
- [x] `src/test_harness/ha_e2e/startup.rs`
  - [x] Update the real startup assertions so `SHOW config_file;` points to `PGDATA/pgtm.postgresql.conf`, not default `PGDATA/postgresql.conf`.
  - [x] Replace the stale expectation variable that still targets `PGDATA/postgresql.conf`; this test currently encodes the old live-config contract and will fail even if the code is correct.
  - [x] Add assertions that the managed config file contains the expected owned paths/settings and remains backup-free.
  - [x] Add replica-focused assertions if this harness already exercises follower bootstrap/startup: `SHOW hot_standby;` and `SHOW primary_conninfo;` should reflect the managed file, not `postgresql.auto.conf`.
- [x] `src/logging/postgres_ingest.rs`
  - [x] This worker-local real test currently appends lines directly to `PGDATA/postgresql.conf` before `StartPostgres`. It must be updated so the logging config lines are injected through the same managed-config serializer or managed-config builder inputs, otherwise the test will stop affecting the live server after the `config_file` cutover.
- [x] `src/worker_contract_tests.rs` and compile-driven sample builders
  - [x] Update any sample `StartPostgresSpec` construction or `RuntimeConfig` literals broken by the new typed fields.
- [x] `docs/src/operator/configuration.md`
  - [x] Document the new `[postgres]` extra-GUC field and state clearly that pgtuskmaster now owns the live PostgreSQL config file under `PGDATA/pgtm.postgresql.conf`.
- [x] `docs/src/contributors/ha-pipeline.md`
  - [x] Replace vague “postgresql.conf-style settings” language with the precise authoritative-config behavior if it becomes stale after execution.

### 5. Required behavioral changes to verify during execution

- Startup and HA-driven starts both materialize the same authoritative config shape and both boot PostgreSQL from `PGDATA/pgtm.postgresql.conf`.
- No generic startup GUC bag remains in code, tests, or docs.
- No PostgreSQL-owned recovery/autoconf path remains in the managed startup contract:
  - `pg_basebackup -R` removed
  - replica starts driven by typed `primary_conninfo`/`hot_standby` plus managed `standby.signal`
- Opaque extra GUCs cannot override owned keys and serialize deterministically.
- TLS ownership is explicit and test-only cert generation remains confined to harnesses.

### 6. Test plan the execution pass must complete

- Add direct unit tests in the new managed-config domain for:
  - fixed owned-setting order
  - extra-GUC deterministic ordering
  - string/path quoting and escaping
  - boolean rendering
  - optional enum/replica field rendering
  - forbidden-key rejection for extra GUCs
- Update `src/postgres_managed.rs` tests to assert:
  - `pgtm.postgresql.conf` contains real rendered settings instead of only a header
  - `config_file` is not emitted as a user-overridable extra GUC
  - TLS files and comments keep the new ownership wording
  - `standby.signal` create/remove behavior works as intended
- Update `src/process/worker.rs` unit tests to assert:
  - `StartPostgres` uses the managed `config_file` override instead of a list of per-setting `-c` flags
  - `BaseBackup` no longer includes `-R`
- Update runtime/HA contract tests affected by the new typed start intent.
- Update real-binary coverage in `src/test_harness/ha_e2e/startup.rs` and `src/logging/postgres_ingest.rs` so the live config path change is exercised, not bypassed.
- Finish with all required gates:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`

### 7. Execution order for the later `NOW EXECUTE` pass

1. Introduce the typed authoritative config domain module and wire `src/postgres_managed.rs` to build/serialize it.
2. Add the new `[postgres]` extra-GUC config surface plus parser validation.
3. Refactor `StartPostgresSpec` and `build_command(...)` so start uses the managed `config_file` path rather than generic setting injection.
4. Refactor runtime startup and HA lowering/dispatch together so both primary and replica starts pass one typed start intent into managed-config materialization; remove the now-redundant no-op `FollowLeader` process-side start contract if that path becomes dead.
5. Remove `pg_basebackup -R` and add pgtuskmaster-owned `standby.signal` handling.
6. Update unit tests first, then real-binary startup/logging tests, then contributor/operator docs.
7. Run the full required gates, then mark `<passes>true</passes>`, task-switch, commit, and push.

### 8. Risks that skeptical review must challenge explicitly

- `StartupMode::ResumeExisting` is now the least obvious path because it can no longer lean on old PostgreSQL-owned recovery files. The verification pass must challenge whether the proposed DCS-derived role selection is sufficient, or whether execution needs a more explicit persisted managed-start marker.
- `ha/apply.rs` ordering is a distraction: the real risk is that `PostgresEffect::Start` and `HaAction::StartPostgres` currently carry no leader context while `FollowLeader` dispatch is process-side skipped. Execution must challenge action/effect shape first, not just bucket ordering.
- `src/logging/postgres_ingest.rs` currently mutates `PGDATA/postgresql.conf` directly inside a real test. The verification pass must not forget that this will silently stop working after the `config_file` cutover unless the test helper is updated.

NOW EXECUTE
