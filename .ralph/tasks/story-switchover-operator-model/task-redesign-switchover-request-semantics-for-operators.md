## Task: Redesign Switchover Request Semantics For Operators <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Replace the current switchover operator model with a clearer and more defensible design. The higher-order goal is to make planned switchovers understandable and controllable from the public API and CLI, so operators do not have to provide a semantically confusing `requested_by` value and do not have to guess which successor the cluster will choose after accepting a switchover request.

**Scope:**
- Investigate and redesign the public switchover request model across API, CLI, HA decision inputs, DCS record shape, and docs.
- Decide whether the public model should support:
- purely generic switchover requests with no caller-supplied identity at all
- explicit audit metadata separate from switchover intent
- explicit target successor selection, if the architecture intends operators to choose a successor
- or a combination of generic request plus optional structured target/audit fields
- Remove or rename the current `requested_by` field if it does not represent a true actor identity with meaningful semantics inside HA.
- Ensure the runtime behavior and public API contract match each other: if successor choice is automatic, expose and document the selection rule clearly; if targeted switchovers are supported, encode and validate the target explicitly.
- Update status surfaces so operators can inspect pending switchover intent in a way that reflects the real semantics rather than leaking an underdesigned DCS field name.

**Context from research:**
- The current DCS switchover record is only `SwitchoverRequest { requested_by: MemberId }` in `src/dcs/state.rs`.
- The API accepts only `requested_by` in `src/api/controller.rs` and writes that record to `/{scope}/switchover`.
- The CLI mirrors that exact model in `src/cli/args.rs` and `src/cli/client.rs`, requiring `ha switchover request --requested-by <value>`.
- In the HA decision path, `requested_by` is propagated into `DecisionFacts.switchover_requested_by`, but the decision logic only checks whether the field is `Some(...)` to trigger primary step-down. The value itself is not used to choose a successor.
- Successor selection currently comes from `available_primary_member_id` and `follow_target(...)` in `src/ha/decision.rs` and `src/ha/decide.rs`. That is derived from observed DCS member role/health, not from the operator-supplied `requested_by` string.
- Live cluster verification showed the practical consequence:
- a generic switchover request moved the cluster from `node-b` to `node-c`
- a second generic request later moved it to `node-a`
- there was no public way to say “switch over specifically to node-a” in either CLI or API
- The codebase also contains `SwitchoverRequestId` in `src/state/ids.rs`, but the current switchover record and public API do not use it, which suggests the model is incomplete or under-evolved.
- Current docs already expose `switchover_requested_by` as an operator-facing state field in `docs/src/how-to/check-cluster-health.md`, which risks cementing a misleading concept.

**Expected outcome:**
- Operators no longer need to guess why they are providing `requested_by`.
- The public API and CLI expose switchover semantics that match actual HA behavior.
- If successor choice is automatic, operators can see the rule and any pending request details clearly.
- If successor choice should be controllable, the API and CLI support explicit target selection and validation.
- Docs, status output, and tests all align with the final model.

</description>

<acceptance_criteria>
- [x] The current switchover request flow is audited across `src/api/controller.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/dcs/state.rs`, `src/ha/decision.rs`, `src/ha/decide.rs`, and operator-facing docs
- [x] The task explicitly decides whether switchover is generic, targeted, or generic-with-optional-target, and that decision is reflected consistently in code and docs
- [x] If `requested_by` is retained, its semantics are real, justified, and exercised beyond simple presence checks
- [x] If `requested_by` is not semantically justified, it is removed or replaced with a better-typed field or fields
- [x] If targeted switchover is part of the chosen design, the API and CLI support an explicit target successor field with validation and clear operator feedback
- [x] If successor choice remains automatic, the selection rule is surfaced clearly in API/CLI/docs and pending switchover state is represented with names that match the real behavior
- [x] HA state output no longer exposes misleading switchover metadata names or values
- [x] Tests are added or updated to cover the chosen public semantics, including acceptance, convergence behavior, and operator-visible state
- [x] Relevant docs are updated so operators understand what a switchover request means and what control they do or do not have over successor choice
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Chosen design

- Make switchover a generic operator request with no caller-supplied identity field.
- Do not add target-successor selection in this task. The current HA engine already selects the replacement primary automatically from observed cluster state, and there is no targeting logic to preserve. Adding targeting now would expand scope across validation, election rules, and operator guarantees without evidence that the architecture is ready for it.
- Replace the misleading `requested_by` naming across API, CLI, DCS state, status output, tests, and docs with names that describe the real semantics: a pending switchover request exists, and successor selection is automatic.

### Code changes

- Update the DCS switchover record in `src/dcs/state.rs` from a typed payload carrying `requested_by: MemberId` to an empty marker-style record such as `SwitchoverRequest {}`.
- Update all DCS fixtures and decoding tests that currently construct or assert `requested_by`, especially in `src/dcs/store.rs`, `src/dcs/etcd_store.rs`, and `src/dcs/worker.rs`, so they encode/decode the new generic request shape cleanly.
- Remove the unused semantic dependency on `MemberId` from switchover request inputs where it only existed to support `requested_by`.

- Update the API request model in `src/api/controller.rs` so `POST /switchover` continues to be the request endpoint, accepts an empty JSON object, and rejects unknown fields.
- Remove `requested_by` validation entirely, because a generic request no longer needs a synthetic actor string.
- Keep `DELETE /ha/switchover` unchanged as the cancellation path.
- Rename the HA state response field in `src/api/mod.rs` and `src/api/controller.rs` from `switchover_requested_by` to `switchover_pending: bool` so the public shape says exactly what the runtime knows.

- Update the CLI argument model in `src/cli/args.rs` so `ha switchover request` takes no `--requested-by` flag.
- Update `src/cli/client.rs` so it POSTs the new empty request body.
- Update `src/cli/mod.rs` and `src/cli/output.rs` so operator-visible output reflects the renamed `switchover_pending` field and no longer prints `switchover_requested_by`.
- Keep command names stable unless review reveals a compelling mismatch; the semantics problem is in the payload and output naming, not the `ha switchover request` verb.

- Update HA decision inputs in `src/ha/decision.rs` to stop propagating a fake actor identity. Replace `switchover_requested_by: Option<MemberId>` with a presence-based field such as `switchover_pending: bool`.
- Update `src/ha/decide.rs` to branch on the renamed presence field only, preserving existing behavior.
- Verify that successor selection still comes exclusively from the observed leader/member state via `follow_target(...)` and document that rule rather than changing the election algorithm in this task.

### Test changes

- Rewrite API controller and worker tests to use the new POST body shape `{}` and to assert the renamed HA state response field.
- Rewrite CLI parsing and client tests so switchover request succeeds without `--requested-by`, and add an assertion that unknown request-body fields are rejected server-side.
- Update unit tests around HA decision facts and phase transitions to assert the new `switchover_pending` representation rather than `requested_by`.
- Update BDD and e2e coverage in `tests/bdd_api_http.rs`, `tests/policy_e2e_api_only.rs`, and `tests/ha/support/multi_node.rs` so switchover requests are exercised without caller identity and status checks observe the renamed field.
- Verify long-running HA tests still cover the automatic successor outcome so the docs can truthfully state that successor choice is automatic and derived from cluster state.

### Documentation changes

- Update all operator-facing docs that currently teach or expose `requested_by` or `switchover_requested_by`, especially:
- `docs/src/how-to/perform-switchover.md`
- `docs/src/how-to/check-cluster-health.md`
- `docs/src/how-to/debug-cluster-issues.md`
- `docs/src/reference/http-api.md`
- `docs/src/reference/pgtuskmasterctl-cli.md`
- `docs/src/reference/dcs-state-model.md`
- `docs/src/explanation/architecture.md`
- `docs/src/explanation/ha-decision-engine.md`
- Because the `update-docs` skill is not available in this session, update docs manually and then refresh or prune any generated/draft mirrors that would otherwise leave stale operator guidance behind, including `docs/draft/docs/...` and relevant `docs/tmp/...` prompt artifacts if they are tracked outputs of the docs workflow.
- Remove stale text that implies operators choose a successor through `requested_by`.
- Add explicit explanation that switchover requests are generic, the current primary steps down when safe, and the successor is chosen automatically from the HA engine’s observed eligible leader/follow target logic.
- Update status/output examples so they show `switchover_pending=true|false` instead of an optional requester string.

### Verification sequence

- Run focused tests first for touched Rust modules to catch shape mismatches early.
- Run the required full gates in this order and only mark the task complete if all pass: `make check`, `make test`, `make test-long`, `make lint`.
- If any doc artifacts or generated book outputs change as part of the docs workflow, include them in the task completion commit.
- After all code, docs, and gates pass, update the task file checkboxes and `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit with the required `task finished ...` message including test evidence, and push.

NOW EXECUTE

## Execution Notes

- Implemented the chosen generic switchover model by removing caller-supplied `requested_by` data from the DCS/API/CLI flow and replacing operator-visible HA state with `switchover_pending: bool`.
- Kept the public request endpoint as `POST /switchover` with an empty JSON body and unknown-field rejection, while preserving the existing automatic successor-selection behavior in the HA engine.
- Updated Rust unit/integration/e2e coverage plus operator docs and key tracked draft mirrors to describe the automatic successor rule and the new status field.
- Verification passed in full on March 8, 2026: `make check`, `make test`, `make test-long`, and `make lint`.
