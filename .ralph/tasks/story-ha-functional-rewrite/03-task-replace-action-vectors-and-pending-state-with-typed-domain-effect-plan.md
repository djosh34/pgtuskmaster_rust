---
## Task: Replace action vectors and pending state with HaDecision plus lowered effect plan <status>not_started</status> <passes>false</passes>

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
- Skip `make test-long` and any direct long HA cargo-test invocations in this task.
- Known long-test failures are deferred until the final story task after the rewrite story lands.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/ha/actions.rs`, `src/ha/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/ha/mod.rs`, `src/api/mod.rs`, `src/api/controller.rs`, `src/cli/client.rs`, `src/cli/output.rs`, and every touched test file that depends on the old action-vector or stringified response shape.
- [ ] Replace `Vec<HaAction>` in production HA planning/output with a high-level `HaDecision` enum and `impl HaDecision { fn lower(&self) -> HaEffectPlan }`.
- [ ] Implement or closely match the explicit lowering contract shape:
  - `impl HaDecision { fn lower(&self) -> HaEffectPlan }`
- [ ] Remove `pending: Vec<HaAction>` from `HaState`.
- [ ] Remove `pending_actions` from the HA state API/CLI surface.
- [ ] Add decision-outcome logging or debug-event emission based on `HaDecision` so observability does not depend on persisted pending plan state.
- [ ] Replace string-based HA phase/trust exposure in API/CLI with typed serialized enums suitable for direct assertions in tests.
- [ ] Define `HaEffectPlan` as a fixed-structure bundle with explicit concern buckets for lease, switchover, replication, postgres lifecycle, and safety.
- [ ] Define the lowered effect enums so that the intended post-restore-removal effect set is represented explicitly:
  - `LeaseEffect::{None, AcquireLeader, ReleaseLeader}`
  - `SwitchoverEffect::{None, ClearRequest}`
  - `ReplicationEffect::{None, FollowLeader { leader }, RecoverReplica { strategy }}`
  - `PostgresEffect::{None, Start, Promote, Demote}`
  - `SafetyEffect::{None, FenceNode, SignalFailSafe}`
- [ ] Define `RecoveryStrategy` explicitly, including `Rewind { leader }`, `BaseBackup { leader }`, and `Bootstrap`.
- [ ] Define `StepDownReason` explicitly, including at least `Switchover`, `ForeignLeaderDetected`, and `FailSafe`.
- [ ] Ensure `HaDecision::lower()` does not accept `DecisionFacts`, `WorldSnapshot`, or any equivalent world-derived context.
- [ ] Ensure the `HaDecision` shape and lowered typed effect plan make duplicate/conflicting effects impossible or explicit enough that post-hoc dedupe is no longer the primary correctness mechanism.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Explicitly skip `make test-long` and direct long HA cargo-test invocations in this task; long-test validation is deferred to task `06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`
</acceptance_criteria>
