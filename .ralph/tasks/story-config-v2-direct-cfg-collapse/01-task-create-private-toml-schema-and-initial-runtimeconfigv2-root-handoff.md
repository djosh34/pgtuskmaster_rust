## Task: Create Private TOML Schema And Initial `RuntimeConfigV2` Root Handoff <status>completed</status> <passes>false</passes>

<description>
**Goal:** Create the first concrete execution task for the config-v2 direct-cfg collapse story. The higher-order goal is to start the migration by making `src/config_v2/parser/private_schema.rs` the only TOML-parsable config shape, adding parse functions in the config-v2 loaders, and switching `src/runtime/node.rs` to root itself in `RuntimeConfigV2` only. This task intentionally does not finish the downstream migration. It must stop at the first compile-failing handoff once the remaining failures are only due to the old `src/config/` corridor that later tasks in this story will delete.

This task is intentionally narrow and must be treated as a handoff boundary for the rest of the story. It exists to force all TOML schema knowledge into one private parser-only module and to force `runtime/node.rs` to stop taking the old config root. The later direct-cfg reduction work belongs to `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md`, not here.

**Hard requirements from user discussion:**
- zero dependencies on the old `src/config/` package from the work introduced here; this is a hard requirement
- `src/config_v2/parser/private_schema.rs` is the only place allowed to define TOML-deserializable shapes for config files
- nothing from `private_schema.rs` may be exported outside parser-internal load paths
- all parsing functions for this task must live in:
  - `src/config_v2/parser/load_config.rs`
  - `src/config_v2/parser/load_operator_config.rs`
- `RuntimeConfigV2` must stay unaltered
- do not add new public config model types for this task
- do not alter `src/config_v2/types.rs` except for the minimum `OperatorConfigV2` top-level surface correction required to make the operator root consumable from `src/cli/config.rs`
- do not start the broader “make all callers use config_v2 directly” refactor in this task; that belongs to task 02
- migrate `src/runtime/node.rs` to use only `RuntimeConfigV2` and pass that root downward
- once the compile check fails only because old/to-be-deleted `src/config/` users still exist in this story, stop
- after creating this task, point `.ralph/current_task.txt` at `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md` and quit immediately

**Scope:**
- `src/config_v2/parser/private_schema.rs`
- `src/config_v2/parser/load_config.rs`
- `src/config_v2/parser/load_operator_config.rs`
- `src/config_v2/types.rs` for `OperatorConfigV2` only
- `src/cli/config.rs`
- `src/runtime/node.rs`
- only the minimum adjacent parser/runtime plumbing required to complete the root handoff

**Implementation direction:**
- in `src/config_v2/parser/private_schema.rs`, define the serde/TOML-only input shape for `config.toml`
- keep that module private to parser internals; do not re-export any of its types, helpers, or fields
- in `src/config_v2/parser/load_config.rs`, parse TOML through the private schema and map directly into the existing `RuntimeConfigV2`
- in `src/config_v2/parser/load_operator_config.rs`, keep parsing logic in the loader and reuse parser-private schema as needed without exporting parser-only types
- limit `src/config_v2/types.rs` changes to `OperatorConfigV2` only, keeping it a flat validated operator root with top-level fields instead of re-exporting source-world TOML DTOs
- in `src/cli/config.rs`, consume `OperatorConfigV2` directly and remove reliance on `crate::config::{InlineOrPath, SecretSource, resolve_inline_or_path_bytes, resolve_secret_string}` for config-v2 inputs
- in `src/runtime/node.rs`, replace old runtime-config root usage with `RuntimeConfigV2` only and pass that reference further down
- accept temporary compile fallout caused by downstream packages that still expect the old config tree, but verify the remaining failures are specifically in the old `src/config/` story corridor
- do not “fix forward” by creating adapter structs, duplicate config trees, or new intermediary config DTOs

**Explicit non-goals:**
- do not alter `RuntimeConfigV2` or any non-operator config-v2 model type
- do not create new public enums or structs to bridge old and new config
- do not migrate all downstream modules to read `RuntimeConfigV2` directly in this task
- do not continue once the root handoff is complete and remaining compile errors are isolated to old-config consumers

If you find any need to do alter types.rs, that is probably cuz we dropped support for stuff:

- removed support for having bytes of certs passed around (fully to be removed), always pass paths
- extra_roles must be gone
- and a lot more...

Only exception made is for 'OperatorConfigV2'
you can only add top_level fields.
they must never be 'Option', instead have default value be set during parse/validation step inside src/config_v2/parse...
always use Duration, no numbers for time

**Expected outcome:**
- parser-private TOML schema exists only in `src/config_v2/parser/private_schema.rs`
- TOML parsing happens only via the config-v2 loader functions
- `RuntimeConfigV2` stays unchanged
- `src/runtime/node.rs` uses `RuntimeConfigV2` as its only runtime config root
- the next engineer can continue from task 02 to collapse one config corridor at a time

</description>

<acceptance_criteria>
- [x] `src/config_v2/parser/private_schema.rs` is the only TOML-deserializable config schema module for this task
- [x] Nothing from `src/config_v2/parser/private_schema.rs` is exported outside parser-internal load paths
- [x] `src/config_v2/parser/load_config.rs` contains parsing logic that maps private schema into unchanged `RuntimeConfigV2`
- [x] `src/config_v2/parser/load_operator_config.rs` contains the operator parsing logic for this step without exporting parser-only schema types
- [x] `src/config_v2/types.rs` changes, if any, are limited to the minimum `OperatorConfigV2` top-level surface correction needed for the operator root handoff
- [x] `src/cli/config.rs` consumes `OperatorConfigV2` directly without relying on old `src/config` source-world materialization helpers for config-v2 inputs
- [x] `src/runtime/node.rs` is migrated to use only `RuntimeConfigV2` and pass that root onward
- [x] Remaining compile-check failure after the handoff is verified to come only from old/to-be-deleted `src/config/` usage in this story
- [x] When this task is done, `.ralph/current_task.txt` is updated to `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md`
- [x] This task clearly instructs the next work to continue in task 02 and not here
</acceptance_criteria>

<boundary_review>
Smell applied from `improve-code-boundaries`: wrong config-ingestion boundary.

Verified repository facts before execution:
- `src/config_v2/parser/private_schema.rs` is empty.
- `src/config_v2/parser/load_config.rs` and `src/config_v2/parser/load_operator_config.rs` are stubs that only return `Not implemented`.
- `src/runtime/node.rs` still roots runtime startup in `crate::config::{load_runtime_config, RuntimeConfig}`.
- `src/config_v2/mod.rs` only re-exports `load_runtime_config`, while `src/cli/config.rs` already imports `config_v2::{load_operator_config, OperatorConfigV2}`.
- `cargo check` currently fails before any meaningful handoff because the `config_v2` public surface is incomplete, `runtime/node.rs` still depends on the old config root, and `src/cli/config.rs` is still written against the old source-world operator schema assumptions.

Verified type-reuse constraints for this task:
- `src/config_v2/types.rs` is authoritative; only the minimum `OperatorConfigV2` top-level surface correction is allowed in this task.
- No new public config DTOs or adapter structs are allowed.
- All TOML-deserializable shapes must live only in `src/config_v2/parser/private_schema.rs`.
- The parser loaders must map directly into the existing `RuntimeConfigV2` / `OperatorConfigV2` types.

Verified design correction needed before execution:
- The existing `OperatorConfigV2` surface in `src/config_v2/types.rs` does not match the validated root that `src/cli/config.rs` needs to consume.
- Concrete mismatch examples:
  - `cli/config.rs` treats `api_base_url` as conditionally present, but `OperatorConfigV2` currently stores it as an unconditional `String`.
  - `cli/config.rs` expects `api_resolve_to`, but `OperatorConfigV2` does not contain that field.
  - `cli/config.rs` expects one validated top-level client TLS view and one validated auth view, but `OperatorConfigV2` is currently split into `api_client_tls`, `api_client_auth`, and `postgres_connection_override`.
  - the CLI tests still exercise inline/path materialization through old `crate::config` helpers even though config-v2 is supposed to stop leaking those source-world ingestion types.
- The correct fix is not to preserve the old source-world operator schema in public types. The correct fix is to allow the minimum `OperatorConfigV2` top-level surface correction and then update `src/cli/config.rs` to consume that validated root directly.

Execution result:
- `cargo check` now reaches the intended root-handoff boundary.
- The remaining failures are only downstream old-config corridor callsites from `src/runtime/node.rs` into modules still typed on `crate::config::RuntimeConfig` / `StateSubscriber<RuntimeConfig>`:
  - `src/logging/core/runtime.rs`
  - `src/process/state.rs`
  - `src/pginfo/startup.rs`
  - `src/dcs/mod.rs`
  - `src/process/startup.rs`
  - `src/api/startup.rs`
  - `src/logging/postgres_ingest.rs`
- No non-corridor compile errors remain in the task-01 parser/operator/runtime-node scope.
</boundary_review>

<plan>
1. Reuse the existing config-v2 ADTs where they already represent validated internal state; do not create new public parser/output shapes.
2. Correct `OperatorConfigV2` in `src/config_v2/types.rs` only as far as needed to expose one flat validated operator root through top-level fields for `src/cli/config.rs`.
3. Implement the private TOML document in `src/config_v2/parser/private_schema.rs` as the single serde/TOML boundary for both runtime and operator config loading.
4. Keep all file I/O, TOML parsing, mapping, and validation in:
   - `src/config_v2/parser/load_config.rs`
   - `src/config_v2/parser/load_operator_config.rs`
5. Add only the minimal adjacent plumbing needed to expose the config-v2 loader entry points without leaking anything from `private_schema.rs`.
6. Update `src/cli/config.rs` to consume `OperatorConfigV2` directly, removing source-world config materialization helpers from the config-v2 path and updating the CLI tests to the validated path-only/operator-root contract.
7. Switch `src/runtime/node.rs` from `crate::config::{load_runtime_config, RuntimeConfig, ConfigError}` to the unchanged config-v2 equivalents and pass the `RuntimeConfigV2` root downward without starting the broader corridor collapse.
8. Run the compile gate after that runtime/operator root handoff and inspect the first remaining failures.
9. If the remaining failures are only downstream packages still typed on the old `src/config::RuntimeConfig` corridor, stop immediately and hand off to task 02.
</plan>

NOW EXECUTE
