---
## Task: Implement pure HA decide engine with exhaustive transition tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement deterministic HA decision logic as a pure function with exhaustive matrix coverage.

**Scope:**
- Implement `src/ha/state.rs`, `src/ha/decide.rs`, `src/ha/actions.rs`, `src/ha/mod.rs`.
- Implement `DecideInput`, `DecideOutput`, `decide`, and action-id dedupe semantics.
- Cover transitions across `HaPhase` including `FailSafe`, `Rewinding`, `Bootstrapping`, and `Fencing`.

**Context from research:**
- Plan marks `ha::decide` exhaustiveness as critical and requires deterministic behavior.

**Expected outcome:**
- `decide` is side-effect free, reproducible, and fully unit-tested with matrix fixtures.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `decide` uses only `DecideInput` and returns `DecideOutput` (no hidden IO).
- [x] Tests cover normal paths plus split-brain prevention and no-quorum fail-safe transitions.
- [x] Action idempotency tests verify duplicate-action suppression via `recent_action_ids`.
- [x] Run targeted `ha::decide` matrix tests.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] On failures, use `$add-bug` and include failing state snapshot fixture.
</acceptance_criteria>
<execution_plan>
## Detailed Implementation Plan

1. Baseline and prerequisite validation
- [x] Confirm blocker `03-task-worker-state-models-and-context-contracts` is done/passing (already marked done; keep as prerequisite evidence in commit).
- [x] Record baseline by running a targeted compile pass for HA paths (`cargo check -q`) before edits so regressions are attributable.
- [x] Re-open `RUST_SYSTEM_HARNESS_PLAN.md` HA section and align naming/semantics before coding to avoid API churn.

2. Lock pure-decision contracts in `ha::state`
- [x] Keep `DecideInput` strictly as `(current, world)` and `DecideOutput` strictly as `(next, actions)` with no hidden context.
- [x] Ensure all decision inputs required by matrix behavior are available via `WorldSnapshot` fields only; if additional fields are required, add them here rather than reading external state in `decide`.
- [x] Preserve `#[derive(Clone, Debug, PartialEq, Eq)]` on decision-facing structs/enums to support deterministic fixture assertions.

3. Expand HA action model for deterministic orchestration
- [x] Replace placeholder-only `HaAction::Noop` with explicit variants needed by transitions (leadership acquire/release, follow, rewind/bootstrap/fence flow, fail-safe signaling).
- [x] Keep `ActionId` stable and comparable (`Ord`) for idempotency set membership.
- [x] Define action payloads minimally but precisely so tests can assert exact emitted action vectors.

4. Implement pure `decide` transition engine in `src/ha/decide.rs`
- [x] Keep `decide(input)` side-effect free: no IO, no clock reads, no global access, no mutation outside returned `next` state.
- [x] Encode transition logic across `HaPhase` including:
- [x] `Init -> WaitingPostgresReachable` bootstrap progression.
- [x] `WaitingPostgresReachable` behavior based on postgres reachability.
- [x] `WaitingDcsTrusted` gating on DCS trust/quorum.
- [x] `Replica/CandidateLeader/Primary` leader election and split-brain prevention behavior.
- [x] `Rewinding`, `Bootstrapping`, `Fencing` transitions based on process state and outcomes.
- [x] `FailSafe` entry/exit on no-quorum or unsafe cluster signals.
- [x] Increment tick deterministically and compute pending/actions deterministically from input snapshots.

5. Implement action-id idempotency and duplicate suppression semantics
- [x] Derive each candidate action’s `ActionId` deterministically from decision context (phase + intent + stable identifiers).
- [x] Suppress duplicate action emission when action ID exists in `recent_action_ids`.
- [x] Update returned `next.recent_action_ids` to include newly emitted IDs while preserving bounded, deterministic ordering semantics.
- [x] Ensure dedupe is tested for repeated identical inputs across consecutive `decide` invocations.

6. Wire module exports with minimal visibility surface
- [x] Keep `src/ha/mod.rs` exports crate-local and avoid widening public API.
- [x] Ensure worker code compiles against updated action/state/decide contracts without introducing behavior in worker loop.
- [x] Update `src/worker_contract_tests.rs` fixture defaults to remain valid after replacing placeholder `HaAction::Noop` and expanded action IDs.

7. Add exhaustive transition matrix unit tests
- [x] Create focused `#[cfg(test)]` matrix tests for every `HaPhase` and key world-snapshot combinations.
- [x] Include explicit split-brain prevention cases (competing primary signals, inconsistent DCS leadership, stale trust) and assert safe fallback behavior.
- [x] Include explicit no-quorum cases asserting transition to/retention of `FailSafe` and absence of unsafe actions.
- [x] Use table-driven fixtures (`struct Case`) so scenario coverage is legible and extendable.

8. Add idempotency tests for `recent_action_ids`
- [x] Test first call emits expected action IDs and second call with equivalent input suppresses duplicates.
- [x] Test mixed sets where one action ID is known and one is new; assert only new action is emitted.
- [x] Test order determinism of emitted actions/IDs and stable state evolution.

9. Run required verification commands (full gate)
- [x] Run targeted HA tests first to shorten debug loop (e.g. `cargo test ha::decide -- --nocapture` or equivalent exact test path).
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] If any required command fails, create bug task(s) using `$add-bug` with failing snapshot fixture + repro command.

10. Task bookkeeping and completion protocol
- [x] Mark acceptance criteria checkboxes in this task file as each criterion is proven.
- [x] Update header tags to done/passing only after all required commands succeed 100%.
- [x] Set `<passing>true</passing>` when done.
- [x] Run `/bin/bash .ralph/task_switch.sh` after completion.
- [x] Commit all files (including `.ralph` updates) with message: `task finished 07-task-ha-decide-pure-matrix-idempotency-tests: <summary with test evidence and challenges>`.
- [x] Append learnings/surprises to `AGENTS.md`.
- [x] Append diary entry to progress log before exit.
</execution_plan>

