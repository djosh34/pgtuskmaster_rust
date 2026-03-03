---
## Task: Implement typed postgres config and conninfo parser renderer <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
**Goal:** Replace raw decisive postgres strings with typed config and strict conninfo parsing.

**Scope:**
- Implement typed `PgConfig`, `PgConnInfo`, and supporting value types in `src/pginfo/state.rs` and/or dedicated config domain module.
- Implement `parse_pg_conninfo` and `render_pg_conninfo`.
- Add strict validation and roundtrip tests.

**Context from research:**
- Plan requires no raw string decisions for critical HA fields.

**Expected outcome:**
- HA-relevant postgres config values are type-safe and validated.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `PgConfig` and `PgConnInfo` contain typed fields from plan.
- [x] Parser rejects invalid syntax, missing required keys, and unsupported ssl modes.
- [x] Roundtrip tests ensure `parse(render(x)) == x` for canonical forms.
- [x] Run targeted parser tests.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] On failure, create `$add-bug` tasks with failing input samples. (No failing gates remained after fixes.)
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, Skeptically Verified)

### Parallel exploration completed (12+ tracks)
- `src/pginfo/state.rs`: current `PgConfig` is only `extra: BTreeMap<String, String>` and is copied across many tests.
- `src/process/jobs.rs` + `src/process/worker.rs`: `PgRewindSpec.source_conninfo` is still raw `String` and only validated for non-empty.
- `src/ha/state.rs` + `src/ha/worker.rs`: `ProcessDispatchDefaults.rewind_source_conninfo` is raw `String`, passed directly into `PgRewindSpec`.
- `src/pginfo/query.rs`: no conninfo parser exists yet; current parser utilities are LSN/timeline only.
- `src/debug_api/worker.rs`, `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/e2e_multi_node.rs`, `src/worker_contract_tests.rs`: all have `PgConfig` construction fallout to update.
- `Cargo.toml`: no additional dependency is required for a first-party parser/renderer implementation.
- Existing test policy and crate lints confirm no `unwrap`/`expect`/`panic` usage is allowed.

### Scope decisions for this task
1. Add strict typed conninfo model and parser/renderer with deterministic canonical output.
2. Replace HA/process critical raw conninfo strings with the new typed model.
3. Make `PgConfig` explicitly typed for HA-relevant fields while preserving extensibility for non-critical values.
4. Keep changes focused to crate-internal API (`pub(crate)`), minimizing public API churn.

### Planned type model
1. Add `PgConnInfo` and supporting types in a dedicated `src/pginfo/conninfo.rs` module, then re-export from `pginfo::state` where needed:
- `PgConnInfo { host: String, port: u16, user: String, dbname: String, application_name: Option<String>, connect_timeout_s: Option<u32>, ssl_mode: PgSslMode, options: Option<String> }`
- `PgSslMode` enum: `Disable`, `Allow`, `Prefer`, `Require`, `VerifyCa`, `VerifyFull`
- `ConnInfoParseError` enum for precise failures:
  - syntax/tokenization errors
  - missing required key
  - duplicate key
  - invalid numeric value/range
  - unsupported key
  - unsupported `sslmode`

2. Replace `PgConfig` map-only shape with typed HA-relevant fields plus explicit extension bucket:
- `port: Option<u16>`
- `hot_standby: Option<bool>`
- `primary_conninfo: Option<PgConnInfo>`
- `primary_slot_name: Option<String>`
- `extra: BTreeMap<String, String>` (for non-critical passthroughs)

3. Keep strict invariants:
- Required conninfo keys are `host`, `port`, `user`, `dbname`.
- `sslmode` is optional and defaults to `prefer`.
- Unknown keys are rejected (strict mode for typo safety, because conninfo drives HA/process decisions).

### Planned parser and renderer behavior
1. `parse_pg_conninfo(input: &str) -> Result<PgConnInfo, ConnInfoParseError>`
- Parse libpq-style whitespace-delimited `key=value` tokens.
- Support quoted values (`'...'`) and escaped quotes/backslashes inside quoted values.
- Enforce end-of-input after each token to avoid silently accepting trailing garbage.
- Reject malformed tokens (missing `=`, empty key, unterminated quotes, trailing garbage).
- Enforce unique keys (duplicate key is an error).
- Parse `port` and `connect_timeout` as bounded integers.
- Map `sslmode` into `PgSslMode`, reject unsupported values.

2. `render_pg_conninfo(info: &PgConnInfo) -> String`
- Emit canonical deterministic key order:
  `host`, `port`, `user`, `dbname`, `application_name`, `connect_timeout`, `sslmode`, `options`.
- Quote and escape values only when required by conninfo syntax.
- Always render `sslmode` explicitly in canonical form to preserve roundtrip determinism.

3. Roundtrip contract:
- For valid typed values: `parse_pg_conninfo(&render_pg_conninfo(x)) == Ok(x.clone())`.
- Canonicalization tests may assert that parse->render normalizes key order and quoting.

### Planned integration points
1. Update process job spec:
- `src/process/jobs.rs`: change `PgRewindSpec.source_conninfo: String` -> `PgConnInfo`.

2. Update job command rendering:
- `src/process/worker.rs`: `build_command` for `ProcessJobKind::PgRewind` should call `render_pg_conninfo(&spec.source_conninfo)` for `--source-server`.
- Remove legacy `trim().is_empty()` string check; rely on typed value validity.

3. Update HA defaults and dispatch:
- `src/ha/state.rs`: `ProcessDispatchDefaults.rewind_source_conninfo` becomes typed `PgConnInfo`.
- `contract_stub()` stays infallible; construct a typed literal default conninfo directly so no `Result`/error plumbing is required across all callsites.
- `src/ha/worker.rs`: pass typed conninfo clone into `PgRewindSpec`.

4. Update all test fixtures and sample states constructing `PgConfig` and `PgRewindSpec` across:
- `src/pginfo/*`
- `src/dcs/*`
- `src/ha/*`
- `src/debug_api/*`
- `src/worker_contract_tests.rs`

### Planned tests (strict and skeptical)
1. Unit tests for conninfo parser/renderer:
- accepts minimal valid canonical input
- accepts quoted values with escapes
- rejects invalid syntax (missing `=`, unterminated quote, empty key)
- rejects missing required keys (`host`, `port`, `user`, `dbname`)
- rejects duplicate keys
- rejects unknown keys
- rejects unsupported `sslmode`
- rejects invalid `port` / `connect_timeout` ranges
- verifies canonical render key order
- verifies `parse(render(x)) == x` for representative variants

2. Integration-adjacent tests:
- process worker command building uses rendered conninfo string and includes `--source-server`.
- HA worker dispatch creates `ProcessJobKind::PgRewind` with typed conninfo.
- existing real `pg_rewind` test path remains executable with typed conninfo.
- add one regression test that parser rejects unknown key typo (for example `sslmdoe=require`) to harden typo safety.

3. Targeted test command before full gates:
- `cargo test pginfo::state::tests::` (or exact new conninfo test module path)

### Execution sequencing
1. Implement types and parser/renderer first (no call-site mutation yet), land parser tests.
2. Migrate process and HA structs to typed conninfo.
3. Fix compile fallout across all fixtures/tests.
4. Run targeted parser tests.
5. Run mandatory gates sequentially (no parallel cargo invocations):
- `make check`
- `make test`
- `make test-long`
- `make lint`
6. If any gate fails, create `$add-bug` task(s) with command, failing sample inputs, and observed errors.

### Completion updates (execution phase only)
1. Tick acceptance criteria only when backed by passing evidence.
2. Set `<status>done</status>` and `<passes>true</passes>` and `<passing>true</passing>` only after all required gates pass.
3. Run `/bin/bash .ralph/task_switch.sh` at task completion.
4. Commit all changes including `.ralph` artifacts with required message format.
</execution_plan>

EXECUTION COMPLETE
