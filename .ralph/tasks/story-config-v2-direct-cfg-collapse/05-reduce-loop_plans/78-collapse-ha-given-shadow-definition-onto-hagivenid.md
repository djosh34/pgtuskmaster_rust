## Plan: Collapse HA Given Shadow Definition Onto `HaGivenId`

### Why this is the next verified reduction target

Plan `77-collapse-api-cert-reload-postmaster-matrix-onto-postmaster-owner.md` is stale in the current tree: `src/api/worker.rs` now keeps only one API-level postmaster failure regression, while the detailed missing-pid / stale-pid / data-dir-mismatch matrix already lives in `src/process/postmaster.rs`.

The next live boundary problem is in the HA cucumber given owner:

- `tests/ha/support/givens/mod.rs` exposes `HaGivenId` as the public API, but the file still stores the real data in a second `HaGivenDefinition` layer plus three static definition constants.
- That shadow layer exists only so `HaGivenId` can bounce through `definition()` and a function-pointer field before returning simple enum-owned facts.
- The same file also splits runtime-config ownership across `render_runtime_config`, `validate_runtime_config_for_host`, and `render_runtime_config_with_process_binaries`, even though only `HaGivenId` knows which roles, DCS routes, and compose variant belong to a given.

### Current overlap verified in code

- `tests/ha/support/givens/mod.rs:49-89` declares:
  - `PLAIN_GIVEN`
  - `CUSTOM_ROLES_GIVEN`
  - `THREE_ETCD_GIVEN`
  - `HaGivenDefinition`
- `tests/ha/support/givens/mod.rs:98-158` shows `HaGivenId::definition()` plus multiple trivial accessors that only forward into that shadow struct:
  - `local_dcs_service_for(...)`
  - `as_str()`
  - `replicator_role()`
  - `rewinder_role()`
  - `dcs_services()`
  - `quorum_majority_dcs_services()`
  - `compose_variant_absolute_path(...)`
- `tests/ha/support/givens/mod.rs:182-239` shows a second helper layer for config materialization:
  - `render_runtime_config(...)`
  - `render_runtime_config_with_process_binaries(...)`
  - `validate_runtime_config_for_host(...)`
- `tests/ha/support/givens/mod.rs:243-249` adds two more indirection helpers only because the shadow struct stores function pointers:
  - `shared_dcs_service_for(...)`
  - `member_local_dcs_service_for(...)`
- External callers in `tests/ha/support/world/mod.rs` and `tests/ha/support/steps/mod.rs` already depend on `HaGivenId` methods directly, not on `HaGivenDefinition`, so the enum is the real owner.

### Execution plan

1. Collapse the shadow metadata layer onto `HaGivenId`.
   - Delete `HaGivenDefinition` and the `PLAIN_GIVEN` / `CUSTOM_ROLES_GIVEN` / `THREE_ETCD_GIVEN` constants.
   - Replace `definition()` and the function-pointer indirection with direct `match self` ownership on `HaGivenId`.
   - Inline the local-DCS routing behavior into `HaGivenId::local_dcs_service_for(...)` so `shared_dcs_service_for(...)` and `member_local_dcs_service_for(...)` can be removed.

2. Collapse given-owned config rendering onto one owner.
   - Keep one private renderer on `HaGivenId` that accepts the process-binary override array and derives the DCS endpoint / role names directly from the enum.
   - Make `render_runtime_config(...)` and host validation reuse that single owner without an extra pretend boundary.
   - Preserve the current host-side validation before returning the container-rendered config, since that is the behavior the file actually owns today.

3. Keep the public surface stable for world/step code.
   - Preserve the existing `HaGivenId` methods used from `tests/ha/support/world/mod.rs` and `tests/ha/support/steps/mod.rs`.
   - Do not introduce a new metadata DTO to replace the deleted one.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not weaken the fixture semantics for shared-etcd versus three-etcd routing.
- Do not skip the existing host validation step before returning rendered member configs.
- Do not add a new enum/struct wrapper around HA-given metadata; the reduction is specifically about deleting the shadow layer.
- Keep the current public `HaGivenId` call sites working without forcing `world` or `steps` to understand a new internal shape.

### Expected yield

- Delete the shadow `HaGivenDefinition` layer and the function-pointer helpers that only support it.
- Keep HA-given facts owned exactly once, on `HaGivenId`.
- Leave the runtime-config rendering logic in the same file, but with one actual owner and fewer wrapper hops.
- Produce a self-contained net-negative slice in `tests/ha/support/givens/mod.rs`.

NOW EXECUTE
