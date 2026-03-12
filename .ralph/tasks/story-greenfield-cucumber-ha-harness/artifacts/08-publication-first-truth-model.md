# HA Refactor Option 8: Publication-First Truth Model

This is a design artifact only. It does not authorize code changes in this task, it does not treat green tests as the goal of this task, and it does not justify fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because the current HA architecture already has the beginnings of a good data model, but it still compresses local truth too early and too narrowly. `src/dcs/worker.rs` writes one `MemberSlot` per node, `src/dcs/state.rs` gives that slot a compact `MemberPostgresView`, and `src/pginfo/state.rs` already preserves useful partial states like `SqlStatus::Unknown` and `Readiness::Unknown`. The differentiator for this option is that it treats publication itself as the primary architectural boundary. Before the HA decider tries to infer leadership, recovery, fencing, or startup behavior, every node must first publish a richer, typed, partial-truth envelope that says what this node actually knows right now about its process, data directory, durability, authority evidence, and convergence progress. The decider then reasons over a `TruthProjection` built from those envelopes. That makes this option materially different from option 1's regime-first classifier, option 2's epoch-story machine, option 3's recovery funnel, option 4's receiver-owned command ledger, option 5's vote matrix, option 6's bootstrap charter machine, and option 7's convergence graph runner. Here the first architectural question is not "what regime are we in?" or "what edge should I run?" It is "what is the richest truthful statement each node can publish, and can HA become a pure function of that truth?"

## Ten option set decided before drafting

These are the ten materially different directions this design study uses. This document fully specifies only option 8.

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
   All startup and replica movement become graph edges between typed data and authority vertices, with the decider choosing the next valid edge from current evidence.
8. `Publication-first truth model`
   Member publication is redesigned first so all other HA logic consumes richer partial-truth envelopes.
9. `Split loops with shared ADTs`
   Startup and steady-state remain separate loops, but they must consume the exact same world and intent ADTs.
10. `Authority contract DSL`
    Leadership, fencing, and switchover are encoded as typed contracts that can be model-checked independently of Postgres actions.

## Diagnostic inputs on March 12, 2026

This option uses the current repository state as input evidence.

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because this option is not based on a claim that the repo lacks typed state. It is based on a claim that the current typed state is too lossy at the publication boundary.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out while waiting for one primary across the two restarted fixed nodes and reported the restarted nodes as unknown. That matters directly to this option because restart after total outage is exactly the kind of situation where richer published partial truth should make durable local evidence visible before leadership is re-established.
- `tests/ha.rs` remains the acceptance surface for future implementation. A later implementation based on this option must satisfy that HA suite or deliberately revise scenario semantics with explicit architectural reasons.

## Current design problems

### 1. Startup logic is still split across `src/runtime/node.rs`

The task research is correct that startup remains a separate architectural story. Even where older startup planning helpers now appear as remnants or disabled tests in `src/runtime/node.rs`, the design pressure is still visible: startup wants to inspect the data directory, probe DCS, choose a startup mode, and only then hand off to the long-running workers. That split means first boot and restart still invite a different reasoning path from steady-state HA. In a publication-first model, startup must stop being a separate planner and become merely the first opportunity to publish truthful local evidence.

### 2. Sender-side dedup still lives in `src/ha/worker.rs`

The current loop in `src/ha/worker.rs` already has the right broad shape: observe, decide, reconcile, publish, apply. The remaining issue is that sender-owned duplicate suppression still exists through `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the sender is still guessing whether a downstream consumer has already durably adopted the intended work. The user explicitly wants that concern moved away from sender-side HA logic. This option agrees, but it also says the sender cannot make good duplicate judgments because the sender does not publish enough explicit truth about what consumers have actually accepted.

### 3. HA reasoning is spread across runtime, decider, and process dispatch

The current architecture divides meaning across:

- `src/ha/worker.rs` for observation and orchestration,
- `src/ha/decide.rs` for authority and role selection,
- `src/ha/reconcile.rs` and `src/ha/lower.rs` for action selection,
- `src/ha/process_dispatch.rs` for late reconstruction of concrete startup and source details,
- `src/runtime/node.rs` for startup and resume thinking,
- `src/dcs/worker.rs` and `src/dcs/state.rs` for cluster publication and trust interpretation.

That spread causes an important weakness: no single published ADT states what each node knows about itself in a way that every other subsystem can consume consistently.

### 4. `src/ha/decide.rs` currently shortcuts non-full quorum into `FailSafe`

Today `decide(...)` routes any non-`DcsTrust::FullQuorum` condition into `decide_degraded(...)`. That means degraded trust is treated too much like missing truth. In reality those are different. A three-node cluster with one node partitioned can still have two nodes publishing mutually corroborating truth, durable data state, leader visibility, and continuity evidence sufficient to continue or elect. The problem is that the current trust model in `src/dcs/state.rs` is coarse: `FullQuorum`, `Degraded`, or `NotTrusted`, with quorum evaluated mostly from membership presence. A publication-first redesign narrows the issue. The question becomes: does the surviving set publish enough authoritative truth to prove one legal writer and safe follower behavior? If yes, degraded membership count alone should not force fail-safe.

### 5. Startup and rejoin remain ambiguous in `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` still owns `start_intent_from_dcs(...)`, `resolve_source_member(...)`, `validate_rewind_source(...)`, and `validate_basebackup_source(...)`. Those helpers mean the process-dispatch layer is still reconstructing the operational meaning of startup and rejoin from DCS state at the moment commands are emitted. That is too late. If a node must rewind, basebackup, resume primary, or wait, that should already be evident from published truth and the pure decision result. Dispatch should only materialize a fully typed action whose source, lineage, and reason were decided earlier.

### 6. Member publication and partial truth are present, but too narrow

This is the most important problem for this option.

`src/dcs/worker.rs` writes one local member record every tick through `build_local_member_slot(...)`. `src/dcs/state.rs` defines `MemberSlot` as:

- lease,
- routing,
- postgres view.

That is already much better than a boolean "healthy" flag, and `src/pginfo/state.rs` contributes useful distinctions like `SqlStatus::{Unknown,Healthy,Unreachable}` and `Readiness::{Unknown,Ready,NotReady}`. But the published truth still omits several critical facts that later logic keeps trying to rediscover:

- whether the node has missing, empty, existing, or divergent local `pgdata`,
- whether the node is alive but currently cannot query Postgres,
- whether a rewind or basebackup is already in progress,
- whether local durable state might still be authoritative after restart,
- whether the node currently sees itself as following a particular leader lineage,
- which intent revision the DCS and process consumers have last accepted,
- what quality of authority evidence the node actually sees locally.

Because those facts are not first-class published truth, the rest of the stack keeps re-deriving them through startup logic, late dispatch logic, and fail-safe shortcuts.

## Core proposal

The system should redesign HA around one richer published ADT called a `TruthEnvelope`.

Every tick, including the first startup tick, each node must:

1. inspect all newest local evidence,
2. compile that evidence into the richest truthful self-description it can make,
3. publish that `TruthEnvelope` to DCS,
4. consume the full set of published envelopes plus lease and switchover records,
5. derive one pure `TruthProjection`,
6. decide one typed `AuthorityOutcome`,
7. lower that outcome into DCS and process intents,
8. let DCS and process consumers own deduplication by published acknowledgement.

The key slogan becomes:

`newest info -> truth envelope -> truth projection -> decide -> typed outcome -> actions`

This still preserves the user's required functional chain. The difference is that "newest info" is no longer fed directly into ad hoc role heuristics. It is normalized first into a richer, published, cluster-visible truth model.

The strongest claim of this option is:

- startup should publish before it tries to be clever,
- restart should publish durable evidence before it tries to promote,
- rejoin should publish divergence evidence before dispatch chooses repair,
- degraded majority operation should continue only when published truth proves it is safe.

## Proposed control flow from startup through steady state

### High-level flow

1. On each tick, the local node collects raw observations:
   `PgInfoState`, local process state, local data-dir inspection, DCS cache, lease records, switchover records, init-lock state, and consumer acknowledgement state.
2. A pure `compile_truth_envelope(...)` function transforms that raw evidence into one `TruthEnvelope` for this member.
3. The DCS worker publishes that envelope as the local member key.
4. The HA worker reads all member envelopes and converts them into one `TruthProjection`.
5. A pure `decide_from_truth_projection(...)` function selects one `AuthorityOutcome`.
6. `lower_authority_outcome(...)` emits DCS intents and process intents tagged with stable truth-derived revision ids.
7. DCS and process consumers apply or ignore those intents based on their own receiver-side ledgers.
8. The next tick repeats from newest evidence; no action is assumed durable unless a consumer acknowledgement is visible in published truth.

### Why this unifies startup and steady state

Startup becomes the first truth compilation, not a separate planner. On the first tick after process launch, the node may know:

- data directory missing,
- data directory initialized but Postgres offline,
- local timeline present but SQL unreachable,
- process currently starting,
- no leader proof yet,
- init-lock visible or not.

That is already enough to publish a useful `TruthEnvelope`. Other nodes can see it. The local node can see its own durable facts in the same schema that remote nodes will later use. The same compile-then-decide loop now applies on:

- initial bootstrap,
- restart after crash,
- restart after all nodes were down,
- ordinary steady-state service,
- failover,
- rejoin after partition,
- rewind and basebackup recovery.

### ASCII diagram: publication-first control flow

```text
 raw local evidence                           remote published evidence
  - pginfo                                    - peer truth envelopes
  - process state                             - leader lease
  - data-dir inspection                       - switchover intent
  - last consumer receipts                    - init lock
  - dcs cache
           |                                           |
           +-------------------+-----------------------+
                               |
                               v
              +----------------------------------+
              | compile_truth_envelope(...)      |
              |                                  |
              | TruthEnvelope {                  |
              |   liveness, process, data,       |
              |   postgres, lineage, recovery,   |
              |   authority_witness, receipts    |
              | }                                |
              +----------------+-----------------+
                               |
                               v
              +----------------------------------+
              | publish local TruthEnvelope      |
              +----------------+-----------------+
                               |
                               v
              +----------------------------------+
              | build_truth_projection(...)      |
              |                                  |
              | - corroborated facts             |
              | - contradictions                 |
              | - majority proof                 |
              | - viable leader set             |
              | - viable follow sources         |
              +----------------+-----------------+
                               |
                               v
              +----------------------------------+
              | decide_from_truth_projection(...)|
              | -> AuthorityOutcome              |
              +----------------+-----------------+
                               |
                               v
              +----------------------------------+
              | lower_authority_outcome(...)     |
              | -> DCS intents + Process intents |
              +----------------+-----------------+
                               |
                     +---------+---------+
                     |                   |
                     v                   v
            +----------------+ +----------------+
            | DCS consumer   | | Process worker |
            | owns dedup     | | owns dedup     |
            +----------------+ +----------------+
```

### ASCII diagram: publication model for one member key

```text
/scope/member/node-a
    |
    +-- routing
    +-- liveness
    +-- postgres observation
    +-- process observation
    +-- data-dir observation
    +-- authority witness
    +-- convergence status
    +-- receipt ledger

Each field may be partial.
Missing proof is represented explicitly.
Unknown is not silently converted into absence.
```

## Proposed typed state machine

This option still needs a state machine, but the state machine is downstream of publication rather than upstream of it.

### Top-level ADTs

```text
struct TruthEnvelope {
    member: MemberId,
    observed_at: UnixMillis,
    routing: RoutingTruth,
    liveness: LivenessTruth,
    postgres: PostgresTruth,
    process: ProcessTruth,
    data: DataTruth,
    authority_witness: AuthorityWitness,
    convergence: ConvergenceTruth,
    receipts: ReceiptTruth,
}

struct TruthProjection {
    now: UnixMillis,
    members: BTreeMap<MemberId, TruthEnvelope>,
    corroboration: CorroborationView,
    contradictions: Vec<TruthContradiction>,
    authority: AuthorityProofSet,
    topology: TopologyProjection,
    local: LocalTruthView,
}

enum AuthorityOutcome {
    ContinuePrimary(PrimaryContract),
    PromoteCandidate(PromotionContract),
    FollowLeader(FollowContract),
    RewindThenFollow(RewindContract),
    BasebackupThenFollow(BasebackupContract),
    BootstrapPrimary(BootstrapContract),
    WaitForMoreTruth(WaitReason),
    Fence(FenceContract),
}
```

### Richer published sub-ADTs

```text
enum LivenessTruth {
    Responsive,
    ProcessAliveButSqlUnknown,
    ProcessAliveButSqlUnreachable,
    ProcessOffline,
    ObservationStale,
}

enum DataTruth {
    MissingDataDir,
    EmptyDataDir,
    ExistingPrimaryCapable(PrimaryDurabilityTruth),
    ExistingReplicaCapable(ReplicaDurabilityTruth),
    Diverged(DivergenceTruth),
    Uninspectable(UninspectableReason),
}

enum PostgresTruth {
    Unknown(UnknownPostgresTruth),
    Primary(PrimaryPostgresTruth),
    Replica(ReplicaPostgresTruth),
}

enum AuthorityWitness {
    LeaseHolder(LeaseWitness),
    SeesLeaseHolder(LeaseWitness),
    NoLeaseVisible,
    SeesContradictoryLease(ContradictoryLeaseWitness),
    CannotTrustLease(LeaseTrustGap),
}

enum ConvergenceTruth {
    Idle(NoConvergenceWork),
    Starting(StartTruth),
    Promoting(PromoteTruth),
    Demoting(DemoteTruth),
    Rewinding(RewindTruth),
    Basebackuping(BasebackupTruth),
    WaitingForLeader(WaitForLeaderTruth),
    WaitingForMoreTruth(WaitForTruthReason),
}

struct ReceiptTruth {
    dcs_revision: Option<RevisionReceipt>,
    process_revision: Option<RevisionReceipt>,
}
```

### Why these types matter

The current `MemberSlot` schema tells peers roughly where to connect and roughly what Postgres role a node believes it has. It does not tell peers whether that belief came from a healthy query, a stale last-known timeline, a currently running rewind, a failed basebackup, or a restart with durable but not yet confirmed primary data. In this redesign, those states are not inferred later. They are explicit published truth.

### State machine phases derived from truth

Once the `TruthProjection` exists, the pure decider derives one `AuthorityOutcome` by classifying the cluster into one of these conceptual phases:

1. `TruthInsufficient`
   There is not enough corroborated publication to prove one safe writer or one safe recovery path.
2. `BootstrapElection`
   There is no durable leader proof yet, but enough truth exists to lawfully select a bootstrap authority.
3. `PrimaryContinuation`
   One member has corroborated authority and may continue serving.
4. `PromotionElection`
   A valid majority-backed candidate may replace a lost or fenced primary.
5. `ReplicaConvergence`
   A member is non-authoritative but has enough truth to follow, rewind, or basebackup toward the chosen leader.
6. `MinorityContainment`
   A member has local data or process state that would be dangerous if treated as authoritative and must fence or wait.
7. `ObservationOnly`
   The node should publish truth but take no disruptive action yet.

These phases are not published directly. They are pure projections derived from the richer member envelopes.

### Invariants

- Every member key must prefer explicit `Unknown` variants over omitted fields.
- Every authoritative decision must cite a majority-backed truth proof, not merely a non-empty member set.
- A node with dangerous ambiguity publishes that ambiguity instead of silently suppressing its member key.
- A process or DCS intent is never considered applied until receiver receipts say so.
- Startup, restart, and rejoin all begin from the same `TruthEnvelope` compilation step.
- A minority-isolated old primary must publish evidence that leads the majority to fence it and itself to stop writes once its authority proof is gone.

## Detailed quorum model

### Why the current quorum model is too coarse

`src/dcs/state.rs` currently exposes `DcsTrust::{FullQuorum,Degraded,NotTrusted}` and `evaluate_trust(...)` largely depends on DCS health plus whether the local member is present in a member set that reaches a simple threshold. That is a sensible first approximation, but it is too coarse for the user's stated boundary. It treats "not full quorum" as "do not trust the decision surface" far too early.

### Publication-first quorum principle

This option replaces coarse trust gating with a two-step model:

1. `Transport trust`
   Can I read and write DCS reliably enough to participate?
2. `Authority truth`
   Do the published envelopes contain enough corroborated evidence to prove one safe writer contract?

These are not the same thing. A cluster can have degraded transport visibility and still have a lawful majority-backed writer proof. Conversely, transport can look healthy while the published truth is contradictory or too incomplete to promote anyone safely.

### Proposed quorum ADTs

```text
enum TransportTrust {
    Healthy,
    DegradedIo,
    Unavailable,
}

enum AuthorityTruth {
    MajorityProven(MajorityProof),
    MajorityVisibleButAmbiguous(AmbiguitySet),
    MinorityOnly(MinorityView),
    Contradictory(ContradictionSet),
    Unknown(UnknownTruthGap),
}
```

### How 2-of-3 operation works here

In a three-node cluster, if two nodes can publish:

- live member envelopes,
- mutually corroborating leader and follower observations,
- consistent lineage or lease witness,
- durable local state compatible with continued service,

then those two nodes have `AuthorityTruth::MajorityProven`, even if a third node is absent. The serving node remains primary or the healthy follower is promoted. The missing third node does not force `FailSafe` merely because the cluster is not at full membership visibility.

### When the node must still fence or wait

This option does not soften safety. It narrows it.

The node must fence or wait when:

- it cannot prove a majority-backed writer contract,
- it sees contradictory leader claims with no tie-breakable proof,
- its local primary state might be stale relative to published majority truth,
- its transport to DCS is lost and no previously valid local authority proof remains within lease bounds,
- its only available recovery source is untrusted or contradictory.

### Re-election rule

Re-election occurs only when the `TruthProjection` can prove:

- the old primary no longer has a valid authority contract,
- a candidate has a sufficiently recent and corroborated durability position,
- the candidate is not lagging beyond the acceptable promotion boundary,
- the majority-published truth makes the promotion uniquely safe.

That decision no longer comes from a bare "best candidate" ranking over a thin peer model. It comes from richer published truth.

## Detailed lease model

### Lease publication remains important, but becomes one witness among several

This option does not remove the lease. It removes the idea that the lease record alone is the whole truth.

The lease becomes one published witness inside the larger truth system:

- who currently holds the leader lease,
- what generation it belongs to,
- which members corroborate seeing that holder,
- whether the holder itself publishes primary-capable or primary-serving truth,
- whether receipts show the writer contract was actually enacted.

### Lease acquisition

Only a node whose `TruthProjection` yields `AuthorityTruth::MajorityProven` for promotion or continuation may lower an `AcquireLease` or `RenewLease` intent. The lease is thus downstream of published truth, not upstream of it.

### Lease expiry or loss

When a primary loses lease visibility, the next outcome depends on published truth:

- if the node still has corroborated majority proof that it remains the lawful writer, it may continue while attempting lease repair only if the design later codifies that boundary explicitly,
- if it cannot prove ongoing authority, it moves to `Fence`,
- if it is offline or ambiguous, it publishes uncertainty and waits.

This is intentionally stricter than "primary always survives degraded transport" and intentionally less blunt than "any non-full quorum means fail-safe."

### Killed primary loses authority

If a primary is killed, its truth envelope eventually shifts from `PrimaryServing` to stale or offline. Surviving nodes then rely on:

- last durable WAL truth,
- lease witness loss or supersession,
- majority-published follower truth,
- consumer receipts indicating who has enacted new leadership.

The old primary, when restarted, does not guess from raw local files alone. It first publishes durable local truth and then receives a recovery contract from the decider.

### Startup and lease interaction

At startup, a node may publish:

- existing primary-capable data,
- no active SQL connection yet,
- no valid lease held locally,
- observed majority truth from peers,
- need for continuity evaluation.

That lets the decider distinguish:

- safe resume as primary,
- safe resume only as replica,
- unsafe ambiguity requiring wait,
- eligible promotion due to absent leader proof.

## Startup reasoning

### Startup becomes "publish durable local truth first"

This option makes startup obey a strict order:

1. Inspect local data directory and process state.
2. Read current DCS snapshot.
3. Compile and publish a local `TruthEnvelope`.
4. Build the full `TruthProjection`.
5. Decide the startup outcome from the same projection used in steady state.

There is no separate startup planner that invents a parallel mode vocabulary.

### Cases this must cover

#### Cluster already up and leader already present

If the cluster already has a corroborated leader and this node has empty or replica-capable data, the node publishes its state and receives either `FollowLeader`, `RewindThenFollow`, `BasebackupThenFollow`, or `WaitForMoreTruth`.

#### Existing members already published

That is ideal for this option. Existing publications become the authoritative context. The restarted node does not need a separate "startup probe" concept because the projection already contains peer truth.

#### Empty vs existing `pgdata`

`DataTruth` explicitly distinguishes:

- missing data dir,
- empty initialized dir,
- existing primary-capable data,
- existing replica-capable data,
- divergent data,
- uninspectable data.

Because these variants are published, other nodes can reason about whether a node should bootstrap, follow, or be fenced.

#### Init lock behavior

The init lock remains a cluster-scoped witness, but bootstrap selection now requires published local truth. A node can only bootstrap if:

- the init lock policy permits it,
- there is no stronger published durability proof for an existing leader,
- the node's own truth envelope shows a valid bootstrap-capable state.

#### Existing local data may still be valid for initialization

This option explicitly allows that possibility. If all nodes were down and a restarting node publishes `ExistingPrimaryCapable` truth with credible lineage and no contradictory higher-authority proof appears, the decider may conclude that the node should resume or continue authority rather than discard data and re-bootstrap. That is exactly the kind of nuance a thin member slot cannot express well enough today.

## Replica convergence as one coherent path

This option does not center the architecture on recovery stages, but it still gives replica convergence a coherent contract because the needed inputs are now published.

### Healthy follow

If the local envelope says:

- replica-capable data,
- leader lineage matches the chosen authority,
- replay position is acceptable,
- no repair job is active,

then `FollowLeader` is selected.

### Tolerable lag

Lag is not automatically disqualifying. The `TruthProjection` evaluates whether the lagging replica remains a valid follower and whether it is still promotable. Publication-first design helps because lag, readiness, and upstream identity are visible to all nodes as published truth rather than only as local transient input.

### Wrong-timeline rewind

If published truth shows:

- the cluster has chosen a leader,
- the local node has durable data on an older compatible branch,
- rewind is possible from a valid published source,

then the outcome is `RewindThenFollow`.

### Basebackup fallback

If the local truth or published receipts show rewind failed or is impossible, the next outcome becomes `BasebackupThenFollow`. That transition should happen in pure decision logic, not in late process-dispatch source reconstruction.

## Partial-information publication

This section is the heart of the option.

### Principle

If pgtuskmaster is up, the member key should contain the richest truthful partial observation it can publish, even when Postgres itself is degraded.

### Examples

If pginfo fails but the process worker still sees a running postmaster, the envelope should be able to say:

- process alive,
- SQL unknown or unreachable,
- data directory exists,
- last known timeline maybe present,
- current readiness unknown or not-ready,
- no safe promotion proof.

If Postgres is offline but durable local data exists, the envelope should say that explicitly instead of disappearing.

If basebackup is running, the envelope should say that explicitly instead of forcing peers to infer it from lack of readiness.

### Proposed publication schema changes

The current `MemberPostgresView` should be replaced or embedded inside a richer member publication model such as:

```text
struct MemberTruthRecord {
    lease: MemberLease,
    routing: MemberRouting,
    truth: TruthEnvelope,
}
```

or, if the design wants clearer separation:

```text
struct MemberTruthRecord {
    identity: MemberIdentity,
    routing: MemberRouting,
    liveness: LivenessTruth,
    postgres: PostgresTruth,
    process: ProcessTruth,
    data: DataTruth,
    authority_witness: AuthorityWitness,
    convergence: ConvergenceTruth,
    receipts: ReceiptTruth,
}
```

### Why this is safer

This design reduces "absence means maybe dead, maybe unknown, maybe busy, maybe stale" ambiguity. Every peer gets the same richer facts, which means:

- promotion decisions can be based on durable truth rather than guesswork,
- restart can reuse local durable evidence without a parallel startup protocol,
- minority containment can be reasoned about from published contradictions,
- the DCS member key becomes a first-class HA input rather than merely a reachability hint.

## Where deduplication moves

Deduplication moves out of sender-side HA code and into the DCS and process consumers.

### Why sender-side dedup is unsafe

The sender can observe intent generation, but it cannot know whether:

- the process worker accepted the job,
- the job is still running,
- the job completed successfully,
- the DCS mutation was already applied,
- the prior intent failed and requires revision.

That truth belongs to the consumers.

### Receiver-owned dedup with published receipts

Each consumer maintains its own ledger keyed by a stable revision identity. The consumer then publishes its last accepted or completed revision back into the member truth envelope or another consumer-owned record.

That means the HA loop receives actual receipt truth, not inferred action truth.

Possible revision identity:

```text
struct IntentRevision {
    cluster_epoch_hint: Option<u64>,
    authority_revision: u64,
    intent_kind: IntentKind,
    target_member: MemberId,
}
```

The exact identity can vary, but the core rule stands: senders emit revisions, receivers acknowledge them, and published receipts become inputs to the next decision.

## Concrete repo files, modules, functions, and types a future implementation would touch

- `src/dcs/state.rs`
  Replace or extend `MemberSlot`, `MemberPostgresView`, and `evaluate_trust(...)` with richer truth-envelope types and a transport-trust versus authority-truth split.
- `src/dcs/worker.rs`
  Replace `build_local_member_slot(...)` usage with `compile_truth_envelope(...)` plus richer publication writes.
- `src/pginfo/state.rs`
  Preserve its partial-truth richness and map it more directly into `PostgresTruth` rather than flattening it into a narrow member publication.
- `src/ha/worker.rs`
  Change observation flow so it compiles local truth and consumes published receipts instead of doing sender-owned duplicate suppression.
- `src/ha/decide.rs`
  Replace early degraded-to-fail-safe branching with `decide_from_truth_projection(...)`.
- `src/ha/reconcile.rs`
  Lower the new `AuthorityOutcome` contracts into ordered DCS and process intents.
- `src/ha/lower.rs`
  Either absorb new truth-derived contracts or become the single lowerer from `AuthorityOutcome` to effects.
- `src/ha/process_dispatch.rs`
  Delete late source rediscovery and startup-intent reconstruction in favor of fully typed inputs.
- `src/ha/decision.rs`
  Likely becomes the natural home for `TruthProjection`, `AuthorityOutcome`, and proof ADTs.
- `src/runtime/node.rs`
  Delete or collapse startup-planning remnants so startup only seeds workers and the first truth publication.
- `src/process/state.rs` and process worker modules
  Add receiver-owned intent ledgers and acknowledgement publication.
- `tests/ha.rs` and HA feature files
  Update or expand assertions where semantics intentionally change around degraded-majority continuation, receipt publication, or restart truth visibility.

## Meaningful changes required for this option

### New types

- `TruthEnvelope`
- `TruthProjection`
- `LivenessTruth`
- `DataTruth`
- `AuthorityWitness`
- `ConvergenceTruth`
- `ReceiptTruth`
- `AuthorityTruth`
- `AuthorityOutcome`
- majority proof and contradiction ADTs

### Deleted or collapsed paths

- startup-only planning vocabulary that is separate from HA reconciliation,
- sender-owned duplicate suppression heuristics in HA worker code,
- late startup source reconstruction in process dispatch,
- trust logic that equates degraded membership visibility with immediate fail-safe.

### Moved responsibilities

- DCS worker becomes responsible for publishing richer local truth,
- HA decider becomes responsible for converting published truth into authority contracts,
- DCS and process consumers become responsible for idempotency and receipt truth,
- startup logic becomes worker bootstrap only, not a separate HA planner.

### Changed transitions

- restart enters through publication of durable truth rather than mode selection,
- failover requires majority-backed truth proofs rather than thin peer ranking alone,
- rejoin chooses follow, rewind, or basebackup from published truth rather than late dispatch heuristics,
- minority old primaries fence because authority proof is contradicted or absent, not merely because trust is not "full quorum."

### Changed effect-lowering boundaries

- lowering receives already-decided authority contracts with explicit source and lineage semantics,
- process dispatch only materializes typed operations,
- DCS lowerers include receipt publication and contradiction visibility.

### Changed DCS publication behavior

- member keys become richer truth records,
- absence is used only for actual expiration, not as a stand-in for uncertainty,
- member truth retains partial data during degraded pginfo cases,
- consumer receipts become visible cluster state.

### Changed startup handling

- no separate startup planner,
- first tick publishes truth,
- authority selection after restart uses the same decider as steady-state HA.

### Changed convergence handling

- repair state becomes explicit published truth,
- peers can see when a node is rewinding or basebackuping,
- promotion candidates are evaluated against published durability and contradiction evidence.

### Required test updates for later implementation

- HA tests around degraded DCS trust must assert the narrower, proof-based continuation boundary,
- restart scenarios should assert that nodes publish durable partial truth before becoming fully ready,
- receipt-ledger tests must assert that duplicate sender intents do not create duplicate consumer actions,
- rejoin tests should assert that rewind versus basebackup selection comes from pure decision state, not dispatch-time rediscovery.

## Logical feature-test verification

This section maps the option logically against the key HA scenarios from `tests/ha.rs`.

### `ha_dcs_quorum_lost_enters_failsafe`

This scenario should still pass when authority truth truly becomes insufficient. The difference is that fail-safe is no longer triggered by coarse degraded membership alone. It is triggered when published truth cannot prove a safe writer contract.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This scenario remains valid. Under this option, the fencing boundary becomes easier to explain: the member publishes that its authority proof is gone or contradicted, and the resulting `Fence` outcome carries the cutoff proof used to block future writes.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

This scenario should remain valid and become clearer. The majority side can promote only if its published truth proves the old primary no longer has lawful authority and the chosen candidate has the best corroborated durability position. The isolated old primary publishes stale or minority-only truth and must not continue as writer.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

This scenario becomes cleaner because the healed old primary first publishes local durable truth. The decider compares that truth against the now-authoritative majority truth and returns either `RewindThenFollow` or `BasebackupThenFollow`.

### `ha_primary_killed_then_rejoins_as_replica`

This remains valid. After the kill, the old primary's truth stops refreshing. A new primary can be chosen from majority-backed published truth. When the old primary returns, it first publishes its local durable state and receives a non-authoritative rejoin contract.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

This scenario should still pass. The crucial benefit here is that the restarted replica can publish useful partial truth immediately, making it visible as a quorum-restoring participant before it is fully caught up.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the scenario that matters most to this option. The current failure report about restarted nodes appearing unknown is exactly what a richer `TruthEnvelope` should address. Restarted nodes should publish:

- durable local data state,
- process liveness,
- authority witness gaps,
- readiness unknown but alive,
- local timeline and WAL evidence if available.

That richer truth should make one lawful restart path visible instead of leaving both nodes in a thin "unknown" condition.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

This should remain valid and become easier to model. The failure of rewind becomes published receipt truth, which changes the next pure outcome from `RewindThenFollow` to `BasebackupThenFollow`.

### `ha_replica_stopped_primary_stays_primary`

This remains valid. The primary continues because the stopped replica's absence or offline truth does not invalidate a majority-backed writer contract.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

This remains valid and arguably becomes safer. A broken rejoin attempt becomes one member's published recovery problem, not a reason to destabilize cluster-wide authority. The cluster sees that the node is in failed rewind or failed basebackup truth and continues if majority authority remains intact.

## Migration sketch

This option requires a future implementation to move deliberately and delete stale paths as it goes.

1. Introduce richer truth ADTs in `src/dcs/state.rs` and `src/ha/decision.rs` without yet deleting the old member view.
2. Teach `src/dcs/worker.rs` to compile a `TruthEnvelope` from existing pginfo, process state, and data-dir observation.
3. Publish the richer truth alongside current member data for one transition period inside the greenfield implementation branch.
4. Build `TruthProjection` and a pure `decide_from_truth_projection(...)` path in parallel with current `decide(...)`.
5. Move source-member selection, rewind/basebackup choice, and startup-mode choice upward into truth-derived outcomes.
6. Add receiver-owned receipts for DCS and process consumers.
7. Switch HA worker execution to consume receipt truth and stop using sender-owned dedup.
8. Delete old `MemberPostgresView`-only assumptions, old degraded trust shortcuts, and old startup-only planning remnants.
9. Remove any fallback path that re-derives authority from thinner data after the richer truth path exists.
10. Rewrite tests to assert the new truth-driven boundaries explicitly and delete stale expectations tied to the old shortcuts.

The critical rule for later implementation is that the repo must not keep both a thin truth model and a rich truth model indefinitely. This project is greenfield. Legacy parallelism should be deleted, not preserved.

## Non-goals

- This option does not attempt to minimize the number of ADTs. It intentionally increases type richness.
- This option does not make the DCS member key tiny. It chooses clarity over compactness.
- This option does not say the lease is unimportant. It says the lease is insufficient on its own.
- This option does not guarantee zero HA test updates. Some tests should change if the degraded-majority boundary is intentionally narrowed to proof-based continuation.
- This option does not claim that publication alone solves every HA problem. It claims that every HA problem becomes easier to solve when publication stops being lossy.

## Tradeoffs

- Richer publication records increase serialization and schema complexity.
- More published truth creates stronger incentives to define precise freshness and staleness rules.
- Engineers may disagree on how much local detail belongs in a member record versus auxiliary keys.
- This option risks turning DCS publication into a dumping ground unless the ADTs are disciplined.
- Compared with graph-first or epoch-first designs, this option is less elegant as a single abstract story and more pragmatic as an information-model correction.

## Q1 Should consumer receipts live inside the member truth record or beside it

Context: this option wants receiver-owned deduplication with published acknowledgement, but that acknowledgement could either be embedded in the member key or emitted as a separate consumer-owned key.

Problem: embedding receipts inside one member record keeps the cluster view unified, but it may create write-amplification and ownership overlap between DCS publication and process consumers.

Restating question in different way: is the cleanest design "one member truth envelope owns every published fact" or "member truth plus separate receipt records that are joined in the projection layer"?

## Q2 How much durable local data evidence should be published before SQL is reachable

Context: restart after outage is a central motivation for this option. A node may know that `PG_VERSION` exists, a timeline file exists, and the process is offline before SQL is queryable.

Problem: publishing too little keeps restart ambiguous, but publishing too much inferred durability from filesystem inspection alone may overstate what the node truly knows.

Restating question in different way: where is the right line between "useful durable truth" and "premature inference" for pre-SQL startup publication?

## Q3 Should majority proof rely only on published envelopes or also on direct local observations not yet published

Context: the local node always has freshest self-observation before DCS reflects it. The decider could choose to include that freshest local truth even before the write round-trip finishes.

Problem: using unpublished local truth may improve reactivity, but it weakens the principle that authority should flow from shared cluster-visible truth.

Restating question in different way: should the decisive proof surface be strictly cluster-published truth, or may the local node temporarily reason over fresher self-truth than peers can yet see?

## Q4 How should contradiction between lease witness and durability witness be resolved

Context: a node may publish that it sees a lease holder, while durability and recovery truth suggest that holder is stale, dead, or superseded.

Problem: if lease witness and durability witness disagree, the projection layer needs an explicit precedence model or a contradiction ADT that blocks action.

Restating question in different way: when the truth system sees "current lease says A" but "majority durability proof favors B," should the result be immediate fence, staged ambiguity, or a higher-order conflict resolution contract?

## Q5 How large is too large for a single member truth record

Context: this option deliberately enriches the member key with liveness, data-dir, process, authority, convergence, and receipt facts.

Problem: one record that is too large or too frequently updated could create DCS churn, but splitting the model too early may recreate the current architecture's information fragmentation.

Restating question in different way: what is the minimal truth envelope that still prevents late rediscovery of startup, rejoin, and authority meaning?
