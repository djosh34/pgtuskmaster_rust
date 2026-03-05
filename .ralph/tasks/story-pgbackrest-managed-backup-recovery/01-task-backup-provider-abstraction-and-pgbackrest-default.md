---
## Task: Introduce backup-provider abstraction with pgBackRest as the default provider <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Add a provider abstraction for backup/restore operations and ship pgBackRest as the first-class default integration with explicit runtime config and binary path support.

**PO Directive (2026-03-05):** Use pgBackRest config-method ownership only. Do not rely on repo-local wrapper/hack paths; use minimal CLI flags, with behavior/config sourced from managed config surfaces first.

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
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Add provider abstraction module(s), e.g. `src/backup/mod.rs`, `src/backup/provider.rs`, with:
- [x] clear trait/enum boundaries between orchestration and provider-specific command rendering
- [x] typed operation inputs/outputs for backup/info/check/restore/archive actions
- [x] Extend config schema in `src/config/schema.rs`:
- [x] add `process.binaries.pgbackrest: PathBuf` as required binary path
- [x] add explicit backup config block (stanza/repo/options/provider selection defaults)
- [x] keep `serde(deny_unknown_fields)` guarantees
- [x] Extend parser/defaults in `src/config/parser.rs` and `src/config/defaults.rs`:
- [x] validate non-empty pgBackRest binary path
- [x] validate backup block required fields and safe defaults
- [x] update parser normalization tests for all new required/optional fields
- [x] Update process command/state surfaces in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`:
- [x] add typed process job kinds/specs for backup provider operations
- [x] build commands with robust arg/env handling and clear error messages
- [x] ensure subprocess output capture remains structured and correlated by job id/job kind
- [x] Update runtime wiring in `src/runtime/node.rs` (or new `src/backup/worker.rs`) to initialize provider-facing state cleanly without ad-hoc globals
- [x] Update real-binary prerequisites in `src/test_harness/binaries.rs`:
- [x] add `pgbackrest` requirement helpers
- [x] include `pgbackrest` in process binary fixtures used by real tests
- [x] Add/extend installation tooling for local/CI prerequisites (expected location under `tools/`, e.g. `tools/install-pgbackrest.sh`)
- [x] Update operator docs for new backup/provider config in `docs/src/operator/configuration.md` and related quick-start pages
- [x] Add focused tests for provider command rendering and config validation failure modes
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### Design decisions (lock these in first)
- [x] **Config surface:** add `process.binaries.pgbackrest` and a new `backup` config block under `config_version = "v2"`, but treat pgBackRest requirements as **conditional on enablement**:
  - [x] `backup.enabled = false` remains a safe default that does *not* require stanza/repo nor `process.binaries.pgbackrest` to be set
  - [x] if `backup.enabled = true` (and provider is pgBackRest), then `process.binaries.pgbackrest` + stanza/repo become required and validated non-empty
  - [x] add `process.backup_timeout_ms` to avoid accidental fallback to unrelated timeouts for backup/restore/check.
- [x] **Provider abstraction boundary:** keep orchestration/HA/API code provider-agnostic by:
  - [x] defining typed *operations* (backup/info/check/restore/archive_{get,push}) in a new `crate::backup` module
  - [x] defining a provider selection enum (`BackupProvider`) in config and a provider-facing command rendering layer that consumes typed operation inputs (no ad-hoc stringly pgBackRest CLI scattered around)
- [x] **Process integration:** implement backup operations as typed `ProcessJobKind` variants so subprocess execution stays centralized (timeouts/cancellation/output logging/job_id correlation stays uniform).
- [x] **Output strategy (this task):** do *not* wire full JSON parsing into runtime state yet (that’s task 06). For now:
  - [x] render pgBackRest commands such that machine output is possible (`--output=json` for `info`/`check` at minimum)
  - [x] rely on existing process-worker line capture + structured log attributes (`job_id`, `job_kind`, `binary`) for observability
  - [x] still define typed output models in `crate::backup` so later tasks can attach parsed JSON without changing public shapes again.
- [x] **Real-binary policy:** add `pgbackrest` to `tools/real-binaries-policy.json` and implement provenance version checking for pgBackRest in `src/test_harness/provenance.rs` (fail-closed like existing tools).

### Exhaustive file/module checklist (execute in roughly this order)

#### 0) New module scaffolding
- [x] `src/lib.rs`: add `pub(crate) mod backup;` (or `pub mod backup;` if API/tests need it public).
- [x] `src/backup/mod.rs`: module root; re-export provider-agnostic operation types + pgBackRest provider implementation entrypoints.
- [x] `src/backup/provider.rs`:
  - [x] define typed inputs/outputs for: `backup`, `info`, `check`, `restore`, `archive_push`, `archive_get`
  - [x] define provider-agnostic command rendering interface (trait or enum-dispatch function) returning a structured “command template” (program-independent: args/env/labels), *not* raw strings
  - [x] define shared validation helpers for option tokens (no empty tokens, no `\\0/\\n/\\r`, and forbid overriding managed fields like stanza/repo via `--stanza*` / `--repo*` tokens).
- [x] `src/backup/pgbackrest.rs` (or `src/backup/providers/pgbackrest.rs`):
  - [x] implement command rendering for each typed operation using pgBackRest CLI semantics
  - [x] ensure stanza/repo are always rendered from typed config (never implicit in free-form options)
  - [x] add unit tests for arg rendering for each operation (pure, no real binaries).

#### 1) Config schema changes
- [x] `src/config/schema.rs`:
  - [x] extend runtime `BinaryPaths` with `pgbackrest: Option<PathBuf>`
  - [x] add `BinaryPathsV2Input` with `Option<PathBuf>` leaves for *all* binaries (preserves current v2 “precise field path” validation instead of serde “missing field” errors)
  - [x] change `ProcessConfigV2Input.binaries` from `Option<BinaryPaths>` to `Option<BinaryPathsV2Input>`
  - [x] add runtime `BackupConfig` + `BackupProvider` + `BackupOptions` types (all `#[serde(deny_unknown_fields)]`)
  - [x] add v2 input wrappers `BackupConfigV2Input` / `BackupOptionsV2Input` with `Option<T>` leaves (for stable missing-field errors)
  - [x] extend `RuntimeConfig` with `backup: BackupConfig`
  - [x] extend `RuntimeConfigV2Input` with `backup: Option<BackupConfigV2Input>`
- [x] `src/config/mod.rs`:
  - [x] re-export new backup config types as needed by runtime/process/backup modules (keep the public surface minimal).

#### 2) Config defaults + normalization + validation
- [x] `src/config/defaults.rs`:
  - [x] add `default_backup_config()` (safe defaults only; keep `enabled=false` by default)
  - [x] update `normalize_process_config(...)` to normalize `BinaryPathsV2Input -> BinaryPaths` with **precise missing-field errors** for required postgres tools, while allowing `pgbackrest: None` when backups are disabled
  - [x] add `normalize_backup_config(input: Option<BackupConfigV2Input>) -> Result<BackupConfig, ConfigError>`:
    - [x] if `[backup]` omitted: use full safe default block
    - [x] if provided partially: merge with defaults
    - [x] default options lists to empty `Vec<String>`
- [x] `src/config/parser.rs`:
  - [x] wire `normalize_backup_config` into `normalize_runtime_config_v2`
  - [x] extend `validate_runtime_config`:
    - [x] **conditional**: if `backup.enabled` then require:
      - [x] `process.binaries.pgbackrest` present + non-empty path
      - [x] `backup.stanza` + `backup.repo` present + non-empty
    - [x] validate backup options tokens (reject empty/whitespace-only tokens; reject override flags `--stanza*` / `--repo*`)
  - [x] update parser normalization tests to include new required fields (both TOML fixtures and struct literal builders).

#### 3) Process job types/specs + command building
- [x] `src/process/jobs.rs`:
  - [x] add typed specs for each provider operation (include `stanza`, `repo`, and per-op options vectors; include `timeout_ms: Option<u64>` like existing specs)
  - [x] add new `ActiveJobKind` variants for backup provider operations.
- [x] `src/process/state.rs`:
  - [x] add `ProcessJobKind` variants for provider ops, each carrying a typed spec.
- [x] `src/process/worker.rs`:
  - [x] extend `timeout_for_kind`, `active_kind`, `job_kind_label` for new variants
  - [x] extend `build_command`:
    - [x] pick program from `config.binaries.pgbackrest` (fail with `InvalidSpec` if unset at runtime)
    - [x] build args via `crate::backup` command rendering (single source of truth)
    - [x] validate all spec fields with clear `ProcessError::InvalidSpec(...)` messages
    - [x] **defense-in-depth:** re-run critical option-token checks here (not just in config parsing) so runtime-generated requests cannot bypass safety rules
    - [x] preserve existing structured log identity (`job_id`, `job_kind`, `binary`) so subprocess output lines remain correlated
  - [x] add/update unit tests for command building for **every** new provider operation (pure, no real binaries)
  - [x] add 1 test that exercises new timeout mapping (`process.backup_timeout_ms` as default when spec timeout unset)
  - [x] add 1 Busy rejection test using a backup job kind
  - [x] add at least 1 real-binary smoke test that executes `pgbackrest --version` via the worker (ensures installer + provenance are wired correctly).

#### 4) Runtime wiring (minimal but clean)
- [x] `src/runtime/node.rs`:
  - [x] ensure `RuntimeConfig` stubs/contract tests include `backup` field (use `default_backup_config()` or an explicit test value)
  - [x] do not introduce globals; rely on passing `RuntimeConfig` through existing subscriber/publisher wiring.
- [x] `src/backup/worker.rs`:
  - [x] add a small provider-facing helper that turns `RuntimeConfig.backup` + typed operation inputs into `ProcessJobRequest`s (so HA/API later can call a stable interface)
  - [x] add at least one concrete caller path (or test-only adapter call) proving runtime uses `crate::backup` rather than rebuilding pgBackRest CLI strings ad-hoc.

#### 5) Test harness + provenance + installers
- [x] `src/test_harness/binaries.rs`:
  - [x] add `require_pgbackrest_bin_for_real_tests()`
  - [x] include `pgbackrest` in the returned `BinaryPaths` fixtures used by real-binary tests
  - [x] consider renaming `require_pg16_process_binaries_for_real_tests()` once it includes non-Postgres tools (optional; keep changes minimal if rename churn is high).
- [x] `src/test_harness/provenance.rs`:
  - [x] extend `ExpectedVersion` with `Pgbackrest { major: u32 }` (or `exact`) and implement verification
  - [x] update remediation messages to include `./tools/install-pgbackrest.sh`
- [x] `tools/real-binaries-policy.json`:
  - [x] add `pgbackrest` entry pointing at `.tools/pgbackrest/bin/pgbackrest`
  - [x] set expected version policy (recommended: major line check, e.g. `major: 2`, to avoid brittle pinning)
  - [x] set `allowed_target_prefixes` to trusted system locations (e.g. `/usr/bin`, `/usr/local/bin`) if using symlink strategy like Postgres installer.
- [x] `tools/install-pgbackrest.sh`:
  - [x] mirror distro gating + hardened command resolution pattern from `tools/install-postgres16.sh`
  - [x] install pgBackRest on Alma/RHEL-like systems by explicitly enabling required repos (likely `crb` + `epel-release`) before `dnf install pgbackrest`; fail with actionable diagnostics if package still unavailable
  - [x] symlink the canonical target into `.tools/pgbackrest/bin/pgbackrest`
  - [x] generate/merge attestation entry via `install-common.sh` (locked atomic merge).

#### 6) Docs updates (operator + quick-start)
- [x] `docs/src/operator/configuration.md`:
  - [x] add `process.binaries.pgbackrest` in the canonical config example
  - [x] add a `### [backup]` section documenting provider, stanza/repo, options, and misconfiguration symptoms
- [x] `docs/src/quick-start/prerequisites.md`: add pgBackRest as a prerequisite + link to configuration docs.
- [x] `docs/src/quick-start/first-run.md` + `docs/src/quick-start/initial-validation.md`: add minimal first-run readiness notes for backup provider.
- [x] `docs/src/contributors/testing-system.md` and `docs/src/contributors/harness-internals.md`: mention the new installer script alongside existing `install-etcd.sh` / `install-postgres16.sh`.

#### 7) Repo-wide compile fixes (mechanical but required)
- [x] Update every `BinaryPaths { ... }` literal to include `pgbackrest: ...` (sources include `src/`, `tests/`, `examples/`).
- [x] Update every `RuntimeConfig { ... }` literal to include `backup: ...`.
- [x] Update any JSON contract fixtures that serialize `process.binaries` (notably `src/test_harness/ha_e2e/startup.rs`).

#### 8) Focused tests to add/extend
- [x] Config validation tests:
  - [x] missing/empty `process.binaries.pgbackrest` fails with a precise field name
  - [x] missing/empty `backup.stanza` / `backup.repo` fails with a precise field name
  - [x] option token validation rejects empty strings and forbidden override flags
- [x] Provider command rendering tests:
  - [x] `pgbackrest info` uses `--output=json` (and includes stanza/repo)
  - [x] `pgbackrest backup` / `check` / `restore` / `archive-*` render subcommand + config-derived flags deterministically.

#### 9) Required gates (must be 100% green before setting `<passes>true</passes>`)
- [x] `make check`
- [x] `make test` (install any missing real binaries first; do not skip)
- [x] `make test-long`
- [x] `make lint`

NOW EXECUTE
