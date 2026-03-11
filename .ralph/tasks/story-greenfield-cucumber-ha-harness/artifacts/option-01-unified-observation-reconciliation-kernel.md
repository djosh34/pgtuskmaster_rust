# Option 1: Unified Observation-Reconciliation Kernel

This is a design artifact only. It does not propose any code changes in this run, it does not attempt to fix tests, and it explicitly treats green test outcomes as a future implementation concern rather than the output of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without needing chat history or repo docs.

## Why this option exists

This option exists to collapse the current split between startup planning and steady-state HA into one typed reconciliation kernel. The differentiator is that there is exactly one authoritative decision entrypoint for the node lifecycle: every tick, including startup, is driven from the newest observable facts into a pure state-machine decision, then into typed lower-level intents, then into receiver-owned idempotent actions.

## Current run diagnostic evidence

The current repo state on March 11, 2026 was used as input evidence for this design.

- `make test` completed successfully in the repo root. That means the default test profile is currently not the source of urgency for this redesign.
- `make test-long` failed in the repo root. The failure pattern is concentrated in HA feature behavior, which is exactly the area this redesign targets.
- Observed failing themes from `target/nextest/ultra-long/junit.xml` and exported logs:
  - `ha_dcs_quorum_lost_enters_failsafe` and `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes` failed because node debug output did not contain `fail_safe`.
  - `ha_old_primary_partitioned_from_majority_majority_elects_new_primary` and `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover` failed because recovery observed no sampled primary under degraded sampling, even though a majority should still be able to elect and expose one.
  - `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum` and `ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken` failed because a node still answered as primary when the scenario expected the query to fail.
  - `ha_targeted_switchover_to_degraded_replica_is_rejected` failed because the targeted switchover succeeded when it should have been rejected.
  - `ha_rewind_fails_then_basebackup_rejoins_old_primary` failed because the expected `pg_rewind wrapper` blocker evidence was not visible in compose logs.
  - `ha_primary_storage_stalled_then_new_primary_takes_over` failed because the stalled primary remained primary past the failover deadline.
  - `ha_primary_killed_with_concurrent_writes` and `ha_replica_flapped_primary_stays_primary` failed on proof-row convergence behavior, which suggests the current authority, demotion, and rejoin path is still too ambiguous.

These results do not prove that this exact option is correct, but they do reinforce the user’s complaint that the current startup, authority, quorum, and convergence boundaries are spread across too many places and do not form one coherent model.

## Ten option set for the overall task

Before writing this document, I fixed a ten-option design set so this artifact can be judged as one option among real alternatives rather than as the only story available.

1. `Option 1: Unified Observation-Reconciliation Kernel`
   One state machine owns startup, steady-state, failover, rejoin, and fencing. That is the option described in this file.
2. `Option 2: Dual-Layer Cluster/Local State Machine`
   A cluster authority state machine chooses desired cluster topology, and a local executor state machine chooses local node behavior.
3. `Option 3: Lease-First Authority Core`
   Lease state becomes the primary abstraction, and leadership/rejoin behavior is derived from lease epoch ownership first, with HA phases secondary.
4. `Option 4: Recovery Funnel Architecture`
   All non-primary paths are collapsed into a single recovery funnel that chooses follow, rewind, or basebackup from one typed recovery planner.
5. `Option 5: Receiver-Owned Work Queue`
   The key change is moving all idempotency and dedup semantics into receiver-side action queues, with the HA decider issuing only monotonic intents.
6. `Option 6: Epoch-and-Generation Topology Model`
   Every leader election, startup decision, and replica rejoin is governed by cluster generation numbers and explicit epoch handoff.
7. `Option 7: Member-Publication-First Control Plane`
   The redesign begins from making DCS member publication authoritative and always populated with partial truth, then drives HA from that stronger substrate.
8. `Option 8: Intent Ledger With Reconciliation Snapshots`
   The decider emits append-only reconciliation intents keyed by observed epochs, and workers materialize them with replay-safe semantics.
9. `Option 9: Safety Gate Plus Role Machine`
   Safety and authority are separate typed layers: one machine answers whether writes are allowed, the other answers what role the node should attempt.
10. `Option 10: Startup-As-Synthetic-Ticks`
   The minimal-change variant keeps most existing modules but removes the special startup planner by replaying synthetic observations through the HA loop.

Option 1 is the most structural of the set without abandoning the current `newest info -> decide -> lower -> actions` functional chain. It does the largest cleanup of responsibility boundaries while still preserving the user-preferred pure decider plus lowerer split.

## Current design problems

### 1. Startup logic is split away from the HA loop

`src/runtime/node.rs` currently contains `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. Those functions make important cluster-role and local-data decisions before the HA worker owns the node lifecycle. That means the actual authoritative logic for "what should this node do now?" is already fragmented before `src/ha/worker.rs` begins its steady-state cycle.

This split creates two different reasoning styles:

- startup uses ad hoc planning from runtime code
- steady-state uses `world_snapshot -> decide -> lower -> publish -> apply`

That is the first architectural break that this option removes.

### 2. Sender-side dedup lives in the HA worker

`src/ha/worker.rs` currently decides not only what the next state is, but also whether some process actions should be suppressed via `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`.

That means the sender is trying to infer receiver progress from stale snapshots. It makes the decider sensitive to process job categories such as `StartPostgres`, `PgRewind`, `BaseBackup`, `Bootstrap`, and `Fencing`. This is exactly the wrong ownership boundary for idempotency. A sender can at most know what it asked for; only the receiver knows whether the work item is active, completed, superseded, or safe to replay.

### 3. HA logic is spread across runtime, decide, lower, and process-dispatch boundaries

The current architecture is directionally good, but the real authority model is still distributed across:

- `src/runtime/node.rs` for startup and cluster discovery
- `src/ha/decide.rs` for phase selection
- `src/ha/lower.rs` for effect selection
- `src/ha/process_dispatch.rs` for startup and recovery process intent derivation

The result is that a human reader cannot point to one place and say, "this is where node lifecycle truth lives."

### 4. Non-full-quorum currently shortcuts directly to fail-safe

`src/ha/decide.rs` currently does:

```text
if !FullQuorum:
    primary -> EnterFailSafe { release_leader_lease: false }
    non-primary -> NoChange in FailSafe
```

That boundary is too blunt for a three-node cluster where two healthy nodes still form a valid majority. The current model effectively conflates:

- majority still available but not full
- minority isolated
- stale or ambiguous visibility

Those are not the same safety state, and the test failures reinforce that the system needs a stronger distinction.

### 5. Startup and rejoin logic is ambiguous inside process dispatch

`src/ha/process_dispatch.rs` still carries authoritative logic about whether a node should start PostgreSQL, follow a leader, rewind, or basebackup. That means some of the lifecycle truth is not in the pure HA decision but in the translation layer that turns decisions into process requests.

The specific smell is that "what should the node become?" and "how do we carry out the local transition?" are still partially interleaved.

### 6. Member publication does not fully embrace partial truth

`src/dcs/worker.rs` publishes `MemberRecord` from current pginfo state, while `src/pginfo/state.rs` already has useful partial states such as:

- `PgInfoState::Unknown`
- `SqlStatus::Unknown`
- `SqlStatus::Unreachable`
- `Readiness::Unknown`
- `Readiness::NotReady`

The user’s requirement is correct: member keys should publish the best obtainable truth even when PostgreSQL health is degraded. Silence is not truth. A node can know "operator alive, SQL probe failed, old role known, local process present" and that should be represented explicitly.

## Proposed control flow

This option introduces one new high-level concept: the `ReconciliationKernel`.

Every node tick, including the very first startup tick, follows this flow:

1. Gather the newest observations from pginfo, DCS, process state, config, local data-dir inspection, init-lock inspection, and runtime host facts.
2. Normalize them into one immutable `ReconcileObservation`.
3. Feed `current_state + observation` into one pure `reconcile_decide(...)` function.
4. Return one `ReconcileOutcome`, which contains the next typed lifecycle state and one typed intent bundle.
5. Lower that bundle into receiver-specific effect envelopes.
6. Publish the state immediately.
7. Let receiver-side workers accept or ignore envelopes based on receiver-owned idempotency keys.

Startup is therefore no longer a separate planner. Startup is just the earliest set of observations, where some facts are `Unknown`.

### Proposed high-level flow diagram

```text
                 +--------------------+
pginfo --------->|                    |
dcs ------------>|                    |
process -------->|                    |
config --------->| Observation        |
data-dir ------->| Assembler          |----+
init-lock ------>|                    |    |
runtime facts -->|                    |    |
                 +--------------------+    |
                                           v
                                  +--------------------+
                                  | reconcile_decide() |
                                  | pure, typed        |
                                  +--------------------+
                                           |
                                           v
                                  +--------------------+
                                  | ReconcileOutcome   |
                                  | - next state       |
                                  | - intent bundle    |
                                  +--------------------+
                                           |
                                           v
                                  +--------------------+
                                  | lower_intents()    |
                                  | pure, typed        |
                                  +--------------------+
                                           |
                +--------------------------+--------------------------+
                |                          |                          |
                v                          v                          v
      +-------------------+     +-------------------+      +-------------------+
      | Process receiver  |     | Lease receiver    |      | DCS receiver      |
      | owns dedup        |     | owns dedup        |      | owns dedup        |
      +-------------------+     +-------------------+      +-------------------+
```

### Responsibility boundary diagram

```text
CURRENT:
runtime startup planner -> ha decide -> lower -> process_dispatch -> process worker
                                 \-> worker-side dedup

PROPOSED:
observation assembler -> reconcile_decide -> lower -> receiver envelopes
                                                  \-> receiver-side dedup only
```

## Proposed typed state machine

The state machine has one top-level node lifecycle enum and a few typed subdomains that make authority, safety, and recovery explicit instead of implicit.

### Top-level lifecycle state

```text
enum NodeLifecyclePhase {
    Discovering,
    JoiningCluster,
    SeekingAuthority,
    PrimaryServing,
    ReplicaServing,
    Recovering,
    Fenced,
    FailSafe,
}
```

### Supporting substate model

```text
struct ReconcileState {
    tick: u64,
    phase: NodeLifecyclePhase,
    local_postgres: LocalPostgresMode,
    authority: AuthorityState,
    recovery: RecoveryState,
    publication: PublicationState,
}

enum LocalPostgresMode {
    Unknown,
    Stopped,
    Starting,
    Primary,
    Replica,
    Unreachable,
}

enum AuthorityState {
    NoQuorum,
    MajorityFollower {
        leader: MemberId,
        lease_epoch: u64,
    },
    MajorityLeader {
        lease_epoch: u64,
        refresh_deadline_ms: u64,
    },
    Candidate {
        election_epoch: u64,
    },
    UnsafeStalePrimary {
        observed_epoch: u64,
    },
}

enum RecoveryState {
    None,
    WaitingForSource,
    RewindRequired {
        source: MemberId,
    },
    BaseBackupRequired {
        source: MemberId,
    },
    CatchingUp {
        source: MemberId,
        lag_bytes: Option<u64>,
    },
}

enum PublicationState {
    Fresh,
    DegradedPartialTruth,
    StaleNeedsRefresh,
}
```

### Transition triggers

The machine is driven entirely by observation facts:

- `Discovering -> JoiningCluster`
  Trigger: local facts are available and DCS visibility exists, even if partial.
- `JoiningCluster -> SeekingAuthority`
  Trigger: PostgreSQL is either reachable or startable and the node has enough facts to classify local data.
- `SeekingAuthority -> PrimaryServing`
  Trigger: the node holds a valid leader lease epoch and majority-based authority remains intact.
- `SeekingAuthority -> ReplicaServing`
  Trigger: another leader is authoritative and the local node can safely follow it.
- `SeekingAuthority -> Recovering`
  Trigger: the node has local data that does not match the authoritative leader timeline or lacks required base state.
- `PrimaryServing -> Fenced`
  Trigger: local write authority must be cut because the node has lost majority-backed lease refresh authority or has seen a newer foreign leader epoch.
- `PrimaryServing -> FailSafe`
  Trigger: cluster authority facts are too incomplete to safely classify majority versus minority.
- `ReplicaServing -> Recovering`
  Trigger: rewind/basebackup/catch-up requirements are detected.
- `Any -> Discovering`
  Trigger: only on process restart. There is no separate runtime startup planner.

### Invariants

This option depends on a few explicit invariants.

1. The decider never performs I/O.
2. Startup and steady-state share the same decision function.
3. A node may continue serving as primary only while it can demonstrate majority-backed lease freshness, not merely full-cluster visibility.
4. A node that cannot demonstrate majority-backed authority must not continue writable primary service.
5. Deduplication is not performed by the HA sender.
6. Member publication always emits the best-known local truth, even when SQL reachability is degraded.

## Quorum model

The main quorum change is to replace the binary "full quorum or fail-safe" shortcut with an explicit `QuorumView`.

```text
struct QuorumView {
    total_voters: u8,
    reachable_voters: u8,
    majority_size: u8,
    local_member_in_view: bool,
    freshness: ViewFreshness,
    kind: QuorumKind,
}

enum QuorumKind {
    Full,
    Majority,
    Minority,
    Unknown,
}
```

### Why degraded-but-valid majority should continue

In a three-node cluster:

- 3/3 visible is `Full`
- 2/3 visible is still `Majority`
- 1/3 visible is `Minority`

The current direct route from anything-not-full into fail-safe throws away the most important distinction in distributed HA: majority is sufficient for authority, full visibility is only a stronger observation state.

Under this option:

- `Full` and `Majority` both permit normal leader election and continued service, subject to lease freshness.
- `Minority` forbids write authority and pushes a primary toward fencing or fail-safe.
- `Unknown` means the node cannot tell whether it is in majority or minority because the evidence itself is stale or contradictory; that is the proper fail-safe boundary.

### Leadership re-election in 2-of-3 cases

If node A is the old primary and becomes partitioned away from nodes B and C:

- B and C observe `Majority`
- A observes `Minority` or `Unknown`
- B and C may elect a new leader under a higher lease epoch
- A must not continue to accept writes once it loses the ability to refresh its majority-backed lease

That is the exact degraded-but-valid operation the user asked for.

## Lease model

This option makes lease state a first-class typed observation rather than an implicit side effect.

```text
struct LeaseView {
    holder: Option<MemberId>,
    epoch: u64,
    refresh_deadline_ms: u64,
    refresh_quorum_kind: QuorumKind,
    observed_from_dcs: bool,
}
```

### Lease acquisition

- Only a node in `SeekingAuthority` may attempt acquisition.
- Acquisition is permitted only with `QuorumKind::Full` or `QuorumKind::Majority`.
- Successful acquisition increments or claims a lease epoch and yields `AuthorityState::MajorityLeader`.

### Lease refresh

- Lease refresh is a periodic receiver-side effect.
- The decider only asks for `MaintainLease { epoch }`.
- The lease receiver decides whether the request is duplicate, still active, expired, or superseded.

### Lease expiry and loss

If a primary cannot refresh majority-backed lease state before `refresh_deadline_ms`:

- the node transitions to `Fenced` or `FailSafe`, depending on how conclusive the observation is
- write authority is removed before or alongside demotion
- the node may later re-enter recovery and rejoin as replica if another leader is authoritative

### How a killed primary loses authority

A killed primary loses authority because authority is not "I was primary last tick." Authority is "I still hold the newest valid lease epoch backed by a majority." Once the process is dead or unable to refresh, the majority partition can elect a higher epoch. On restart, the old primary sees:

- a newer leader epoch
- an authoritative foreign leader
- a local timeline that may require rewind or basebackup

That forces a coherent rejoin path instead of a half-remembered startup shortcut.

## Startup reasoning

This option removes the dedicated startup planner entirely. Startup is modeled as a sequence of observations with high uncertainty early on.

### Startup observation set

The observation assembler must capture:

- whether PostgreSQL is running, reachable, unreachable, or stopped
- whether `pgdata` exists and whether it looks initialized
- whether an init lock exists
- whether DCS already has a leader
- whether DCS already has this node’s member record
- whether local data appears on the current timeline, an older timeline, or an unknown timeline
- whether the process worker is already doing local recovery work

### Cluster already up

If startup sees an existing leader and local data matches replica expectations:

- the node enters `JoiningCluster`
- then `ReplicaServing` or `Recovering`, depending on timeline and lag facts

### Cluster leader already present

If a leader exists and local PostgreSQL is stopped:

- the decider emits `EnsurePostgresRunningForInspection` if local inspection needs running PostgreSQL
- then chooses `FollowLeader`, `Rewind`, or `BaseBackup` from one typed recovery path

### Existing members already published

Published member keys are treated as evidence, not as authority by themselves. They help answer:

- whether this node was previously seen alive
- whether it was previously primary or replica
- whether local data might still be valid

But they do not override newer lease or timeline evidence.

### Empty versus existing `pgdata`

- Empty `pgdata` means the node is a bootstrap or clone candidate.
- Existing `pgdata` is not automatically valid or invalid.
- Existing `pgdata` must be classified into:
  - locally valid and followable
  - requires rewind
  - cannot be rewound safely and must be replaced by basebackup

### Init lock behavior

Init lock should not be interpreted by a separate startup planner. It should become an observation field:

```text
enum InitLockState {
    Absent,
    PresentFresh,
    PresentStale,
}
```

The decider then uses that together with DCS and local data facts to choose:

- fresh bootstrap
- resume bootstrap
- abandon stale bootstrap intent and rejoin from existing cluster state

### When existing local data may still be valid for initialization

Existing local data can remain initialization-valid when:

- DCS does not yet show an established cluster
- local data is self-consistent
- there is no conflicting higher leader epoch
- the init lock is locally attributable and fresh

If those conditions are not true, local data becomes either a rejoin candidate or a discard-and-basebackup candidate.

## Replica convergence as one coherent path

The current system spreads convergence decisions across HA decide, process dispatch, and runtime startup. This option creates one explicit convergence planner.

```text
enum ReplicaConvergencePlan {
    HealthyFollow {
        source: MemberId,
    },
    CatchUp {
        source: MemberId,
        lag_bytes: u64,
    },
    Rewind {
        source: MemberId,
    },
    BaseBackup {
        source: MemberId,
        reason: BaseBackupReason,
    },
}
```

### Healthy follow

If local timeline and role are consistent and lag is acceptable, the plan is `HealthyFollow`.

### Tolerable lag

Lag does not trigger a role transition by itself. The node can remain in `ReplicaServing` with a `CatchUp` convergence plan while still being a healthy replica candidate or a disqualified promotion candidate, depending on thresholds.

### Wrong-timeline rewind

If local data is behind or divergent but still rewindable, the planner chooses `Rewind`.

### Basebackup fallback

If rewind cannot succeed because prerequisites are missing or validation fails, the planner advances to `BaseBackup`. This path must be typed and explicit rather than inferred by side effects inside process dispatch.

## Partial information publication

This option treats DCS member publication as an always-on projection of best-known truth.

### Proposed member publication shape

```text
struct ObservedMemberHealth {
    agent_state: AgentState,
    sql_status: SqlStatus,
    readiness: Readiness,
    local_role_hint: RoleHint,
    process_presence: ProcessPresence,
    observation_age_ms: u64,
    last_error: Option<String>,
}
```

Key point: if pginfo fails but the node process is up, the publication is not omitted. It becomes a degraded partial-truth record such as:

```text
agent_state = Running
sql_status = Unreachable
readiness = NotReady
local_role_hint = LastKnownPrimary
process_presence = Present
```

That is much more useful to peer decision-making than silence.

## Where deduplication moves

Deduplication moves out of `src/ha/worker.rs` and into effect consumers.

### Why the current sender-side approach is unsafe

`should_skip_redundant_process_dispatch(...)` only sees:

- current HA state
- next HA state
- coarse current process state

It does not know:

- whether the receiver already completed the exact request
- whether a prior request failed and needs replay
- whether the currently running job belongs to the same authority epoch
- whether the request is stale but the active job is from an older world view

### Proposed safer boundary

Every lowered action gets a receiver-owned idempotency key:

```text
struct EffectEnvelope {
    receiver: ReceiverKind,
    key: EffectKey,
    payload: EffectPayload,
}

struct EffectKey {
    authority_epoch: u64,
    phase: NodeLifecyclePhase,
    intent_kind: IntentKind,
    target_member: Option<MemberId>,
}
```

The process worker, lease worker, and DCS worker each keep their own view of active and completed keys. If they receive the same key again:

- replay can be ignored if already complete
- replay can be coalesced if already running
- replay can replace an older conflicting key if the authority epoch advanced

This keeps idempotency where the real execution truth lives.

## Concrete repo files, modules, functions, and types a future implementation would touch

### Primary files

- `src/runtime/node.rs`
- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/ha/decision.rs`
- `src/ha/lower.rs`
- `src/ha/process_dispatch.rs`
- `src/dcs/worker.rs`
- `src/dcs/state.rs`
- `src/pginfo/state.rs`
- `src/process/state.rs`
- `src/process/jobs.rs`
- `src/debug_api/view.rs`
- `src/debug_api/snapshot.rs`
- `tests/ha.rs`
- `tests/ha/features/*`
- `tests/ha/support/*`

### Existing functions and concepts likely to change or disappear

- `plan_startup(...)`
- `plan_startup_with_probe(...)`
- `execute_startup(...)`
- `build_startup_actions(...)`
- `should_skip_redundant_process_dispatch(...)`
- `decision_is_already_active(...)`
- current top-level non-`FullQuorum` shortcut in `decide_phase(...)`
- start-intent derivation in `src/ha/process_dispatch.rs`

### New likely types

- `ReconcileObservation`
- `ReconcileState`
- `ReconcileOutcome`
- `NodeLifecyclePhase`
- `AuthorityState`
- `QuorumView`
- `LeaseView`
- `ReplicaConvergencePlan`
- `ObservedMemberHealth`
- `EffectEnvelope`
- `EffectKey`

## All meaningful changes required for this option

### New types

- Introduce one typed observation struct spanning runtime, DCS, pginfo, process, and local data inspection.
- Introduce one typed state struct spanning lifecycle phase, authority, recovery, and publication state.
- Introduce typed quorum and lease views that separate `Full`, `Majority`, `Minority`, and `Unknown`.
- Introduce receiver-side effect envelopes with idempotency keys.

### Deleted paths

- Delete the separate startup planner/executor path from `src/runtime/node.rs`.
- Delete sender-side process dedup from `src/ha/worker.rs`.
- Delete HA lifecycle truth hidden in `src/ha/process_dispatch.rs`.

### Moved responsibilities

- Move startup classification into observation assembly plus pure reconciliation.
- Move dedup/idempotency into process, lease, and DCS receivers.
- Move convergence choice into the pure decision layer, leaving only mechanical translation in lower/apply layers.

### Changed transitions

- Replace the current automatic non-full-quorum fail-safe shortcut with explicit majority/minority/unknown handling.
- Allow degraded-majority election and continued service.
- Force minority or unknown-authority primaries into fencing or fail-safe.

### Changed effect-lowering boundaries

- Lowerer accepts typed intents, not partially inferred lifecycle guesses.
- Process dispatch becomes a narrow translation layer, not an authority planner.

### Changed DCS publication behavior

- Member records publish degraded partial truth instead of disappearing or flattening uncertainty.
- Lease metadata becomes clearer and more authoritative.

### Changed startup handling

- Remove one-off startup planning.
- Model startup as initial observations with unknowns.

### Changed convergence handling

- Use one convergence planner for follow, lag catch-up, rewind, and basebackup.
- Make rewind-to-basebackup fallback explicit in typed state.

### Test updates a later implementation would need

- Update HA tests and support code to assert majority-based degraded operation instead of assuming full visibility is required for service.
- Expand debug API expectations so fail-safe, fenced, and degraded-majority states are distinguishable.
- Add unit coverage around quorum classification, lease expiry, and receiver-owned idempotency keys.

## Migration sketch

This option is large enough that implementation order matters. A later implementation should not try to swap everything in one patch.

### Step 1: Introduce observation and outcome types without deleting old behavior

- Add `ReconcileObservation` and `ReconcileOutcome`.
- Build them from current worker inputs.
- Keep the old state machine temporarily, but log the new model in parallel if needed.

### Step 2: Route startup through synthetic reconciliation

- Make runtime startup produce initial observations only.
- Stop making lifecycle decisions in `src/runtime/node.rs`.
- Preserve existing process launches through adapters during the transition.

### Step 3: Move dedup into receivers

- Add effect keys to process, lease, and DCS receivers.
- Delete `should_skip_redundant_process_dispatch(...)`.
- Verify duplicate requests are ignored or coalesced by the receiver itself.

### Step 4: Replace quorum semantics

- Introduce `QuorumView`.
- Replace direct non-full-quorum fail-safe behavior with majority/minority/unknown transitions.
- Update debug projections so operators can see the difference.

### Step 5: Replace convergence planning

- Move rewind/basebackup/follow selection into the decider.
- Shrink `src/ha/process_dispatch.rs` to translation only.

### Step 6: Delete stale legacy paths aggressively

- Remove startup planner code from runtime.
- Remove transitional adapters and dead enums.
- Update tests to refer only to the new lifecycle names and semantics.

This project is explicitly greenfield with no backward-compatibility requirement, so once the new path is authoritative the old one should be deleted, not retained behind flags.

## Non-goals

- This option does not try to preserve existing phase names for compatibility.
- This option does not try to be the smallest patch set.
- This option does not move side effects into the decider.
- This option does not claim that all test failures are caused solely by quorum semantics; it claims the architectural shape makes several of those failures harder to fix coherently.

## Tradeoffs

- The refactor is structurally large and will touch many modules.
- More typed state means a larger upfront modeling cost.
- Receiver-owned idempotency is operationally cleaner but requires stronger receiver bookkeeping.
- Unifying startup and steady-state removes ambiguity, but it also removes familiar separation that some maintainers may have been using as a mental shortcut.

## Logical feature-test verification

This section maps the option against the key HA scenarios named in the task. The goal here is not to pretend they would be green immediately, but to show how this architecture gives each scenario a coherent home.

### `ha_dcs_quorum_lost_enters_failsafe`

If a node cannot determine whether it still has majority-backed authority, `QuorumKind::Unknown` drives `FailSafe`. The debug API should expose this state directly, which addresses the current symptom where debug output did not contain `fail_safe`.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

If a primary loses lease refresh authority and falls to `Minority` or `Unknown`, the decider emits a fencing-oriented intent before any continued primary service is allowed. That gives a clearer safety boundary than "non-full-quorum but keep lease."

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority partition classifies itself as `Majority`, not fail-safe. It can elect a new leader with a higher lease epoch. The isolated old primary sees either `Minority` or a foreign higher epoch and must stop serving primary writes. This is one of the strongest fits for this option.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

On healing, the old primary does not re-enter via a separate startup planner. It observes a foreign higher epoch and goes into `Recovering`, where the convergence planner chooses rewind or basebackup. That gives one coherent rejoin path.

### `ha_primary_killed_then_rejoins_as_replica`

A restarted old primary begins in `Discovering`, observes that another node owns the newer epoch, then transitions through `JoiningCluster -> Recovering -> ReplicaServing`. No startup-side shortcut is allowed to bypass that.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica returns, the cluster should be able to move from `Minority` back to `Majority`. The design explicitly allows service restoration under majority without requiring full visibility, while still preventing minority-side stale primaries from continuing to answer.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes can form a valid majority and elect leadership under the same reconciliation model used during normal operation. The final node rejoins via the standard recovery planner, not via a special startup classification branch. That directly addresses the current timeout around observing the rejoined node as replica.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The convergence planner represents rewind and basebackup as one typed path. If rewind prerequisites are not met or rewind fails validation, the state advances to `BaseBackupRequired`. That makes the fallback part of the model instead of an incidental secondary process branch.

### `ha_replica_stopped_primary_stays_primary`

A healthy primary with majority-backed lease freshness stays in `PrimaryServing`. Replica stoppage alone does not cause unnecessary role churn.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica’s recovery work remains local to its `Recovering` state. Because authority and role are not re-derived from sender-side process guesses, the broken replica does not destabilize the majority partition merely by attempting rejoin work.

### Boundary clarification for changed interpretations

This option changes one major interpretation boundary: "degraded but healthy enough" is no longer synonymous with full-cluster visibility. The new split is:

- `Full` or `Majority` with fresh lease evidence:
  service may continue or election may proceed
- `Minority` with stale or absent lease authority:
  primary service must fence or demote
- `Unknown` because the system cannot safely classify majority versus minority:
  enter fail-safe

That distinction is required to make the majority-partition scenarios and the fail-safe scenarios both correct at the same time.

## Q1 Which state owns cluster bootstrap finality

In this option, bootstrap is not a separate startup planner, but there is still a real question about whether initial cluster creation should live inside `JoiningCluster`, `SeekingAuthority`, or a dedicated `BootstrapLeader` substate.

The problem is that bootstrap combines two concerns:

- creating first authoritative cluster state
- serving as the first lease-backed primary

Restated question: should first-cluster creation be modeled as just another authority acquisition path, or should bootstrap remain a dedicated substate with extra invariants?

## Q2 How much lease metadata belongs in DCS versus local derived state

This option introduces `LeaseView`, but the exact line between persisted lease metadata and locally derived authority classification still needs design discipline.

The problem is that too much lease data in DCS can make the control plane heavy, while too little keeps the current ambiguity where local code infers too much from incomplete facts.

Restated question: what is the minimum persisted lease shape that still lets restarted nodes distinguish stale local primary state from current majority-backed authority?

## Q3 Should degraded partial-truth publication include last-known role

Publishing `LastKnownPrimary` or `LastKnownReplica` is useful, but it can also mislead peers if the value is treated as fresh truth rather than historical hint.

The problem is balancing richer publication against the risk that future code will over-trust historical role hints.

Restated question: should member records publish historical role hints directly, or should they publish only current probe truth plus a separate freshness-qualified role history field?

## Q4 Where should authority epoch live in receiver idempotency keys

This option proposes receiver-owned `EffectKey` values keyed by authority epoch. That is directionally right, but the exact key shape determines whether superseded work is safely ignored or accidentally retained.

```text
candidate key:
(authority_epoch, phase, intent_kind, target_member)
```

The problem is that if the key is too broad, valid retries may be dropped; if it is too narrow, duplicate work may re-run.

Restated question: what exact effect-key fields let receivers suppress stale work without blocking valid retries inside the same authority epoch?

## Q5 Should fail-safe and fenced remain separate top-level phases

This option keeps `Fenced` and `FailSafe` distinct. `Fenced` means the node has positively determined it must cut write authority. `FailSafe` means observations are too incomplete to classify authority safely.

The problem is that operators may perceive both as "node cannot serve writes," and implementation complexity rises when two safety-oriented phases exist.

Restated question: does the operational value of distinguishing "positively fenced" from "insufficiently certain" justify two separate top-level phases, or should one safety phase carry typed reasons instead?
