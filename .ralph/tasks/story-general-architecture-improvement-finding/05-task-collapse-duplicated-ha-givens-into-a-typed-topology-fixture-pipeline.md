## Task: Collapse Duplicated HA Givens Into A Typed Topology Fixture Pipeline <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Refactor the HA given-fixture architecture so topology variants are represented by typed fixture data or a small generation/materialization pipeline instead of full copied directory trees. The higher-order goal is to remove duplicated configs, duplicated secrets/TLS assets, and directory-copy boilerplate that currently make HA scenario topology changes expensive, error-prone, and difficult to reason about. This task should aggressively reduce duplication in `tests/ha/givens/` while preserving real end-to-end fidelity.

**Original general architectural request that this task must preserve:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"

**Problem statement from current research:**
- HA given selection is currently directory-based. `tests/ha/support/givens/mod.rs` resolves a given name directly to a directory under `tests/ha/givens/`, and `tests/ha/support/world/mod.rs` copies that entire directory tree into a per-run working directory before the harness starts.
- That means topology variation is currently represented by copying whole fixture trees rather than by expressing only the differences.
- The strongest concrete example is `tests/ha/givens/three_node_plain/` versus `tests/ha/givens/three_node_custom_roles/`. Each directory currently contains 23 files, but the diff is tiny relative to the copied surface:
  - one `compose.yml` difference
  - three node runtime config differences
  - three observer config differences
  - the rest of the files are duplicated static assets such as secrets, TLS material references, `pg_hba.conf`, and `pg_ident.conf`
- This means a small topology/role variation currently requires a full copied directory and duplicated file maintenance.
- The cost of that duplication is already visible in other backlog work: config-related tasks and role-related tasks both need to touch HA given configs, and copied fixture trees multiply that work.
- The current harness contract keeps this duplication alive because it accepts only a directory name and blindly copies a directory tree. That is a structural smell, not just a fixture tidy-up.

**Concrete repo evidence from research:**
- `tests/ha/support/givens/mod.rs`
  - `given_root(...)` resolves a given only by directory existence under `tests/ha/givens/`
- `tests/ha/support/world/mod.rs`
  - `HarnessShared::initialize(...)` copies the entire selected given directory into the run directory with `copy_directory(...)`
- `tests/ha/givens/three_node_plain/`
  - current baseline three-node topology fixture
- `tests/ha/givens/three_node_custom_roles/`
  - near-duplicate copy of the baseline fixture with mostly the same tree and only a small number of meaningful differences
- `diff -rq tests/ha/givens/three_node_plain tests/ha/givens/three_node_custom_roles`
  - shows only the compose file plus six runtime/observer config files differ, while each given directory still contains 23 files
- Existing HA backlog such as `.ralph/tasks/story-ctl-operator-experience/09-task-add-a-three-etcd-ha-given-and-design-real-dcs-majority-features.md`
  - already points toward adding another full given directory, which increases the risk of more copied topology trees if the underlying fixture architecture stays the same

**Required architectural direction:**
- The harness should stop requiring topology variants to be expressed primarily as whole copied directories.
- Shared fixture assets should live once, with typed overlays/templates/materialization for the parts that actually vary.
- Small differences such as role names, endpoint layouts, or compose capability toggles should be represented as data or narrowly scoped overrides, not wholesale copied trees.
- The final design must preserve real compose/runtime-config/TLS/secrets fidelity. This is not a request to replace black-box topology fixtures with mocks.

**Important non-goals for this task:**
- Do not weaken the HA suite into unit tests or fake topology simulation.
- Do not preserve duplicate copied trees just because they already exist.
- Do not solve this by inventing a huge generic templating system that is more complex than the duplicated fixtures themselves. Prefer the smallest typed generation/materialization model that removes the current duplication.

**Scope:**
- Refactor the HA given loading/materialization path under `tests/ha/support/` so the harness can build or materialize topology variants without requiring full copied source trees for each near-duplicate variant.
- Collapse duplicated fixture structure between the current three-node givens while preserving behavior.
- Update any existing givens, support code, and docs/comments needed so topology variants are represented by a smaller, more maintainable architecture.
- Keep the resulting fixture system compatible with real HA test execution and future topology variants such as multi-etcd givens.

**Expected outcome:**
- HA topology variants are represented by a smaller typed fixture/materialization pipeline instead of copied directory trees.
- Shared assets and shared config structure live once rather than being copied into each near-duplicate given.
- Adding new topology variants or role variants requires expressing real differences only.
- The HA harness becomes easier to maintain because fixture ownership and variation points are explicit.

</description>

<acceptance_criteria>
- [x] Refactor `tests/ha/support/givens/mod.rs` and related harness loading code so given selection is no longer limited to blindly resolving and copying a near-duplicate directory tree for every variant.
- [x] Refactor `tests/ha/support/world/mod.rs` and any related materialization/copy logic so the harness can materialize or assemble topology fixtures from shared assets plus explicit variation points.
- [x] Collapse duplicated fixture structure between `tests/ha/givens/three_node_plain/` and `tests/ha/givens/three_node_custom_roles/`, keeping only the meaningful differences as data/overrides/materialized output rather than full copied trees.
- [x] Preserve real HA fixture fidelity, including compose startup, runtime config generation/materialization, TLS material references, and secrets wiring.
- [x] Ensure the resulting fixture architecture is compatible with additional topology variants such as the multi-etcd given work, without requiring another large copied tree for small variations.
- [x] Update any focused tests or harness assertions needed to prove the new fixture/materialization path is correct.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Finish the typed HA given boundary so scenario selection parses into `HaGivenId`, resolves into a `HaGivenDefinition`, and stops leaking raw directory names/paths through the harness API.
2. Implement the new fixture materialization pipeline around explicit ADTs (`HaTopologyFixture`, `FixtureMaterialization`, `SharedFixtureEntry`, `RenderedFixtureFile`) so the harness assembles a run workspace from shared assets plus rendered variation points instead of copying a full given tree.
3. Rework `HarnessShared` around typed workspace/compose state (`HarnessWorkspace`, `WorkspacePaths`, `ComposeStack`) and update the world/support helpers to consume that smaller boundary rather than separate loose path fields.
4. Materialize the shared three-node fixture layout once, then render only the meaningful differences for the current variants:
   - compose observer capability (`NET_ADMIN` enabled vs disabled)
   - postgres role naming (`replicator` / `rewinder` vs `mirrorbot` / `rewindbot`)
   - member runtime and observer config outputs per node
5. Remove the duplicated `three_node_plain` / `three_node_custom_roles` fixture trees in favor of the shared asset layout plus the typed render/materialization pipeline, while preserving real TLS/secrets/compose fidelity and compatibility with future topology variants.
6. Add or update focused harness tests around given resolution/materialization so future topology additions prove differences via typed data rather than new copied trees.
7. After the design is wired end-to-end, run the required validation gates in repo-preferred order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
8. Only after all checks pass, update docs for the new fixture/materialization flow using the `k2-docs-loop` skill, remove stale HA fixture docs if needed, then complete task closeout (`<passes>true</passes>`, task switch, commit, push).

### Constraints for execution
- Keep the fixture boundary typed. Do not fall back to a raw `given_name -> directory copy` API just to make the implementation easier.
- Preserve real black-box HA fidelity: shared TLS assets, secrets wiring, runtime configs, and compose startup still need to be materialized into a real runnable workspace.
- Prefer a narrow generation model for the known variation points over a generic text-templating system.
- If execution shows the current ADTs are still wrong, switch this task back to `TO BE VERIFIED`, explain the type/design gap in the task file, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
