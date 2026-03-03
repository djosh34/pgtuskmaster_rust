---
## Task: Expose full HA admin API read and write surface <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a first-class HA admin API that exposes operational read endpoints and write actions needed to control cluster behavior without touching DCS directly.

**Scope:**
- Extend `src/api/controller.rs` with typed request/response handlers for HA admin actions beyond switchover.
- Extend `src/api/worker.rs` routing/auth to expose a complete admin/read API surface.
- Add/update API contracts in `src/api/mod.rs` and config/auth wiring in `src/config/schema.rs` (and parser/defaults if needed).
- Add behavior tests in `tests/bdd_api_http.rs` and module unit tests for request validation and DCS write semantics.

**Context from research:**
- Current API only exposes `/switchover`, fallback routes, and debug routes, while e2e currently uses direct DCS mutations in `src/ha/e2e_multi_node.rs`.
- New e2e requirements demand that tests read and control HA only through exposed API surfaces.
- Keep strict typed payload handling and deny-unknown-fields patterns already used in controller inputs.

**Expected outcome:**
- Operators and tests can observe HA state and request HA control actions over HTTP API only, with no need for direct DCS access.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: `src/api/controller.rs` (typed handlers), `src/api/worker.rs` (routes/authz), `src/api/mod.rs` (shared API types/errors), `src/config/schema.rs` + parser/defaults (admin/read auth config), `tests/bdd_api_http.rs` + new API contract tests (read/write route assertions)
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 1)

### Research snapshot (parallel exploration complete)
- Completed parallel exploration across 12 tracks touching `src/api/controller.rs`, `src/api/worker.rs`, `src/api/mod.rs`, `src/config/{schema,defaults,parser}.rs`, `tests/bdd_api_http.rs`, `src/ha/e2e_multi_node.rs`, and DCS typed writer surfaces in `src/dcs/store.rs`.
- Confirmed current API surface is narrow: `POST /switchover`, fallback routes, and debug routes; there is no first-class HA admin read API for leader/switchover/member visibility.
- Confirmed auth plumbing already supports read/admin role decisions in `api::worker`, but runtime config currently only exposes legacy `security.auth_token`; there is no explicit read/admin token config in schema/defaults/parser.
- Confirmed typed DCS write helpers already exist for HA semantics (`write_leader_lease`, `delete_leader`, `clear_switchover`) and should be reused to avoid raw key-path duplication.
- Confirmed e2e currently performs direct DCS mutations in `src/ha/e2e_multi_node.rs`, so this task must expose API actions that later tasks can call instead of direct `write_path`/`delete_path`.

### Planned API contract for this task
1. Add typed HA admin read endpoint(s):
- `GET /ha/state`: return typed operational snapshot with fields needed by operators/tests:
- cluster identity (`cluster_name`, `scope`, `self_member_id`);
- DCS view (`leader`, `switchover_requested_by`, `member_count`);
- local HA view (`ha_phase`, `ha_tick`, `pending_actions`).
- Data source: use API worker snapshot subscriber (same underlying snapshot channel used for debug views), but keep this endpoint independent of `debug.enabled`. If snapshot source is unavailable, return `503`.

2. Add typed HA admin write endpoint(s) beyond switchover:
- `POST /ha/leader` with typed body `{ "member_id": "node-x" }` to set leader record via typed DCS writer.
- `DELETE /ha/leader` to clear leader record via typed DCS writer.
- `DELETE /ha/switchover` to clear pending switchover via typed DCS writer.
- Keep existing `POST /switchover` behavior for backward compatibility.

3. Keep strict serde contracts:
- Every new request struct must use `#[serde(deny_unknown_fields)]`.
- Every handler validates non-empty identifiers/tokens and maps errors to `ApiError::{BadRequest,DcsStore,Internal}`.

### Planned implementation phases (execute exactly in order during NOW EXECUTE)
1. Extend shared API types in `src/api/mod.rs`
- Add typed response structs shared across controller/worker for HA read endpoints.
- Keep `AcceptedResponse` for mutation ACKs; add explicit read response types for HA state fields.

2. Extend controller handlers in `src/api/controller.rs`
- Add request/response types and handlers:
- `post_set_leader(...)`
- `delete_leader(...)`
- `delete_switchover(...)`
- `get_ha_state(...)` (typed projection from snapshot inputs)
- Reuse `DcsHaWriter` for leader/switchover mutation operations.
- Add unit tests for:
- unknown-field rejection on new inputs;
- exact typed DCS write/delete semantics and target keys;
- bad input validation (empty member IDs).

3. Extend API worker routing in `src/api/worker.rs`
- Add route matches for new endpoints and map them to controller handlers.
- Add method to attach/read HA snapshot source for `GET /ha/state`; return `503` when unavailable.
- Keep `POST /switchover` route unchanged.
- Ensure `api_error_to_http` covers all controller failure paths without panic/unwrap.

4. Extend auth role mapping in `src/api/worker.rs`
- Mark new mutation endpoints as `EndpointRole::Admin`.
- Keep new read endpoint as `EndpointRole::Read`.
- Preserve legacy behavior: admin token can read; read token cannot invoke admin mutations.
- Add/adjust worker tests verifying `401/403/200/202` behavior on new routes.

5. Wire explicit API read/admin auth config in `src/config/schema.rs`
- Add explicit token fields under API config:
- runtime: `api.read_auth_token`, `api.admin_auth_token` (both `Option<String>`);
- partial config mirrors.
- Keep `security.auth_token` as legacy fallback for backward compatibility.

6. Add defaults + validation in `src/config/defaults.rs` and `src/config/parser.rs`
- Defaults:
- new API tokens default to `None`.
- Parser validation:
- reject configured blank/whitespace `api.read_auth_token` / `api.admin_auth_token`;
- keep existing timeout/scope/binary invariants.
- Add/update config unit tests for:
- defaults preserve `None`;
- explicit tokens roundtrip;
- invalid blank token rejected with field-specific validation error.

7. Expand BDD/API integration coverage in `tests/bdd_api_http.rs`
- Add black-box HTTP tests for:
- `GET /ha/state` success path (with seeded snapshot input);
- `POST /ha/leader` creates expected typed DCS write;
- `DELETE /ha/leader` and `DELETE /ha/switchover` produce expected DCS delete actions;
- auth role behavior on new endpoints (read token denied for admin writes, admin token allowed).
- Keep current switchover/fallback/debug tests intact.

8. Synchronize runtime-config constructors across test modules
- Update all `RuntimeConfig` test fixtures that construct `ApiConfig` literals so new schema fields compile and preserve existing semantics.
- Do not add linter exceptions; handle all errors explicitly.

9. Verification and closeout
- Run full required gates in order:
- `make check`
- `make test`
- `make test-bdd`
- `make lint`
- If failures occur, fix immediately or create bugs via `add-bug` per AGENTS policy before completion.

### Skeptical verification requirements before execution
- When this task is in `TO BE VERIFIED`, run a deep skeptical plan review with at least 16 parallel verification tracks.
- Mandatory: alter at least one concrete plan item (contract shape, sequencing, validation rule, or test gate) and document the exact delta with rationale in this file.
- Replace `TO BE VERIFIED` with `NOW EXECUTE` only after that alteration is recorded.

### Risks and controls
- Risk: API read endpoint depends on unavailable snapshot data.
- Control: explicit `503` contract and dedicated tests for missing-subscriber behavior.
- Risk: auth regression where read token can perform admin mutations.
- Control: enforce endpoint-role matrix in worker tests and BDD tests.
- Risk: config drift from introducing new `ApiConfig` fields.
- Control: compile-fix sweep across all runtime-config fixtures plus parser/defaults tests.
- Risk: direct raw DCS path handling duplicated in API code.
- Control: route all leader/switchover mutations through typed DCS writer helpers.
</execution_plan>

TO BE VERIFIED
