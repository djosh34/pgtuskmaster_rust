## Bug: Remove Primary And Replica API Commands And Derive CLI Output From State <status>done</status> <passes>true</passes>

<description>
`pgtm primary` / `pgtm replicas` can disagree with the cluster `state` view during HA scenarios. This was detected while restoring the older HA harness: ultra-long HA tests exposed cases where the authoritative state view showed one primary expectation while the primary-resolution command returned something different or no authoritative primary at all.

Do not paper over this inside the harness. Explore and research the codebase first, then fix the product/API design so this mismatch becomes impossible. Remove the separate primary/replica command path from the full API surface. Instead, make the CLI derive primary/replica output from `/state` plus its own config using the same exact input data as the state response, so `state`, `primary`, and `replicas` cannot diverge.

The explicit target flow is:

- `/state`, `/primary`, and `/replica(s)` all route into the same server-side state-based derivation path.
- API code that needs primary/replica information first gets the latest state through the same function / same data path used by `/state`.
- Primary/replica projections are produced from that state on the server, never reconstructed independently on the client.
- If conversion is needed, it happens after that shared state-based derivation, not through a separate source of truth.
- `pgtm` adds TLS / SSL connection material from its own local config.toml only, not from whatever the server returned.
- The CLI and API converge on one command-domain output model: `CommandOutputDto` as the enum of possible command outputs before choosing text or JSON rendering.
- Rendering then becomes format-only:
  `CommandOutput` / `CommandOutputDto` -> `Display` / `to_string()`
  or
  `CommandOutput` / `CommandOutputDto` -> serde JSON
- The same command uses the same output struct regardless of whether the caller asked for JSON or text.

The investigation should identify every place that currently computes or exposes primary/replica results independently of `/state`, remove that split-brain path, and update the HA harness/tests so they validate the unified behavior rather than hiding inconsistencies with fallback routing.
</description>

<acceptance_criteria>
- [x] Explore the current `state`, `primary`, and `replicas` implementation paths first and document where they can diverge.
- [x] The full API no longer has a separate primary/replica computation path that can disagree with `/state`.
- [x] `/state`, `/primary`, and `/replica(s)` go through the same server-side state acquisition / derivation path.
- [x] Primary/replica outputs are derived from the latest state on the server, not from a second independent computation path.
- [x] `pgtm` uses its own config.toml only for local TLS / SSL connection material when rendering connection outputs, not as a second state source.
- [x] The CLI/API command layer has one shared `CommandOutputDto`-style enum per command result family, reused for both text and JSON output.
- [x] Text output comes from `Display` / `to_string()` on the same command output model that serde uses for JSON output.
- [x] HA harness helpers fail loudly on state/primary divergence instead of masking it with direct-route fallback logic.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Research summary
- The current `state` path is `src/api/worker.rs:get_state -> src/api/controller.rs:build_node_state`, and that is the only server route today. The CLI `primary` / `replicas` commands do not share a command-domain ADT with it; they reconstruct their own connection view in `src/cli/connect.rs`.
- `src/cli/connect.rs:resolve_primary_view` derives the primary from `authority_primary_member(state)` and then renders a dedicated `ConnectionView`, while `src/cli/status.rs` separately renders `/state` as either raw `NodeState` JSON or `ClusterStatusView` text. These are two projections over the same source, but they are modeled and rendered independently.
- The strongest built-in divergence is `src/cli/connect.rs:build_primary_connection_target`, which can override the primary host/port from `pgtm.primary_target`. That lets `pgtm primary` produce a connection target that is not the target described by cluster state, while `replicas` still use DCS member metadata directly.
- The connection helper model itself is split-brain: `ConnectionTarget` mixes server-derived member identity with CLI-local DSN rendering and TLS path material, so the authoritative state-derived target and the local rendering concern are not represented as separate types.
- The HA harness still masks disagreement in multiple places. `tests/ha/support/steps/mod.rs:sql_target_for_member` falls back from pgtm-derived routing to `direct_connection_target`, and `tests/ha/support/workload/mod.rs:resolve_primary_target` collapses the primary contract to a raw `(member_id, dsn)` tuple. Both paths can hide the exact state-derived route that was validated.
- `tests/ha/support/observer/pgtm.rs` currently parses `pgtm primary --json` / `pgtm replicas --json` into the dedicated connection-view DTO instead of a shared command-domain enum, so the harness reinforces the current split rather than exercising the final unified model.

### Type design completed in this pass
- Added a new shared command-domain module at `src/command/mod.rs` with:
  - `CommandOutputDto` as the enum for command-family results before text/JSON rendering
  - `StateCommandOutputDto` and `StateProjectionDto` for state-rooted command metadata
  - `StateDerivedConnectionCommandDto` and `StateDerivedConnectionTargetDto` for primary/replica projections derived from state only
  - `LocalConnectionMaterialization` and `PathBackedClientTlsDto` so local TLS material is modeled separately from state-derived topology
- Removed the config type that let CLI primary resolution replace state-derived topology: `PgtmPrimaryTargetConfig` and the `pgtm.primary_target` field were removed from the configuration schema/export surface.
- Re-shaped `src/cli/connect.rs` around the new ADTs so primary/replica command construction now targets `CommandOutputDto` and state-derived target DTOs instead of the old DSN-bearing `ConnectionView` / `ConnectionTarget` pair.
- Re-shaped the harness-facing target type in `tests/ha/support/world/mod.rs` from a DSN-bearing connection target to `StateDerivedConnectionTargetDto`, so later execution must thread explicit local rendering/writability instead of carrying an already-rendered DSN as if it were authoritative state.
- Re-shaped `tests/ha/support/observer/pgtm.rs` so the observer now parses `primary` / `replicas` output as the shared `CommandOutputDto` instead of the dedicated connection-view DTO.

### Execution plan
1. Finish the command-domain migration so `src/cli/output.rs`, `src/cli/status.rs`, and the CLI call sites all render from `CommandOutputDto` and no longer keep a separate connection-helper DTO tree.
2. Add one pure state-derivation path that starts from `NodeState` / the same snapshot used by `/state` and produces:
   - state projection metadata
   - primary projection
   - replica projection
   That path must be the only source of truth for primary/replica command results.
3. Remove all remaining host/port override and independent reconstruction paths so the CLI can only add local TLS / SSL connection material after state-derived target selection is complete.
4. Thread the new ADTs through the HA harness:
   - replace `CommandOutputDto` pattern matches where the harness still expects `targets`
   - eliminate direct-route fallback in `sql_target_for_member`
   - replace raw `(member_id, dsn)` workload resolution with a typed writable-primary contract
5. Add or update focused tests around:
   - shared state-derived command projection
   - local TLS materialization rules
   - primary/replica/state convergence
   - harness failure behavior when state and primary output disagree
6. If execution proves the ADT boundary is still wrong, switch this task back to `TO BE VERIFIED`, explain the design gap precisely, and stop immediately.
7. After the design still proves correct, run the required validation gates in order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
8. Only after all checks pass, update docs using `k2-docs-loop`, then set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

### Constraints for execution
- Do not reintroduce config-backed primary host/port overrides; local config is only for local TLS / SSL connection material after state-derived target selection.
- Do not keep separate DTO trees for `state` versus `primary` / `replicas`; the command layer must converge on the shared `CommandOutputDto` family.
- Do not restore harness fallback routing that can silently hide state/primary disagreement.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

### Execution result
- `make check`
- `make lint`
- `make test`
- `make test-long`
- `make docs-lint`
