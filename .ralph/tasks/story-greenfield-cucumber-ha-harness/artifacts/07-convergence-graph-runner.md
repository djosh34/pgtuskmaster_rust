# HA Refactor Option 7: Convergence Graph Runner

This is a design artifact only. It does not authorize code changes in this task, it does not treat green tests as the goal of this task, and it does not justify fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because the current architecture still treats replica repair, startup, restart, rewind, basebackup, ordinary following, and minority demotion as adjacent but separate stories. The live tree already hints that these are one family of problems: `src/runtime/node.rs` still carries old startup planning ideas, `src/ha/decide.rs` distinguishes bootstrap versus failover through candidacy helpers, `src/ha/process_dispatch.rs` reconstructs start intent from DCS facts, and `src/ha/lower.rs` still expresses recovery as a small enum that later code has to reinterpret. The differentiator for this option is that the system becomes a typed convergence graph runner. Instead of asking "what action should I do next" in a mostly imperative way, every tick first asks "what convergence vertex am I currently in, what target vertex is currently authoritative, and which graph edge is valid now?" Startup, rejoin, rewind, basebackup, normal following, promotion, and safe waiting become graph edges with typed prerequisites and typed ownership. That makes this option materially different from option 1's regime classification, option 2's lease-epoch storytelling, option 3's single recovery funnel, option 4's receiver-owned ledger, option 5's peer-evidence vote matrix, and option 6's bootstrap-specific charter machine.

## Ten option set decided before drafting

These are the ten materially different directions this design study uses. This document fully specifies only option 7.

1. `Regime-first reconciliation`
   The system first derives a cluster regime ADT, then derives a local contract from that regime.
2. `Lease-epoch story machine`
   The system is organized around explicit lease epochs and handoff stories, with every transition anchored to epoch ownership.
3. `Startup-as-recovery funnel`
   Startup is deleted as a special case and replaced by one recovery funnel that handles empty, existing, diverged, and stale data uniformly.
4. `Receiver-owned command ledger`
   HA produces stable intent revisions while DCS and process consumers own all idempotency, duplicate suppression, and apply-progress truth.
5. `Peer-evidence vote matrix`
   Leadership is derived from an explicit matrix of peer evidence, majority proofs, and contradiction handling instead of a single trust enum and best-candidate helper.
6. `Bootstrap charter machine`
   Bootstrap is elevated into its own charter state machine with explicit init-lock substates, bootstrap proofs, and durability claims.
7. `Convergence graph runner`
   All startup and replica movement become graph edges between typed data/authority vertices, with the decider choosing the next valid edge from current evidence.
8. `Publication-first truth model`
   Member publication is redesigned first so all other HA logic consumes richer partial-truth envelopes.
9. `Split loops with shared ADTs`
   Startup and steady-state remain separate loops, but they must consume the exact same world and intent ADTs.
10. `Authority contract DSL`
    Leadership, fencing, and switchover are encoded as typed contracts that can be model-checked independently of Postgres actions.

## Diagnostic inputs on March 12, 2026

This option uses the current repository state as input evidence.

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because the graph-runner argument is not "everything is broken." The argument is that the remaining HA complexity is caused by state transitions being implicit, reconstructed late, or split across startup and reconciliation.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out while waiting for one primary across the two restarted fixed nodes and reported the restarted nodes as unknown. That matters directly to this option because a graph runner makes "restart with partially known durable state" an explicit vertex and edge-selection problem instead of a side effect of bootstrap or fail-safe heuristics.
- `tests/ha.rs` remains the acceptance surface for a future implementation. A later implementation based on this option must satisfy that HA suite or deliberately update scenario semantics with explicit architectural reasons.

## Current design problems

### 1. Startup logic is still split across `src/runtime/node.rs`

The repository still shows startup as a separate mental model. `src/runtime/node.rs` carries the remnants of `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, `build_startup_actions(...)`, and `select_resume_start_intent(...)`. Even if some of that code is disabled or no longer the primary path, the architecture still reflects that split: startup decides from one set of facts, then the HA worker later decides from another set of facts. That violates the user's desired invariant that the same newest observations plus the same typed state should produce the same actions on first boot, restart, and normal ticks.

### 2. Sender-side dedup still lives in `src/ha/worker.rs`

The current worker path already resembles the wanted flow: world snapshot, pure decision, lower, publish, apply. The remaining problem is that the sender still owns duplication judgments through `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the node making the decision is also guessing which downstream effects are already in flight. Under faults and retries, that is the wrong authority boundary. The decider should say which graph edge is now valid. The DCS consumer and process consumer should decide whether they already applied that edge.

### 3. HA reasoning is spread across runtime, decider, and process dispatch

Important authority facts are divided among:

- `src/ha/decision.rs` for `DecisionFacts` and `HaDecision`,
- `src/ha/decide.rs` for phase selection and fail-safe shortcuts,
- `src/ha/lower.rs` for effect lowering,
- `src/ha/process_dispatch.rs` for reconstructing start intent and source-member choices,
- `src/runtime/node.rs` for startup and resume concepts.

That spread means no single pure ADT fully answers: what is this node's current convergence state, what is the authoritative target state for the cluster, and what edge is valid right now?

### 4. `src/ha/decide.rs` currently shortcuts non-full quorum into `FailSafe`

The user already called out the current boundary as wrong. The current path routes any non-`DcsTrust::FullQuorum` situation too quickly toward `HaPhase::FailSafe`, and a primary under non-full-quorum may return `HaDecision::EnterFailSafe { release_leader_lease: false }`. In a three-node cluster where two healthy nodes still form the valid majority, the right model is not "trust dropped, therefore fail-safe." The right model is "the cluster graph still has an authoritative majority-backed path to one leader, while the minority must lose authority." This option makes that distinction explicit by separating majority-backed target vertices from minority-isolated local vertices.

### 5. Startup and rejoin remain ambiguous in `src/ha/process_dispatch.rs`

`start_intent_from_dcs(...)` and related helpers in `src/ha/process_dispatch.rs` still reconstruct how a node should start from leader records, member slots, local data-dir shape, and recovery hints. That is too late. By the time process dispatch runs, the pure decider should already have selected a typed convergence edge:

- `InitializePrimaryFromEmpty`
- `ResumePrimaryFromValidData`
- `FollowLeaderFromExistingReplica`
- `RepairByRewind`
- `RepairByBasebackup`
- `DemoteThenFollow`
- `WaitForAuthoritativeSource`

Dispatch should materialize the chosen edge, not rediscover the edge from incomplete downstream clues.

### 6. Member publication and partial truth are present, but not yet rich enough

`src/dcs/worker.rs` and `src/pginfo/state.rs` already preserve useful partial truth: `PgInfoState::Unknown`, `SqlStatus::{Unknown,Healthy,Unreachable}`, and `Readiness::{Unknown,Ready,NotReady}`. That is the right foundation. The missing piece is convergence-specific truth. The member key should also be able to say:

- whether local `pgdata` appears empty, initialized, divergent, or unusable,
- whether a convergence edge is currently running,
- whether the node knows a local timeline and replay point,
- whether the node is alive but cannot currently query Postgres,
- whether the node sees enough evidence to follow but not to promote.

Without that richer publication, other nodes cannot reason about graph edges safely and the local node cannot restart into the same graph vertex deterministically.

## Core proposal

The system should replace the current loosely coupled startup/recovery/action story with a typed convergence graph.

The graph runner has three pure steps:

1. Build one `ObservationEnvelope` containing newest local and DCS facts.
2. Derive one `ConvergenceGraphView` that says:
   the local vertex, the authoritative target vertex, available edges, and disqualifying blockers.
3. Select one `ConvergenceIntent` representing the next valid graph edge, or explicitly select `NoEdge`.

The key idea is that a node is always somewhere in a graph, even when it is not healthy:

- a stopped node with valid primary data is in one vertex,
- a replica with wrong timeline is in another vertex,
- a node with empty data but valid majority evidence is in another vertex,
- a minority-isolated old primary is in another vertex,
- a healthy primary with valid lease proof is in another vertex.

The pure decider does not ask "what command should I issue?" It asks "which graph edge, if any, preserves the authority invariants and moves this node toward the currently authoritative target?"

That yields a cleaner slogan than the current code:

`newest info -> graph view -> selected edge -> lowered intents -> applied effects`

## Proposed control flow from startup through steady state

### High-level flow

1. Every worker tick collects newest evidence:
   DCS lease state, member slots, switchover state, process state, pginfo, local data-dir inspection, and last applied edge receipts.
2. The HA worker converts those observations into one `ObservationEnvelope`.
3. A pure `derive_convergence_graph(...)` function determines:
   the local vertex, globally credible target vertices, majority-backed authority proofs, and graph edges currently allowed.
4. A pure `select_convergence_edge(...)` function chooses exactly one next edge or chooses `NoEdge`.
5. `lower_convergence_intent(...)` converts that selected edge into DCS intents and process intents.
6. DCS and process consumers own idempotency by comparing the lowered `EdgeToken` against their last-applied tokens.
7. The next tick reevaluates from newest evidence. The graph runner never assumes a previous process action succeeded just because it was requested.

### Why this unifies startup and steady state

Startup is just a first graph evaluation where local process state is mostly empty. Restart after outage is a graph evaluation where local durable state exists but live authority may not. Rejoin after partition is a graph evaluation where local data may be stale relative to the authoritative leader vertex. Steady following is simply the graph remaining in `ReplicaFollowing` and selecting `MaintainFollow` or `NoEdge`.

The important architectural change is that the pure graph view owns the question "what state transition is valid now?" instead of splitting it across startup helpers, failover heuristics, and process-dispatch reconstruction.

### ASCII diagram: graph-runner control boundary

```text
     newest local + remote evidence
                 |
                 v
    +-----------------------------------+
    | ObservationEnvelope               |
    | - dcs view                        |
    | - pg view                         |
    | - process view                    |
    | - data-dir view                   |
    | - edge receipts                   |
    +----------------+------------------+
                     |
                     v
    +-----------------------------------+
    | derive_convergence_graph(...)     |
    |                                   |
    | local_vertex                      |
    | target_vertex_set                 |
    | authority_proof                   |
    | allowed_edges                     |
    | blocked_edges                     |
    +----------------+------------------+
                     |
                     v
    +-----------------------------------+
    | select_convergence_edge(...)      |
    | -> EdgeChoice                     |
    |    - initialize_primary           |
    |    - promote_with_lease           |
    |    - follow_current_leader        |
    |    - rewind_then_follow           |
    |    - basebackup_then_follow       |
    |    - demote_and_wait              |
    |    - observe_only                 |
    |    - no_edge                      |
    +----------------+------------------+
                     |
                     v
    +-----------------------------------+
    | lower_convergence_intent(...)     |
    | -> DcsIntent + ProcessIntent      |
    |    tagged with EdgeToken          |
    +----------------+------------------+
                     |
           +---------+---------+
           |                   |
           v                   v
  +------------------+ +-------------------+
  | DCS consumer     | | Process consumer  |
  | owns dedup       | | owns dedup        |
  +------------------+ +-------------------+
```

### ASCII diagram: conceptual convergence graph

```text
 [ObserveOnly]
      |
      | enough majority-backed evidence
      v
 [CanChooseTarget] ------> [WaitForLeaseAuthority]
      |                           |
      | empty data + win          | lease valid + healthy primary
      v                           v
 [InitializePrimary] -------> [PrimaryServing]
      ^                           |
      | resume valid primary data | minority isolated / lease lost
      |                           v
 [ResumePrimary] <--------- [DemotedOrFenced]
      |
      | follower target selected
      v
 [ReplicaAttachable] ---> [ReplicaFollowing]
      |                     |
      | divergence          | target changed
      v                     v
 [NeedsRewind] -------> [Rewinding]
      | failure or impossible
      v
 [NeedsBasebackup] --> [Basebackuping] --> [ReplicaFollowing]
```

## Proposed typed state machine

### Top-level ADTs

```text
struct ObservationEnvelope {
    now: UnixMillis,
    local_member_id: MemberId,
    dcs: DcsObservation,
    pg: PgObservation,
    process: ProcessObservation,
    data_dir: DataDirObservation,
    receipts: ReceiptObservation,
}

struct ConvergenceGraphView {
    local_vertex: LocalVertex,
    authority: AuthorityProof,
    target: TargetVertex,
    allowed_edges: Vec<ConvergenceEdge>,
    blocked_edges: Vec<BlockedEdge>,
}

enum LocalVertex {
    Unknown(UnknownVertex),
    EmptyData(EmptyDataVertex),
    InitializedButOffline(OfflineDataVertex),
    PrimaryCapable(PrimaryCapableVertex),
    PrimaryServing(PrimaryServingVertex),
    ReplicaAttachable(ReplicaAttachableVertex),
    ReplicaFollowing(ReplicaFollowingVertex),
    DivergedReplica(DivergedReplicaVertex),
    CloneRequired(CloneRequiredVertex),
    Fenced(FencedVertex),
}

enum TargetVertex {
    ObserveOnly(ObserveOnlyTarget),
    PrimaryAuthority(PrimaryAuthorityTarget),
    ReplicaOf(MemberId),
    WaitForAuthoritativeLeader(WaitTarget),
}

enum AuthorityProof {
    NoMajorityEvidence(NoMajorityEvidence),
    MajorityVisible(MajorityProof),
    MajorityBackedLeader(MajorityLeaderProof),
    LeaseBackedLeader(LeaseLeaderProof),
    BootstrapMajority(BootstrapMajorityProof),
}

enum ConvergenceEdge {
    PublishTruthOnly(PublishTruthEdge),
    InitializePrimary(InitializePrimaryEdge),
    ResumePrimary(ResumePrimaryEdge),
    AcquireLeaseAndServe(AcquireLeaseEdge),
    FollowLeader(FollowLeaderEdge),
    RewindToLeader(RewindEdge),
    BasebackupFromLeader(BasebackupEdge),
    DemoteAndFollow(DemoteEdge),
    FenceAndObserve(FenceEdge),
    ClearStaleIntents(ClearIntentsEdge),
    NoOp(NoOpEdge),
}

struct EdgeToken {
    authority_scope: AuthorityScope,
    local_member_id: MemberId,
    edge_kind: EdgeKind,
    graph_revision: GraphRevision,
}
```

### Local-vertex meaning

`LocalVertex` is deliberately not a role enum. It describes both durable data shape and live authority posture.

#### `Unknown`

Use this when the node does not yet have enough local evidence to classify itself safely. This is a temporary observation state, not a stable authority state. The graph may allow only `PublishTruthOnly` or `ObserveOnly` from here.

#### `EmptyData`

The local data directory is empty or equivalent to empty for cluster purposes. The node may initialize as primary only if the authority proof says this node is the selected bootstrap or restart winner. Otherwise it may only wait for a leader to clone from.

#### `InitializedButOffline`

The node has recognizable managed data, but Postgres is not healthy or not currently running. This vertex exists specifically to unify startup and restart. A node with valid prior primary data is not the same as an empty node, and a node with valid replica data is not the same as a node needing basebackup.

#### `PrimaryCapable`

The local data appears internally coherent for primary service if authority is granted. This does not mean the node is already the primary. It means a valid edge exists from local durable state to `PrimaryServing` if majority and lease proofs allow it.

#### `PrimaryServing`

The node is running as primary, local health is sufficient, and the authority proof still supports primary service. Losing that proof does not immediately jump to a raw fail-safe role. It transitions to either `FenceAndObserve` or `DemoteAndFollow` depending on whether a valid majority-backed successor exists.

#### `ReplicaAttachable`

The node has enough local state to start or resume as a replica of the currently authoritative leader without destructive repair.

#### `ReplicaFollowing`

The node is already following the current leader on a valid lineage. This vertex supports either `NoOp`, `MaintainFollow`, or a transition toward repair if later evidence shows divergence.

#### `DivergedReplica`

The node has usable durable data, but its timeline or replay position proves that normal follow is unsafe. The graph runner must decide between `RewindToLeader` and `BasebackupFromLeader`.

#### `CloneRequired`

The node cannot safely converge from local data. Either no valid lineage exists, rewind is impossible, or local durable state is unusable. Only a destructive clone edge is allowed here.

#### `Fenced`

The node is intentionally not authoritative and not allowed to accept writes. This vertex is important because the current code treats fail-safe, demotion, and fencing as a blur. Under this option, `Fenced` is explicit and may still lead to a safe replica edge later.

### Target-vertex meaning

The local node is not deciding from local shape alone. It is deciding relative to an authoritative target.

- `ObserveOnly`
  No safe target can be chosen yet, so the node should publish truth and avoid authority moves.
- `PrimaryAuthority`
  The cluster has enough evidence that exactly one leader target is valid now. This may or may not be the local node.
- `ReplicaOf(MemberId)`
  The leader target is another node, and the current node should converge toward following it.
- `WaitForAuthoritativeLeader`
  The cluster likely has enough evidence to reject local primary authority, but not enough to choose a concrete source leader yet.

### Authority-proof meaning

This option intentionally separates "how sure are we" from "what should we become."

#### `NoMajorityEvidence`

The node cannot prove a safe majority-backed target. The only valid edges are truth publication, fencing, or waiting.

#### `MajorityVisible`

The node sees a majority of fresh member facts, but not yet a final leader proof. This commonly appears during restart or after simultaneous node returns.

#### `MajorityBackedLeader`

The node can prove that a particular leader is the only safe target according to fresh majority evidence. This should be sufficient for replicas to attach or remain attached.

#### `LeaseBackedLeader`

The node can prove not only majority visibility but also the currently valid leader lease. This is the strongest ordinary steady-state authority proof.

#### `BootstrapMajority`

The node sees a majority-backed basis for selecting an initial or resumed primary before normal steady-state lease authority is fully re-established.

### Edge selection rules

`ConvergenceEdge` is the center of the design. The pure decider selects one edge, not a generic decision enum that lower layers must reinterpret.

#### `PublishTruthOnly`

Allowed when local information changed but no authority move is safe. This preserves the user's requirement that member keys publish partial truth rather than silence.

#### `InitializePrimary`

Allowed only from `EmptyData` with `BootstrapMajority` or a stronger proof, and only when the graph proves this node is the selected winner.

#### `ResumePrimary`

Allowed from `InitializedButOffline` or `PrimaryCapable` when local data is still authoritative and the graph proves there is no safer competing leader target. This is the explicit answer to the restart-after-total-outage case that currently feels half bootstrap, half resume.

#### `AcquireLeaseAndServe`

Allowed when the node is already converged to primary-capable data and may now acquire or renew the ordinary leader lease. This edge lets the graph distinguish "durable data is suitable" from "operator-visible authority is fully established."

#### `FollowLeader`

Allowed from `EmptyData`, `InitializedButOffline`, or `ReplicaAttachable` when another node is the leader target and no destructive repair is required.

#### `RewindToLeader`

Allowed from `DivergedReplica` only when the graph proves a common ancestry and the leader source is suitable.

#### `BasebackupFromLeader`

Allowed from `CloneRequired` or from `DivergedReplica` when rewind is impossible or failed.

#### `DemoteAndFollow`

Allowed when the local node is or might be primary-capable but majority-backed authority now points elsewhere. This edge is the typed answer to "old primary partitioned then healed." The local node does not wander through generic fail-safe first. It explicitly demotes and targets the new authoritative leader.

#### `FenceAndObserve`

Allowed when local authority is unsafe and no safe follow edge is yet available. This is how a minority-isolated old primary or DCS-blind leader loses write authority without pretending the cluster has vanished.

#### `ClearStaleIntents`

Allowed when previously emitted edges are now superseded. This edge is purely about downstream cleanup and helps keep consumers from holding stale in-flight assumptions forever.

### Graph invariants

The graph runner must preserve these invariants:

1. A node may enter `PrimaryServing` only through an edge backed by majority evidence.
2. `PrimaryServing` requires both durable suitability and authority proof. Health alone is insufficient.
3. A node in `Fenced` may publish truth but may not lower write-authority intents.
4. A node may select `RewindToLeader` only when the source leader is already authoritative.
5. A node may select `BasebackupFromLeader` only when rewind is impossible, invalid, or previously failed for the current source.
6. The same observation envelope must always derive the same graph view and edge choice.
7. Sender-side dedup is forbidden. Repeated selection of the same edge is acceptable because consumers own idempotency via `EdgeToken`.

## Edge-state transitions

This option is not a simple funnel because multiple edges may lead into the same target vertex from different starting vertices.

### Transition family: startup from no process

- `Unknown -> PublishTruthOnly`
  First tick after process start when local inspection or DCS cache is incomplete.
- `EmptyData -> InitializePrimary`
  Selected winner for initial or restart bootstrap.
- `EmptyData -> FollowLeader`
  Empty node should clone from existing authoritative leader.
- `InitializedButOffline -> ResumePrimary`
  Local durable state matches the authoritative primary target.
- `InitializedButOffline -> FollowLeader`
  Local durable state is replica-attachable and another node is authoritative.
- `InitializedButOffline -> RewindToLeader`
  Durable state is close enough to repair non-destructively.
- `InitializedButOffline -> BasebackupFromLeader`
  Durable state cannot safely attach or rewind.

### Transition family: steady primary under topology changes

- `PrimaryServing -> AcquireLeaseAndServe`
  Lease renewal or visibility refresh on stable majority.
- `PrimaryServing -> FenceAndObserve`
  Majority proof lost and no safe successor chosen.
- `PrimaryServing -> DemoteAndFollow`
  Another majority-backed leader target exists or is being established safely.

### Transition family: replica life cycle

- `ReplicaAttachable -> FollowLeader`
  Normal start or restart as a replica.
- `ReplicaFollowing -> NoOp`
  Continue current state because the graph target is unchanged.
- `ReplicaFollowing -> RewindToLeader`
  Divergence was detected after leader change.
- `DivergedReplica -> BasebackupFromLeader`
  Rewind is impossible or failed.
- `CloneRequired -> BasebackupFromLeader`
  The only safe path is destructive clone.

### Transition family: outage recovery

- `InitializedButOffline + BootstrapMajority -> ResumePrimary`
  Two nodes restart after full outage and one has the best authoritative data proof.
- `InitializedButOffline + MajorityBackedLeader(other) -> FollowLeader`
  Restarting node already knows it should be a replica.
- `PrimaryCapable + NoMajorityEvidence -> FenceAndObserve`
  Existing primary data is not sufficient when the cluster majority cannot be proven.

## Redesigned quorum model

The current `DcsTrust` boundary is too coarse for graph selection. The graph runner needs a richer quorum envelope.

### Proposed quorum ADT

```text
enum QuorumEnvelope {
    MajorityFresh(MajorityFreshEnvelope),
    MajorityFreshLeaderKnown(MajorityLeaderEnvelope),
    MajorityFreshLeaderUnknown(MajorityUnresolvedEnvelope),
    MinorityIsolated(MinorityEnvelope),
    DcsBlindLeaseGrace(LeaseGraceEnvelope),
    NoSafeMajority(NoSafeMajorityEnvelope),
}
```

### Meaning of each quorum state

- `MajorityFresh`
  A majority of fresh member facts is available. This is enough to keep evaluating leadership and convergence. It is not enough by itself to promote.
- `MajorityFreshLeaderKnown`
  The graph can identify the authoritative leader target. This is the normal state for a stable cluster and for 2-of-3 degraded but healthy operation.
- `MajorityFreshLeaderUnknown`
  The cluster majority is visible, but target selection still needs more evidence. This commonly appears right after simultaneous restart.
- `MinorityIsolated`
  The local node is clearly in the minority or sees only minority evidence. The local node must never remain primary from here.
- `DcsBlindLeaseGrace`
  The local node temporarily cannot refresh DCS but still has not yet crossed a safe lease cutoff. This is explicitly transitional and time-bounded.
- `NoSafeMajority`
  There is not enough evidence to choose or preserve primary authority safely.

### Why degraded-but-valid majority must continue

In a three-node cluster where two healthy nodes remain connected, the correct graph is still:

- one `PrimaryAuthority` target,
- one leader or promotable leader vertex on the majority side,
- one `MinorityIsolated` old-primary or missing member vertex on the minority side.

That means the majority side should continue service and the minority side should lose authority. Treating this whole situation as fail-safe is too blunt and causes the wrong behavior for `ha_old_primary_partitioned_from_majority_majority_elects_new_primary` and related recovery cases.

### When the node must fence or demote

- Fence when local write authority is unsafe and the graph cannot yet prove a safe replica target.
- Demote and follow when the graph proves another leader target and local data can converge toward it.
- Continue serving only when majority-backed primary authority still points to the local node.

## Lease model

This option keeps lease handling but demotes it from "the whole HA story" to "one class of authority proof."

### Lease phases

```text
enum LeasePosture {
    Absent,
    Candidate,
    HeldByLocal(LeaseRecord),
    HeldByPeer(MemberId, LeaseRecord),
    Expiring(LeaseRecord),
    Expired,
}
```

### Lease interaction with graph edges

- `AcquireLeaseAndServe` may only be selected when the graph already says the local node is the correct primary target.
- `ResumePrimary` is allowed before operator-visible leadership is complete, but it must flow into `AcquireLeaseAndServe` before the node is considered authoritative for writes.
- `DemoteAndFollow` and `FenceAndObserve` explicitly clear or supersede local lease assumptions.
- A killed or partitioned primary loses authority when either:
  the majority side establishes a new target and local lease can no longer be refreshed safely, or
  the local node reaches the lease cutoff without majority-backed renewal proof.

### Startup interaction

Startup uses lease proof only after the graph has classified the node's durable data and chosen whether the cluster target is local primary, peer primary, or unresolved majority. That is the key difference from treating startup as an early lease heuristic.

## Startup reasoning

This option treats startup as graph classification, not a dedicated startup planner.

### Case: cluster already up and leader already present

The restarting node inspects local data and DCS member truth. If the graph view proves another node is the authoritative leader and local data is attachable, the edge is `FollowLeader`. If local data is divergent, the edge is `RewindToLeader` or `BasebackupFromLeader`. There is no separate startup mode enum to invent this later.

### Case: empty cluster or first initialization

An empty local data directory is only one fact. The graph still needs majority-backed authority to choose `InitializePrimary`. If this node loses the winner selection or sees another authoritative leader, it moves toward `FollowLeader` instead.

### Case: existing members already published

The graph should use those publications directly as vertex evidence. A node with empty data and visible authoritative leader should not even consider primary edges. A node with valid primary-capable data and majority-backed absence of any surviving leader may select `ResumePrimary`.

### Case: empty versus existing `pgdata`

This option explicitly encodes the difference:

- empty data means initialize or clone,
- existing replica-compatible data means attach or repair,
- existing primary-capable data means resume or demote,
- unusable data means clone required.

That explicit split is what makes the graph materially different from option 3's single recovery funnel. The graph has multiple durable-state vertices, not one recovery stream.

### Case: init lock behavior

This option does not make bootstrap charter the central ADT, but it still treats `init_lock` as graph evidence. The presence of an init lock contributes to the authority proof and may block some edges. It does not independently decide the whole control flow. That keeps this option distinct from option 6 while still respecting bootstrap safety.

### Case: existing local data reused for initialization or restart

The graph runner may reuse existing local data when it lands in `PrimaryCapable` or `InitializedButOffline` and the authority proof says the local node is the restart winner. Reuse is not allowed merely because the node has data. Reuse requires both:

- local data integrity and lineage evidence,
- majority-backed proof that no safer competing primary target exists.

## Replica convergence under this option

Replica convergence becomes one connected subgraph instead of scattered conditional logic.

### Convergence subgraph

```text
ReplicaAttachable --FollowLeader--> ReplicaFollowing
ReplicaFollowing --leader changed--> DivergedReplica
DivergedReplica --rewind valid--> RewindToLeader
DivergedReplica --rewind impossible--> BasebackupFromLeader
CloneRequired --clone done--> ReplicaAttachable
Fenced --authoritative leader known--> FollowLeader
```

### Why this is better than the current recovery split

The current design carries the right pieces in several places:

- `RecoveryStrategy` in `src/ha/decision.rs`,
- `ReplicationEffect` in `src/ha/lower.rs`,
- start-intent selection in `src/ha/process_dispatch.rs`.

The missing piece is one typed graph that decides those transitions before lowering. With the graph runner, rewind and basebackup are not alternate branch outputs from a generic recovery decision. They are explicit edges chosen from a proven `DivergedReplica` or `CloneRequired` vertex.

## Partial-truth publication redesign

The member key must publish convergence truth, not only role-like observations.

### Proposed publication shape

```text
struct PublishedConvergenceTruth {
    member_id: MemberId,
    pg_shape: PublishedPgShape,
    data_state: PublishedDataState,
    convergence_vertex: PublishedVertex,
    in_flight_edge: Option<PublishedEdge>,
    timeline: Option<TimelineId>,
    flush_lsn: Option<Lsn>,
    replay_lsn: Option<Lsn>,
    sql_status: SqlStatus,
    readiness: Readiness,
    evidence_freshness: FreshnessWindow,
}
```

### Why this matters

If pginfo is degraded but the process is alive, the member key should still say something like:

- process alive,
- data directory present,
- last known timeline `T`,
- last known replay `L`,
- convergence vertex still believed to be `ReplicaAttachable`,
- current SQL check unavailable.

That preserves the user's requirement that "pginfo failed but pgtuskmaster is up" should publish partial truth rather than disappear. It also gives peer nodes enough evidence to choose graph edges safely.

## Deduplication boundary

This option moves deduplication entirely to receivers by making the selected graph edge the idempotency unit.

### New rule

The HA worker may select the same `ConvergenceEdge` on multiple ticks. That is acceptable. The downstream consumer must suppress duplicates by `EdgeToken`.

### Why this is safer than `should_skip_redundant_process_dispatch(...)`

The sender does not know:

- whether the process consumer already launched the job,
- whether the job completed but the acknowledgement was delayed,
- whether the DCS write partially succeeded,
- whether a retry should materialize the same desired edge.

The consumer does know its own apply state. Therefore:

- DCS consumer dedups by last applied `EdgeToken` for DCS intents,
- process consumer dedups by last launched or completed `EdgeToken` per job family.

This preserves the user's desired functional decider while removing transport-specific guesswork from the HA worker.

## Concrete file, module, function, and type changes a future implementation would touch

A future implementation of this option would almost certainly touch these areas:

- `src/ha/decision.rs`
  Replace or heavily refactor `DecisionFacts`, `HaDecision`, and `RecoveryStrategy` into graph-oriented ADTs.
- `src/ha/decide.rs`
  Replace phase-first branching with `derive_convergence_graph(...)` and `select_convergence_edge(...)`.
- `src/ha/lower.rs`
  Replace generic `HaDecision` lowering with lowering from `ConvergenceIntent`.
- `src/ha/worker.rs`
  Build `ObservationEnvelope`, remove sender-side dedup, and publish graph-based receipts.
- `src/ha/process_dispatch.rs`
  Stop inferring start intent from DCS as the primary source of truth; consume explicit graph edges instead.
- `src/runtime/node.rs`
  Remove remaining startup-planner architecture and route startup through the same graph runner.
- `src/dcs/state.rs`
  Extend member publication types to include convergence truth and richer majority evidence.
- `src/dcs/worker.rs`
  Publish the richer truth envelope even under partial pginfo failure.
- `src/pginfo/state.rs`
  Possibly split raw observation from published convergence-ready facts so graph classification can preserve unknowns cleanly.
- `tests/ha.rs`
  Add or adapt graph-oriented assertions where scenario semantics become more explicit.

## Meaningful implementation changes required by this option

- Introduce new ADTs for `ObservationEnvelope`, `ConvergenceGraphView`, `LocalVertex`, `TargetVertex`, `AuthorityProof`, `ConvergenceEdge`, and `EdgeToken`.
- Delete the remaining architectural split between startup planning and steady-state reconciliation.
- Move rewind/basebackup/start-follow selection into the pure decider.
- Change effect lowering so it consumes graph edges rather than reinterpreting broad decisions.
- Move idempotency ownership to DCS and process consumers.
- Extend DCS publication with convergence-specific partial truth.
- Replace the coarse non-full-quorum shortcut with richer majority and minority graph evidence.
- Encode restart-after-total-outage as a normal graph transition, not an implicit bootstrap side path.

## Migration sketch

This option requires an aggressive implementation, because leaving legacy paths behind would undercut the entire point.

### Step 1: Introduce graph ADTs beside current decisions

Add the graph-view types without yet removing current behavior. Map current world snapshots into a parallel `ObservationEnvelope`.

### Step 2: Implement graph derivation in shadow mode

Run `derive_convergence_graph(...)` beside existing decision logic and emit debug-only comparisons. The purpose is to prove the graph vocabulary is expressive enough before it becomes authoritative.

### Step 3: Make lower layers consume `EdgeToken`

Teach DCS and process consumers to persist last-applied edge receipts. Do not yet switch edge selection authority; only establish the receiver-side idempotency substrate.

### Step 4: Replace replica recovery branching first

Move `FollowLeader`, `RewindToLeader`, and `BasebackupFromLeader` into graph-derived selection before touching promotion and startup. This is the cleanest thin slice because it already spans `decision`, `lower`, and `process_dispatch`.

### Step 5: Replace startup planner and restart heuristics

Delete the separate startup-planner architecture from `src/runtime/node.rs` and enter the graph runner from the first tick.

### Step 6: Replace primary/fail-safe branching

Switch promotion, demotion, fencing, and restart-after-outage handling to graph selection. Remove the coarse `DcsTrust` to `FailSafe` shortcut once the graph's authority proofs are complete.

### Step 7: Delete stale legacy paths

Remove unused startup-mode helpers, redundant recovery enums, and sender-side dedup functions. This repo is explicitly greenfield and should not carry both architectures.

## Logical feature-test verification

This section maps the option against the key HA scenarios in `tests/ha.rs`.

### `ha_dcs_quorum_lost_enters_failsafe`

Under this option, nodes that cannot prove a safe majority-backed target enter `FenceAndObserve` and publish a fenced vertex. The operator-visible result still matches the current scenario intent: no authoritative primary and no dual-primary evidence. The semantics become clearer because "fail-safe" is implemented as a graph state where no write-authority edge is allowed.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

The graph runner forces the old primary from `PrimaryServing` to `FenceAndObserve` when the lease cutoff is reached without safe majority-backed renewal. The write cutoff is therefore a typed edge boundary: commits before the `FenceAndObserve` edge may stand; commits after that edge must be rejected.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

This option directly improves that scenario. The majority side remains in `MajorityFreshLeaderKnown` and selects a promotion edge for the best eligible replica. The isolated old primary shifts to `MinorityIsolated` and then `FenceAndObserve`. The majority does not fail-safe just because full quorum disappeared.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

After heal, the old primary's graph view sees `TargetVertex::ReplicaOf(new_primary)`. If the old primary's data is attachable it takes `DemoteAndFollow`; if divergence is proven it takes `RewindToLeader` or `BasebackupFromLeader`. The old primary never regains write authority because the graph target is already established elsewhere.

### `ha_primary_killed_then_rejoins_as_replica`

Once the killed node restarts, its first tick lands in `InitializedButOffline`. The authoritative target is already the new majority-backed leader. Therefore the only valid edges are follow or repair edges toward that leader. The node cannot re-promote itself because it lacks majority-backed primary authority.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one healthy replica returns, the cluster transitions from insufficient majority visibility to `MajorityFreshLeaderKnown`. The original primary may remain `PrimaryServing` if it still holds the correct majority-backed target, and the restarted replica takes `FollowLeader`. This is exactly the kind of degraded-but-valid case the coarse fail-safe shortcut mishandles.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the central scenario for the graph runner. The two restarted fixed nodes both classify their local durable state. The graph then uses majority-backed evidence to select one `ResumePrimary` or `AcquireLeaseAndServe` path and one `FollowLeader` path. The third node later rejoins through the same convergence subgraph. The scenario should stop timing out because restart-after-outage is now a first-class vertex/edge problem instead of an accidental blend of startup and failover logic.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

This option makes the fallback explicit. The node first enters `DivergedReplica`, selects `RewindToLeader`, publishes that in-flight edge, and if rewind fails the next graph view moves to `CloneRequired`, which authorizes `BasebackupFromLeader`. The fallback is typed and visible rather than buried in lower-layer repair branching.

### `ha_replica_stopped_primary_stays_primary`

The primary remains in `PrimaryServing` because majority-backed authority still points to it. The stopped replica is simply absent or in `InitializedButOffline`. No promotion or demotion edge is selected because the target vertex is unchanged.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

The broken replica may oscillate among `InitializedButOffline`, `DivergedReplica`, `CloneRequired`, or `Fenced`, but those local vertices do not alter the majority-backed target while the healthy primary remains authoritative. A failed repair edge affects only that node's convergence subgraph, not cluster leadership.

## Tradeoffs

- This option introduces more types than the current design and more graph vocabulary than some engineers will initially want.
- It requires disciplined separation between graph derivation and lowering, otherwise the design collapses back into imperative branching.
- Receiver-owned idempotency is non-optional here. Without it, repeated edge selection would create noisy retries.
- The graph may feel abstract until debug output and test fixtures render vertices and edges clearly.
- Bootstrap remains an important subproblem, but this option intentionally does not make bootstrap the central ADT. Teams that want bootstrap to dominate the model may prefer option 6.

## Non-goals

- This option does not propose implementing the graph in this task.
- This option does not change production code, tests, docs, configuration, or behavior in this run.
- This option does not attempt to preserve every current enum and helper. A later implementation should delete stale paths aggressively.
- This option does not treat sender-side retry suppression as salvageable. That boundary should move.
- This option does not claim lease proof alone is sufficient for restart-after-outage authority.

## Q1 [Should the graph expose one universal vertex enum or separate durable-state and authority-state enums]

The current proposal uses one `LocalVertex` plus separate `AuthorityProof` and `TargetVertex`, which keeps edge selection centralized but may still mix durable and authority concerns more than necessary.

Would implementers understand transitions better if the graph were decomposed into `DurableStateVertex` and `AuthorityStateVertex`, with legal edge combinations derived from their product?

Is it better to keep one richer vertex for simpler debug output, or should the design make the composition explicit so impossible state pairs are rejected by type construction instead of convention?

## Q2 [How much source-selection detail should live inside the pure edge versus lowering]

`FollowLeader`, `RewindToLeader`, and `BasebackupFromLeader` all need concrete source-member details, timeline proofs, and sometimes transport parameters, but putting every transport detail into the pure graph risks leaking lower-level concerns upward.

Should the edge carry a fully resolved `SourceMemberProof` with lineage evidence and required preconditions, while leaving network/process flags to lowering?

Where is the correct boundary between a graph edge that is semantically complete and a lowerer that still has room to choose operational details without reinterpreting authority?

## Q3 [What is the right restart-winning proof after total outage]

The hardest case remains the full-stop restart where several nodes return with existing durable state and no live leader. The graph needs a majority-backed way to choose `ResumePrimary` for exactly one node without inventing a hidden startup side channel.

Should the winning proof prioritize last known lease owner, highest durable timeline/LSN, explicit DCS resume metadata, or some combination with strict tie-break order?

If two restarted nodes present plausible primary-capable data, what exact proof vocabulary makes the restart winner deterministic and auditable rather than heuristic?

## Q4 [Should published convergence truth include last failed edge attempts]

This design publishes current vertex and in-flight edge, but fallback decisions such as rewind to basebackup may need memory of why a prior edge failed, especially across restarts.

Would it be safer to publish a bounded `last_failed_edge` record with failure class and source member so the next graph view can choose `BasebackupFromLeader` without locally remembered hidden state?

How much failure memory belongs in DCS member truth versus purely local receipts, given that too much shared failure state can become noisy but too little can cause repeated bad edge selection?

## Q5 [Should fencing be a dedicated vertex or a modifier on any vertex]

This document models `Fenced` as its own vertex because the operator-visible and write-authority semantics are important, but another approach would represent fencing as a modifier layered on top of `PrimaryCapable`, `ReplicaAttachable`, or `InitializedButOffline`.

Would a dedicated `Fenced` vertex make reasoning simpler by forcing all authority loss through one visible state, or would it obscure the underlying durable shape that still matters for the next follow or repair edge?

Is it clearer to say "this node is now in `Fenced(PrimaryCapable)`" as a composite type, or to keep fencing as one standalone convergence destination before later transitions reclassify the durable state?
