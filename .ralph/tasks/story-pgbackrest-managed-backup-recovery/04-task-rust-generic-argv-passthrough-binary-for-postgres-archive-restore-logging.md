---
## Task: Rust WAL Passthrough Binary for Postgres Archive Restore Logging <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Reintroduce archive/restore observability with a Rust binary command invoked by Postgres that performs passthrough execution and logs invocations via pgtuskmaster.

**Scope:**
- Add a generic Rust passthrough executable command surface dedicated to Postgres archive/restore invocation passthrough with strict exit-code fidelity.
- Keep Postgres config model with explicit `archive_command` / `restore_command` strings, but route commands to the Rust binary instead of shell wrapper generation, with no shell parsing.
- Implement logging flow where passthrough process sends structured event payloads to pgtuskmaster (API or equivalent local interface) and pgtuskmaster emits normal structured logs.
- Support both operation kinds (`archive-push`, `archive-get`) and preserve full command argument semantics.
- Enforce argv-only execution model: no command-string parsing, no shell eval, no quoting reconstruction; child processes are spawned from already-tokenized args.
- Enforce absolute-path-only executable configuration for passthrough target commands and `pgbackrest`; do not perform PATH lookup or fallback resolution.

**Context from research:**
- Existing shell wrapper logs by writing JSONL directly to file and returning pgBackRest exit code.
- Desired architecture: no generated scripts, no shell JSON construction, and no wrapper-specific file format contract.
- Existing config and managed postgres wiring already controls archive/restore command assignment and can be adapted to point at Rust binary command lines.

**Expected outcome:**
- Postgres calls a Rust binary command for archive and restore actions.
- Rust passthrough executes target backup command, forwards/stores output safely, and returns exact exit status.
- Each invocation is logged through pgtuskmaster with structured fields equivalent or better than current backup event model.
- No bash or shell-wrapper code path remains.
- Command execution is fully deterministic from configured argv vectors with absolute executable paths only.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Add Rust binary/subcommand implementation (for example under [src/bin/](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/bin)) that supports explicit modes equivalent to archive push/get and validates required args.
- [x] Implement strict passthrough behavior: execute configured backup command/toolchain from tokenized argv vectors, capture bounded output, and return exact child exit code to caller.
- [x] Passthrough command execution must be argv-native only (`std::process::Command` style argument vector usage): no shell invocation, no command-string parsing, and no re-tokenization logic.
- [x] Implement structured event emission from passthrough binary to pgtuskmaster logging surface (API or internal endpoint), with robust error handling that never masks underlying backup command exit semantics.
- [x] Update configuration schema/defaults and managed-postgres wiring in [src/config/schema.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/schema.rs), [src/config/defaults.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/defaults.rs), [src/config/parser.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/parser.rs), and [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) so Postgres command strings target the Rust passthrough binary.
- [x] Config surface for passthrough target commands must be array-of-args only (no free-form command strings); executable path must be absolute.
- [x] Config validation must reject non-absolute executable paths and must reject PATH-based command resolution for passthrough targets and `process.binaries.pgbackrest`.
- [x] Align `pgbackrest` handling with other binaries: mandatory explicit configured absolute path, never PATH lookup, never implicit command-name execution.
- [x] Preserve or improve existing backup event fields (`provider`, `event_kind`, `invocation_id`, `status_code`, `success`, output truncation indicator, and relevant WAL path fields) in centralized pgtuskmaster logs.
- [x] Add integration tests covering:
- [x] Postgres archive push path invokes Rust binary and returns child success.
- [x] Postgres archive get path invokes Rust binary and returns child failure exactly.
- [x] Structured log events are emitted through pgtuskmaster with expected fields under concurrent invocations.
- [x] Special-character/space/single-quote argument paths are handled correctly through argv vectors without shell-wrapper quoting hacks.
- [x] Tests prove non-absolute binary paths are rejected during config validation and PATH is never consulted.
- [x] Update docs to describe Rust passthrough architecture and remove obsolete wrapper-file guidance in [docs/src/operator/observability.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/observability.md), [docs/src/operator/configuration.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/configuration.md), and relevant lifecycle docs.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan

### Notes / constraints

- Deep skeptical verification was done at `2026-03-05 04:12 CET` using 20+ explore_spark subagents after the usage-limit window cleared. Key plan adjustments from that review:
  - Use existing `reqwest` dependency via the `blocking` client for helper -> node event emission (no ad-hoc tokio runtime).
  - `src/backup/archive_command.rs` is already argv-native (`RenderedCommand { program, args }`); keep command rendering tokenized and conf-string rendering as a narrow boundary.
  - Replace the shell-execution test (`/bin/sh -c ...`) with direct `pgtuskmaster wal ...` invocation tests; keep only a small “Postgres conf string contains expected quoting/placeholders” test.
  - Harden `/events/wal` with loopback-only peer enforcement in addition to token auth.
- Current state already has a `pgtuskmaster wal ...` helper subcommand (`src/bin/pgtuskmaster.rs`) + a helper JSON config file in `PGDATA/pgtm.pgbackrest.archive.json` (`src/backup/archive_command.rs`) + managed Postgres wiring that injects `archive_command` / `restore_command` to call that helper (`src/postgres_managed.rs`).
- The main missing pieces relative to this task are: (1) strict absolute-path enforcement (no PATH lookup, no “absolutize”), (2) bounded stdout/stderr capture with truncation metadata, and (3) structured event emission into *central* pgtuskmaster logs (not “helper prints JSONL itself”).

### 1) Enforce absolute-path-only binaries (no PATH)

- [x] Update config validation to reject any non-absolute binary paths:
  - [x] `src/config/parser.rs`: call `validate_absolute_path(...)` for every required `process.binaries.*` path:
    - `postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, `psql`
    - and `pgbackrest` when present (and required when `backup.enabled=true`).
  - [x] Add/adjust tests in `src/config/parser.rs` to prove relative paths are rejected (and that “pgbackrest” as a bare name is rejected).
- [x] Remove any “make relative absolute” behavior for WAL helper config:
  - [x] `src/backup/archive_command.rs`: replace `absolutize_path(...)` with `validate_absolute_path(...)`-style checks and return `ArchiveCommandError::InvalidConfig` when a path is not absolute.
  - [x] Ensure `ArchiveCommandConfig.pgbackrest_bin` and `ArchiveCommandConfig.pg1_path` are **required absolute paths**, not “relative joined to cwd”.
- [x] Eliminate any helper-path PATH fallback in managed-Postgres wiring:
  - [x] `src/self_exe.rs`: replace `get_or_fallback()` with a non-PATH behavior (either “must be initialized” or “use current_exe()”), so `archive_command` never points to a PATH-resolved `pgtuskmaster`.
  - [x] `src/postgres_managed.rs`: treat failure to resolve an absolute helper path as configuration error.
- [x] Defense-in-depth: enforce “no PATH” at execution boundary too:
  - [x] `src/process/worker.rs` (near `build_command` / spawn): reject a `program` that is not absolute before calling `Command::new(...)`.
  - [x] `src/bin/pgtuskmaster.rs` `run_wal`: reject a rendered target `program` that is not absolute (should already be guaranteed by config validation, but keep this as a last line of defense).

### 2) Add a structured WAL event ingest surface into the running node

Goal: WAL helper (short-lived, invoked by Postgres) emits a structured payload to the *running node*, and the node’s `LogHandle` emits it to configured sinks.

- [x] Add a new API endpoint that accepts WAL helper events:
  - [x] `src/api/controller.rs` (or a new small controller module): define a request type like `WalEventIngestInput` with `#[serde(deny_unknown_fields)]`.
  - [x] `src/api/worker.rs`:
    - [x] Add `("POST", "/events/wal")` route.
    - [x] Parse JSON body into the input type, validate minimal invariants (non-empty ids/kind, exit status present, etc).
    - [x] Emit a single structured event via `ctx.log.emit_event(...)` with stable naming:
      - `event.domain = "backup"`
      - `event.name = "backup.wal_passthrough"`
      - `event.result = "ok" | "error"`
      - attributes include the acceptance-criteria fields: `provider`, `event_kind`, `invocation_id`, `status_code`, `success`, truncation flags, plus WAL identifiers (`wal_path` or `wal_segment` + `destination_path`).
  - [x] Auth policy:
    - [x] Treat this endpoint as `EndpointRole::Read` (so either read/admin token works; admin token already always works).
    - [x] Keep existing “no tokens configured => allow all” behavior; in that case WAL helper can emit without a token.
  - [x] Local-only enforcement:
    - [x] Reject requests unless the peer address is loopback (`127.0.0.1` / `::1`). This avoids exposing a telemetry-ingest surface remotely even if the API listen address is non-loopback.

### 3) Extend the WAL helper to be a strict passthrough with bounded output + event emission

- [x] Execution behavior (strict exit-code fidelity + bounded IO):
  - [x] Implement a small reusable helper module (new file) like `src/wal_passthrough.rs`:
    - Inputs: `pgdata`, `kind`, WAL args (`wal_path` / `wal_segment` + `destination_path`), and max capture sizes.
    - Render the target command via existing `crate::backup::archive_command::render_*_from_pgdata`.
    - Spawn the child with **argv vectors only** (`std::process::Command`), **never** via shell.
    - Capture stdout/stderr in a bounded way (e.g. `64KiB` each):
      - Forward child stdout/stderr through to the helper’s stdout/stderr (so Postgres logs still contain tool output).
      - Also keep the first N bytes for event payload (record `stdout_truncated` / `stderr_truncated` booleans).
    - Compute duration (`started_at_ms`, `duration_ms`).
    - Map exit status faithfully:
      - normal exit code `0..=255` => return exact same code
      - Unix signal termination => return `128 + signal` when available
      - otherwise fall back to `1` (and include raw status details in the event).
  - [x] Update `src/bin/pgtuskmaster.rs` `run_wal(...)` to use this new passthrough runner.

- [x] Event emission (best-effort, never changes child exit code):
  - [x] Extend `ArchiveCommandConfig` (the `PGDATA/pgtm.pgbackrest.archive.json` file) to include enough info for the helper to reach the node:
    - [x] `api_local_addr` (e.g. `127.0.0.1:8080`), derived from the node’s API listen port but always loopback.
    - [x] `api_token` (optional; pick `read_token` if present, else `admin_token`, else `None`)
  - [x] `src/backup/archive_command.rs` `materialize_archive_command_config(...)` populates these fields from `RuntimeConfig.api` at materialization time.
  - [x] WAL helper sends `POST http://{api_local_addr}/events/wal` with JSON body:
    - Enable the `reqwest` `blocking` client feature (already a dependency) and use `reqwest::blocking::Client` with a strict total timeout (target `250ms`, no retries).
    - If the HTTP request fails: write a compact diagnostic to stderr and continue returning the child’s exit code.
  - [x] Security / permissions:
    - [x] Change the helper config file mode from `0644` to `0600` because it may include a token.

### 4) Tests (must be deterministic; no skipping)

- [x] Config validation tests:
  - [x] `src/config/parser.rs`: add tests asserting relative paths in `process.binaries.*` (including `pgbackrest`) are rejected.
  - [x] `src/backup/archive_command.rs`: add tests asserting relative `pgbackrest_bin` is rejected during materialization.

- [x] WAL passthrough execution tests (no real Postgres needed):
  - [x] Add direct helper invocation tests (avoid shell execution entirely):
    - Use a temporary **stub pgbackrest executable** (a small script written under `/tmp/...` with an absolute path) that:
      - writes its received argv to a file for assertions
      - prints enough stdout/stderr to trigger truncation behavior
      - exits with a chosen code (0 for push, nonzero for get) to validate exit-code fidelity
    - Run `pgtuskmaster wal --pgdata <PGDATA> archive-push <wal_path>` as a subprocess and assert:
      - exact exit code is preserved
      - stub `argv` matches expected tokens exactly (includes tokens with spaces/single-quotes unchanged)
      - event emission failure (if API not running) does not affect exit code (stderr may include a compact diagnostic)
    - Run `pgtuskmaster wal --pgdata <PGDATA> archive-get <wal_segment> <destination_path>` similarly and assert failure exit code propagation.
  - [x] Keep a small “Postgres config string rendering” test only:
    - Materialize managed Postgres config and assert `archive_command` / `restore_command` strings include the fixed helper invocation + correctly quoted `%p` / `%f` placeholders (do not execute the string).

- [x] Event ingest tests:
  - [x] Start an `ApiWorkerCtx` bound to `127.0.0.1:0` with a `TestSink` `LogHandle`.
  - [x] Send a sample `/events/wal` payload and assert a structured event is emitted with expected fields.
  - [x] Concurrency: spawn multiple helper invocations (or multiple POSTs) concurrently and assert all are accepted and logged.

- [x] Special-character argv correctness:
  - [x] Put option tokens containing spaces and single-quotes into `backup.pgbackrest.options.archive_push/archive_get` and verify the stub binary receives those tokens unchanged (proves argv-only, no retokenization).

### 5) Docs updates (remove stale wrapper guidance)

- [x] `docs/src/operator/observability.md`:
  - Explain that Postgres invokes `pgtuskmaster wal ...` and that this helper emits a structured event into the node via `/events/wal`.
  - Document which fields appear in logs and how to correlate with `invocation_id`.
- [x] `docs/src/operator/configuration.md`:
  - Update binary configuration section to state **all** `process.binaries.*` paths must be absolute.
  - Make `process.binaries.pgbackrest` explicitly required when `backup.enabled=true` and `backup.provider=pgbackrest`.
- [x] `docs/src/operator/recovery-bootstrap-runbook.md`:
  - Keep the “helper config file exists” check, but remove any references to shell wrappers or wrapper JSONL formats.

### 6) Verification gates (no exceptions)

- [x] Ensure required real binaries are installed/attested before running test suites:
  - `./tools/install-etcd.sh`, `./tools/install-postgres16.sh`, `./tools/install-pgbackrest.sh`
- [x] Run and pass (100%):
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`

NOW EXECUTE
