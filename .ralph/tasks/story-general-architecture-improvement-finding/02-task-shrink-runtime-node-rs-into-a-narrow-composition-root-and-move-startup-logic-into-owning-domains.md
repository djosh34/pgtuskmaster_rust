## Task: Shrink `runtime/node.rs` Into A Narrow Composition Root And Move Startup Logic Into Owning Domains <status>completed</status> <passes>true</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>

<description>
**Goal:** Refactor `src/runtime/node.rs` so it becomes a small runtime composition boundary instead of a giant mixed-responsibility startup file. The higher-order goal is to make runtime startup boring and readable: `node.rs` should orchestrate top-level startup/lifecycle only, while each domain owns its own startup preparation, internal defaults, config projection, and handle construction. This task should aggressively reduce how much domain knowledge `node.rs` must carry about DCS, HA, API, process, pginfo, logging, TLS, and startup state fabrication.

**Specific request that motivated this task:**
- `node.rs` currently has far too much logic inside it that belongs in other places
- the DCS worker should handle its own startup and only return the handle that other workers need to use
- far too much defaulting, default logic, config handling, and state copying is currently done in `node.rs`
- the desired direction is to scope the domain of `node.rs` much more tightly, make its interface far smaller, and move functionality into the domains it controls

**Original general architectural request that this task must preserve:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"

**Problem statement from current research:**
- `src/runtime/node.rs` is currently `1190` lines long while `src/runtime/` contains only `mod.rs` and `node.rs`. That file is acting as both composition root and architectural dumping ground.
- `run_node_from_config(...)` currently does all of the following:
  - validates runtime config
  - bootstraps logging
  - creates and emits startup log events
  - derives process defaults from config
  - creates required directories and permissions for startup paths
  - fabricates initial state values for pginfo, DCS, process, and HA
  - creates state channels and shares subscribers/publishers manually
  - constructs DCS, HA, API, process, pginfo, and logging-ingest worker contexts directly
  - opens multiple DCS store connections itself
  - binds the API listener itself
  - builds API TLS config itself
  - manually joins all worker futures
- That means `node.rs` currently knows a lot of domain-private details that should not be a runtime orchestrator concern.

**Concrete repo evidence from research:**
- `src/runtime/node.rs` imports DCS internals directly:
  - `crate::dcs::etcd_store::EtcdDcsStore`
  - `crate::dcs::state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx}`
- `src/runtime/node.rs` manually fabricates initial DCS cache/state, including empty maps and `last_emitted_*` bookkeeping values that look like worker internals rather than runtime composition concerns.
- `src/runtime/node.rs` manually constructs `DcsWorkerCtx` with:
  - `scope`
  - local routing values
  - pg subscriber
  - publisher
  - raw store
  - internal cache
  - member TTL
  - last emitted store/trust flags
- That is exactly the kind of startup ownership inversion this task should fix. The DCS domain should own DCS startup and return only the narrow handle/surface needed by other components.
- `src/runtime/node.rs` derives `ProcessDispatchDefaults` in `process_defaults_from_config(...)`, but the type itself lives in `src/ha/state.rs` and is also consumed by `src/ha/source_conn.rs`. That is a leaky ownership split: runtime currently knows how HA/process defaults should be projected from config.
- `src/runtime/node.rs` contains `advertised_postgres_port(...)`, `advertised_operator_api_url(...)`, and `local_postgres_conninfo(...)`, which are domain projection helpers embedded in runtime instead of living near the owning domain.
- `src/runtime/node.rs` contains `ensure_start_paths(...)`, which performs filesystem preparation for PostgreSQL/process-related runtime paths. That is startup domain logic mixed into the runtime composition layer.
- `src/runtime/node.rs` constructs `HaWorkerCtx` via `HaWorkerCtx::contract_stub(...)` and then mutates many fields afterward:
  - `poll_interval`
  - `now`
  - `process_defaults`
  - `log`
- That pattern strongly suggests the HA public construction API is wrong for production wiring. Runtime should not need to create a stub and then patch it into a real worker.
- `src/runtime/node.rs` binds the API listener and then constructs `ApiWorkerCtx`, sets subscribers, builds TLS config, configures TLS mode, and sets client-cert requirements. The API startup boundary is therefore also split awkwardly between runtime and API.
- `src/runtime/node.rs` still carries dead disabled startup-test code under `#[cfg(all(test, any()))]`. That test module references old startup/DCS shapes such as `LeaderRecord`, `MemberRecord`, and `MemberRole`, which indicates legacy startup logic and tests have been left in place as archaeological noise rather than removed.

**Required architectural direction:**
- `src/runtime/node.rs` should become a narrow composition root whose main responsibilities are limited to:
  - loading and validating top-level runtime config
  - top-level runtime bootstrapping that truly must be global
  - assembling domain-owned startup handles/components
  - starting the worker tasks / top-level lifecycle
  - mapping failures into `RuntimeError`
- `src/runtime/node.rs` should not remain the place where domain defaults are derived, internal worker state is fabricated, or domain-private startup policy is encoded.
- Startup/config/default logic should move toward the domain that owns it, even when that means adding domain-specific startup helpers/builders/constructors.
- Domain internals should become more private as a result. Runtime should depend on smaller public entry points and read-only/public handles rather than raw ctx structs with many fields.

**Important non-goal for this task:**
- Do not solve this by merely splitting `node.rs` into smaller runtime-local helper files while keeping the same ownership model.
- The point is not file-size cosmetics. The point is to move responsibility to the owning domains and shrink the runtime interface surface.

**Scope:**
- Refactor `src/runtime/node.rs` and any newly introduced runtime modules so runtime becomes a narrow composition layer.
- Refactor DCS startup so runtime no longer constructs raw DCS internals directly and the DCS domain owns its own startup/bootstrap details.
- Refactor HA startup so runtime no longer relies on `contract_stub(...)` plus post-construction mutation for the real worker wiring path.
- Refactor API startup/binding/TLS setup boundary if runtime currently owns API details that should live with the API domain.
- Refactor process / pginfo / postgres-startup-related helper logic currently embedded in runtime when it actually belongs in those domains.
- Relocate config projection/default derivation helpers such as process defaults, advertised routing values, local PG conninfo, and startup path preparation into the owning domain or a smaller domain-specific startup module.
- Remove dead legacy runtime startup code/tests that no longer match the current architecture.

**Context from research:**
- `src/runtime/mod.rs` currently just re-exports `run_node_from_config`, `run_node_from_config_path`, and `RuntimeError` from `node.rs`.
- `src/dcs/state.rs` currently exposes a large `DcsWorkerCtx` that requires runtime to supply internal cache/store/startup values. That is a concrete place to challenge the current boundary.
- `src/dcs/worker.rs` currently runs off a fully assembled `DcsWorkerCtx`; the user specifically wants the DCS worker/domain to handle its own startup and only return the handle other workers actually need.
- `src/ha/state.rs` currently keeps `ProcessDispatchDefaults` in HA but runtime owns `process_defaults_from_config(...)`. That split is a concrete example of config/default logic living in the wrong place.
- `src/api/worker.rs` currently exposes an `ApiWorkerCtx::new(...)` plus setter-style configuration for live state, TLS, and client-cert requirement, which pushes runtime into API-private setup detail.
- `src/runtime/node.rs` currently owns several small helper functions whose placement is suspicious:
  - `process_defaults_from_config(...)`
  - `advertised_postgres_port(...)`
  - `advertised_operator_api_url(...)`
  - `local_postgres_conninfo(...)`
  - `ensure_start_paths(...)`
- The dead disabled test block in `src/runtime/node.rs` is important repo evidence that legacy runtime-startup logic has already accumulated and should be deleted rather than carried forward.

**Required outcome properties, without dictating the exact solution:**
- `node.rs` becomes dramatically smaller and easier to read.
- Runtime no longer imports or constructs broad domain-internal startup structs unless those are truly the intentional public startup interface.
- DCS startup becomes domain-owned and exposes only the narrow startup result/handle the rest of the runtime actually needs.
- Config projection/defaulting/path-preparation logic moves out of runtime into the components that own those concerns.
- Runtime wiring no longer depends on "construct stub, then mutate many fields into real state" patterns for production startup.
- The startup/test surface becomes cleaner and less legacy-laden.
- The resulting code has less duplication, less state copying, fewer leaked internals, and clearer ownership boundaries.

**Out of scope:**
- Do not redesign unrelated runtime behavior or HA policy beyond what is needed to fix ownership and startup boundaries.
- Do not preserve dead startup compatibility layers merely to avoid touching call sites.
- Do not stop at partial extraction if `node.rs` still owns the same domain knowledge afterward.

**Expected outcome:**
- `src/runtime/node.rs` is reduced to a real composition root with a much smaller interface and much less direct domain knowledge.
- DCS, HA, API, process, pginfo, and related startup concerns own more of their own startup/config/default/path logic.
- Runtime depends on smaller public constructors/handles and fewer mutable setup steps.
- Dead legacy runtime startup helpers/tests are removed.
- The overall startup architecture is less spaghetti, more private, and easier to test and maintain.

</description>

<acceptance_criteria>
- [x] Refactor `src/runtime/node.rs` so it no longer acts as a mixed-responsibility dumping ground for domain startup logic, and reduce it to a narrow runtime composition/lifecycle boundary.
- [x] Audit every helper currently in `src/runtime/node.rs` and either keep only truly runtime-global orchestration concerns there or move the helper into the domain that owns it; this includes at minimum `process_defaults_from_config(...)`, `advertised_postgres_port(...)`, `advertised_operator_api_url(...)`, `local_postgres_conninfo(...)`, and `ensure_start_paths(...)`.
- [x] Refactor the DCS startup boundary so runtime no longer constructs raw DCS internals such as `EtcdDcsStore`, internal `DcsCache` values, or a broad `DcsWorkerCtx` assembled field-by-field from `node.rs`; DCS startup must become domain-owned and return only the narrow handle/surface runtime and other workers need.
- [x] Refactor the HA startup boundary so production wiring no longer depends on `HaWorkerCtx::contract_stub(...)` followed by manual field mutation in `src/runtime/node.rs`; replace that with a real production-oriented boundary owned by the HA domain.
- [x] Refactor the API startup boundary so runtime does not continue owning unnecessary API-private setup detail such as piecemeal TLS/client-cert/live-state wiring if those concerns can be moved behind a smaller API-owned startup interface.
- [x] Re-evaluate ownership of `ProcessDispatchDefaults` and related config/default projection logic currently split between `src/runtime/node.rs`, `src/ha/state.rs`, and `src/ha/source_conn.rs`; the final boundary must make domain ownership clearer and reduce runtime knowledge.
- [x] Re-evaluate initial state fabrication and cross-worker state-channel setup currently embedded in `src/runtime/node.rs`; move domain-specific initialization out of runtime where practical and reduce duplicated/manual state copying.
- [x] Remove dead or disabled legacy startup code/tests in `src/runtime/node.rs`, including the `#[cfg(all(test, any()))]` block and any stale helper logic or references that no longer match current architecture.
- [x] Update affected modules and tests across `src/runtime/`, `src/dcs/`, `src/ha/`, `src/api/`, `src/process/`, `src/pginfo/`, and any related startup/test helpers so the new narrower runtime boundary is enforced by code structure and module privacy.
- [x] The final implementation leaves `node.rs` substantially smaller, with less direct domain knowledge, fewer setter/mutation-style startup steps, and a smaller public/runtime-facing interface than before.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>
- [ ] Rewire `src/runtime/node.rs` to construct only domain bootstrap ADTs, remove runtime-owned helper projections/state fabrication, and adopt the narrower startup boundaries introduced in `src/process/state.rs`, `src/pginfo/state.rs`, `src/dcs/state.rs`, `src/ha/state.rs`, and `src/api/worker.rs`.
- [ ] Move domain-owned startup execution into the owning modules: process startup path preparation, pginfo local probe target materialization, DCS local advertisement/store bootstrap assembly, HA worker bootstrap construction, and API serving-plan derivation from runtime config.
- [ ] Update all affected worker modules, tests, and harnesses to the new bootstrap/runtime ADTs; delete stale runtime startup helpers and legacy disabled runtime startup code; then run `make check`, `make lint`, `make test`, `make test-long`, and only afterward do docs updates with the `k2-docs-loop` skill.
NOW EXECUTE
</plan>

### Execution plan
1. Finish the runtime composition-root shrink by keeping `src/runtime/node.rs` limited to config validation, logging bootstrap, top-level startup event emission, domain runtime assembly, and worker lifecycle orchestration.
2. Complete the new startup ADT rollout in `src/process/startup.rs`, `src/pginfo/startup.rs`, `src/dcs/startup.rs`, `src/ha/startup.rs`, and `src/api/startup.rs` so runtime depends on domain-owned requests/runtimes instead of raw worker ctx construction.
3. Remove the remaining leaked ownership in runtime by deleting runtime-local config/default/path/routing helpers and moving the real projection logic behind the owning domain startup boundary.
4. Narrow the DCS production boundary so the startup module owns store/channel/bootstrap fabrication and runtime only receives the read model plus command handle it must pass to HA/API.
5. Replace the HA stub-and-mutate production wiring path with a real HA-owned bootstrap path, and tighten the process/API/pginfo startup seams so runtime no longer needs domain-private field knowledge.
6. After the type design is fully wired and compile errors are resolved, run the required validation gates in repo-preferred order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after all checks pass, update docs for the new startup/runtime ownership model using the `k2-docs-loop` skill, remove stale startup architecture docs if needed, then complete task closeout (`<passes>true</passes>`, task switch, commit, push).

### Constraints for execution
- Keep the new startup boundary domain-owned. Do not slide back to `node.rs` assembling raw worker internals or recreating domain-private defaulting logic.
- Prefer typed startup requests/runtimes and shared identity ADTs over ad hoc tuples, loose strings, or setter-style mutation.
- Remove legacy/dead startup code instead of preserving compatibility surfaces that only exist to avoid touching call sites.
- If execution shows the current ADTs are still wrong, switch this task back to `TO BE VERIFIED`, explain the type/design gap in the task file, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE

<implementation_plan>
- [x] Inspect `src/runtime/node.rs` and every current startup boundary it owns directly so the redesign changes the owning domain types first instead of doing file-size-only extraction.
- [x] Replace the flat worker startup/context shapes in HA, DCS, API, pginfo, and process with narrower ADT-style bundles that separate identity, observed state, state channels, control plane, cadence, and runtime services.
- [x] Rewire `src/runtime/node.rs` to speak only in domain startup requests/results and stop constructing domain-private startup details directly.
- [x] Repair compiler fallout by implementing the new constructors and moving remaining helper logic into the owning domains.
- [x] Run `make check`, `make lint`, `make test`, `make test-long`, then update docs with the `k2-docs-loop` skill once the implementation is complete.
NOW EXECUTE
</implementation_plan>
