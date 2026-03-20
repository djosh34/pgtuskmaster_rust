## Task: Create Private TOML Schema And Initial `RuntimeConfigV2` Root Handoff <status>not_started</status> <passes>false</passes>

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
- do not alter `src/config_v2/types.rs`
- do not start the broader “make all callers use config_v2 directly” refactor in this task; that belongs to task 02
- migrate `src/runtime/node.rs` to use only `RuntimeConfigV2` and pass that root downward
- once the compile check fails only because old/to-be-deleted `src/config/` users still exist in this story, stop
- after creating this task, point `.ralph/current_task.txt` at `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md` and quit immediately

**Scope:**
- `src/config_v2/parser/private_schema.rs`
- `src/config_v2/parser/load_config.rs`
- `src/config_v2/parser/load_operator_config.rs`
- `src/runtime/node.rs`
- only the minimum adjacent parser/runtime plumbing required to complete the root handoff

**Implementation direction:**
- in `src/config_v2/parser/private_schema.rs`, define the serde/TOML-only input shape for `config.toml`
- keep that module private to parser internals; do not re-export any of its types, helpers, or fields
- in `src/config_v2/parser/load_config.rs`, parse TOML through the private schema and map directly into the existing `RuntimeConfigV2`
- in `src/config_v2/parser/load_operator_config.rs`, keep parsing logic in the loader and reuse parser-private schema as needed without exporting parser-only types
- in `src/runtime/node.rs`, replace old runtime-config root usage with `RuntimeConfigV2` only and pass that reference further down
- accept temporary compile fallout caused by downstream packages that still expect the old config tree, but verify the remaining failures are specifically in the old `src/config/` story corridor
- do not “fix forward” by creating adapter structs, duplicate config trees, or new intermediary config DTOs

**Explicit non-goals:**
- do not alter `src/config_v2/types.rs`
- do not create new public enums or structs to bridge old and new config
- do not migrate all downstream modules to read `RuntimeConfigV2` directly in this task
- do not continue once the root handoff is complete and remaining compile errors are isolated to old-config consumers

**Expected outcome:**
- parser-private TOML schema exists only in `src/config_v2/parser/private_schema.rs`
- TOML parsing happens only via the config-v2 loader functions
- `RuntimeConfigV2` stays unchanged
- `src/runtime/node.rs` uses `RuntimeConfigV2` as its only runtime config root
- the next engineer can continue from task 02 to collapse one config corridor at a time

</description>

<acceptance_criteria>
- [ ] `src/config_v2/parser/private_schema.rs` is the only TOML-deserializable config schema module for this task
- [ ] Nothing from `src/config_v2/parser/private_schema.rs` is exported outside parser-internal load paths
- [ ] `src/config_v2/parser/load_config.rs` contains parsing logic that maps private schema into unchanged `RuntimeConfigV2`
- [ ] `src/config_v2/parser/load_operator_config.rs` contains the operator parsing logic for this step without exporting parser-only schema types
- [ ] `src/config_v2/types.rs` is explicitly treated as out of scope and remains unaltered
- [ ] `src/runtime/node.rs` is migrated to use only `RuntimeConfigV2` and pass that root onward
- [ ] Remaining compile-check failure after the handoff is verified to come only from old/to-be-deleted `src/config/` usage in this story
- [ ] When this task is done, `.ralph/current_task.txt` is updated to `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md`
- [ ] This task clearly instructs the next work to continue in task 02 and not here
</acceptance_criteria>
