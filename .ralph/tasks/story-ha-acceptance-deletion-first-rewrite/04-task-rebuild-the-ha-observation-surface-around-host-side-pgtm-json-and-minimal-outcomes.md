## Task: Rebuild The HA Observation Surface Around Host-Side `pgtm` JSON And Minimal Outcomes <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Replace the current observer-container exec model with a host-side `pgtm` observation surface that queries each node individually, parses JSON directly, and exposes only the minimal outcome checks the PO asked for. The higher-order goal is to make the harness reason from simple direct observations instead of from observer-container indirection, seed selection heuristics, and layered test-only DTOs.

**Context from research:**
- `tests/ha/support/observer/pgtm.rs` currently shells into an observer container, fans out across configs, chooses a "best seed", and parses `CommandOutputDto` through helper types like `SelectedSeed`.
- `tests/ha/support/observer/sql.rs` currently shells into `/usr/lib/postgresql/16/bin/psql` inside that observer container.
- The PO explicitly asked for:
  - no observer container
  - `pgtm` from outside the container to contact all 3 nodes individually
  - JSON output only
  - parse directly via serde
  - at most one serde DTO per command
  - a result shape where each node observation is `Option<dto>` only when the command failed
- The minimal scenario outcomes are:
  - `Then cluster becomes healthy`
  - `Then cluster becomes unhealthy`
  - the switchover exception `Then node-a becomes primary`

**Scope:**
- Rewrite `tests/ha/support/observer/pgtm.rs`, `tests/ha/support/observer/sql.rs`, and any adjacent support modules that currently assume an observer container.
- Reuse the host process execution layer in `tests/ha/support/process/mod.rs` instead of inventing a second command-launch abstraction.
- Extend `tests/ha/support/config.rs` and `tests/ha/harness.toml` as needed so host-side `pgtm` and SQL execution are configured coherently.
- Remove seed-selection logic and replace it with explicit per-node observations.
- Rework `tests/ha/support/steps/mod.rs` so outcome steps use the new observation surface.
- Update `tests/ha/support/runner/mod.rs` or adjacent lifecycle code so the host runs the new observation commands directly.

**Required behavior:**
- For each relevant check, run `pgtm ... --json` against each node individually.
- Parse each result into `Option<Dto>` where `None` means the command failed for that node.
- Reuse existing production DTOs where possible; do not introduce a new test-only struct tree.
- Keep one serde DTO per command at most. Do not recreate the current nested helper layers.

**Minimal outcome definitions:**
- `cluster becomes healthy`
  - wait until the per-node self-reports are compatible with one primary
  - confirm that the healthy cluster also has a writable primary
- `cluster becomes unhealthy`
  - wait until the unhealthy condition is visible through the same direct observation surface
  - do not layer extra scenario-specific assertions on top
- `"<alias>" becomes primary`
  - this is the narrow switchover exception
  - it should reuse the same direct observation surface as `cluster becomes healthy`, but also require the requested member to be the reported primary

**Expected outcome:**
- The HA harness uses host-side `pgtm` JSON as a simple direct observation mechanism.
- The outcome step surface is tiny and aligned with the PO request.
- The old observer-container seed-selection logic is gone.

</description>

<acceptance_criteria>
- [x] Remove the observer-container exec path from `tests/ha/support/observer/pgtm.rs` and `tests/ha/support/observer/sql.rs`.
- [x] Rebuild the observation path so it queries each node individually from the host and parses JSON into `Option<dto>` results.
- [x] Keep serde DTO usage to at most one DTO per command; reuse existing production DTOs where they already express the JSON shape correctly.
- [x] Remove `SelectedSeed`, "best seed" selection, and any equivalent host-side heuristics that are only needed by the old observer model.
- [x] Implement only the reduced outcome step family on top of the new observation surface: `cluster becomes healthy`, `cluster becomes unhealthy`, and `"<alias>" becomes primary`.
- [x] Ensure these outcome steps use the same direct observation model that later invariant runners will use.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Delete the observer-container assumptions from the observation helpers.
2. Build a host-side `pgtm` command runner that queries each node individually and returns `Option<dto>` values.
3. Keep the SQL/writeability path equally direct and minimal so the healthy/unhealthy outcome checks do not depend on container exec indirection.
4. Rebuild the surviving outcome steps on top of that direct observation API.
5. Run the required repo validation gates after the reduced observation surface compiles and drives the reduced feature suite.

### Constraints for execution
- Do not preserve the old "best seed" abstraction.
- Do not add a new test-only DTO graph; reduce DTO count instead.
- Keep errors explicit. A failed per-node observation should surface as `None` or as a task-level error, not as hidden fallback behavior.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
