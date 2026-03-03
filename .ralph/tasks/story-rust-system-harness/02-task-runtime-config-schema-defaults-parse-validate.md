---
## Task: Implement runtime config schema defaults parser and validation <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>01-task-core-types-time-errors-watch-channel</blocked_by>

<description>
**Goal:** Define and validate the full typed runtime configuration model.

**Scope:**
- Create `src/config/schema.rs`, `src/config/defaults.rs`, `src/config/parser.rs`, `src/config/mod.rs`.
- Implement `RuntimeConfig`, nested config structs, `ProcessConfig`, and `BinaryPaths`.
- Implement `load_runtime_config`, `apply_defaults`, and `validate_runtime_config`.

**Context from research:**
- Build Order step 2 and typed-runtime-input rule in plan.
- Config must fully control runtime behavior; no hidden magic constants.

**Expected outcome:**
- Runtime config can be loaded from file, defaulted, and rejected on invalid settings.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] All config structs from plan are present and wired through `RuntimeConfig`.
- [x] Validation covers mandatory binary paths, timeout bounds, and required HA/DCS settings.
- [x] Table-driven tests cover valid configs, missing fields defaulting, and invalid-file rejections.
- [x] Run targeted config tests.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] If any fail, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/`.
</acceptance_criteria>
<execution_plan>
## Detailed Implementation Plan

1. Pre-flight and phase bookkeeping
- [x] Set this task status tag from `not_started` to `in_progress` at execution start.
- [x] Confirm this task is unblocked by checking task 01 is complete (already true).
- [x] Keep task-file updates incremental: check each acceptance item only after concrete evidence (test output or code diff).

2. Resolve authoritative config contract from plan and existing codebase
- [x] Extract all required runtime config types and function signatures from `RUST_SYSTEM_HARNESS_PLAN.md` and map them to concrete Rust modules:
  - [x] `src/config/schema.rs`
  - [x] `src/config/defaults.rs`
  - [x] `src/config/parser.rs`
  - [x] `src/config/mod.rs`
- [x] Enumerate the full struct set required by task and plan references:
  - [x] Root: `RuntimeConfig`
  - [x] Nested: `ClusterConfig`, `PostgresConfig`, `DcsConfig`, `HaConfig`, `ProcessConfig`, `ApiConfig`, `DebugConfig`, `SecurityConfig`, `BinaryPaths`
  - [x] Parsing support: `PartialRuntimeConfig` (+ partial nested structs as needed)
  - [x] Errors: `ConfigError`
- [x] Confirm visibility follows plan intent (private-by-default); export only surfaces needed by tests and near-future worker wiring.

3. Dependency and crate wiring
- [x] Add serde/config parsing dependencies in `Cargo.toml` (at minimum `serde` with derive and `toml` or equivalent parser crate).
- [x] Ensure `src/lib.rs` exports `pub mod config;` and preserves existing `state` exports.
- [x] Add `src/config/mod.rs` re-exports for key types/functions:
  - [x] `RuntimeConfig`
  - [x] `ProcessConfig`
  - [x] `BinaryPaths`
  - [x] `load_runtime_config`
  - [x] `apply_defaults`
  - [x] `validate_runtime_config`
  - [x] `ConfigError`

4. Schema model implementation (`src/config/schema.rs`)
- [x] Implement strongly typed config structs and derive traits needed for tests (`Clone`, `Debug`, `PartialEq`, `Eq`).
- [x] Keep fields explicit; avoid hidden constants in worker code (all runtime knobs belong here).
- [x] Implement `ProcessConfig` exactly with timeout fields and binary paths:
  - [x] `pg_rewind_timeout_ms`
  - [x] `bootstrap_timeout_ms`
  - [x] `fencing_timeout_ms`
  - [x] `binaries: BinaryPaths`
- [x] Implement `BinaryPaths` exactly with:
  - [x] `postgres`
  - [x] `pg_ctl`
  - [x] `pg_rewind`
  - [x] `initdb`
  - [x] `psql`
- [x] Define `PartialRuntimeConfig` and partial nested structs with `Option<T>` fields for defaulting logic.

5. Defaulting layer (`src/config/defaults.rs`)
- [x] Implement `apply_defaults(raw: PartialRuntimeConfig) -> RuntimeConfig`.
- [x] Centralize all default constants in this file; do not scatter numeric defaults across parser/validation.
- [x] Keep defaults only for genuinely optional fields; do **not** synthesize placeholder values for required fields.
- [x] Merge logic rules:
  - [x] fill missing optional values with deterministic defaults
  - [x] preserve caller-provided values verbatim
  - [x] carry required-field absence forward as explicit validation failures (or decode failures), never silent placeholders
- [x] Add focused unit tests for defaults behavior (field-by-field table tests).

6. Parser and validation layer (`src/config/parser.rs`)
- [x] Implement `load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError>` pipeline:
  - [x] read file bytes/string
  - [x] decode into `PartialRuntimeConfig`
  - [x] apply defaults
  - [x] validate
  - [x] return fully typed runtime config
- [x] Implement `validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError>` with mandatory checks required by task acceptance:
  - [x] all binary paths non-empty
  - [x] timeout bounds sane and non-zero
  - [x] HA/DCS required settings present and internally consistent
  - [x] include at least one cross-field invariant check (for example, interval/timeout relationship) where both fields exist
  - [x] include clear, typed, actionable errors for each failed validation branch
- [x] Implement `ConfigError` variants for:
  - [x] file I/O failures
  - [x] decode/parse failures
  - [x] validation failures (field-specific)
- [x] Use strict parse semantics for config typo safety (e.g., `serde(deny_unknown_fields)` on partial structs) so unknown keys are rejected.

7. Test strategy (table-driven and integration-like)
- [x] Add unit tests in config modules for:
  - [x] valid complete config passes
  - [x] missing optional fields are defaulted
  - [x] malformed config file is rejected
  - [x] unknown keys are rejected
  - [x] invalid required values (timeouts/paths/HA-DCS invariants) are rejected with expected error type
- [x] Add at least one test that exercises full `load_runtime_config` from temp file input.
- [x] Keep tests deterministic and minimal; avoid external services.

8. Acceptance criteria mapping and checklist closure
- [x] Map each acceptance checkbox to a concrete test or code location and tick only after evidence:
  - [x] struct presence and wiring through `RuntimeConfig`
  - [x] validation of binary paths/timeouts/HA-DCS settings
  - [x] table-driven valid/default/invalid parsing tests
  - [x] targeted config tests executed (`cargo test config --all-targets`)
  - [x] `make check` passed
  - [x] `make test` passed
  - [x] `make lint` passed
  - [x] `make test` passed

9. Failure handling protocol
- [x] If any required command fails, do not mark pass tags true.
- [x] Create bug task(s) via `$add-bug` skill with repro command and relevant logs.
- [x] Keep this task status `in_progress` until all failures are resolved or handed off.

10. Finalization protocol (only after all commands pass)
- [x] Update this task header tags:
  - [x] `<status>done</status>`
  - [x] `<passes>true</passes>`
  - [x] `<passing>true</passing>`
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes with required message format:
  - [x] `task finished 02-task-runtime-config-schema-defaults-parse-validate: <summary + test evidence + notable challenges>`
- [x] Append learnings/surprises to `AGENTS.md`.
- [x] Append diary entry to progress log before ending the turn.
</execution_plan>

NOW EXECUTE
