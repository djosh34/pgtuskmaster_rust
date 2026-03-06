---
## Task: Introduce a typed authoritative managed Postgres config model and serializer <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add one authoritative managed-config domain module and use it as the only serializer for `pgtm.postgresql.conf`
- [ ] Replace `ManagedPostgresConfig.extra_settings: BTreeMap<String, String>` with typed managed-config output; no new generic stringly startup setting bag may remain
- [ ] Add one explicit config field for pgtuskmaster-owned opaque extra GUCs and validate GUC names deterministically
- [ ] Serializer output is deterministic and covered by direct unit tests for ordering, escaping, booleans, enums, and opaque extra GUC rendering
- [ ] Task text and code comments make TLS ownership explicit: production inputs are user-supplied, pgtuskmaster only materializes managed copies, and test-generated certs are test-only
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [ ] `src/postgres_managed.rs`
  - [ ] Stop exposing a generic `extra_settings` bag as the main managed-config product.
  - [ ] Introduce or call one typed authoritative managed-config builder.
  - [ ] Keep managed file materialization for `pgtm.pg_hba.conf`, `pgtm.pg_ident.conf`, `pgtm.server.crt`, `pgtm.server.key`, and optional `pgtm.ca.crt`.
  - [ ] Make TLS-file comments and errors explicit that production contents come from user config, not generated by pgtuskmaster.
- [ ] `src/config/schema.rs`
  - [ ] Add one explicit config surface for pgtuskmaster-owned opaque extra GUCs.
  - [ ] Do not introduce a second parallel config surface for unmanaged startup flags.
- [ ] `src/config/parser.rs`
  - [ ] Validate opaque extra GUC names and values sufficiently to prevent malformed serialization or option injection.
  - [ ] Keep validation errors field-specific and actionable.
- [ ] `src/config/defaults.rs`
  - [ ] Do not synthesize security-sensitive managed Postgres settings here.
  - [ ] Only add defaults if they are safe and explicitly part of the managed-config contract.
- [ ] `src/config/mod.rs`
  - [ ] Re-export the new typed managed-config-facing fields cleanly if required.
- [ ] `src/lib.rs`
  - [ ] Export any new managed-config module needed by the rest of the crate.
- [ ] New module(s), likely under `src/postgres_managed/` or a new dedicated managed-config module
  - [ ] Define the typed authoritative config model.
  - [ ] Define deterministic serialization to `.conf`.
  - [ ] Keep this code free of backup-specific concepts.
- [ ] Direct unit tests in the managed-config domain
  - [ ] Assert stable rendering order.
  - [ ] Assert correct quoting/escaping for string GUC values.
  - [ ] Assert booleans and enums serialize correctly.
  - [ ] Assert opaque extra GUCs are merged deterministically and cannot override forbidden owned keys unless that override policy is intentionally and explicitly designed.
