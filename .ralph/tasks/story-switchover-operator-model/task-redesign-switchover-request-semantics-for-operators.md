## Task: Redesign Switchover Request Semantics For Operators <status>not_started</status> <passes>false</passes>

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
- [ ] The current switchover request flow is audited across `src/api/controller.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/dcs/state.rs`, `src/ha/decision.rs`, `src/ha/decide.rs`, and operator-facing docs
- [ ] The task explicitly decides whether switchover is generic, targeted, or generic-with-optional-target, and that decision is reflected consistently in code and docs
- [ ] If `requested_by` is retained, its semantics are real, justified, and exercised beyond simple presence checks
- [ ] If `requested_by` is not semantically justified, it is removed or replaced with a better-typed field or fields
- [ ] If targeted switchover is part of the chosen design, the API and CLI support an explicit target successor field with validation and clear operator feedback
- [ ] If successor choice remains automatic, the selection rule is surfaced clearly in API/CLI/docs and pending switchover state is represented with names that match the real behavior
- [ ] HA state output no longer exposes misleading switchover metadata names or values
- [ ] Tests are added or updated to cover the chosen public semantics, including acceptance, convergence behavior, and operator-visible state
- [ ] Relevant docs are updated so operators understand what a switchover request means and what control they do or do not have over successor choice
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
