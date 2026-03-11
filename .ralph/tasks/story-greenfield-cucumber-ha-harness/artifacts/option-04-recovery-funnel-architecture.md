# Option 4: Recovery Funnel Architecture

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA design appears to have too many separate branches for "what should a non-primary do next?" The repo has startup-specific branching, rejoin-specific branching, process-dispatch-specific branching, rewind fallback branching, and quorum/failsafe branching. The user complaint about drift is not only about leader election. It is also about the fact that once a node is not the active leader, too many places compete to decide whether that node should follow, wait, rewind, basebackup, bootstrap, or stand down.

The differentiator of Option 4 is that every non-primary lifecycle path is collapsed into one typed recovery funnel. Leadership still matters, and quorum still matters, but once the pure decider concludes that this node should not presently act as writable primary, all remaining node behavior is derived from one recovery planner with one ordered convergence sequence. Startup is no longer special. Rejoin is no longer special. "Old primary after failover" is no longer a special case with ad hoc branching. They all become inputs into one recovery funnel that chooses among a small, explicit set of convergence stages.

Option 1 centered a unified lifecycle kernel. Option 2 centered the split between cluster intent and local execution. Option 3 centered lease authority. Option 4 instead asks a narrower but still structural question: if we can make the non-primary path exact, monotonic, and unified, can we remove most of the current ambiguity around restart, failover, rejoin, rewind, basebackup, and service restoration? This option says yes.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign studies.
- Relevant failure themes previously observed from `target/nextest/ultra-long/junit.xml` and exported logs remain applicable:
  - quorum-loss scenarios did not consistently surface the expected `fail_safe` evidence
  - degraded-majority scenarios did not consistently expose a new primary from the healthy majority
  - some restore-service and restart scenarios left a node writable when the scenario expected service to remain blocked
  - targeted switchover toward a degraded replica succeeded when it should have been rejected
  - rewind-to-basebackup fallback evidence was not consistently visible
  - storage-stall and killed-primary scenarios left old-primary authority active too long
  - rejoin and convergence behavior after failover remained ambiguous rather than following one clean path

These failures do not prove that Option 4 is correct. They do reinforce the user's complaint that the current startup, authority, and convergence logic is too spread out and that too many branches are deciding the non-primary path.

## Ten option set for the overall task

This document remains one member of the fixed ten-option design set:

1. `Option 1: Unified Observation-Reconciliation Kernel`
2. `Option 2: Dual-Layer Cluster/Local State Machine`
3. `Option 3: Lease-First Authority Core`
4. `Option 4: Recovery Funnel Architecture`
5. `Option 5: Receiver-Owned Work Queue`
6. `Option 6: Epoch-and-Generation Topology Model`
7. `Option 7: Member-Publication-First Control Plane`
8. `Option 8: Intent Ledger With Reconciliation Snapshots`
9. `Option 9: Safety Gate Plus Role Machine`
10. `Option 10: Startup-As-Synthetic-Ticks`

Option 4 is the most recovery-centric proposal in the set. The main design bet is that the cleanest way to fix startup/rejoin drift is to stop modeling startup, replica repair, old-primary repair, and fresh-node onboarding as different architectural families. Instead, there is one recovery funnel with one typed ordering of choices.

## Option differentiator

The specific differentiator of this option is:

- there is a single pure `RecoveryPlanner`
- every non-primary tick flows through it
- the planner always evaluates the same ordered convergence ladder
- recovery choices are represented as a typed funnel stage rather than a scattered collection of booleans and helper branches

The funnel order is:

1. `ContinueFollower`
2. `WaitForLeaderEvidence`
3. `AcquireMissingState`
4. `RewindToLeader`
5. `BasebackupFromLeader`
6. `BootstrapNewClusterMember`
7. `HoldFenced`

The ordering is not cosmetic. It encodes the user's requested convergence preference:

- healthy follow if already good
- tolerate acceptable lag without unnecessary churn
- rewind on wrong timeline when possible
- fall back to basebackup when rewind is impossible
- treat previously-primary, previously-replica, and freshly-restored nodes as variations of the same convergence path when a healthy leader exists

That ordering becomes the heart of the architecture.

## Current design problems

### 1. Startup logic is split across `src/runtime/node.rs`

`src/runtime/node.rs` currently contains `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. Those functions make decisions that are not mere boot mechanics. They decide whether existing local state is usable, whether the node should initialize, whether it should follow, and how it should begin participating.

That split means startup is a separate family of decision-making with its own vocabulary. Later, once workers are running, the HA loop uses a different vocabulary and a different control surface. This is the exact drift the user wants removed.

In Option 4, startup is simply the first pass through the same recovery funnel. The node starts with many facts unknown, but the planner is the same planner.

### 2. Sender-side dedup in `src/ha/worker.rs` is answering a receiver-owned question

`src/ha/worker.rs` currently includes sender-side suppression such as `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the HA sender is trying to infer whether the process receiver is already doing equivalent work.

This is a poor fit for a recovery-heavy system because recovery work is precisely where long-running, retrying, superseding actions occur. A rewind may be underway. A basebackup may be half complete. A follow request may already be satisfied. A sender only knows what it asked for, not the actual receiver state.

Option 4 moves all deduplication into receiver-owned execution slots keyed by recovery stage and leader identity. The HA decider emits monotonic recovery intents. The process runner owns replay suppression, supersession, and active-work replacement rules.

### 3. HA logic is spread across too many boundaries

The current architecture has truth distributed across:

- `src/runtime/node.rs` for startup planning
- `src/ha/decide.rs` for phase selection
- `src/ha/lower.rs` for effect selection
- `src/ha/process_dispatch.rs` for local process and rejoin derivation
- `src/dcs/worker.rs` and `src/dcs/state.rs` for publication and trust interpretation

This spread is particularly harmful on the recovery side. A future implementer must inspect multiple modules to understand whether a stopped node should remain stopped, follow, rewind, basebackup, or attempt bootstrap.

Option 4 narrows the answer:

- leadership authority is still decided in the pure HA decider
- any non-primary path is then delegated to one pure `RecoveryPlanner`
- lowering converts that planner output into typed actions
- receivers own execution and idempotency

That means the system now has one authoritative place for non-primary convergence rules.

### 4. The current non-full-quorum shortcut in `src/ha/decide.rs` is too blunt

The current design routes any non-`DcsTrust::FullQuorum` state into a fail-safe-oriented path. That erases important distinctions:

- healthy majority versus isolated minority
- partial membership visibility versus no safe authority signal
- degraded-but-valid election context versus actual unsafe split-brain risk

This matters directly to the recovery funnel because a node should not enter the same recovery posture in all three cases. A minority old primary should fence and enter `HoldFenced`. A healthy majority replica should continue or promote according to authority. A recovering node in a healthy majority should still use the recovery funnel to converge toward service restoration.

Option 4 therefore separates:

- `AuthorityStatus`, which answers whether a writable leader may exist
- `RecoveryEligibility`, which answers whether non-primary convergence may proceed

The node does not blindly fall into the same fail-safe bucket.

### 5. Startup and rejoin logic in `src/ha/process_dispatch.rs` is too branchy

`src/ha/process_dispatch.rs` currently contains startup intent bridging and source selection logic such as `start_intent_from_dcs(...)`, `start_postgres_leader_member_id(...)`, and source validation around rewind and basebackup. Those are all legitimate concerns, but their current placement makes recovery behavior feel like a collection of helper-condition exceptions.

Option 4 does not deny the need for that detail. It changes the shape. Process dispatch stops choosing the recovery strategy. It only materializes the already-chosen funnel stage into concrete process work.

### 6. Member publication needs to preserve partial truth, especially during recovery

The user explicitly wants "pginfo failed but pgtuskmaster is up" style partial truth to remain publishable. `src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already suggest the system can model partial truth, but the current flow does not treat recovery status as a first-class published signal.

Option 4 adds typed recovery publication fields so a node can publish:

- process reachability
- last-known local role
- recovery funnel stage
- leader target, if any
- uncertainty reason, if observation is partial

That means a recovering node is not silent just because it is not yet healthy enough to serve.

## Proposed architecture overview

This option introduces a new pure planning shape with three successive layers:

1. `ObservationAssembler`
   Builds the newest unified snapshot from DCS facts, pginfo facts, process facts, local persistent markers, and timing metadata.
2. `AuthorityDecider`
   Determines whether this node is the leader, may compete for leadership, must stand down, or must fence.
3. `RecoveryPlanner`
   If the node is not the active writable leader, chooses the single current recovery funnel stage and the typed reasons behind it.

The high-level rule is:

- leader path answers whether writes may occur
- non-leader path always runs through the same recovery funnel

## Proposed control flow from startup through steady-state

There is no startup planner separate from the HA model.

Every tick, including the first tick after process start, executes this sequence:

```text
Newest observations
    |
    v
+-----------------------+
| ObservationAssembler  |
| builds UnifiedFacts   |
+-----------------------+
    |
    v
+-----------------------+
| AuthorityDecider      |
| decides authority     |
+-----------------------+
    |
    +------------------------------+
    |                              |
    v                              v
leader-authorized?               not leader-authorized
    |                              |
    v                              v
+-------------------+      +----------------------+
| LeaderPlan        |      | RecoveryPlanner      |
| keep/acquire/lose |      | pick one funnel step |
+-------------------+      +----------------------+
    |                              |
    +--------------+---------------+
                   |
                   v
         +----------------------+
         | Lowering layer        |
         | typed effect intents  |
         +----------------------+
                   |
                   v
         +----------------------+
         | Receivers own dedup   |
         | and work execution    |
         +----------------------+
```

The key consequence is that startup now becomes a natural case:

- if there is already a healthy leader and local data is aligned, the first tick chooses `ContinueFollower`
- if there is already a healthy leader but local data is on the wrong timeline, the first tick chooses `RewindToLeader`
- if there is already a healthy leader and rewind cannot work, the first tick chooses `BasebackupFromLeader`
- if no valid leader exists but this node has majority-backed authority, the leader branch handles promotion/bootstrap
- if authority is unsafe or ambiguous, the first tick chooses `HoldFenced` or `WaitForLeaderEvidence`

There is no separate startup family after this change.

## Typed state machine

This option keeps a pure decision core and introduces explicit recovery stages.

### Core observation types

The future implementation would introduce a unified snapshot such as:

```text
UnifiedFacts
- cluster_observation: ClusterObservation
- local_observation: LocalObservation
- lease_observation: LeaseObservation
- publication_observation: PublicationObservation
- timing: TimingObservation
```

Important derived facts include:

- `majority_status`
- `leader_evidence`
- `local_data_state`
- `timeline_relation_to_leader`
- `local_process_health`
- `latest_member_truth`
- `can_publish_dcs`

### Authority decision types

The first pure output is:

```text
AuthorityOutcome
- authority_status: AuthorityStatus
- leader_target: Option<LeaderTarget>
- fencing_requirement: FencingRequirement
- promotion_eligibility: PromotionEligibility
- reason_chain: Vec<AuthorityReason>
```

Where:

- `AuthorityStatus::ActiveLeader`
- `AuthorityStatus::LeaderCandidate`
- `AuthorityStatus::FollowerRequired`
- `AuthorityStatus::UnsafeMinority`
- `AuthorityStatus::UnknownAuthority`

This is deliberately narrower than a full lifecycle state machine. The point is to decide authority first and then hand off any non-leader action to the recovery funnel.

### Recovery planner types

The second pure output exists whenever the node is not presently an active writable leader:

```text
RecoveryOutcome
- stage: RecoveryStage
- target_leader: Option<LeaderTarget>
- local_data_policy: LocalDataPolicy
- publication_mode: RecoveryPublicationMode
- retry_policy: RetryPolicy
- reason_chain: Vec<RecoveryReason>
```

Where `RecoveryStage` is:

- `ContinueFollower`
- `WaitForLeaderEvidence`
- `AcquireMissingState`
- `RewindToLeader`
- `BasebackupFromLeader`
- `BootstrapNewClusterMember`
- `HoldFenced`

The planner evaluates those stages in order and stops at the first valid stage with strongest confidence.

### Funnel-stage invariants

`ContinueFollower`
- A valid leader target exists.
- Local data is already aligned enough to stream or continue streaming.
- No stronger repair operation is needed.
- The node remains read-only if it is not the leader.

`WaitForLeaderEvidence`
- The node is not allowed to invent a leader.
- Current observations are insufficient to choose safe follow, rewind, basebackup, or bootstrap.
- Local writes remain fenced.
- Publication must explain the uncertainty rather than falling silent.

`AcquireMissingState`
- A leader target likely exists, but local observation is too incomplete to choose between follow/rewind/basebackup.
- Examples: pginfo unavailable, control data unreadable, or timeline relation unknown.
- Additional observation or local introspection work is required before process actions.

`RewindToLeader`
- A valid leader target exists.
- Local data is on the wrong timeline or reflects a previously-primary identity that can be safely rewound.
- Rewind prerequisites are satisfied or close enough to attempt.

`BasebackupFromLeader`
- A valid leader target exists.
- Follow is impossible and rewind is impossible or failed terminally.
- Fresh replica acquisition is the next safe convergence step.

`BootstrapNewClusterMember`
- No valid leader exists and authority logic allows this node to become part of cluster creation.
- Local data policy allows bootstrap or initialization.
- This stage is never reached merely because follow failed; it requires explicit cluster-creation conditions.

`HoldFenced`
- The node must not write.
- The node must not attempt speculative promotion.
- This includes minority old-primary cases, lease-unsafe cases, and post-cutoff authority loss.

### State transitions

The funnel transitions are monotonic with explicit reasons:

```text
ContinueFollower -> RewindToLeader
  when leader exists and timeline mismatch is detected

RewindToLeader -> BasebackupFromLeader
  when rewind prerequisites are impossible or rewind fails terminally

AcquireMissingState -> ContinueFollower
  when new local facts show alignment is already acceptable

AcquireMissingState -> RewindToLeader
  when new local facts show wrong-timeline divergence

AcquireMissingState -> BasebackupFromLeader
  when new local facts show local state is unusable for rewind

WaitForLeaderEvidence -> ContinueFollower
  when majority-backed leader evidence becomes available

WaitForLeaderEvidence -> BootstrapNewClusterMember
  when cluster creation conditions become valid and authority allows it

Any non-fenced stage -> HoldFenced
  when majority authority is lost or lease safety is lost
```

This makes the recovery path understandable. A future implementer can reason about one ordered ladder instead of several helper families.

## Quorum model redesign

This option does not treat any loss of full quorum as equivalent.

The future implementation should replace the single blunt boundary with explicit categories:

- `MajorityHealthy`
- `MajorityDegradedButValid`
- `MinorityIsolated`
- `NoSafeClusterView`

The leader branch and the recovery funnel consume these categories differently.

### Degraded-but-valid majority

If two nodes out of three remain healthy and can still observe enough DCS state to elect or preserve a leader, the cluster is degraded but still valid. In that case:

- the current leader may remain leader if it still holds safe authority
- a new leader may be elected if the old primary is outside the healthy majority
- replicas in the healthy majority continue through the normal funnel toward service restoration
- old primary outside the majority must enter `HoldFenced`

This directly addresses the user complaint that the system should not blindly demote into the current primary fail-safe path when a valid majority can still function.

### Minority isolation

If a node is isolated from the healthy majority, it does not get to reinterpret that as a local degraded mode. It enters `HoldFenced`. If it was previously primary, it must relinquish any local write authority and publish that it no longer has safe majority backing.

### No safe cluster view

If observations are too incomplete to determine whether a valid majority-backed leader exists, the system uses `WaitForLeaderEvidence` or `AcquireMissingState`, not speculative promotion. This is especially important on startup and immediately after partial DCS outages.

## Lease model redesign

Option 4 does not make lease semantics the only abstraction, but it does make lease state decisive for leader permission and for recovery posture.

### Lease acquisition

Only the leader branch may pursue lease acquisition or renewal. The recovery funnel never attempts writable leadership actions.

### Lease expiry and leader loss

If the node loses leader lease backing, that is not treated as a routine role shuffle. It causes one of two outcomes:

- if the node still has majority-backed renewal evidence and the lease is renewable, it remains leader
- otherwise it exits leader authorization and enters the recovery side, typically `HoldFenced` first, then later a non-primary convergence stage once a replacement leader is identified

### Killed primary and lost authority

A killed primary or stalled primary loses authority because lease renewal and majority backing cannot both remain true indefinitely. Option 4 makes that visible in the planner:

- first, leader authority fails
- second, the old primary becomes a non-leader node
- third, if it later returns, it goes through the same recovery funnel as any other non-primary

There is no separate "old primary rejoin mode." That is a direct simplification.

### Lease interaction with startup

On startup, lease evidence is only one observation input. If a valid leader lease is already present, startup enters the non-primary funnel and follows that leader. If no valid leader exists and this node has authority to bootstrap or promote, the leader branch handles that. If lease evidence is ambiguous, the node waits fenced rather than speculating.

## Startup reasoning redesign

Startup is now a standard funnel entry rather than a separate code family.

### Case: cluster already up and leader healthy

The first tick sees leader evidence and local state facts. The planner asks:

- Is local data already aligned?
- Is the node already a suitable follower?
- Is rewind required?
- Is basebackup required?

The answer is one funnel stage, not a runtime-specific startup path.

### Case: leader already present but local member publication absent

The node may still enter `ContinueFollower` or `AcquireMissingState` while publishing partial truth. Publication absence does not force special startup logic.

### Case: empty `pgdata`

If a valid leader exists, empty local data naturally leads to `BasebackupFromLeader`.

If no leader exists and this node wins the init lock under safe authority rules, the leader branch may allow bootstrap. The recovery funnel itself does not invent bootstrap merely because `pgdata` is empty.

### Case: existing `pgdata`

Existing `pgdata` is treated as evidence to classify:

- reusable follower state
- rewindable divergent state
- unusable state needing basebackup
- bootstrap-eligible local state only when cluster creation conditions explicitly allow it

This directly addresses the user's request to reconsider whether existing `pgdata` may still be used when the node wins the init lock.

## Replica convergence as one coherent sequence

This is the core of Option 4.

The convergence sequence is always:

1. prefer `ContinueFollower` if already healthy enough
2. otherwise gather missing evidence if required
3. if divergence exists and rewind is valid, choose `RewindToLeader`
4. if rewind is impossible or terminally failed, choose `BasebackupFromLeader`
5. only use bootstrap when creating cluster membership is truly the intended authority outcome

That means previously-primary, previously-replica, and freshly-restored nodes are no longer separate design families. They are different inputs to the same convergence funnel.

### Healthy follow

If the node has acceptable replication alignment, it remains on `ContinueFollower`. Minor lag is tolerated according to typed thresholds rather than forcing churn.

### Wrong timeline

If the node diverged because it was an old primary or otherwise followed a dead branch, the funnel selects `RewindToLeader` when rewind prerequisites permit.

### Basebackup fallback

If rewind is impossible because required WAL history or prerequisites are unavailable, or if rewind failed terminally, the funnel escalates once to `BasebackupFromLeader`.

### Broken replica does not destabilize quorum

If a single replica is broken, the funnel keeps that node in repair stages without destabilizing leader authority or majority-backed service. This is a direct benefit of separating authority from recovery stage.

## Partial-truth member publication

This option explicitly strengthens publication during recovery.

The future `MemberRecord` or equivalent published structure should include fields such as:

- `agent_reachable`
- `pginfo_status`
- `last_known_postgres_role`
- `recovery_stage`
- `leader_target_member_id`
- `local_data_state`
- `timeline_relation`
- `write_safety`
- `observation_uncertainty`

This preserves the user's requirement that "pginfo failed but pgtuskmaster is up" remains publishable truth.

Examples:

- A node with unreachable pginfo but an alive process manager publishes `agent_reachable=true`, `pginfo_status=unknown`, `recovery_stage=AcquireMissingState`.
- A fenced old primary publishes `write_safety=fenced`, `recovery_stage=HoldFenced`, and the last known leader target if available.
- A node midway through basebackup publishes `recovery_stage=BasebackupFromLeader` rather than going silent.

That richer publication model also gives other nodes better inputs for leader choice and convergence reasoning.

## Deduplication boundary redesign

Option 4 moves dedup entirely away from sender-side HA logic.

The HA side emits typed intents like:

- `EnsureFollower { leader_id, stage_token }`
- `EnsureRewind { leader_id, stage_token, source_timeline }`
- `EnsureBasebackup { leader_id, stage_token }`
- `EnsureFenced { reason_token }`

Receivers then own dedup using stable semantic keys:

- one active fence job per node
- one active rewind job per `(leader_id, local_generation)`
- one active basebackup job per `(leader_id, bootstrap_generation)`
- one active follow configuration per `(leader_id, recovery_generation)`

This is safer than sender-side suppression because only the receiver knows:

- whether a job is active
- whether it already completed successfully
- whether it failed terminally
- whether a new intent supersedes an older one

The sender simply keeps describing desired state. The receiver decides whether any physical action is necessary.

## Concrete repo areas a future implementation would touch

This option would require later changes in at least these areas:

- `src/runtime/node.rs`
  remove or collapse `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)` into unified observation assembly and first-tick execution
- `src/ha/worker.rs`
  change from mixed phase-plus-dedup orchestration into observation assembly, authority decision, recovery planning, and intent emission
- `src/ha/decide.rs`
  narrow leadership and safety decisions to explicit authority outputs rather than blunt phase branching
- `src/ha/decision.rs`
  add `AuthorityOutcome`, `RecoveryOutcome`, `RecoveryStage`, and typed reason structures
- `src/ha/lower.rs`
  lower leader plans and recovery funnel stages into receiver-owned intents
- `src/ha/process_dispatch.rs`
  stop choosing strategy, only materialize chosen recovery stages into concrete process jobs
- `src/dcs/worker.rs`
  publish richer partial-truth recovery status
- `src/dcs/state.rs`
  support stronger trust categories and richer member publication types
- `src/pginfo/state.rs`
  ensure missing/unknown/local-state facts map cleanly into `AcquireMissingState` and other funnel stages
- `tests/ha.rs`
  keep scenario inventory aligned to new state names and new observability signals
- `tests/ha/features/`
  later update assertions to verify the new recovery-stage observability and authority boundaries

## All meaningful changes required for this option

The future implementation would need to make all of these changes explicitly:

- introduce a pure `ObservationAssembler`
- introduce a pure `AuthorityDecider`
- introduce a pure `RecoveryPlanner`
- add a typed `RecoveryStage` enum with ordered semantics
- delete special startup planner paths from runtime
- remove sender-side process dedup logic from `src/ha/worker.rs`
- move process idempotency and supersession into receivers
- replace blunt non-full-quorum fail-safe branching with explicit majority categories
- add leader-target identity and timeline-relation facts to the unified snapshot
- treat old-primary repair and ordinary replica repair as one funnel family
- publish recovery-stage information into member records
- make `process_dispatch` a materializer instead of a strategist
- add explicit escalation from rewind to basebackup
- add explicit waiting states for missing evidence rather than overloading fail-safe
- ensure bootstrap is reachable only from safe cluster-creation authority, not as an accidental recovery fallback
- update logging/telemetry so operators can see funnel stage and stage transitions

If any of those changes are omitted, the design risks retaining legacy branch families and therefore losing the main benefit of the option.

## Migration sketch

The future implementation should migrate in stages without keeping stale parallel paths alive.

### Stage 1: introduce types before deleting logic

Add the new `AuthorityOutcome`, `RecoveryOutcome`, and `RecoveryStage` types in `src/ha/decision.rs`. Add structured logs for stage decisions.

### Stage 2: make `process_dispatch` materialize only explicit stages

Refactor `src/ha/process_dispatch.rs` so it can consume an already-selected recovery stage. At this point it may still be fed by temporary adapter code, but it must stop embedding strategy choice.

### Stage 3: add `RecoveryPlanner`

Move rejoin/follow/rewind/basebackup strategy selection into the pure planner. Preserve behavior temporarily by mapping old decision inputs into the new planner outputs.

### Stage 4: collapse startup into normal ticks

Delete `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)` once the first-tick observation path can drive the funnel correctly.

### Stage 5: remove sender-side dedup

Delete `should_skip_redundant_process_dispatch(...)`, `decision_is_already_active(...)`, and any related sender inference. Replace them with receiver-owned semantic dedup keys.

### Stage 6: tighten trust categories and member publication

Expand DCS trust interpretation and member publication so the planner gets the richer categories it needs.

### Stage 7: delete stale legacy branches

Remove any adapter shims that duplicate the funnel decision path. This project has no users and should not keep backward-compatible dead architecture around.

## Non-goals

This option intentionally does not try to:

- preserve the current startup planner as a compatibility layer
- keep sender-side dedup for convenience
- model every non-primary path as a separate dedicated sub-architecture
- treat all non-full-quorum situations as operationally identical
- use silent absence instead of partial publication during recovery

## Tradeoffs

This option has real costs.

- It strongly centralizes non-primary behavior, which may make the recovery planner large.
- It depends on good observation typing; poor fact modeling would make the funnel obscure rather than clarifying.
- It does not by itself solve every leader-authority nuance as explicitly as Option 3.
- Receiver-side dedup requires stronger execution-state reporting from process workers.
- Migration discipline matters. If the implementation keeps both old startup paths and the new funnel, the architecture will get worse, not better.

The reason to accept these tradeoffs is that the current pain appears concentrated in recovery and rejoin ambiguity. This option aims directly at that center of gravity.

## Logical feature-test verification

This section explains how Option 4 would logically satisfy the key HA scenarios named in the task. This is not an implementation claim. It is a design-level mapping from the proposed architecture to expected behavior.

### `ha_dcs_quorum_lost_enters_failsafe`

If a node loses safe majority backing and cannot confirm writable authority, `AuthorityOutcome` becomes `UnsafeMinority` or `UnknownAuthority`. The recovery side chooses `HoldFenced`. Operator-visible state should surface fencing explicitly rather than burying it behind an ambiguous role update.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

The same `HoldFenced` stage blocks writes after authority loss. Because fencing is a stable recovery stage with receiver-owned idempotency, the system can keep reasserting fence intent without sender guesswork.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side still has valid authority and may elect or preserve a leader. The isolated old primary becomes `HoldFenced`. Healthy replicas in the majority use `ContinueFollower` or promotion on the leader side. The key boundary change is that degraded-but-valid majority does not collapse into the same path as minority isolation.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

Once the old primary reconnects, it is no longer a special role family. It becomes a non-primary node entering the recovery funnel. If its timeline diverged, the funnel chooses `RewindToLeader`; if rewind is impossible, it chooses `BasebackupFromLeader`. That is exactly the kind of path simplification this option is meant to provide.

### `ha_primary_killed_then_rejoins_as_replica`

The killed primary loses authority. When it later returns, the leader branch does not restore writable authority just because of local history. It enters the recovery funnel and converges to the new leader through `ContinueFollower`, `RewindToLeader`, or `BasebackupFromLeader` as required.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

The restarted node does not need a startup-special branch. It observes the healthy majority context, identifies the valid leader, and enters the funnel. If its local state is already suitable, it lands on `ContinueFollower`, restoring quorum-backed service without unsafe promotion.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first restarted nodes use authority rules to determine whether a valid leader may be established or restored. The later node then joins through the same non-primary funnel. This avoids the current ambiguity where restart order can accidentally imply different architecture paths.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

This option directly encodes that escalation. `RewindToLeader` is an explicit stage with a defined terminal escalation path to `BasebackupFromLeader`. Telemetry should make that transition visible so the absence of expected rewind evidence becomes diagnosable rather than mysterious.

### `ha_replica_stopped_primary_stays_primary`

A broken or stopped replica remains inside the recovery funnel while leader authority remains unchanged. Because authority and recovery are separated, the broken replica does not destabilize the primary.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

The broken replica can cycle between `AcquireMissingState`, `RewindToLeader`, and `BasebackupFromLeader` without affecting the majority-backed leader. Recovery work is local to the broken node's funnel stage, not a cluster-wide authority shock.

## Q1 Should the funnel own bootstrap or should bootstrap remain a leader-side path?

Context:
Option 4 places bootstrap near the funnel because it is part of the ordered "what should a non-primary-capable node do next?" story. But bootstrap is different from rewind or basebackup because it is cluster creation, not recovery toward an existing leader.

```text
No leader exists
    |
    +--> leader branch allows cluster creation?
             |
             +--> yes: bootstrap possible
             +--> no: remain fenced / wait
```

Problem:
If bootstrap sits inside the funnel, the funnel is no longer strictly "non-primary repair." If bootstrap stays leader-side, the funnel remains cleaner but startup may feel split again.

Restated question:
Should `BootstrapNewClusterMember` be a formal funnel stage, or should the funnel stop at "not leader yet" and let only the leader-authority branch own bootstrap?

## Q2 How much local introspection belongs in `AcquireMissingState`?

Context:
`AcquireMissingState` exists to avoid overloading fail-safe when local facts are incomplete. It may need to trigger control-data reads, process probes, or additional pginfo attempts before the planner can choose follow, rewind, or basebackup.

Problem:
If too much work is hidden inside this stage, the planner becomes vague. If too little work is allowed, the system may bounce between uncertainty and unnecessary destructive repair.

Restated question:
What exact local observations are allowed to be actively acquired inside `AcquireMissingState`, and which ones must remain passive inputs only?

## Q3 Should rewind failure always escalate to basebackup immediately?

Context:
The funnel currently says terminal rewind failure becomes `BasebackupFromLeader`.

```text
RewindToLeader
    |
    +--> transient failure -> retry?
    |
    +--> terminal failure -> BasebackupFromLeader
```

Problem:
Some rewind failures are clearly terminal; others may be environmental and worth retrying. Too eager an escalation may destroy useful state. Too conservative a retry policy may stall convergence.

Restated question:
What exact criteria should distinguish retryable rewind failure from terminal rewind failure that must escalate to basebackup?

## Q4 How should publication encode uncertainty without creating false confidence?

Context:
This option publishes richer recovery truth, including partial information and uncertainty reasons.

Problem:
More detail improves diagnosis, but publication fields may be misread as stronger evidence than they really are. For example, "last known role primary" must not be read as "safe to write now."

Restated question:
Which publication fields should be explicitly tagged as historical or uncertain so operators and other nodes do not confuse stale local memory with current authority?

## Q5 Should the funnel stage be persisted locally across restarts?

Context:
If a node restarts midway through rewind or basebackup, the first tick could recompute the funnel stage from observations alone, or it could consult a persisted local marker indicating previously chosen stage and attempt identity.

Problem:
Pure recomputation is cleaner, but persisted markers may improve continuity and operator debugging. Persisted markers also risk preserving stale intent longer than they should.

Restated question:
Should recovery funnel stages be fully recomputed from fresh observations on every restart, or should there be a bounded persisted execution marker that informs the first post-restart tick?
