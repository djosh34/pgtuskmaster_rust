# Option 10: Roundbook Receipt Protocol

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. This document exists only to describe one complete redesign option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA architecture still behaves too much like a collection of local conditionals and too little like an explicit protocol. The user's complaints all point at missing protocol structure:

- startup and steady-state use different decision entrypoints
- the HA worker still decides too much about whether downstream work is "already active"
- degraded-majority cases do not move through an explicit re-election protocol and instead fall too quickly into fail-safe
- leader loss, replica repair, rewind, and basebackup are all inferred through scattered branches instead of one visible sequence of protocol stages

The differentiator of Option 10 is that every HA tick is interpreted as part of a typed protocol roundbook. A roundbook is a compact, deterministic description of what round the cluster is in, what guards must hold, what receipts prove progress, and what next round is allowed. The pure decider does not answer "what role am I?" first. It answers "what protocol round is valid right now, what receipts are still missing, and what local execution card follows from that round?"

The resulting chain is:

`newest observations -> build roundbook -> choose active round -> emit round outcome and required receipts -> lower into effects -> effect consumers produce receipts`

Startup is not special because it simply begins in the first round. Rejoin is not special because it is another round transition. Deduplication is not sender-owned because consumers own receipt production and therefore own idempotency. This is materially different from the first nine options because the primary abstraction is not a kernel, split state machine, lease core, recovery funnel, generation ledger, evidence table, obligation graph, constitution, or case verdict. It is a visible protocol script with explicit rounds and receipts.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` completed successfully in the repo root. That means the default test profile is not the reason for this design task.
- `make test-long` failed in HA-oriented scenarios, which is exactly the area this redesign targets.
- Observed failing themes from prior diagnostic collection in this task:
  - `ha_dcs_quorum_lost_enters_failsafe` and `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes` indicate the current system does not expose the intended fail-safe boundary consistently.
  - `ha_old_primary_partitioned_from_majority_majority_elects_new_primary` and `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover` indicate degraded-majority re-election and old-primary repair are still not represented clearly enough.
  - `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum` and related service-restoration cases indicate the current transition structure still allows stale authority or stale local activity to outlive the safety boundary expected by the feature suite.
- These outcomes are diagnostic inputs only. This task does not fix them. This option describes one architecture that could later fix them by making the protocol rounds explicit.

## Current design problems

### Startup logic is still split away from the long-running HA protocol

`src/runtime/node.rs` still contains separate startup planning and execution paths such as `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. That means the most dangerous authority decisions can happen before the main HA loop has even built a consistent ongoing model. When startup uses a different planner from steady-state, it becomes easy for the same facts to produce one action at process launch and a different action a few moments later once the worker loop begins.

### Sender-side dedup still lives in `src/ha/worker.rs`

The HA worker still contains `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the sender is trying to predict downstream execution state instead of letting consumers prove whether they already applied a command. This is exactly the wrong place for idempotency in a distributed control loop. A sender can only infer. A consumer can know.

### HA reasoning is spread across too many module boundaries

The current architecture already resembles `world snapshot -> decide -> lower -> publish -> apply`, but the meaning of each stage is not protocol-visible. `src/ha/decide.rs`, `src/ha/lower.rs`, `src/ha/process_dispatch.rs`, `src/runtime/node.rs`, and `src/dcs/worker.rs` each carry a piece of the authority story. There is no single protocol object that says:

- what round the node thinks the cluster is in
- what evidence made that round valid
- what work is still outstanding
- what proof would allow the next round

Without that, the operator sees symptoms instead of a protocol narrative.

### The non-full-quorum to `FailSafe` shortcut is too blunt

`src/ha/decide.rs` still routes any non-`DcsTrust::FullQuorum` state into `HaPhase::FailSafe`. That shortcut hides an important distinction between:

- no safe majority exists
- a safe acting majority exists but full visibility does not
- this specific node is outside the acting majority and must stop serving
- the old leader lost authority and a challenger may now proceed

The user explicitly wants 2-of-3 style degraded-majority service to continue when a valid majority still exists. The current shortcut prevents the protocol from expressing that middle ground.

### Startup and rejoin logic remain too implicit in `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` still carries the authoritative bridge from HA decision to process start intent, including data-source validation, rewind vs basebackup routing, and leader-member selection. That makes it hard to tell whether startup, rejoin, and repair are part of the same policy or just accumulated dispatcher behavior. The result is hidden protocol state encoded as dispatch heuristics.

### Member publication still risks collapsing partial truth into weak inputs

`src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already contain some notion of partial truth, but the architecture still does not force partial truth to stay protocol-visible. "pginfo failed but pgtuskmaster is alive" should remain publishable and useful. Silence is not equivalent to unknown, and unknown is not equivalent to unhealthy. A protocol redesign has to preserve those distinctions all the way into the decider.

## Core idea

The pure decider emits a `HaRoundbook`, and the active roundbook contains:

- the newest merged observation set
- the currently valid `ProtocolRound`
- the guards that justify that round
- the receipts already satisfied
- the receipts still required before the round is considered complete
- the `RoundOutcome` for this specific node
- the next rounds that are legally reachable

The active round is a cluster protocol stage, not merely a local role. Examples:

- `Discover`
- `BootstrapClaim`
- `LeaderEstablish`
- `LeaderServe`
- `ReplicaConverge`
- `FormerPrimaryRepair`
- `QuorumSuspend`
- `FenceIsolate`

Each round may have local execution variants, but the protocol stage comes first. The key idea is that every dangerous transition now becomes a guarded round transition with explicit proof of completion.

For example, a node does not merely say "I should be primary." It says:

- the active round is `LeaderEstablish`
- this node is the designated leader candidate for the current acting majority
- it must satisfy `LeaseHeld`, `PromotionComplete`, `PublicationRefreshed`, and `WritePermitGranted` receipts before entering `LeaderServe`

Likewise, a node does not merely say "I should rejoin as a replica." It says:

- the active round is `FormerPrimaryRepair` or `ReplicaConverge`
- the node lacks a valid serving receipt for the current authority epoch
- it must satisfy `LeaderSourceConfirmed`, `TimelineFitnessChecked`, `RewindComplete` or `BasebackupComplete`, and `FollowConfigApplied` receipts before it can enter steady replica following

This gives the user what they asked for:

- newest observations first
- pure decide step second
- typed outcome third
- all side effects below the decider
- startup folded into the same state evolution
- dedup moved out of sender logic

## Proposed control flow from startup through steady state

Every tick, including the first startup tick, follows one entrypoint:

1. Gather newest inputs from DCS state, pginfo state, local process state, persisted local receipts, and runtime probes.
2. Build one `ObservationBundle`.
3. Convert that bundle into one `HaRoundbook`.
4. Select the active `ProtocolRound` and `RoundOutcome`.
5. Lower the outcome into effect plans.
6. Send effect plans to consumers.
7. Consumers execute or reject work idempotently and emit receipts.
8. Receipts feed the next tick.

### Roundbook flow diagram

```text
           +------------------------------+
           | DCS snapshots + pginfo +     |
           | process facts + local disk   |
           | receipts + timers            |
           +---------------+--------------+
                           |
                           v
              +------------+-------------+
              | ObservationBundle        |
              | newest partial truths    |
              +------------+-------------+
                           |
                           v
              +------------+-------------+
              | build_roundbook(...)     |
              | pure protocol selection  |
              +------------+-------------+
                           |
        +------------------+------------------+
        |                                     |
        v                                     v
+-------+--------+                  +---------+--------+
| ProtocolRound  |                  | RoundOutcome     |
| valid stage    |                  | local execution  |
+-------+--------+                  +---------+--------+
        |                                     |
        +------------------+------------------+
                           |
                           v
              +------------+-------------+
              | lower_round_outcome(...) |
              | typed effect plans       |
              +------------+-------------+
                           |
                           v
              +------------+-------------+
              | effect consumers         |
              | own idempotency + emit   |
              | receipts                 |
              +------------+-------------+
                           |
                           v
              +------------+-------------+
              | next ObservationBundle   |
              +--------------------------+
```

### Responsibility boundary diagram

```text
+--------------------+       +----------------------+       +----------------------+
| DCS worker         |       | HA roundbook         |       | Effect consumers     |
| reads/writes etcd  | ----> | pure selection only  | ----> | Postgres / fencing / |
| publishes member   |       | no IO, no dedup      |       | replication / DCS    |
| truth + receipts   |       |                      |       | consumers own        |
+--------------------+       +----------------------+       | receipt emission     |
         ^                                                         |
         |                                                         |
         +---------------------- persisted receipts ---------------+
```

The HA layer is now easy to describe in one sentence: it chooses a protocol round and a receipt plan from the newest available facts. It does not guess whether work has already happened. Consumers say that by emitting receipts.

## Proposed typed state model

### `ObservationBundle`

`ObservationBundle` is the pure input type for the HA decider. It contains:

- freshest local `PgInfoState`
- freshest DCS cluster membership and lease records
- freshest local worker/process observations
- persisted receipt cache for recent epochs
- time-derived freshness judgments
- boot context such as empty-vs-existing `pgdata`, init-lock visibility, and runtime incarnation

The important point is that startup and steady-state both build the same bundle shape.

### `HaRoundbook`

Suggested future shape:

```text
struct HaRoundbook {
    authority_epoch: AuthorityEpoch,
    cluster_view: ClusterView,
    active_round: ProtocolRound,
    round_guards: RoundGuardSet,
    satisfied_receipts: Vec<ReceiptRecord>,
    required_receipts: Vec<RequiredReceipt>,
    local_outcome: RoundOutcome,
    next_rounds: Vec<ProtocolRound>,
}
```

This type becomes the main output of pure decision logic. The current `DecisionFacts` in `src/ha/decision.rs` would likely survive as part of the internal evidence assembly, but the external product of deciding would shift from "decision plus scattered interpretation" to "roundbook plus required proofs."

### `ProtocolRound`

Suggested conceptual enum:

```text
enum ProtocolRound {
    Discover,
    BootstrapClaim,
    BootstrapFinalize,
    LeaderEstablish,
    LeaderServe,
    ReplicaConverge,
    FormerPrimaryRepair,
    QuorumSuspend,
    FenceIsolate,
}
```

Meaning of each round:

- `Discover`: initial or restarted node gathers evidence and decides whether a valid leader or bootstrap path already exists.
- `BootstrapClaim`: no healthy cluster exists yet, a node may contend for initialization authority.
- `BootstrapFinalize`: a winner with init authority validates local data and creates the initial authoritative cluster epoch.
- `LeaderEstablish`: a node with acting-majority legitimacy is promoting or confirming itself into authority.
- `LeaderServe`: the leader has completed establishment receipts and may continue writable service while refreshing required proofs.
- `ReplicaConverge`: a non-authoritative node is aligning itself to the active leader.
- `FormerPrimaryRepair`: a node that used to be authoritative but lost that right must first prove demotion, timeline correction, and recovery safety.
- `QuorumSuspend`: no safe acting majority is visible for serving, but the node may continue publishing truth and waiting for evidence.
- `FenceIsolate`: the node has crossed a hard safety boundary and must preserve non-writability until later rounds become legal.

### `RoundGuardSet`

Every round is legal only if its guards hold. Conceptual guard families:

- `MajorityGuard`
- `LeaseGuard`
- `DataFitnessGuard`
- `PublicationGuard`
- `IsolationGuard`
- `BootstrapGuard`
- `RecoverySourceGuard`

Example:

- `LeaderServe` requires a valid acting-majority guard plus a held lease guard plus a publication freshness guard.
- `ReplicaConverge` requires a valid recovery-source guard.
- `FormerPrimaryRepair` requires an isolation guard proving the node no longer retains serving authority.

### `RequiredReceipt`

Receipts are the central idempotency and progress primitive. A required receipt contains:

- a stable receipt key
- the authority epoch it belongs to
- the round it satisfies
- the consumer class responsible for producing it
- freshness/expiry rules
- the contradictory receipts that invalidate it

Examples:

- `LeaseHeld(epoch=42, holder=node-b)`
- `PromotionComplete(epoch=42, leader=node-b)`
- `WritePermitGranted(epoch=42, leader=node-b)`
- `PostgresStopped(epoch=42, node=node-a)`
- `RewindComplete(epoch=42, node=node-a, source=node-b)`
- `BasebackupComplete(epoch=42, node=node-a, source=node-b)`
- `FollowConfigApplied(epoch=42, node=node-a, source=node-b)`
- `MemberPublished(epoch=42, node=node-a, freshness=recent)`

### `RoundOutcome`

`RoundOutcome` is what later replaces today's more role-like top-level HA decision. Suggested conceptual shape:

```text
enum RoundOutcome {
    WaitForEvidence { reasons: Vec<Blocker> },
    ClaimBootstrap { bootstrap_plan: BootstrapPlan },
    FinalizeBootstrap { finalize_plan: BootstrapFinalizePlan },
    EstablishLeadership { establish_plan: EstablishPlan },
    ContinueLeadership { serve_plan: ServePlan },
    ConvergeAsReplica { converge_plan: ConvergePlan },
    RepairFormerPrimary { repair_plan: RepairPlan },
    SuspendService { suspension_plan: SuspensionPlan },
    EnforceFence { fence_plan: FencePlan },
}
```

This is still pure. It contains typed intent, not IO.

### `LocalProtocolCursor`

A later implementation may also want a persisted local cursor that remembers the last accepted round and recent receipt records. The cursor is not authority by itself. It is only a local replay aid that lets the node:

- explain its most recent round
- avoid reissuing equivalent consumer work when valid receipts already exist
- expose operator-friendly debugging without sender-side dedup guesses

### Transition triggers

Examples of meaningful transitions:

- `Discover -> LeaderServe` when a healthy leader already exists, this node is that leader, and establishment receipts remain valid.
- `Discover -> ReplicaConverge` when a healthy leader exists and this node must follow or repair toward it.
- `Discover -> BootstrapClaim` when no authoritative cluster exists and bootstrap guards are satisfied.
- `LeaderServe -> QuorumSuspend` when acting-majority guard is lost but hard split-brain proof does not yet require full fence.
- `LeaderServe -> FenceIsolate` when lease is lost or contradictory leader proof exists.
- `ReplicaConverge -> LeaderServe` never happens directly; the node must first pass through `LeaderEstablish` under a valid authority epoch.
- `FormerPrimaryRepair -> ReplicaConverge` after stop, data-fitness, and source-validation receipts all satisfy the current epoch's repair requirements.

### Invariants

This option depends on a few non-negotiable invariants:

- every writable-serving state belongs to one explicit authority epoch
- no node may remain in `LeaderServe` without fresh lease and publication receipts
- no node may skip from "former primary" to "healthy follower" without proving data compatibility for the current leader
- startup uses the same `Discover` round as a runtime restart; there is no separate startup planner
- sender code never decides that work is already done; only receipts from the responsible consumer can prove completion

## Detailed round model

### Round A: `Discover`

`Discover` is the mandatory entry round for every process start and every major evidence discontinuity. It asks:

- do we already see an authoritative epoch?
- does a safe acting majority exist?
- is there an existing leader whose authority is still defensible?
- is local `pgdata` compatible with joining or serving in that epoch?
- is bootstrap even legal?

No local side effects are decided here beyond typed requests to gather missing evidence or publish current partial truth.

### Round B: `BootstrapClaim`

This round exists only when no authoritative cluster is live and bootstrap guards say initialization is permitted. It explicitly separates winning initialization authority from actually becoming serving primary. That prevents the current architecture's tendency to blur "I won the init lock" with "I may now assume all local data is valid and start serving."

Key rules:

- init-lock ownership is only one guard, not the entire bootstrap proof
- existing local `pgdata` must still be examined for compatibility
- bootstrap requires a `BootstrapAuthorityGranted` receipt before local data preparation may begin
- other nodes remain in `QuorumSuspend` or `Discover`, not speculative follower states

### Round C: `BootstrapFinalize`

The bootstrap winner uses this round to turn initialization into a real authority epoch. Required receipts may include:

- `InitLockHeld`
- `BootstrapDataAccepted`
- `InitialPublicationComplete`
- `LeaseHeld`
- `PromotionComplete`

Only after these are complete may the node transition to `LeaderServe`.

### Round D: `LeaderEstablish`

This round is used for both first bootstrap serving and later failover serving. That is a major simplification. Promotion is not a side effect hidden behind a "be primary" decision. It is one explicit round with required receipts.

Typical receipts:

- acting-majority proof
- leadership selection proof
- lease acquisition proof
- PostgreSQL promotion proof
- publication freshness proof
- write permission proof

This makes degraded-majority re-election explicit. A 2-of-3 majority can enter `LeaderEstablish` and complete it if the acting-majority guard holds, even when full visibility is gone.

### Round E: `LeaderServe`

This is the steady writable-serving round. The leader may remain here only while its guards remain true. The roundbook continuously revalidates:

- acting-majority health
- lease freshness
- publication freshness
- contradiction checks against challenger evidence

If the acting majority is still valid but visibility is incomplete, `LeaderServe` may continue. If the acting majority disappears, the node drops to `QuorumSuspend`. If the lease is explicitly lost or a contradictory leader is proven, it drops to `FenceIsolate`.

### Round F: `ReplicaConverge`

This is the ordinary non-primary steady path. The node can remain in `ReplicaConverge` across startup, restart, transient lag, and healthy following because the round contains substeps instead of scattering them elsewhere:

- `ObserveLeader`
- `AssessFitness`
- `MinorCatchup`
- `Rewind`
- `Basebackup`
- `StartFollowing`
- `VerifyFollowing`

These are not separate top-level protocols. They are one replica-convergence round with ordered substates.

### Round G: `FormerPrimaryRepair`

This round exists specifically for nodes that previously had serving authority or still have data that may reflect a superseded leadership epoch. Treating them like ordinary replicas immediately is unsafe. The round requires the node to:

- prove serving authority is gone
- stop writable service if still running
- verify leader source
- assess timeline compatibility
- prefer rewind when legal
- fall back to basebackup when rewind is impossible or has failed

Only after those receipts are complete may the node return to ordinary `ReplicaConverge`.

### Round H: `QuorumSuspend`

This round is the replacement for the current too-blunt fail-safe path in ambiguous non-full-quorum situations. It means:

- this node may not serve writable traffic right now
- this node should still publish partial truth
- this node should still evaluate evidence and be ready to move
- this node is not yet permanently fenced

It is intentionally reversible when new evidence restores a valid acting majority.

### Round I: `FenceIsolate`

This is the hard safety round. It is used when:

- lease loss is explicit
- contradictory leadership proof exists
- this node is outside the acting majority and still retains dangerous local state
- the protocol must guarantee non-writability before any later recovery

`FenceIsolate` is stricter than `QuorumSuspend`. It is what the design uses to satisfy post-cutoff write-blocking expectations.

## Redesigned quorum model

This option replaces the current "full quorum or fail-safe" boundary with a three-layer quorum interpretation.

### Layer 1: Visibility quorum

How much of the cluster can currently be observed? This is useful, but not enough for authority decisions by itself.

### Layer 2: Acting majority quorum

Is there a presently connected and mutually visible majority set that can lawfully establish or retain leadership for the current authority epoch? This is the key safety boundary for degraded operation. A 2-of-3 majority can satisfy this even when one node is missing.

### Layer 3: Local participation status

Is this node inside the acting majority or outside it? The answer matters because a safe majority may exist somewhere, but this specific node may still have to suspend or fence.

### Why degraded-but-valid majority should continue

In a three-node cluster where two healthy members can still exchange DCS truth and one is gone, the system should not pretend that no authority path exists. The remaining majority should be able to:

- conclude that the old isolated node cannot retain valid majority-backed authority
- establish or retain one leader
- issue replica convergence instructions to any rejoining node later

The current architecture does not express that cleanly enough. The roundbook does because `LeaderEstablish` and `LeaderServe` rely on acting-majority guards, not perfect visibility guards.

### When the node must still suspend or fence

The node must not continue writable service merely because some data is present. It must:

- enter `QuorumSuspend` when acting-majority proof is absent but hard contradiction is also absent
- enter `FenceIsolate` when lease loss, contradictory leader proof, or explicit exclusion from the acting majority is established

This gives a more precise answer to the user's complaint. The system no longer collapses every imperfect cluster view into one path, but it still fences aggressively when authority is truly broken.

## Redesigned lease model

This option treats lease as a receipt-backed guard rather than an incidental fact.

### Lease acquisition

`LeaderEstablish` requires a `LeaseHeld` receipt for the active authority epoch. The pure decider may request lease acquisition, but only the DCS consumer can produce the receipt.

### Lease retention

`LeaderServe` requires periodic lease-refresh receipts. If the refresh is late but the acting majority still exists and no contradictory leader is proven, the node may move briefly through `QuorumSuspend` rather than pretending the lease is fine. That transitional clarity is important for operators and for tests.

### Lease loss

Explicit lease loss invalidates all writable-serving receipts for that epoch. The node must leave `LeaderServe` immediately. Whether it goes to `QuorumSuspend` or `FenceIsolate` depends on whether contradictory authority is proven. If the cluster proves a new leader, the old leader goes straight to `FenceIsolate` and then `FormerPrimaryRepair`.

### Killed primary and lost authority

A killed primary loses authority because it stops refreshing the lease and stops producing fresh publication receipts. A challenger in the acting majority can then move through `LeaderEstablish`. When the old primary returns, it starts in `Discover`, sees that its prior epoch is no longer current, and enters `FormerPrimaryRepair` rather than attempting to resume service.

## Startup reasoning

Startup becomes one roundbook evaluation problem rather than a separate planner.

### Cluster already up with a healthy leader

If startup observations show a healthy leader with valid serving receipts, this node does not speculate. It enters `ReplicaConverge` or `FormerPrimaryRepair` depending on local data fitness and prior authority status.

### Cluster leader already present but local `pgdata` exists

Existing local data is not enough to determine behavior. `Discover` classifies the local store:

- compatible follower data
- stale but rewindable former-leader data
- incompatible data requiring basebackup
- empty data requiring initial clone

The round decides from those categories.

### Existing members already published

Published member truth is part of the `ObservationBundle`, not an afterthought. Startup uses the same member table that steady-state uses. Partial truth is allowed to remain partial; a missing SQL probe does not erase the fact that the agent is alive and publishing.

### Empty versus existing `pgdata`

`BootstrapClaim` and `ReplicaConverge` both use explicit data-fitness guards. Empty data may allow clone or bootstrap. Existing data may allow follow, rewind, or safe reuse under bootstrap, but only if the round guards approve it. There is no generic "existing data means start Postgres and see."

### Init lock behavior

Winning the init lock moves the node into `BootstrapClaim`; it does not skip directly to service. The node must still complete bootstrap receipts and create a first authority epoch.

### Using existing `pgdata` when the node wins init lock

This option explicitly allows that existing local data can still be valid for bootstrap if it satisfies `BootstrapDataAccepted` guards. For example, if the node has clean local data for an uninitialized cluster generation, it may reuse it instead of discarding it. The important rule is that this must be a typed guard decision, not a side effect of whatever the current startup helper happens to do.

## Replica convergence as one coherent path

This option keeps one convergence round with ordered substates instead of separate ad hoc dispatch branches.

### Healthy follow

If the node already follows the current leader and the data-fitness guard says the timeline and replay position are acceptable, the roundbook simply requires freshness receipts such as `FollowConfigApplied` and `FollowerHealthy`. No restart or repair is necessary.

### Tolerable lag

Lag within configured tolerances remains in `ReplicaConverge::MinorCatchup`. The node does not bounce between broad role phases; it remains in the same round with different outstanding receipts.

### Wrong timeline with rewind possible

The roundbook enters `ReplicaConverge::Rewind` or `FormerPrimaryRepair::Rewind` depending on prior authority status. Required receipts include source confirmation, rewind success, and post-rewind validation.

### Rewind impossible or failed

The roundbook escalates to `Basebackup` inside the same convergence family. It does not invent a separate architecture path. That directly supports the user's request for one coherent sequence: follow when healthy, tolerate minor lag, rewind when possible, basebackup when necessary.

### Previously-primary, previously-replica, freshly-restored nodes

These are all variations of the same convergence framework. The difference is guard strictness and required receipts, not a different control-plane architecture.

## Partial-truth member publication

This design depends on publication being explicit enough to build a trustworthy roundbook.

### Publication principle

Every member key should publish the newest obtainable truth, even if incomplete. Examples:

- `pgtuskmaster_alive = true`, `pginfo_sql_status = unknown`
- `postgres_process_observed = false`, `last_known_timeline = 18`
- `readiness = unknown`, `agent_loop_timestamp = fresh`

Absence must be reserved for truly unavailable fields, not used as a shortcut for discomfort.

### Publication fields this option wants

The later implementation would likely want DCS member records to expose:

- agent freshness
- pginfo freshness
- SQL reachability classification
- readiness classification
- last known role claim
- last known leader source
- last known timeline and LSN summary
- last satisfied local receipts that are safe to externalize
- local prior-authority hint such as "was serving leader in epoch 41"

### Why receipts matter for publication

The roundbook becomes easier to compute if DCS records include a minimal public receipt summary. For example, a fresh `LeaseHeld(epoch=42)` published by the claimed leader is evidence that the cluster is in `LeaderServe`, while a stale or absent publication is evidence that a round transition may be needed. Only DCS workers may write etcd, but they can publish receipt summaries produced by local consumers.

## Deduplication boundary

### Current problem

The current HA worker tries to infer whether process dispatch is already active. That is fundamentally speculative because the worker does not own the actual Postgres process, replication artifacts, fencing state, or DCS write result.

### Proposed replacement

This option moves deduplication entirely into receipt-producing consumers.

Examples:

- the Postgres consumer decides whether `PromotionComplete(epoch=42)` already exists
- the replication consumer decides whether `RewindComplete(epoch=42, source=node-b)` already exists
- the DCS consumer decides whether `MemberPublished(epoch=42, freshness=recent)` already exists
- the safety/fencing consumer decides whether `WritesBlocked(epoch=42)` already exists

The HA worker no longer contains `should_skip_redundant_process_dispatch(...)`. Instead, it issues effect plans keyed by round and receipt. Consumers compare requested work against their own durable execution records and either:

- emit a new receipt
- reaffirm an existing receipt
- emit a contradiction or failure receipt

This is safer because idempotency is anchored to the component that actually knows whether work happened.

## Concrete future code areas that would change

The following areas would be central to a later implementation:

- `src/runtime/node.rs`
  - remove or collapse `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`
  - replace with one startup path that builds an `ObservationBundle` and enters `Discover`
- `src/ha/worker.rs`
  - replace sender-side dedup checks with roundbook assembly and effect dispatch keyed by receipt requirements
- `src/ha/decide.rs`
  - replace current phase selection with `build_roundbook(...)`
  - remove the non-full-quorum shortcut that maps directly to `HaPhase::FailSafe`
- `src/ha/decision.rs`
  - retain and refactor `DecisionFacts` into evidence assembly for roundbook construction
  - add types such as `ProtocolRound`, `RoundGuardSet`, `RequiredReceipt`, `ReceiptRecord`, `RoundOutcome`, and `AuthorityEpoch`
- `src/ha/lower.rs`
  - lower `RoundOutcome` into typed effect plans keyed by required receipts
- `src/ha/process_dispatch.rs`
  - strip policy selection out of dispatch
  - retain only consumer-facing process execution planning
- `src/dcs/worker.rs`
  - publish expanded partial-truth member records plus public receipt summaries
- `src/dcs/state.rs`
  - extend trust evaluation to distinguish visibility, acting majority, and local participation
- `src/pginfo/state.rs`
  - preserve and surface partial truth needed by `ObservationBundle`
- `tests/ha.rs` and `tests/ha/features/`
  - later align test assertions with explicit protocol rounds, receipts, and round transitions rather than opaque phase text

## All meaningful changes required for this option

### New types to add

- `ObservationBundle`
- `HaRoundbook`
- `ProtocolRound`
- `RoundGuardSet`
- `RoundGuard`
- `RequiredReceipt`
- `ReceiptRecord`
- `ReceiptConflict`
- `AuthorityEpoch`
- `RoundOutcome`
- `BootstrapPlan`
- `EstablishPlan`
- `ServePlan`
- `ConvergePlan`
- `RepairPlan`
- `SuspensionPlan`
- `FencePlan`
- `LocalProtocolCursor`

### Existing paths to remove or collapse

- separate startup-planning path in `src/runtime/node.rs`
- sender-owned "already active" logic in `src/ha/worker.rs`
- direct mapping from non-full-quorum to fail-safe in `src/ha/decide.rs`
- policy-heavy start-intent inference in `src/ha/process_dispatch.rs`

### Responsibilities to move

- move startup authority reasoning from runtime startup helpers into roundbook construction
- move action dedup from HA worker senders into Postgres, replication, safety, and DCS consumers
- move public protocol narration into explicit round and receipt types instead of logs stitched across modules
- move degraded-majority interpretation into guard evaluation inside the roundbook builder

### State transitions to redefine

- define `Discover` as the only startup entrypoint
- define `LeaderEstablish` as mandatory before `LeaderServe`
- define `FormerPrimaryRepair` as mandatory for returned prior leaders before ordinary follow
- distinguish `QuorumSuspend` from `FenceIsolate`
- define legal next-round sets explicitly for every round

### Effect-lowering boundary changes

- `src/ha/lower.rs` should lower receipts and round outcomes into concrete effect requests
- consumers should report receipt success, failure, or contradiction back into observation state
- lowerers should not infer completion from prior decisions; they should key requests by receipt identity

### DCS publication changes

- always publish freshest partial truth even when SQL probes fail
- publish public receipt summaries that help other nodes assess leadership freshness and repair progress
- publish prior-authority hints so returning old primaries can be classified safely

### Startup handling changes

- startup no longer calls a separate planner
- startup always builds an `ObservationBundle` and enters `Discover`
- bootstrap, existing-leader follow, and former-primary repair all become round transitions instead of startup-specific branches

### Convergence handling changes

- convergence becomes one round family with ordered substates and explicit receipts
- rewind and basebackup are alternate receipt plans within the same convergence family
- former-primary repair becomes a strict precursor when needed, not an implicit variation

### Test updates a future implementation would need

- assert protocol rounds and receipts in debug outputs or events so HA feature scenarios become easier to diagnose
- update degraded-majority expectations to reflect acting-majority semantics instead of full-visibility semantics
- ensure fencing tests assert `FenceIsolate` or receipt-backed write blocking explicitly
- ensure restart/rejoin tests assert the `FormerPrimaryRepair -> ReplicaConverge` path where relevant

## Migration sketch

### Step 1: Introduce round and receipt types beside current decision machinery

Add the new types to `src/ha/decision.rs` or nearby modules without yet deleting the old path. Make the compiler understand the roundbook vocabulary first.

### Step 2: Build `ObservationBundle` and `Discover` in parallel with current startup helpers

Teach startup to assemble the same pure inputs that the worker loop will use later. Keep existing behavior temporarily, but generate shadow roundbooks for comparison.

### Step 3: Replace the non-full-quorum shortcut with guard evaluation

Implement visibility quorum, acting majority, and local participation classification. Delete the direct "non-full-quorum means fail-safe" path once guard-driven rounds exist.

### Step 4: Add `LeaderEstablish`, `LeaderServe`, `QuorumSuspend`, and `FenceIsolate`

This step gives leadership retention and loss a more exact protocol structure before replica recovery is changed.

### Step 5: Move startup and rejoin selection into `Discover`

Delete runtime startup planners once `Discover` can route bootstrap, follow, and former-primary repair correctly.

### Step 6: Introduce receipt-driven consumer idempotency

Teach Postgres, replication, DCS, and safety consumers to emit receipts and contradictions. Delete sender-side dedup helpers once consumer receipts are authoritative.

### Step 7: Collapse repair paths into `ReplicaConverge` and `FormerPrimaryRepair`

Remove stale branching from `src/ha/process_dispatch.rs` and replace it with execution plans derived from round outcomes.

### Step 8: Delete legacy compatibility paths aggressively

Because this is a greenfield project with no backward-compatibility requirement, a later implementation should remove stale startup helpers, stale decision enums, stale dispatch shortcuts, and stale debug narratives instead of supporting both architectures indefinitely.

## Non-goals

- This option does not attempt to preserve the current phase names if they obscure the protocol.
- This option does not treat DCS as a general workflow engine; DCS remains a storage and lease substrate, not a second pure decider.
- This option does not move Postgres or etcd IO into the HA decider.
- This option does not claim that every receipt must be globally published; some receipts may remain local as long as the necessary public summaries exist.
- This option does not fix failing tests in this task. It only proposes an architecture that could later do so.

## Tradeoffs

### Strengths

- Makes startup and steady-state genuinely the same protocol entrypoint.
- Makes degraded-majority behavior explicit and testable.
- Gives operators a visible answer to "what round are we in and what proof is missing?"
- Moves idempotency to the components that actually know execution truth.
- Makes former-primary repair a first-class safety path instead of an inference.

### Costs

- Introduces more explicit types and therefore more up-front design surface.
- Requires consumers to persist and expose receipts carefully.
- Needs disciplined epoch design so receipts do not linger across incompatible authority changes.
- May require test harnesses and debug output to learn new vocabulary.

### Risks

- If receipt granularity is too coarse, the protocol becomes vague again.
- If receipt granularity is too fine, the protocol becomes noisy and hard to reason about.
- If public receipt summaries reveal too little, cross-node decision quality suffers.
- If public receipt summaries reveal too much unstable detail, DCS churn could rise unnecessarily.

## Logical feature-test verification

### `ha_dcs_quorum_lost_enters_failsafe`

Under this option, the exact outcome depends on whether acting-majority guards remain valid. If no safe acting majority exists, the leader leaves `LeaderServe` and enters `QuorumSuspend` or `FenceIsolate` depending on whether explicit authority loss is proven. The logical correction is that the test should observe an explicit suspension/fence round rather than relying on the current blunt fail-safe shortcut.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This scenario is handled by `FenceIsolate` plus a required `WritesBlocked` receipt from the safety consumer. The old leader cannot remain writable after its serving receipts are invalidated. The feature should pass once post-cutoff write attempts are blocked by receipt-backed fencing rather than inferred phase text.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The surviving majority remains eligible for `LeaderEstablish` because acting-majority guards still hold. The isolated old primary loses the ability to retain `LeaderServe` because it no longer has valid majority-backed receipts. That directly addresses the user's complaint that 2-of-3 majority re-election should keep operating.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

On healing, the old primary begins in `Discover`, sees a newer authority epoch, and enters `FormerPrimaryRepair`. It cannot jump back into service. It must satisfy stop, source-validation, and rewind/basebackup receipts before entering `ReplicaConverge`, then finally healthy following.

### `ha_primary_killed_then_rejoins_as_replica`

When the old primary is killed, it stops refreshing lease and publication receipts. The surviving acting majority may establish a new leader. On return, the old primary follows the same `Discover -> FormerPrimaryRepair -> ReplicaConverge` path and therefore rejoins as a replica.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica comes back and a safe acting majority is restored, the roundbook allows `LeaderServe` to continue or `LeaderEstablish` to complete on the surviving majority side. The restarted node enters `ReplicaConverge`, not a speculative service path, so quorum can be restored without stale self-promotion.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes begin in `Discover`. If they can form an acting majority and establish leadership safely, they move through `LeaderEstablish` and `LeaderServe`. The final node later enters `ReplicaConverge` or `FormerPrimaryRepair` depending on its data. Startup is unified because all three nodes use the same roundbook logic.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

Inside `FormerPrimaryRepair` or `ReplicaConverge`, failure to satisfy `RewindComplete` simply advances the repair plan to `BasebackupComplete`. This is one coherent convergence family, not a cross-module special case.

### `ha_replica_stopped_primary_stays_primary`

Stopping a replica does not invalidate the leader's acting-majority and lease receipts when enough members still remain. The leader stays in `LeaderServe`. The stopped replica returns later to `ReplicaConverge`.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica cannot destabilize the cluster because it does not own any leadership receipts. It remains in `ReplicaConverge` with missing repair receipts, while the active leader remains in `LeaderServe` so long as its own guards hold. Rejoin failure is localized to the broken node's missing receipts.

### Boundary clarification for changed interpretations

This option intentionally changes how the architecture narrates several scenarios:

- "fail-safe" is split into `QuorumSuspend` and `FenceIsolate`
- leadership service requires receipt-backed proof, not just inferred continuity
- degraded-majority operation is explicitly legal when acting-majority guards hold
- old-primary return is always a repair protocol, never an optimistic role resume

The feature suite should become easier to reason about because each scenario now maps to visible round transitions rather than opaque combinations of phase, lease, and process-dispatch state.

## Open questions

## Q1 How much of the receipt set should be published into DCS?

Context: this option gets much of its clarity from receipts, but not every receipt needs to be visible cluster-wide. Publishing too little makes remote decision quality weak. Publishing too much could create DCS churn and expose noisy local detail.

Problem or decision point: the later implementation must choose which receipts are public protocol evidence and which remain local implementation evidence.

Restated question: what is the smallest public receipt surface that still lets every node reconstruct the same roundbook safely?

## Q2 Should `QuorumSuspend` preserve read-only service hints or be strictly non-serving?

Context: this option separates `QuorumSuspend` from `FenceIsolate`, but that still leaves a product question about what kinds of local service, if any, remain acceptable while the node is waiting for evidence.

Problem or decision point: a looser suspend mode might help availability, but it could also confuse operators or blur the safety boundary the user wants clarified.

Restated question: should `QuorumSuspend` permit any read-only continuity semantics, or should it mean "publish truth only, serve nothing"?

## Q3 Should a returned old primary always enter `FormerPrimaryRepair`, even if its data already looks follower-compatible?

Context: the strict version of this option always treats prior leaders as special on return. That is safer, but it may introduce extra steps in cases where the data already appears completely aligned.

Problem or decision point: the architecture must decide whether prior authority alone is enough to require the repair round, or whether some nodes may skip directly to ordinary convergence.

Restated question: is "was recently authoritative" itself a mandatory repair trigger, or can proven data compatibility waive that extra round?

## Q4 How should receipt invalidation interact with authority epoch rollover?

Context: receipts are only useful if later rounds know when earlier receipts no longer count. Epoch rollover is the obvious boundary, but some receipts may remain useful across adjacent epochs while others must be invalidated immediately.

Problem or decision point: the invalidation rules need to be strict enough to prevent stale authority but not so strict that harmless work is repeated needlessly.

Restated question: which receipt classes should be epoch-scoped strictly, and which may be reused as stable infrastructure facts across epoch changes?

## Q5 Should bootstrap use the same `LeaderEstablish` machinery after claim, or keep a distinct `BootstrapFinalize` round?

Context: this option currently keeps `BootstrapFinalize` distinct so the first cluster birth is narratively explicit. A more minimal version could collapse bootstrap completion directly into ordinary leader establishment.

Problem or decision point: a dedicated bootstrap-finalization round adds clarity, but it also adds another top-level protocol stage.

Restated question: is the first-cluster-birth path special enough to justify its own round, or should bootstrap become just an input condition for `LeaderEstablish`?
