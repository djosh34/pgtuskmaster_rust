## Plan: Collapse HA Given Fixture Ownership

### Why this reduction target

The config-parser slice is no longer the best next deletion seam. The larger wrong-place accumulation is now in the HA harness:

- `tests/ha/support/givens/mod.rs` expresses fixture topology through seven parallel `match` trees on the same three `HaGivenId` variants: `local_dcs_service_for`, `replicator_role`, `rewinder_role`, `dcs_services`, `quorum_majority_dcs_services`, `compose_variant_relative_path`, and `shared_fixture_relative_paths`.
- `tests/ha/support/world/mod.rs` then reassembles that same fixture knowledge inside `materialize_given_fixture`, `materialize_runtime_config`, `materialize_compose_include_file`, and `compose_variant_absolute_path`, even though those functions are not harness-runtime behavior and only exist to materialize a chosen given.
- The result is wrong-place fixture/config ownership plus repeated per-given branching spread across two owners, while `world/mod.rs` remains one of the largest files in the repo at 1705 lines.
- Callers only need “materialize this given” and “what DCS layout does this given use”; they do not benefit from parallel per-variant match functions plus a second world-owned rendering path.

That is wrong-placeism plus config-not-reduced: fixture description should live with the given owner, and the harness should consume that reduced description instead of reconstructing it.

### Current overlap already verified

- `HaGivenId::{local_dcs_service_for, replicator_role, rewinder_role, dcs_services, quorum_majority_dcs_services, compose_variant_relative_path, shared_fixture_relative_paths}` are all variant tables over the same three givens in `tests/ha/support/givens/mod.rs`.
- `tests/ha/support/world/mod.rs` lines in the materialization block consume those tables piecemeal to derive runtime config contents and compose include contents, even though that logic is static fixture knowledge, not runtime orchestration.
- The materialization path already depends only on `HaGivenId`, `ClusterMember`, and the parser’s `render_ha_member_runtime_config_toml(...)`; it does not need `HarnessShared` state and can be owned by the given layer.

### Execution plan

1. Introduce one shared descriptor for HA givens.
   - Add a single `HaGivenDefinition`-style owner in `tests/ha/support/givens/mod.rs` that holds the per-given static data now spread across parallel `match` functions.
   - Reuse existing enums and constants; do not create a second “builder” layer or new fixture DTO hierarchy.

2. Move fixture-specific rendering/lookup to the given owner.
   - Collapse the per-given branching in `givens/mod.rs` onto the shared descriptor.
   - Move compose-variant path resolution and HA runtime-config rendering helpers out of `tests/ha/support/world/mod.rs` and into the given owner, so `world/mod.rs` just performs file IO/copying/materialization orchestration.
   - Keep the external surfaces used by the harness equivalent: callers should still be able to ask a given for DCS layout info and rendered runtime config/compose inputs.

3. Delete world-owned fixture knowledge and narrow the harness.
   - Rewrite `materialize_given_fixture(...)` in `tests/ha/support/world/mod.rs` to consume the reduced given descriptor/renderers instead of re-deriving fixture details itself.
   - Remove the now-dead parallel `HaGivenId` match helpers and world-local compose/runtime materialization helpers.
   - Update or relocate the corresponding tests so they assert the reduced owner, not the deleted split boundary.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+3926 -6304 diff: -2378` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Keep `world/mod.rs` focused on harness lifecycle, docker control, fault injection, and invariant orchestration; do not replace deleted helpers with a second fixture wrapper module.
- Reuse `HaGivenId`, `ClusterMember`, and existing constants directly; do not introduce a new enum set or duplicate topology types.
- If moving fixture ownership into `givens/mod.rs` starts adding more glue types or pass-through helpers than it deletes, switch this plan back to `TO BE VERIFIED`, document the mismatch, retarget the task, and stop immediately.

NOW EXECUTE
