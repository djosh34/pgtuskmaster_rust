---
## Task: Introduce backup-provider abstraction with pgBackRest as the default provider <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a provider abstraction for backup/restore operations and ship pgBackRest as the first-class default integration with explicit runtime config and binary path support.

**Scope:**
- Add a new backup provider model that keeps pgBackRest as default while leaving a clean extension seam for future tools.
- Extend config schema/parser/defaults so backup features are explicitly configured and validated under `config_version = "v2"`.
- Add pgBackRest binary path configuration (`process.binaries.pgbackrest`) and propagate that path through runtime/process/test harness surfaces.
- Define command/spec types for backup lifecycle operations (`backup`, `info`, `check`, `restore`, archive commands) without coupling API/HA logic to pgBackRest CLI details.

**Context from research:**
- PostgreSQL 16 removed `recovery.conf`; recovery settings now come from normal config files and signal files, so operator tooling must manage recovery config deterministically: https://www.postgresql.org/docs/16/recovery-config.html
- `pg_basebackup` includes config files from the source cluster, which can inject bad runtime settings into restores if unmanaged: https://www.postgresql.org/docs/16/app-pgbasebackup.html
- pgBackRest documents stable JSON output via `--output=json`, which should be consumed instead of brittle text parsing: https://pgbackrest.org/command.html
- pgBackRest has no built-in scheduler, so scheduling must be integrated in our runtime layer: https://pgbackrest.org/user-guide.html

**Expected outcome:**
- Backup provider abstraction exists and is used by runtime/HA/API paths rather than hardcoding pgBackRest command strings throughout the codebase.
- pgBackRest is integrated as the default provider and fully wired in config validation, docs, and harness binary prerequisites.
- Future providers can be added without rewriting orchestration/state logic.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add provider abstraction module(s), e.g. `src/backup/mod.rs`, `src/backup/provider.rs`, with:
- [ ] clear trait/enum boundaries between orchestration and provider-specific command rendering
- [ ] typed operation inputs/outputs for backup/info/check/restore/archive actions
- [ ] Extend config schema in `src/config/schema.rs`:
- [ ] add `process.binaries.pgbackrest: PathBuf` as required binary path
- [ ] add explicit backup config block (stanza/repo/options/provider selection defaults)
- [ ] keep `serde(deny_unknown_fields)` guarantees
- [ ] Extend parser/defaults in `src/config/parser.rs` and `src/config/defaults.rs`:
- [ ] validate non-empty pgBackRest binary path
- [ ] validate backup block required fields and safe defaults
- [ ] update parser normalization tests for all new required/optional fields
- [ ] Update process command/state surfaces in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`:
- [ ] add typed process job kinds/specs for backup provider operations
- [ ] build commands with robust arg/env handling and clear error messages
- [ ] ensure subprocess output capture remains structured and correlated by job id/job kind
- [ ] Update runtime wiring in `src/runtime/node.rs` (or new `src/backup/worker.rs`) to initialize provider-facing state cleanly without ad-hoc globals
- [ ] Update real-binary prerequisites in `src/test_harness/binaries.rs`:
- [ ] add `pgbackrest` requirement helpers
- [ ] include `pgbackrest` in process binary fixtures used by real tests
- [ ] Add/extend installation tooling for local/CI prerequisites (expected location under `tools/`, e.g. `tools/install-pgbackrest.sh`)
- [ ] Update operator docs for new backup/provider config in `docs/src/operator/configuration.md` and related quick-start pages
- [ ] Add focused tests for provider command rendering and config validation failure modes
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
