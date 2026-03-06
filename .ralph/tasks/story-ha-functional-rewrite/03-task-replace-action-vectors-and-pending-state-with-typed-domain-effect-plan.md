---
## Task: Replace action vectors and pending state with HaDecision plus lowered effect plan <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Replace `Vec<HaAction>` planning with a high-level `HaDecision` enum plus an inherent `HaDecision::lower(&self) -> HaEffectPlan` step, and remove `pending` entirely from HA state.

**Scope:**
- Edit `src/ha/{actions,state,decide,worker,mod}.rs`, `src/api/{mod,controller}.rs`, `src/cli/{client,output}.rs`, and all affected tests.
- Replace `Vec<HaAction>`-style planning with:
  - a high-level `HaDecision` enum chosen by `decide`
  - an inherent `impl HaDecision { fn lower(&self) -> HaEffectPlan }`
- Remove `pending` from `HaState` completely and eliminate `pending_actions` from state/API surfaces.
- Replace stringly typed HA phase/trust API exposure with typed serialized enums wherever HA state is surfaced externally.
- Log the chosen `HaDecision` outcome for observability instead of persisting the previous plan in HA state.

**Context from research:**
- We discussed that “no more vec” is part of the real design goal: the state machine should first choose a high-level domain decision, and only after that lower into executable effects.
- A plain vector makes contradictory effects and duplicate cleanups too easy to represent, which is why the current code needs post-hoc dedupe.
- `pending` in `HaState` is not true machine state; it is the previous plan leaking execution details into persisted HA state and should be removed rather than renamed.
- Logging the chosen `HaDecision` gives better observability than persisting last tick's plan in state.
- API/CLI currently stringify `ha_phase` and `dcs_trust`, which weakens type safety in the exact surfaces we use for tests and operations.
- We agreed the lowered plan should be structured by concern, not by append order. After restore removal, the intended concern buckets are:
  - `LeaseEffect`
  - `SwitchoverEffect`
  - `ReplicationEffect`
  - `PostgresEffect`
  - `SafetyEffect`
- We also agreed the lowering boundary must be strict: if lowering still needs `DecisionFacts`, then `HaDecision` is too weak and must carry more payload.
- The intended first-pass lowering contract should be treated as explicit target shape:
  - `impl HaDecision { fn lower(&self) -> HaEffectPlan }`
- The intended support enums should be defined explicitly rather than inferred ad hoc later:
  - `StepDownReason`
  - `RecoveryStrategy`
  - `LeaseEffect`
  - `SwitchoverEffect`
  - `ReplicationEffect`
  - `PostgresEffect`
  - `SafetyEffect`

**Expected outcome:**
- The decision layer returns a high-level `HaDecision`, and `HaDecision::lower()` turns that into a typed effect plan whose shape prevents contradictions by construction as much as practical.
- `pending` is removed from core HA state.
- HA API/CLI surfaces use typed enums and stop normalizing the machine into string comparisons.
- Dedupe becomes unnecessary or dramatically smaller because the plan model is structured, not append-only.
- Decision observability comes from logs/debug events of `HaDecision`, not from storing a copied plan in `HaState`.
- The lowered plan is a fixed-structure bundle, not a renamed vector:
  - `LeaseEffect::{None, AcquireLeader, ReleaseLeader}`
  - `SwitchoverEffect::{None, ClearRequest}`
  - `ReplicationEffect::{None, FollowLeader { leader }, RecoverReplica { strategy }}`
  - `PostgresEffect::{None, Start, Promote, Demote}`
  - `SafetyEffect::{None, FenceNode, SignalFailSafe}`
- `RecoverReplica { strategy }` should use an explicit recovery strategy payload such as:
  - `Rewind { leader }`
  - `BaseBackup { leader }`
  - `Bootstrap`
- The expected first-pass supporting decision payloads should also be explicit:
  - `StepDownReason` should distinguish at least `Switchover`, `ForeignLeaderDetected`, and `FailSafe`
  - `RecoveryStrategy` should distinguish at least `Rewind { leader }`, `BaseBackup { leader }`, and `Bootstrap`

**Story test policy:**
- This task is not complete unless `make check`, `make test`, `make test-long`, and `make lint` all pass.
- Long HA coverage was executed in this task and is no longer deferred.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Modify the actual affected production and docs surfaces for this rewrite, including `src/ha/{state,decide,decision,lower,worker}.rs`, `src/api/{mod,controller,worker}.rs`, `src/cli/{client,output}.rs`, `src/dcs/state.rs`, `docs/src/contributors/{ha-pipeline,api-debug-contracts}.md`, and the HA end-to-end coverage that validates the final behavior.
- [x] Replace `Vec<HaAction>` in production HA planning/output with a high-level `HaDecision` enum and `impl HaDecision { fn lower(&self) -> HaEffectPlan }`.
- [x] Implement or closely match the explicit lowering contract shape:
  - `impl HaDecision { fn lower(&self) -> HaEffectPlan }`
- [x] Remove `pending: Vec<HaAction>` from `HaState`.
- [x] Remove `pending_actions` from the HA state API/CLI surface.
- [x] Add decision-outcome logging or debug-event emission based on `HaDecision` so observability does not depend on persisted pending plan state.
- [x] Replace string-based HA phase/trust exposure in API/CLI with typed serialized enums suitable for direct assertions in tests.
- [x] Define `HaEffectPlan` as a fixed-structure bundle with explicit concern buckets for lease, switchover, replication, postgres lifecycle, and safety.
- [x] Define the lowered effect enums so that the intended post-restore-removal effect set is represented explicitly:
  - `LeaseEffect::{None, AcquireLeader, ReleaseLeader}`
  - `SwitchoverEffect::{None, ClearRequest}`
  - `ReplicationEffect::{None, FollowLeader { leader }, RecoverReplica { strategy }}`
  - `PostgresEffect::{None, Start, Promote, Demote}`
  - `SafetyEffect::{None, FenceNode, SignalFailSafe}`
- [x] Define `RecoveryStrategy` explicitly, including `Rewind { leader }`, `BaseBackup { leader }`, and `Bootstrap`.
- [x] Define `StepDownReason` explicitly, including at least `Switchover`, `ForeignLeaderDetected`, and `FailSafe`.
- [x] Ensure `HaDecision::lower()` does not accept `DecisionFacts`, `WorldSnapshot`, or any equivalent world-derived context.
- [x] Ensure the `HaDecision` shape and lowered typed effect plan make duplicate/conflicting effects impossible or explicit enough that post-hoc dedupe is no longer the primary correctness mechanism.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

<plan>
## Execution plan draft (2026-03-06)

### 1. Start from the actual current gap, not from the stale task header

The repository already contains a substantial part of this task:

- `src/ha/decision.rs` exists and already holds `DecisionFacts`, `PhaseOutcome`, and `HaDecision`.
- `src/ha/state.rs` already removed `pending` and stores `decision` in `HaState`.
- `src/ha/decide.rs` already selects domain decisions through pure phase handlers.
- `src/ha/lower.rs` exists as a separate lowering boundary.

But the task is not actually complete yet because the implementation still misses the key contract this task is about:

- lowering still returns `Vec<HaAction>` through `lower_decision(...)`
- `HaDecision` does not yet provide the required inherent `fn lower(&self) -> HaEffectPlan`
- the lowered plan is not yet a fixed concern-bucket structure
- API and CLI still expose `dcs_trust` and `ha_phase` as strings instead of typed serialized enums
- decision detail is still flattened into string labels/details instead of typed serialized decision data
- contributor docs still describe the lowered result as an action list rather than the typed plan boundary

Execution must therefore treat the existing code as partial progress and finish the architectural replacement rather than rewriting everything from scratch.

### 2. Freeze the target architecture before editing

The implementation target for this task should be:

- decision layer:
  - pure `DecisionFacts` gathering
  - pure `PhaseOutcome { next_phase, decision }`
  - no execution-order reasoning here
- lowering layer:
  - `impl HaDecision { fn lower(&self) -> HaEffectPlan }`
  - no access to `DecisionFacts`, `DecideInput`, or `WorldSnapshot`
  - one fixed bundle with explicit buckets:
    - `LeaseEffect`
    - `SwitchoverEffect`
    - `ReplicationEffect`
    - `PostgresEffect`
    - `SafetyEffect`
- dispatch layer:
  - execute the lowered plan deterministically
  - if dispatch still needs a linear sequence, derive that sequence from the typed plan at the worker boundary instead of keeping `Vec<HaAction>` as the primary model
- presentation layer:
  - API/CLI/debug surfaces should serialize typed HA enums directly, not `format!("{:?}", ...)`

Do not keep both models alive as co-equal sources of truth. `HaEffectPlan` must become the canonical lowered form, and any linear action list must become a temporary dispatch/view derived from it.

### 3. Replace the current lowering contract with a typed effect bundle

Implementation sequence:

1. Introduce a new typed lowered-plan model, preferably alongside the decision model so the contract is close to the domain:
   - `struct HaEffectPlan`
   - `enum LeaseEffect`
   - `enum SwitchoverEffect`
   - `enum ReplicationEffect`
   - `enum PostgresEffect`
   - `enum SafetyEffect`
2. Shape the plan so contradictions are impossible or explicit by construction:
   - one lease effect per tick
   - one switchover effect per tick
   - one replication effect per tick
   - one postgres effect per tick
   - one safety effect per tick
3. Move lowering to an inherent method:
   - `impl HaDecision { pub(crate) fn lower(&self) -> HaEffectPlan }`
4. Remove the current `lower_decision(&HaDecision) -> Vec<HaAction>` primary contract.
5. If dispatch still needs ordered `HaAction`s, add a secondary pure conversion such as `HaEffectPlan::to_actions()` or equivalent, but make it clear that:
   - `HaEffectPlan` is the source of truth
   - linear action order is derived from bucket order, not authored directly by decision code

Required first-pass mapping target:

- `LeaseEffect::{None, AcquireLeader, ReleaseLeader}`
- `SwitchoverEffect::{None, ClearRequest}`
- `ReplicationEffect::{None, FollowLeader { leader }, RecoverReplica { strategy }}`
- `PostgresEffect::{None, Start, Promote, Demote}`
- `SafetyEffect::{None, FenceNode, SignalFailSafe}`

### 4. Strengthen decision payloads so lowering is fully fact-free

Current code already carries more structure than raw actions, but it still does not match the requested payload shape. Make these adjustments:

- replace or rename step-down reasoning so it matches explicit domain meanings:
  - `StepDownReason::Switchover`
  - `StepDownReason::ForeignLeaderDetected { leader }` or a very close equivalent
  - `StepDownReason::FailSafe`
- replace the current split `RecoveryPlan { strategy, leader_member_id }` shape if needed so the recovery payload itself carries what lowering needs
- define `RecoveryStrategy` exactly or very close to:
  - `Rewind { leader }`
  - `BaseBackup { leader }`
  - `Bootstrap`

The rule is strict: once a `HaDecision` exists, lowering must not need to inspect world facts. If that means moving leader identity inside recovery strategy variants rather than carrying it next to them, do that.

### 5. Adapt decision selection with minimal churn but correct payloads

Keep the pure decision machine already present in `src/ha/decide.rs`, but update it to emit the stronger payloads required by the typed plan:

- `Replica` branches that follow a leader should emit the typed leader payload directly.
- recovery branches should choose `Rewind { leader }`, `BaseBackup { leader }`, or `Bootstrap` explicitly.
- primary step-down branches should distinguish:
  - switchover-driven step-down
  - foreign-leader-detected step-down
  - fail-safe step-down if the chosen design routes demotion through a unified step-down path
- fail-safe entry must preserve whether leader lease release is required in a way that lowering can derive without re-reading facts.

Do not reintroduce `mut`-driven accumulation or `Vec<HaAction>` construction inside `decide.rs`.

### 6. Rework worker dispatch around the new plan, without reintroducing a vector boundary

`src/ha/worker.rs` should be updated in this order:

1. call `decide(...)`
2. call the inherent `output.outcome.decision.lower()`
3. emit observability for both the chosen decision and the typed effect plan
4. dispatch the typed effect plan directly through a new plan-aware worker boundary such as `dispatch_effect_plan(...)`
5. only derive tiny local per-bucket sequencing inside the dispatcher where unavoidable, not a general-purpose `Vec<HaAction>`
6. publish `HaState` with the chosen decision

Specific constraints:

- remove the old dependency on `lower_decision(...)`
- do not add `HaEffectPlan::to_actions()` as a new long-lived public or canonical API; that would keep the vector model alive under a different name
- keep dispatch sequencing deterministic and explicit through fixed bucket order inside the dispatcher:
  - postgres demotion before lease release for primary step-down
  - lease release before switchover cleanup when both are required
  - destructive recovery preparation before base-backup/bootstrap start
  - lease release before fail-safe signaling when both are required
- if action-shaped logging remains useful, derive logging metadata from each dispatched bucketed effect at the worker boundary rather than restoring action-list planning

### 7. Replace stringly typed API and CLI state surfaces with one shared transport contract

Current API/CLI code still uses:

- `dcs_trust: String`
- `ha_phase: String`
- `ha_decision: String`
- `ha_decision_detail: Option<String>`

That needs to become a typed serialization contract suitable for direct assertions.

Execution target:

- introduce one shared serializable response contract, defined once and reused by both API and CLI, for:
  - HA phase
  - DCS trust
  - HA decision
- stop building these fields with `format!("{:?}", ...)`, `label()`, and `detail()`
- update API controller mapping to populate the typed shared schema
- update CLI client to deserialize the exact same shared response structs/enums instead of maintaining mirror copies
- update CLI text rendering to print the typed values in a human-readable way without weakening the JSON contract

Because this is one crate, prefer a single source of truth over transport mirrors. Only introduce separate transport enums if reusing internal/domain enums would create a real semantic leak; if that happens, the dedicated transport types must still live in one shared module that both API and CLI import.

### 8. Update tests in the same pass as the contract changes

The minimum required test updates are:

- `src/ha/lower.rs` unit tests rewritten around `HaEffectPlan` equality and any derived action-order checks
- `src/ha/decide.rs` tests updated for the stronger step-down and recovery payload shapes
- `src/ha/worker.rs` tests updated for the new lowering/dispatch boundary and observability
- API/controller tests updated to assert typed serialized values instead of strings
- CLI/client and CLI/output tests updated for the new response schema and text rendering
- debug snapshot or API-surface tests updated anywhere they still expect `pending_actions` or stringified phase/trust values

Do not keep old string fields in parallel just to preserve tests. The tests should move to the stronger contract.

### 9. Update docs and remove stale wording

At minimum update:

- `docs/src/contributors/ha-pipeline.md`

The doc currently says the decision lowers into an ordered `HaAction` list. That should be rewritten to describe:

- domain decision selection
- typed effect-plan lowering
- dispatch derivation from the effect plan
- typed external HA state surfaces

Remove stale wording rather than documenting both the old and new designs.

### 10. Final execution/verification checklist for the later `NOW EXECUTE` pass

When this task is promoted to `NOW EXECUTE`, execute in this order without fresh exploration:

1. Implement typed `HaEffectPlan` plus effect enums and inherent `HaDecision::lower()`.
2. Refactor decision payloads (`StepDownReason`, `RecoveryStrategy`, related structs/enums) until lowering is fact-free.
3. Update `src/ha/worker.rs` to use the new lowering boundary and direct typed-plan dispatch, not a replacement vector conversion.
4. Update API/CLI typed response contracts and rendering.
5. Update docs to match the new architecture.
6. Update all affected tests.
7. Run all required gates with no skipping:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
8. Only after all four pass:
   - tick acceptance boxes truthfully
   - set `<passing>true</passing>`
   - run `/bin/bash .ralph/task_switch.sh`
   - commit all tracked and untracked changes, including `.ralph` files
   - push with `git push`

### 11. Skeptical review result recorded on 2026-03-06

This plan was materially changed during the skeptical pass before promotion:

- the previous draft allowed a derived `Vec<HaAction>` boundary to survive as `HaEffectPlan::to_actions()`, which would have preserved the model this task is supposed to delete; execution must now dispatch the typed plan directly
- the previous draft allowed API and CLI to maintain separate mirror transport schemas, which would likely drift; execution must now define one shared transport contract used by both surfaces

Remaining execution-time judgment calls are narrower:

- whether `HaEffectPlan` should live in `decision.rs`, `lower.rs`, or its own module
- whether fail-safe should be modeled as `StepDownReason::FailSafe` or remain a separate `HaDecision::EnterFailSafe`
- whether `FollowLeader` belongs only in `ReplicationEffect` or also needs a decision-level payload adjustment

</plan>

NOW EXECUTE

<verification>
- `make check` passed via gate run `20260306T055316Z-4154757-21275`.
- `make test` passed via gate run `20260306T055323Z-4154932-23013`.
- `make test-long` passed via gate run `20260306T055403Z-4156348-3870`.
- `make lint` passed via gate run `20260306T061448Z-4163120-10414`.
</verification>
