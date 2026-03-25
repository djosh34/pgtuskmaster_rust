## Task: Delete Observer Compose Rendering And Collapse HA Fixtures To Two Static Compose Variants <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Remove the current observer-container-oriented compose/rendering machinery and collapse the HA fixture materialization model to two on-disk Docker Compose variants plus a tiny include-based per-run compose file. The higher-order goal is to stop rendering large compose files and stop treating the observer container as a first-class fixture concept. The PO direction is explicit: all docker composes are effectively the same with only two variants, so the harness should stop templating them and should stop rendering observer-specific config files.

**Context from research:**
- `tests/ha/support/world/mod.rs` currently materializes a per-run `compose.yml` by rendering `ComposeTemplate` through `render_fixture_template(...)` and `render_compose_template(...)`.
- `HarnessShared::initialize(...)` currently brings up DCS services and then brings up an `observer` service, and stores `observer_container` on the harness.
- `tests/ha/support/givens/mod.rs` currently models rendered compose/runtime/observer outputs with:
  - `FixtureRenderTarget`
  - `FixtureTemplate`
  - `ComposeTemplate`
  - `ObserverTemplate`
  - `RenderedFixtureFile`
- The PO explicitly asked for:
  - no observer container at all
  - only two docker compose variants on disk
  - include syntax from a tiny otherwise-empty compose file
- If this story conflicts with `.ralph/tasks/story-general-architecture-improvement-finding/06-task-move-ha-scenario-execution-into-a-per-scenario-runner-container-and-remove-docker-daemon-polling.md`, follow this story. The older task moved more logic into a runner container, but the new PO direction is the opposite: remove observer containers entirely and drive `pgtm` from outside the containers.

**Scope:**
- Rewrite the fixture/given materialization path under `tests/ha/support/givens/mod.rs`.
- Rewrite the compose-materialization logic in `tests/ha/support/world/mod.rs`.
- Rewrite `tests/ha/support/topology.rs` so observer-specific service/config path concepts disappear from the topology model.
- Replace rendered compose templates with two real compose variant files checked into the repo.
- Delete observer-specific compose/config render targets and any dead fixture metadata that only existed to support the observer container.

**Required architectural direction:**
- Keep only two compose variants on disk. The likely split is:
  - shared single DCS service
  - colocated three-member DCS layout
- Do not create a third compose variant just for custom roles. Custom role differences belong in runtime config data, not in a separate compose topology.
- The per-run compose file should exist only to include the chosen on-disk compose variant, not to restate the whole topology.

**Expected outcome:**
- The HA fixture system no longer renders large compose YAML blobs per run.
- The observer service disappears from the fixture model.
- Compose ownership becomes obvious because the repo contains only the actual variant files in source control.

</description>

<acceptance_criteria>
- [x] Delete the `observer` compose service from the HA fixture model and from harness startup/teardown logic.
- [x] Replace rendered compose generation with exactly two checked-in compose variant files plus a tiny include-only per-run compose file.
- [x] Remove observer-specific render targets and templates from `tests/ha/support/givens/mod.rs`; do not preserve `ObserverTemplate`-style indirection.
- [x] Remove observer-specific topology concepts from `tests/ha/support/topology.rs`, including service/config-path data that only existed for the observer container.
- [x] Remove `render_compose_template(...)` and any equivalent large compose string-generation path from `tests/ha/support/world/mod.rs`.
- [x] Ensure custom role variants no longer require their own compose variant; only topology differences should decide which compose file is included.
- [x] Keep fixture materialization simple enough that a reader can see all compose variants directly on disk without mentally expanding a render function.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Delete observer-oriented compose/config render targets from the given/materialization types.
2. Check in the two actual compose variant files and rework fixture materialization to include one of them from a tiny per-run compose file.
3. Remove observer service startup and observer container tracking from the harness.
4. Keep only the runtime-config generation that still matters after the observer service is gone.
5. Run the required repo validation gates after the simplified fixture model is wired end to end.

### Constraints for execution
- Do not preserve rendered compose text generation "just in case".
- Do not reintroduce a hidden observer service under a different name.
- Keep the variant count at two unless the current repo proves that even fewer is correct.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
