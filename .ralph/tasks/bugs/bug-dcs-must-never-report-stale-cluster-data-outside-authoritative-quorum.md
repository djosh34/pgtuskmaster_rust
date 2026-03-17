## Bug: DCS must never report stale cluster data outside authoritative quorum <status>done</status> <passes>true</passes>

<description>
The intended product invariant is stricter than the behavior that was reintroduced during `.ralph/tasks/story-general-architecture-improvement-finding/07-task-collapse-duplicate-struct-trees-into-canonical-domain-adts-and-prove-the-struct-count-went-down.md`: DCS has only two meaningful states, and stale or reused cluster data is never allowed outside authoritative quorum.

The required model is:
- `Quorum(data in dcs)` when there is authoritative quorum majority
- `NoQuorum` when there is not

There is no third middle state, no degraded-but-still-visible member set, no observed-members exception, and no retention of old cluster pictures when DCS authority is lost. If DCS is not authoritative, the system must go directly into fail-safe and expose only the minimal local/disconnected state. It must not keep, reuse, or republish stale member visibility for any reason.

This bug was detected from the postmortem on the referenced task: tests were over-asserting unimportant behavior and encoded a different product decision than the intended safety invariant. That caused the redesign to preserve information that should have been dropped, which reintroduced a three-state model. Under the intended semantics, any test that expects visible members outside authoritative quorum is wrong and must be rewritten rather than driving product behavior.

Explore and research the codebase first, then fix the DCS/HA state model and the tests together so the product enforces the strict two-state invariant. Do not preserve or mask stale authority through disconnect-path retention, cached observed-members views, or any other form of reused cluster state.
</description>

<acceptance_criteria>
- [x] DCS and HA state handling expose only two authority states in this area: authoritative quorum with cluster data, or non-authoritative fail-safe with no reused cluster data
- [x] No code path keeps, republishes, or derives visible member state from stale DCS data once authoritative quorum is lost
- [x] Tests that currently require member visibility or reused cluster pictures outside authoritative quorum are rewritten to assert the stricter fail-safe behavior instead
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this change impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Research summary
- `src/dcs/state.rs` still exposes a three-state DCS model: `NotTrusted`, `Degraded(DcsObservedCluster)`, and `Quorum(DcsQuorumState)`. The `Degraded` branch still carries member visibility and observed leadership outside authoritative quorum, which directly violates the bug invariant.
- `src/ha/worker.rs` currently derives `LeadershipView`, `PrimaryObservation`, `peers`, `switchover`, replica upstream resolution, and local identity fallbacks from `dcs.members()` and `dcs.observed_leadership()` even when DCS is not authoritative.
- `src/ha/decide.rs` encodes non-authoritative DCS as a publication state via `NoPrimaryProjection::DcsDegraded` and also reuses stale observed lease data through `NoPrimaryProjection::StaleObservedLease`.
- Operator surfaces and tests still treat non-authoritative DCS visibility as a real cluster picture. The main examples are `src/command/mod.rs`, `src/cli/status.rs`, `tests/ha/support/steps/mod.rs`, `tests/ha/support/world/mod.rs`, and `tests/ha/support/observer/pgtm.rs`, all of which count or rank `status.dcs.members()` outside quorum.

### Type design completed in this pass
- `src/dcs/state.rs` now collapses DCS authority to the only two valid states for this bug:
  - `DcsSnapshot::Quorum(DcsQuorumState)`
  - `DcsSnapshot::NoQuorum`
- Non-authoritative DCS payloads were deleted entirely in the type layer. There is no longer a `DcsObservedCluster`, no `Degraded` variant, and no `observed_leadership` field on `NoQuorum`.
- `current_snapshot(...)` now returns `NoQuorum` whenever etcd is unreachable or member quorum is missing, which makes stale-member retention impossible at the DCS ADT boundary.
- `src/ha/types.rs` now reshapes coordination around authority instead of around always-present DCS snapshots:
  - `GlobalKnowledge` no longer carries top-level `switchover` or `peers`
  - `CoordinationState` is now `NoQuorum | Quorum(QuorumCoordinationState)`
  - `QuorumCoordinationState` owns authoritative `dcs`, `leadership`, `primary`, `switchover`, and `peers`
- `LeadershipView` no longer has `StaleObservedLease`. Outside quorum there is no authoritative lease view at all; the state must be represented as `CoordinationState::NoQuorum`.
- `NoPrimaryProjection` no longer models degraded or stale-observed DCS as separate states. It now begins with the explicit fail-safe boundary `NoQuorum { fence }`, and any remaining no-primary reasons are quorum-scoped (`LeaseOpen`, `Recovering`, `SwitchoverRejected`).
- This intentionally breaks compilation. The next pass must thread the new ADTs through HA, API/CLI, and HA harness code instead of recreating the deleted stale-data paths.

### Execution plan
1. Refactor `src/ha/worker.rs` so `build_global_knowledge` returns `CoordinationState::NoQuorum` without peers, switchover, observed primary, or lease-derived leadership when DCS is not authoritative.
2. Rework `src/ha/decide.rs` to branch on `CoordinationState::{NoQuorum, Quorum(..)}` instead of `dcs.is_quorum()`, and replace `DcsDegraded` / `StaleObservedLease` publication with the single fail-safe projection `NoPrimaryProjection::NoQuorum { fence }`.
3. Fix `src/ha/reconcile.rs`, `src/ha/state.rs`, and related HA tests to consume `QuorumCoordinationState` only when authoritative data exists.
4. Update API and CLI surfaces so switchover validation, state projections, warnings, and table rendering do not expose or depend on member visibility outside quorum.
5. Rewrite HA harness helpers and features that currently assert visible members or reused cluster pictures after quorum loss so they assert the stricter fail-safe semantics instead.
6. After the design still proves correct in execution, run the required validation gates in repo order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after all checks pass, update docs using `k2-docs-loop`, remove stale docs if needed, then set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

### Constraints for execution
- Do not reintroduce any DCS variant or helper that carries members, switchover state, or leadership outside authoritative quorum.
- Do not rebuild stale member visibility in HA, CLI, or tests via cached `PublicationState`, prior `NodeState`, observer seed selection, or any other indirect retention path.
- If execution shows this ADT boundary is still wrong, switch this task back to `TO BE VERIFIED`, explain the type/design gap precisely, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

NOW EXECUTE
