# Option 2: Dual-Layer Cluster/Local State Machine

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make tests green. Green outcomes for `make check`, `make test`, `make test-long`, and `make lint` are future implementation concerns outside this task. This document exists only to describe one complete redesign option in enough detail that a later implementer can execute it without needing chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current architecture mixes two different questions inside one HA path:

- What cluster topology is safe and desired right now?
- What should this specific node do locally in order to match that topology?

The differentiator of Option 2 is that it treats those as two separate but composable pure state machines. The first pure machine decides cluster intent from newest observations. The second pure machine decides local-node execution from that cluster intent plus local facts. Startup is unified with steady-state because both machines run on the very first tick as well as every later tick. The design keeps the user-preferred functional style of `newest info -> decide -> typed outcome -> actions`, but it stops forcing one state enum to answer both cluster-level authority and local recovery details at the same time.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the redesign target for this task.
- The observed failure themes from `target/nextest/ultra-long/junit.xml` and exported logs remain relevant:
  - quorum-loss scenarios did not consistently surface `fail_safe` evidence when expected
  - degraded-majority failover scenarios did not consistently expose a new primary from the healthy majority
  - some restart and restore-service scenarios left a node writable when the scenario expected service to stay blocked
  - targeted switchover toward a degraded replica succeeded when it should have been rejected
  - rewind-to-basebackup fallback evidence was not consistently visible
  - stalled-primary failover timing remained too weak, with the old primary holding authority too long
  - primary-kill and replica-flap scenarios showed convergence ambiguity rather than one clean authority handoff

These results do not prove that Option 2 is the right answer. They do show that the current authority, startup, failover, and rejoin logic is not yet expressed as one coherent model.

## Option differentiator

Option 1 collapsed everything into one authoritative reconciliation kernel with one large lifecycle state machine. Option 2 deliberately keeps two pure machines:

- a `ClusterIntentMachine` that answers cluster topology, lease authority, safety posture, and leader requirements
- a `LocalExecutionMachine` that answers what this node must do locally to satisfy the current cluster intent

That split is the point of the design. The current system is spread across too many boundaries, but the answer does not have to be one giant enum. The cleaner answer may be to separate "global desired shape" from "local operational path" while still making both machines run on every tick from the same newest observation bundle.

## Current design problems

### Startup logic is split across runtime code instead of being part of the main HA model

`src/runtime/node.rs` currently owns `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. Those functions decide things that are not mere bootstrapping mechanics. They decide topology-relevant behavior such as whether the node should initialize, follow, or continue. That means the repo currently has one reasoning system before workers start and another reasoning system after workers start.

Option 2 removes that split by making startup a first-class observation pass through both pure machines. There is no special startup planner. There is only an early tick where many facts are still `Unknown`.

### Sender-side dedup in `src/ha/worker.rs` is answering the wrong question at the wrong layer

The current `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)` logic asks the sender to guess whether the receiver is already doing the right thing. That is not trustworthy. A sender only knows what it last emitted. It does not know whether the process receiver accepted, superseded, retried, partially completed, or rolled back a request.

Option 2 moves idempotency and dedup to receiver-owned work items. The HA machines emit monotonic intent envelopes with semantic keys. Process, DCS, and lease receivers own replay suppression.

### HA truth is spread across runtime, decide, lower, and process-dispatch logic

The current architecture has relevant truth in at least four places:

- `src/runtime/node.rs` for startup planning
- `src/ha/decide.rs` for phase selection
- `src/ha/lower.rs` for typed effects
- `src/ha/process_dispatch.rs` for role/start/rejoin path derivation

That spread is why the architecture feels drifted. A reader cannot say where cluster truth ends and local execution truth begins.

Option 2 makes the answer explicit:

- cluster truth lives in `ClusterIntentMachine`
- local execution truth lives in `LocalExecutionMachine`
- lowering turns those two outputs into typed effect envelopes

### The current non-full-quorum shortcut is too blunt

`src/ha/decide.rs` currently routes any non-`DcsTrust::FullQuorum` state toward fail-safe behavior. That erases essential distinctions:

- healthy majority but missing one member
- isolated minority
- stale cluster visibility
- no lease visibility
- no usable DCS writes at all

A 2-of-3 majority is not the same thing as unsafe uncertainty. Option 2 makes cluster-majority analysis the first responsibility of `ClusterIntentMachine`, so degraded-majority operation remains possible while minority partitions are fenced.

### Startup and rejoin intent is still partially buried in `src/ha/process_dispatch.rs`

The current `start_intent_from_dcs(...)` and related source-validation logic still decides too much of the lifecycle path. It is not just translating a decision into work. It is materially determining what the local node should become and how.

Option 2 changes that boundary. The `LocalExecutionMachine` chooses one local target path:

- remain stopped
- start primary
- start replica
- converge to leader
- rewind then follow
- basebackup then follow
- bootstrap new cluster
- fence locally

`process_dispatch` then becomes a dumb translator from typed local execution plan to receiver commands.

### Member publication still under-uses partial truth

`src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already contain much of the structure needed to publish partial truth, but the current control plane still leans toward absence when probes are degraded. The user’s requirement is stronger: if pgtuskmaster is alive, local process state is partially known, and SQL probing fails, the member key should still say that clearly.

Option 2 treats publication as part of local execution state, not as a side-effect afterthought. Every tick computes a `LocalPublicationView` that represents best-known truth whether PostgreSQL is healthy, starting, unreachable, or uncertain.

## Proposed control flow

Every tick, including the first startup tick, runs the same flow:

1. Gather newest observations from DCS, pginfo, local process inspector, local data-dir inspector, init-lock inspector, and runtime host facts.
2. Normalize those facts into a single immutable `ObservationBundle`.
3. Feed `ObservationBundle` into `ClusterIntentMachine`.
4. Receive a pure `ClusterIntent`.
5. Feed `ObservationBundle + ClusterIntent + prior local execution state` into `LocalExecutionMachine`.
6. Receive a pure `LocalExecutionPlan`.
7. Lower both outputs into typed effect envelopes.
8. Send envelopes to receivers that own idempotency and dedup.
9. Publish derived local member truth through the DCS receiver.

The main change from the current model is not extra complexity. It is ordering clarity:

- first decide what the cluster safely wants
- then decide what this node should do to align with that cluster intent

### High-level flow diagram

```text
                 +-------------------------+
pginfo --------->|                         |
dcs ------------>| ObservationBundle       |
process -------->| assembler               |
data-dir ------->| newest facts only       |
init-lock ------>|                         |
runtime -------->|                         |
                 +-------------------------+
                              |
                              v
                 +-------------------------+
                 | ClusterIntentMachine    |
                 | pure cluster topology   |
                 | and authority choice    |
                 +-------------------------+
                              |
                              v
                 +-------------------------+
                 | LocalExecutionMachine   |
                 | pure local node path    |
                 | selection               |
                 +-------------------------+
                              |
                              v
                 +-------------------------+
                 | lower_to_effects()      |
                 | typed receiver intents  |
                 +-------------------------+
                      |        |         |
                      v        v         v
                 Process    Lease      DCS
                 receiver   receiver   receiver
                 owns dedup owns dedup owns dedup
```

### Responsibility boundary diagram

```text
CURRENT
-------
runtime startup planner
  -> ha decide
  -> lower
  -> process_dispatch
  -> worker-side dedup

OPTION 2
--------
ObservationBundle
  -> ClusterIntentMachine
  -> LocalExecutionMachine
  -> lower_to_effects
  -> receiver-owned idempotency
```

## Proposed typed model

### Shared observations

The first important type is a richer shared observation bundle.

```text
struct ObservationBundle {
    tick: u64,
    now: InstantSnapshot,
    dcs: DcsObservation,
    pginfo: PgObservation,
    local_process: LocalProcessObservation,
    local_data: LocalDataObservation,
    init_lock: InitLockObservation,
    config: ConfigObservation,
    self_identity: SelfIdentityObservation,
    prior_cluster_intent: Option<ClusterIntent>,
    prior_local_execution: Option<LocalExecutionState>,
}
```

This bundle must preserve partial truth rather than collapsing unknowns into absence.

### Cluster-level machine

The first pure machine decides cluster topology and authority rules. It does not decide detailed local steps such as rewind or basebackup. It answers what the cluster safely wants to be.

```text
enum ClusterIntent {
    NoClusterAuthority {
        reason: NoAuthorityReason,
    },
    BootstrapCluster {
        generation: u64,
        bootstrap_leader: MemberId,
    },
    MaintainLeader {
        generation: u64,
        leader: MemberId,
        lease_epoch: u64,
        quorum_mode: QuorumMode,
    },
    ElectLeader {
        generation: u64,
        candidate: MemberId,
        election_epoch: u64,
        quorum_mode: QuorumMode,
    },
    FollowLeader {
        generation: u64,
        leader: MemberId,
        lease_epoch: u64,
        quorum_mode: QuorumMode,
    },
    FenceClusterWrites {
        reason: FenceReason,
        last_safe_generation: Option<u64>,
    },
}
```

Supporting cluster types:

```text
enum QuorumMode {
    FullQuorum,
    DegradedMajority,
    MinorityIsolated,
    NoWritableQuorum,
    VisibilityAmbiguous,
}

enum NoAuthorityReason {
    EmptyClusterNoBootstrapWinner,
    DcsUnavailable,
    LeaseStateUnknown,
    ClusterGenerationConflict,
}
```

The invariant is simple: `ClusterIntentMachine` never says how to execute. It only says what cluster shape is valid and safe.

### Local-level machine

The second pure machine answers what this node should do locally in order to align with current cluster intent.

```text
struct LocalExecutionPlan {
    next_state: LocalExecutionState,
    postgres_target: PostgresTarget,
    convergence: ConvergencePlan,
    publication: LocalPublicationView,
    safety: LocalSafetyPlan,
}

enum LocalExecutionState {
    IdleUnknown,
    WaitingForAuthority,
    Bootstrapping {
        generation: u64,
    },
    StartingPrimary {
        generation: u64,
        lease_epoch: u64,
    },
    ServingPrimary {
        generation: u64,
        lease_epoch: u64,
    },
    StartingReplica {
        leader: MemberId,
        generation: u64,
    },
    ServingReplica {
        leader: MemberId,
        generation: u64,
    },
    Rewinding {
        leader: MemberId,
        generation: u64,
    },
    BaseBackuping {
        leader: MemberId,
        generation: u64,
    },
    Fenced {
        reason: FenceReason,
    },
}
```

Convergence is a first-class type, not a side decision hidden in process dispatch.

```text
enum ConvergencePlan {
    None,
    ContinueHealthyFollow {
        leader: MemberId,
    },
    ContinueReplicaWithLagTolerance {
        leader: MemberId,
        max_lag_bytes: u64,
    },
    PromoteToPrimary,
    RewindToLeader {
        leader: MemberId,
    },
    BaseBackupFromLeader {
        leader: MemberId,
    },
    StopAndFence,
}
```

The invariant here is also simple: the local machine never invents cluster authority. It consumes cluster authority chosen by the cluster machine.

## Cluster authority model

Option 2 fixes the degraded-quorum problem by making quorum interpretation explicit and upstream.

### Core cluster rules

- `FullQuorum` means all expected members are visible and the current leader lease is healthy.
- `DegradedMajority` means a strict majority of expected voters is healthy and mutually visible, even if one or more members are absent.
- `MinorityIsolated` means this node cannot form or confirm a majority and therefore must not retain write authority.
- `NoWritableQuorum` means DCS or lease writes are unavailable enough that safe authority cannot be refreshed.
- `VisibilityAmbiguous` means observations conflict or are too stale to trust leader continuity.

### Resulting behavior

- A node in a healthy majority may continue or elect leadership under `DegradedMajority`.
- A node in a minority partition may not keep primary write authority, even if it was primary moments earlier.
- A node with ambiguous visibility does not immediately invent a new leader. It fences writes until visibility becomes safe again.
- The difference between "degraded but valid" and "unsafe ambiguity" becomes central rather than accidental.

### Why this helps 2-of-3 behavior

In a three-node cluster, losing one replica should not force the two surviving healthy nodes into the same path as a total DCS blackout. The current architecture compresses those cases too aggressively. Option 2 makes majority viability a cluster-level property, so the local node only follows cluster-safe intent rather than trying to infer it from local role plus non-full-quorum.

## Lease model

Lease reasoning becomes part of cluster intent, not a local side effect.

### Lease invariants

- A leader is authoritative only if the cluster machine can justify that leader under the current generation and lease epoch.
- A node cannot remain `ServingPrimary` locally unless cluster intent is `MaintainLeader` or `ElectLeader` for that same node.
- Lease refresh failure does not wait for process symptoms. It changes cluster intent first, which then changes local execution.
- A killed primary loses authority because it stops refreshing its lease and because surviving majority members can advance generation or lease epoch beyond it.

### Lease flow

```text
1. Cluster machine reads lease facts from DCS observations.
2. It classifies the lease as healthy, stale, lost, conflicting, or unknown.
3. It combines lease classification with quorum mode.
4. It emits cluster intent:
   - maintain current leader
   - elect a new leader
   - fence writes
   - wait for bootstrap authority
5. Local machine enforces that outcome on this node.
```

### Why this is stronger than the current model

The current design lets local role and current phase carry too much implied authority. Option 2 requires explicit cluster intent for authority. Local execution cannot "accidentally remain primary" because there is no legal local primary state without corresponding cluster intent.

## Startup reasoning

Startup is no longer a separate planner. It is the first run of the same two machines.

### Startup observations that matter

- Is the DCS cluster empty, partially populated, or already led?
- Is there a visible leader lease?
- Is there an init lock, and who owns it?
- Does local `pgdata` exist?
- Does local `pgdata` indicate prior primary or replica identity?
- Is PostgreSQL already running?
- Is local SQL reachable, unreachable, or unknown?
- Are local control files on the expected timeline?

### Startup rules

- If the cluster is empty and this node wins bootstrap authority, the cluster machine emits `BootstrapCluster`.
- If the cluster already has a leader, the cluster machine emits `FollowLeader` or `MaintainLeader`; the local machine then decides whether local data can follow directly, requires rewind, or requires basebackup.
- If local `pgdata` exists and matches the chosen role path, the local machine reuses it rather than assuming bootstrap from scratch.
- If local `pgdata` exists but conflicts with cluster intent, the local machine chooses the convergence path explicitly.
- If the node sees an init lock but no safe cluster authority yet, the local machine waits in `WaitingForAuthority` rather than pre-committing to local changes.

### Bootstrap rethink

Bootstrap should have substates rather than one opaque step:

```text
BootstrapLocalState:
- WaitingForInitAuthority
- PreparingLocalData
- InitializingPrimary
- PublishingBootstrapLeader
- ConfirmingClusterGeneration
```

This allows a later implementation to keep or reuse valid existing `pgdata` when the node wins bootstrap authority, instead of assuming that winning the init lock always implies destructive reinitialization.

## Replica convergence as one coherent path

Option 2 keeps convergence in one typed local plan instead of scattering it across decision and dispatch helpers.

### Convergence rules

- If a healthy leader is visible and local state already follows correctly, use `ContinueHealthyFollow`.
- If a healthy leader is visible and replica lag is within tolerated recovery limits, continue following without unnecessary rebuild.
- If local timeline diverges and rewind preconditions are met, use `RewindToLeader`.
- If rewind is impossible or fails decisively, choose `BaseBackupFromLeader`.
- Previously-primary, previously-replica, and freshly-restored nodes all use the same convergence planner. Their historical role is input, not architecture.

### Why this is different from current behavior

Today, startup/rejoin intent derivation is partially hidden in `src/ha/process_dispatch.rs`. Option 2 moves all convergence choice into the local pure machine. That means the later process dispatcher becomes a mechanical translation layer rather than an architectural decision-maker.

## Partial-truth member publication

Option 2 treats publication as a first-class output of local execution planning.

```text
struct LocalPublicationView {
    member_id: MemberId,
    observed_role: Option<RoleObservation>,
    sql_status: SqlStatus,
    readiness: Readiness,
    process_state: ProcessStateObservation,
    data_state: DataStateObservation,
    authority_claim: AuthorityClaimView,
    last_observation_time: Timestamp,
}
```

### Publication rules

- If pginfo is healthy, publish full observed role, readiness, and SQL health.
- If pginfo fails but the agent is up and the local process inspector sees PostgreSQL running, publish that the process appears up while SQL status is `Unknown` or `Unreachable`.
- If the node is bootstrapping, rewinding, or basebackuping, publish that explicitly so peers can reason about non-serving states.
- Never encode uncertainty as silence when a better partial truth exists.

This satisfies the user’s requirement that “pginfo failed but pgtuskmaster is up” remains publishable information.

## Deduplication and receiver ownership

Sender-side dedup is removed from the HA worker entirely.

### New dedup rule

- HA machines emit typed effect envelopes with stable semantic keys.
- Receivers own replay detection based on those keys plus their own in-flight state.
- Receivers may accept, collapse, supersede, or reject repeated envelopes.

### Example

Instead of the HA worker deciding whether `StartPostgres` is redundant, the process receiver gets:

```text
ProcessEnvelope {
    key: "generation-42/start-replica/member-b",
    desired_target: StartReplica { leader: member-a },
    supersedes: ["generation-41/start-primary/member-b"],
}
```

The process receiver can then say:

- already active, no-op
- superseded, cancel previous
- invalid due to newer generation
- accepted and now in progress

This is safer because only the receiver knows actual execution state.

## Concrete future code areas

A future implementation of this option would likely touch all of the following areas:

- `src/runtime/node.rs`
  - remove special startup planner/executor pathways
  - replace them with observation assembly and one shared tick bootstrap
- `src/ha/worker.rs`
  - split current one-machine flow into cluster-intent pass and local-execution pass
  - remove sender-side dedup helpers
- `src/ha/decide.rs`
  - replace or narrow it into `ClusterIntentMachine`
  - move degraded-majority classification here
- `src/ha/decision.rs`
  - add shared cluster/local typed outputs
  - likely become the home for the new cross-machine types
- `src/ha/lower.rs`
  - lower `ClusterIntent + LocalExecutionPlan` into effect envelopes
- `src/ha/process_dispatch.rs`
  - stop deciding lifecycle paths
  - only translate local execution targets into process receiver commands
- `src/dcs/worker.rs`
  - publish `LocalPublicationView`
  - own DCS write-side idempotency
- `src/dcs/state.rs`
  - enrich quorum and trust classification for degraded-majority versus unsafe ambiguity
- `src/pginfo/state.rs`
  - preserve partial truth input into publication and local execution decisions
- `tests/ha.rs`
  - align scenario assertions with explicit cluster-intent and local-execution transitions
- `tests/ha/features/`
  - likely expand observability assertions to cover degraded-majority versus minority fencing

## Meaningful implementation changes required by this option

- Introduce `ObservationBundle` as the canonical newest-facts input.
- Introduce `ClusterIntentMachine` as the only pure cluster authority selector.
- Introduce `LocalExecutionMachine` as the only pure local path selector.
- Delete the concept of separate startup planning in runtime code.
- Delete sender-side dedup checks from the HA worker.
- Change `src/ha/process_dispatch.rs` from hybrid decider/dispatcher into translator only.
- Add explicit quorum mode classification for `FullQuorum`, `DegradedMajority`, `MinorityIsolated`, `NoWritableQuorum`, and `VisibilityAmbiguous`.
- Add generation and lease-epoch coupling between cluster intent and legal local primary states.
- Add publication planning as an explicit typed output of the local machine.
- Add bootstrap substates rather than a single opaque bootstrap path.
- Unify previously-primary and previously-replica rejoin logic through one convergence planner.
- Remove stale legacy paths after migration rather than preserving backward-compatible duplicate logic.

## Migration sketch

The later implementation should not attempt a giant flag-day rewrite in one commit, but it also should not preserve long-lived duplicate architectures.

### Suggested migration order

1. Create shared `ObservationBundle` and make both startup and steady-state gather through it.
2. Extract current degraded-quorum classification into an explicit cluster-intent prototype.
3. Introduce `ClusterIntent` type in `src/ha/decision.rs`.
4. Move local startup/rejoin selection into a draft `LocalExecutionPlan`.
5. Simplify `process_dispatch` to consume plan outputs instead of deriving them.
6. Move dedup out of the HA sender and into receivers.
7. Delete old startup planner code from `src/runtime/node.rs`.
8. Delete any remaining shadow lifecycle decisions from dispatcher helpers.
9. Update HA tests and observability to assert the new boundaries directly.

### Deletion rule

Every time a new machine absorbs a responsibility, the previous copy of that responsibility must be deleted in the same change series. This is a greenfield repo with no backward-compatibility requirement, so the implementation should not keep old and new authority logic alive together.

## Non-goals

- This option does not attempt to preserve the current phase enum shape for compatibility.
- This option does not claim the cluster machine should directly read PostgreSQL or directly write etcd.
- This option does not solve observability by adding logs alone; it changes authority boundaries.
- This option does not argue that every failure becomes recoverable. Minority isolation and authority ambiguity must still fence writes.

## Tradeoffs

- Two machines are easier to reason about conceptually, but they introduce one more typed interface that must stay crisp.
- If the boundary between cluster intent and local execution is defined poorly, the design could simply move confusion rather than remove it.
- Later implementation work will need discipline to stop `process_dispatch` from regaining decision authority.
- More explicit types may temporarily feel heavier than the current phase-based shortcuts, but they pay for themselves in testability and authority clarity.

## Logical feature-test verification

This section explains how a later implementation of Option 2 should satisfy the key HA scenarios without changing code in this task.

### `ha_dcs_quorum_lost_enters_failsafe`

If DCS visibility falls to `NoWritableQuorum` or `VisibilityAmbiguous`, the cluster machine emits `FenceClusterWrites` rather than pretending local primary state still implies authority. The local machine transitions the node to `Fenced` or `WaitingForAuthority` depending on process reality. The key difference is that fail-safe is driven by lost cluster authority, not by an incidental local role check.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

The same cluster-intent boundary ensures that once the node cannot justify continued authority, the local machine produces a safety plan that blocks writes. Because local primary service requires corresponding cluster intent, post-cutoff writes cannot remain legal on the isolated or ambiguity-trapped node.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The healthy majority classifies itself as `DegradedMajority`, not as generic non-full-quorum. The cluster machine on majority-side nodes may emit `ElectLeader`, allowing one of them to become leader with a new epoch while the old isolated primary falls into `MinorityIsolated` and fences.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

After healing, the old primary sees a newer cluster generation and a healthy leader. The cluster machine emits `FollowLeader`, and the local machine chooses the proper convergence plan: direct follow if possible, rewind if timeline-diverged but rewindable, or basebackup if rewind is impossible.

### `ha_primary_killed_then_rejoins_as_replica`

When the killed primary returns, it no longer has valid cluster intent to serve as primary. The cluster machine sees the newer leader generation and emits `FollowLeader`. The local machine must therefore choose a replica-serving or convergence state instead of resuming primary service.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica restarts into a topology where majority can be restored, the cluster machine recognizes recovered majority viability. It may keep or re-elect the valid leader depending on lease state. The restarted node’s local machine then selects the correct replica startup/convergence path rather than defaulting to an unsafe or stale local role.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

Startup is handled by the same two-machine model, so the first two restarted nodes can determine whether they form a viable cluster authority and which node should lead. The final rejoining node sees established cluster intent and follows the convergence path instead of improvising a local startup role.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The local execution machine models rewind and basebackup as explicit convergence choices. A later implementation can therefore express the exact fallback: `RewindToLeader` first, then if receiver feedback marks rewind impossible or terminally failed, recompute into `BaseBackupFromLeader`.

### `ha_replica_stopped_primary_stays_primary`

If the leader still holds a valid lease and quorum mode remains `FullQuorum` or `DegradedMajority`, the cluster machine emits `MaintainLeader`. Replica absence alone does not force a role change on the healthy primary. The local machine on the primary stays in `ServingPrimary`.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

Because cluster intent is decided before local execution and because broken-replica recovery is a local convergence issue, a single bad replica does not destabilize overall cluster authority. The cluster machine still maintains the valid leader, while the broken node’s local machine cycles through its own recovery plan without changing cluster topology.

## Why this option may be attractive

This option is attractive if the implementation team believes the current design failed mainly because cluster-wide authority reasoning and node-local recovery reasoning were forced into one muddled chain. It gives each question a pure home while still unifying startup and steady-state behavior.

## Why this option may still be rejected

This option should be rejected if the team concludes that two state machines would introduce too much interface complexity and that one larger lifecycle machine would actually be clearer in practice. It should also be rejected if there is no appetite to rigorously police the boundary so that `process_dispatch` remains a translator instead of silently becoming a third decider.

## Q1 Where should cluster generation advance?

Context: the cluster machine uses `generation` and `lease_epoch` to distinguish old authority from new authority, especially after failover or healed partitions.

Problem: generation can advance on bootstrap, failover, forced fencing, or lease-conflict repair. If generation moves in too many cases, the system churns. If it moves in too few cases, an old primary may not recognize that it is definitively obsolete.

Restated question: what exact events should increment cluster generation versus only increment lease epoch, and which component should own that rule?

## Q2 How strict should degraded-majority continuation be?

Context: this design allows `DegradedMajority` to remain operational rather than collapsing all non-full-quorum states into fail-safe.

Problem: some degraded states are safe enough to continue, but others may have too much stale visibility to trust leadership continuity. The machine needs a crisp boundary that later tests can assert directly.

Restated question: what observation thresholds distinguish healthy degraded-majority continuation from ambiguity that must still fence writes?

## Q3 Should bootstrap reuse existing `pgdata`?

Context: the bootstrap path includes substates and explicitly allows reuse of valid local data when this node wins bootstrap authority.

Problem: reusing data may reduce unnecessary rebuilds, but it also increases the burden of proving the data is truly safe and belongs to the intended new cluster generation.

Restated question: under what exact local-data proofs may a bootstrap winner keep existing `pgdata` instead of reinitializing it?

## Q4 How much receiver feedback should re-enter the pure machines?

Context: dedup and execution-state ownership move to receivers, but the local machine still needs enough feedback to decide when rewind failed, basebackup completed, or a primary start is still in progress.

Problem: too little feedback makes the pure model blind; too much feedback risks letting operational noise dominate decision logic.

Restated question: what is the minimal typed receiver feedback surface needed so the local machine stays informed without becoming effect-coupled?

## Q5 Should publication include explicit authority confidence?

Context: the publication view already carries partial truth such as SQL status, readiness, and process state.

Problem: peers and tests may also benefit from knowing whether a node currently believes it has full authority, degraded-majority authority, or no authority. Exposing that can aid debugging, but it can also create another compatibility surface inside DCS records.

Restated question: should member publication include an explicit authority-confidence field, or should that remain derivable only from raw cluster facts and intent logic?
