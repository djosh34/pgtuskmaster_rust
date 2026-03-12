# HA Refactor Option 2: Lease-Epoch Story Machine

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history or repo documentation.

## Why this option exists

This option exists because the current HA architecture has several typed pieces, but it still does not make one question impossible to miss: "which lease epoch story are we inside right now?" The differentiator for this option is that every startup, failover, demotion, rejoin, and switchover path is modeled as movement through explicit lease epochs and epoch handoff stories. Instead of first classifying a broad cluster regime, this option first classifies lease authority, lease lineage, and authority transfer state. That makes it different from options centered on regime classification, recovery graphs, bootstrap charters, or shared ADTs with split loops.

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 2.

1. `Regime-first reconciliation`
   The system first derives a cluster regime ADT, then derives a local contract from that regime.
2. `Lease-epoch story machine`
   The system is organized around explicit lease epochs and handoff stories, with every transition anchored to epoch ownership.
3. `Startup-as-recovery funnel`
   Startup is deleted as a special case and replaced by one recovery funnel that handles empty, existing, diverged, and stale data uniformly.
4. `Receiver-owned command ledger`
   HA produces stable intent ids while DCS/process consumers own all idempotency and duplicate suppression.
5. `Peer-evidence vote matrix`
   Leadership is derived from a typed peer-evidence matrix instead of a single trust enum and a best-candidate helper.
6. `Bootstrap charter machine`
   Bootstrap is elevated into its own charter state machine with explicit init-lock substates and durability claims.
7. `Convergence graph runner`
   Replica repair, rewind, basebackup, and steady following become one graph of convergence edges with typed prerequisites.
8. `Publication-first truth model`
   Member publication is redesigned first so all other HA logic consumes richer partial-truth envelopes.
9. `Split loops with shared ADTs`
   Startup and steady-state remain separate loops, but they must consume the exact same world and intent ADTs.
10. `Authority contract DSL`
    Leadership, fencing, and switchover are encoded as typed contracts that can be model-checked independently of Postgres actions.

## Diagnostic inputs on March 12, 2026

This option uses the current repository state as input evidence.

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because this option is not justified by "the repo is red"; it is justified by authority clarity, handoff clarity, and the need to make restart/failover reasoning legible.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out waiting for one primary across the two restarted fixed nodes and reported both restarted nodes as unknown. That is directly relevant to this option because a full-stop restart is fundamentally a lease-lineage problem: the system must determine whether a new epoch can begin, whether old local data still carries usable lineage, and how a final rejoiner discovers which epoch story it belongs to.
- `tests/ha.rs` remains the acceptance contract surface for later implementation work. A future implementation based on this option must either satisfy those scenarios or update them with explicit new semantics.

## Current design problems

### 1. Startup reasoning is still architecturally separate from steady-state reasoning

The live runtime path now enters HA through `run_node_from_config(...)` and then `run_workers(...)` in `src/runtime/node.rs`, not through the older startup planner names described in the task context. That is an improvement, but it still means startup framing happens in runtime wiring before the HA loop has declared a first-class authority model. Older explicit startup-planning names such as `plan_startup_with_probe(...)` and `build_startup_actions(...)` still exist as stale disabled references, which is itself evidence of architectural drift. The user wanted startup to follow the same "newest information plus same state produces same actions" rule as later ticks. The current tree still does not center startup on the same typed authority object that steady-state failover would use.

### 2. HA reasoning is still spread across multiple "almost central" modules

The current live path still splits meaning across `src/ha/worker.rs`, `src/ha/decide.rs`, `src/ha/reconcile.rs`, and `src/ha/process_dispatch.rs`. `src/ha/decision.rs` and `src/ha/lower.rs` also define a second typed decision/effect vocabulary that is real and useful, but not the singular live center of the current worker loop. This makes future changes risky because lease meaning, authority meaning, startup intent, and process-command meaning are still partially reconstructed in different places. A lease-epoch design should eliminate the ambiguity by making every module answer to one typed epoch story.

### 3. Sender-side dedup ownership has been reduced, but delivery identity still originates from the sender

The older sender skip helpers called out in the task context are no longer the dominant shape, which is good. The remaining issue is subtler: `src/ha/process_dispatch.rs` still manufactures `ProcessJobRequest.id` by calling `process_job_id(...)` from the HA side using `(scope, self, action, action_index, ha_tick)`. That means the sender still defines command identity and therefore still partially owns duplicate handling. The user explicitly wants that concern to move away from sender-side HA logic. This option treats lease epochs as the stable story identifier and leaves final idempotency to consumers.

### 4. The quorum and fail-safe boundary is still too compressed

`decide(...)` in `src/ha/decide.rs` still sends every non-`DcsTrust::FullQuorum` world through `decide_degraded(...)`. Inside that branch, a primary falls into `FailSafeGoal::PrimaryMustStop(...)` when there is an active or observed epoch, or `FailSafeGoal::WaitForQuorum` otherwise. That is safer than pretending degraded trust is healthy, but it still compresses multiple distinct authority stories into one degraded branch. A 2-of-3 majority that can still authoritatively continue one epoch is not the same as a total authority blackout, and a full-stop restart is not the same as a primary partitioned away from its majority.

### 5. Startup and rejoin logic still crosses a dangerous boundary in process dispatch

`start_intent_from_dcs(...)`, `resolve_source_member(...)`, `validate_basebackup_source(...)`, and `validate_rewind_source(...)` in `src/ha/process_dispatch.rs` still force the dispatch layer to rediscover or confirm startup meaning from DCS state. That means "what lineage should I join?" and "is this local data from the winning epoch or a stale branch?" can still be answered too late. In this option, dispatch should only lower a fully typed epoch-scoped intent such as "join epoch 42 as follower from source node-b using rewind" rather than re-deriving leader/source meaning at the last moment.

### 6. Partial-truth publication is richer than before, but not treated as an epoch contract

`src/dcs/worker.rs` now builds a `MemberSlot` with richer `MemberPostgresView` detail, and `src/pginfo/state.rs` preserves partial truth through `PgInfoState::{Unknown,Primary,Replica}` plus `SqlStatus` and `Readiness`. That is strong groundwork. The missing step is to make published truth explicit about epoch participation. A node that has process liveness but degraded pginfo should still publish "I am alive, my local lease token is X if any, my last observed epoch is Y if any, my readiness is unknown" rather than silently disappearing from epoch reasoning.

## The central proposal

Replace the current decision center with an epoch-story pipeline:

1. `Observe`: collect the newest local and global evidence into an immutable `ObservationEnvelope`.
2. `InferEpochStory`: derive a typed `EpochStory` that answers which lease lineage is currently authoritative, whether that lineage is continuous or broken, and whether this node belongs to it, must wait, must fence, or may begin a new epoch.
3. `ResolveRoleContract`: derive a `RoleContract` for this node within that epoch story.
4. `LowerIntent`: produce epoch-scoped intents and actions for DCS and process consumers.

The critical design rule is simple: no HA decision may be expressed without naming the current epoch story. Startup is therefore not "before HA"; startup is just the earliest epoch-story evaluation. Switchover is not a special imperative path; it is an epoch handoff story with an expected next holder. Rejoin is not a dispatcher concern; it is an epoch reattachment story with explicit lineage proof or clone fallback.

## Control flow from startup through steady state

### High-level method

At process boot:

1. Start the observation workers exactly as today: DCS subscriber, pginfo worker, process subscriber, config stream, and data-dir inspection helpers.
2. Do not pre-compute startup actions outside the unified HA loop.
3. As soon as the first observation snapshot exists, run `infer_epoch_story(...)`.
4. If the node cannot yet prove which epoch is authoritative, it publishes truth and waits. It does not guess at a primary or issue ad hoc startup commands.
5. Once the epoch story becomes decidable, the resolver chooses one contract:
   `HoldEpochLeader`, `JoinEpochFollower`, `RecoverIntoEpoch`, `InitiateEpochBootstrap`, `YieldDuringHandoff`, or `FenceForSafety`.
6. Lowering emits intents carrying both a stable `EpochIntentId` and an `EpochRef`.

During steady state:

1. Every observation change recomputes the epoch story from scratch.
2. The decider compares the newly inferred story with the last acknowledged story.
3. If the epoch is unchanged and the node is already correctly aligned, the resulting contract is a no-op or publication-only update.
4. If the epoch changed, the new story explicitly says whether this is:
   `SameEpochContinue`, `SameEpochRepair`, `EpochHandoff`, `EpochSuperseded`, `EpochLost`, or `EpochBootstrap`.
5. The lowerer emits only intents valid for that story.
6. Consumers apply idempotently by `(consumer, epoch_ref, intent_kind, intent_revision)` rather than by sender tick.

### ASCII diagram

```text
                    startup tick or steady-state tick
                                 |
                                 v
                  +------------------------------------+
                  | ObservationEnvelope                |
                  | - dcs cache + trust                |
                  | - local pginfo                     |
                  | - local process state              |
                  | - local data-dir evidence          |
                  | - last acknowledged epoch story    |
                  +------------------+-----------------+
                                     |
                                     v
                  +------------------------------------+
                  | infer_epoch_story(...)             |
                  |                                    |
                  | EpochStory                         |
                  | - epoch authority                  |
                  | - epoch lineage                    |
                  | - handoff status                   |
                  | - local membership                 |
                  | - safety posture                   |
                  +------------------+-----------------+
                                     |
                                     v
                  +------------------------------------+
                  | resolve_role_contract(...)         |
                  |                                    |
                  | HoldLeader / JoinFollower /        |
                  | Recover / Bootstrap / Yield /      |
                  | Fence / Wait                       |
                  +------------------+-----------------+
                                     |
                                     v
                  +------------------------------------+
                  | lower_epoch_intents(...)           |
                  |                                    |
                  | EpochIntentId + DCS intents +      |
                  | Process intents                    |
                  +------------+------------+----------+
                               |            |
                               v            v
                    +----------------+   +-------------------+
                    | DCS consumer   |   | Process consumer  |
                    | owns idempotency|  | owns idempotency  |
                    +----------------+   +-------------------+


  Story examples

  [No authoritative epoch] ---> [BootstrapEpoch proposed] ---> [Epoch 41 held by node-a]
            |                                  |                           |
            v                                  v                           v
       wait/publish                    init-lock + bootstrap         node-b joins epoch 41

  [Epoch 41 held by node-a] ---> [HandoffTo node-b] ---> [Epoch 42 held by node-b]
            |                              |                           |
            v                              v                           v
      old leader demotes             target proves eligibility    old leader rejoins as follower
```

## Proposed typed state machine

### Core top-level ADTs

```text
enum EpochStory {
    Unauthoritative(UnauthoritativeStory),
    Active(ActiveEpochStory),
    Handoff(HandoffStory),
    Bootstrap(BootstrapStory),
    Recovery(RecoveryEpochStory),
}

struct EpochRef {
    generation: u64,
    holder: MemberId,
}

enum RoleContract {
    Wait(WaitContract),
    Fence(FenceContract),
    HoldLeader(LeaderContract),
    JoinFollower(FollowerContract),
    RecoverIntoEpoch(RecoveryContract),
    InitiateEpochBootstrap(BootstrapContract),
    YieldDuringHandoff(HandoffYieldContract),
}

struct EpochIntentId {
    epoch: EpochRef,
    consumer: ConsumerKind,
    intent_kind: IntentKind,
    revision: u32,
}
```

### Observation ADTs

This option needs the observation layer to become explicit about lineage evidence, not just current status.

```text
struct ObservationEnvelope {
    now: UnixMillis,
    local_member_id: MemberId,
    dcs: DcsObservation,
    local_pg: LocalPgObservation,
    local_process: LocalProcessObservation,
    local_storage: LocalStorageObservation,
    local_data_dir: LocalDataDirObservation,
    previous_story: Option<AckedEpochStory>,
}

struct DcsObservation {
    trust: DcsAuthorityView,
    leader_lease: Option<LeaderLeaseView>,
    switchover: Option<SwitchoverView>,
    init_lock: Option<InitLockView>,
    members: BTreeMap<MemberId, PublishedMemberEvidence>,
}

enum DcsAuthorityView {
    AuthoritativeMajority,
    DegradedMajority,
    LocalButUnauthoritative,
    Unreachable,
}

struct PublishedMemberEvidence {
    routing: RoutingEvidence,
    postgres: PublishedPostgresEvidence,
    observed_epoch: Option<EpochRef>,
    local_epoch_token: Option<LeaseTokenRef>,
    freshness: EvidenceFreshness,
}

enum LocalDataDirObservation {
    Empty,
    Existing(DataLineageEvidence),
    Uninspectable { reason: String },
}

struct DataLineageEvidence {
    system_identifier_known: bool,
    latest_control_timeline: Option<TimelineId>,
    recovery_conf_present: bool,
    standby_signal_present: bool,
    has_history_files: bool,
}
```

The addition that matters most is `observed_epoch` on published member evidence and `previous_story` on local observation. The node must reason from "what epoch does the cluster claim exists?" and "what epoch did I most recently accept?" instead of only from "who looks like leader right now?"

### Epoch authority ADTs

```text
enum UnauthoritativeStory {
    StoreBlind {
        local_primary: bool,
        last_known_epoch: Option<EpochRef>,
    },
    NoMajorityProof {
        discoverable_members: BTreeSet<MemberId>,
        last_known_epoch: Option<EpochRef>,
    },
    ConflictingAuthority {
        competing_epochs: Vec<EpochRef>,
    },
}

struct ActiveEpochStory {
    epoch: EpochRef,
    authority: EpochAuthority,
    local_relation: LocalEpochRelation,
    continuity: EpochContinuity,
}

enum EpochAuthority {
    ConfirmedMajority,
    DegradedButAuthoritative,
}

enum LocalEpochRelation {
    Holder,
    AlignedFollower { source: MemberId },
    RecoverableFormerHolder,
    DivergedFormerParticipant,
    NotYetAttached,
}

enum EpochContinuity {
    Continuous,
    HolderRestarting,
    MajorityRestartedWithoutProcessLeader,
}
```

The major design shift is that degraded-majority does not become a generic fail-safe branch. It remains inside `ActiveEpochStory` when the cluster can still authoritatively continue one lease lineage.

### Handoff ADTs

```text
struct HandoffStory {
    from_epoch: EpochRef,
    expected_target: MemberId,
    handoff_state: HandoffState,
    old_holder_state: OldHolderState,
    target_state: TargetState,
}

enum HandoffState {
    RequestedButNotPrepared,
    OldHolderDemoting,
    TargetQualifying,
    TargetPromoting,
    NewEpochPublishing { new_epoch: EpochRef },
    FailedAndRevertable,
}

enum OldHolderState {
    StillPrimary,
    Demoting,
    Demoted,
    Unreachable,
}

enum TargetState {
    NotReady,
    ReadyToAcquire,
    Promoting,
    PrimaryWithNewEpoch,
}
```

This option treats switchover as a separate story, not a boolean modifier on a normal failover decision. That creates room for deterministic handoff rules and explicit rollback behavior.

### Bootstrap ADTs

```text
struct BootstrapStory {
    charter: BootstrapCharter,
    local_bootstrap_relation: LocalBootstrapRelation,
}

enum BootstrapCharter {
    NoClusterHistory,
    ClusterHistoryExistsButNoAuthoritativeEpoch,
    InitLockHeldByPeer(MemberId),
    InitLockHeldByMe,
}

enum LocalBootstrapRelation {
    EligibleSeeder,
    WaitingForSeeder,
    MustNotBootstrapBecauseExistingDataNeedsRecovery,
}
```

Bootstrap is still expressed in epoch terms: the cluster is deciding whether epoch generation `1` or a new post-blackout generation may begin.

### Recovery ADTs

```text
struct RecoveryEpochStory {
    target_epoch: EpochRef,
    local_relation: RecoverableRelation,
    recovery_path: RecoveryPath,
}

enum RecoverableRelation {
    FormerLeader,
    FormerFollower,
    NewReplica,
}

enum RecoveryPath {
    StartAsFollower,
    RewindIntoEpoch { source: MemberId },
    BaseBackupIntoEpoch { source: MemberId },
    WaitForSourceQualification,
}
```

Recovery is therefore not "some process dispatch helper inferred a source member." Recovery is a first-class epoch story with a typed target epoch and a typed path into that epoch.

### Contract ADTs

```text
struct LeaderContract {
    epoch: EpochRef,
    lease_goal: LeaseGoal,
    postgres_goal: LeaderPostgresGoal,
    publication_goal: PublicationContract,
}

struct FollowerContract {
    epoch: EpochRef,
    source: MemberId,
    convergence_goal: ConvergenceGoal,
    publication_goal: PublicationContract,
}

struct RecoveryContract {
    epoch: EpochRef,
    path: RecoveryPath,
    publication_goal: PublicationContract,
}

struct FenceContract {
    safety_reason: FenceReason,
    cutoff: Option<FenceCutoff>,
    publication_goal: PublicationContract,
}

struct WaitContract {
    reason: WaitReason,
    publication_goal: PublicationContract,
}
```

These contracts keep the user's desired structure intact: newest information first, then a pure typed outcome, then lower layers turn that outcome into actions.

## Transition rules

### Startup transitions

1. If DCS is unreachable and no authoritative epoch can be proven, emit `EpochStory::Unauthoritative(StoreBlind { ... })` and return `RoleContract::Wait`.
2. If DCS shows an active leader lease and that lease corresponds to one coherent member publication set, emit `EpochStory::Active`.
3. If no leader lease exists but there is authoritative majority and no cluster history, emit `EpochStory::Bootstrap`.
4. If no process is up after a full-cluster outage but authoritative majority exists and the member evidence plus local lineage still point to one valid continuation candidate, emit `ActiveEpochStory { continuity: MajorityRestartedWithoutProcessLeader }`.
5. If local existing data clearly belongs to a prior epoch but another member already holds the new epoch, emit `RecoveryEpochStory`.

### Steady-state transitions

1. `Active -> Active` when the same epoch holder remains authoritative and the local role stays aligned.
2. `Active -> Handoff` when a switchover request is authoritative and a target has been selected.
3. `Active -> Recovery` when this node loses leadership, detects foreign epoch authority, or proves its local timeline diverged.
4. `Active -> Unauthoritative` when authority proof disappears entirely and no degraded-majority continuation is defensible.
5. `Unauthoritative -> Active` only when majority proof and a single epoch lineage become re-established.

### Switchover transitions

1. `Handoff::RequestedButNotPrepared` begins when an authoritative switchover request exists.
2. `OldHolderDemoting` starts only after the old leader has a contract that preserves write safety and publishes that it is yielding.
3. `TargetQualifying` requires the target to prove readiness and acceptable lag within the old epoch.
4. `NewEpochPublishing` begins only after the target owns the new lease epoch and publishes `EpochRef { generation = old + 1, holder = target }`.
5. `FailedAndRevertable` exists so the system can explicitly decide whether to continue the old epoch, retry the handoff, or fence both sides.

## Redesigned quorum model

This option breaks the current single compressed degraded branch into four authority classes:

1. `ConfirmedMajority`
   DCS is healthy, this node sees the majority-backed member set, and one epoch lineage is authoritative.
2. `DegradedButAuthoritative`
   DCS/member evidence is reduced but still sufficient to prove a valid majority-backed epoch lineage. This is the key 2-of-3 continuation case.
3. `LocalButUnauthoritative`
   DCS may be reachable locally, but the node cannot prove majority-backed authority for any epoch. It must not create or continue leadership.
4. `Unreachable`
   The store is not usable, so only prior safety obligations remain. A prior primary may need fencing, but it may not continue writes merely because it remembers an old epoch.

The reason this matters is that majority reasoning must answer two separate questions:

- "Can this epoch continue?"
- "Can a new epoch be created?"

In a 3-node cluster where one node is down and two healthy nodes still agree, the answer is often "yes" to continuation and sometimes "yes" to creation if the old holder is gone and a new authoritative handoff/failover can be proven. The current `!FullQuorum -> degraded` approach treats too much of that as one bucket. The epoch-story model preserves degraded-majority operation without relaxing split-brain safety.

### Leadership re-election in 2-of-3 cases

The rule becomes:

1. If a valid epoch holder is still provably authoritative through degraded majority, continue that epoch.
2. If the holder is gone but a degraded majority can prove that the old epoch can no longer safely continue, one eligible follower may create `generation + 1`.
3. The new epoch must always supersede the prior one; it never coexists with it.
4. Any node carrying local data from the old epoch but not owning the new one must rejoin through recovery, not self-promote.

That allows correct degraded-majority continuation and failover without reducing all non-perfect health to generic fail-safe.

## Lease model

### Lease rules

The core lease model is:

1. Every authoritative leader owns an `EpochRef { generation, holder }`.
2. An epoch is not only a TTL-backed lease; it is also a lineage marker used by every publication, recovery, and demotion decision.
3. Leadership continuity means "the same epoch remains authoritative."
4. Failover means "a higher generation supersedes the prior epoch."
5. Switchover is "an intentionally created higher generation with expected target."

This differs from using the lease as merely a liveness gate. In this option, lease generations are the narrative backbone of the HA system.

### Lease expiry, loss, and killed primaries

If a primary is killed:

1. It stops renewing the current epoch lease.
2. Once authoritative majority determines the holder is gone and a new candidate is eligible, a new epoch generation is created.
3. The old primary, when restarted, must inspect publications and conclude "my local data belongs to a superseded epoch."
4. Its only valid contracts are `RecoverIntoEpoch` or `Fence`.

This directly answers the user complaint that a killed primary must lose authority in a way the restarted node can understand structurally, not heuristically.

### Lease interaction with startup

Startup is evaluated in terms of epoch participation:

1. "Cluster already up" means "an active epoch already exists."
2. "Leader already present" means "this node observes a foreign active epoch holder."
3. "Empty cluster" means "no authoritative epoch exists."
4. "Existing local data" means "this node may already be on some lineage that either matches or conflicts with the active epoch."

The startup question is therefore never "what command should runtime run first?" It is "what epoch story describes this node's first observation?"

## Startup reasoning in this option

### Cluster already up

If DCS shows an active authoritative epoch held by another node and member evidence supports that conclusion, this node joins that epoch as follower or recovery participant. No startup-only planner should decide that independently.

### Cluster leader already present

If the leader is already present and authoritative, the local node asks only:

1. Do my local files appear aligned enough to start following directly?
2. Do I need rewind?
3. Is clone required?

All three outcomes are still contracts within the observed epoch.

### Existing members already published

Existing member publication is read as epoch evidence, not as loose hints. Every published member record should help answer:

- which epoch the cluster currently believes exists
- who holds it
- whether this member thinks it is attached to that epoch
- how fresh that claim is

### Empty versus existing `pgdata`

`Empty` local data does not automatically imply bootstrap. If an authoritative epoch already exists elsewhere, empty local data simply means `BaseBackupIntoEpoch`.

Existing `pgdata` also does not automatically imply bootstrap denial. Existing local data can be:

1. aligned with the current epoch and ready to start following
2. from the same lineage but behind, requiring normal replica start
3. from a divergent timeline that still shares history, requiring rewind
4. too far gone or insufficiently provable, requiring basebackup

### Init lock behavior

Init lock becomes part of the bootstrap charter for epoch generation creation:

1. If no authoritative epoch exists and no durable cluster history exists, a member may attempt to acquire bootstrap charter through init lock.
2. If init lock is held by a peer, every other node publishes and waits.
3. If init lock holder disappears before completing bootstrap, the next node may attempt to claim a fresh bootstrap charter after timeout and evidence review.
4. Bootstrap charter must be converted into epoch generation `1` only once Postgres bootstrap succeeds and the leader publication is written.

### When existing local data may still be valid for initialization

This option distinguishes "bootstrap seeding" from "reusing existing data." Existing data may still be valid for initializing the cluster if:

1. there is no authoritative epoch
2. authoritative evidence does not show another live cluster history
3. local data has the expected cluster identity and no conflicting higher epoch
4. the bootstrap charter specifically allows reuse

That rule prevents needless wiping while still making bootstrap decisions typed and reviewable.

## Replica convergence as one coherent path

Replica convergence becomes "enter the active epoch safely," with exactly four recovery shapes:

### `StartAsFollower`

The local timeline and data lineage already align with the active epoch. The node starts as replica following the epoch holder.

### `RewindIntoEpoch`

The node has reusable data but diverged from the active epoch. The node rewinds using an approved source member from the active epoch and then starts as follower.

### `BaseBackupIntoEpoch`

The node cannot safely prove rewind viability or rewind has already failed for the target epoch. It wipes or replaces the data directory and clones from an approved source inside the active epoch.

### `WaitForSourceQualification`

The active epoch exists, but the node cannot yet confirm a safe source member, likely because publication is stale or partial. The correct action is to publish, wait, and retry rather than guessing in dispatch.

The design intent is to unify healthy follow, tolerable lag, rewind, and basebackup into one typed "attach to epoch" contract family.

## Partial-truth publication model

### New publication shape

This option expands member publication so every node always publishes the richest truth it can without pretending certainty it does not have.

```text
struct MemberEpochPublication {
    member_id: MemberId,
    observed_at: UnixMillis,
    process_health: ProcessHealthClaim,
    postgres_view: MemberPostgresView,
    observed_epoch: Option<EpochRef>,
    local_epoch_token: Option<LeaseTokenRef>,
    lineage_hint: Option<LineageHint>,
    attachment_state: AttachmentState,
}
```

Important details:

1. `process_health` must still be publishable even when pginfo is degraded.
2. `postgres_view` can remain `Unknown` while still publishing readiness and any known timeline.
3. `observed_epoch` is the node's best current belief about the authoritative epoch.
4. `attachment_state` says whether the node believes it is holder, follower, recovering, fenced, or waiting.

That satisfies the user requirement that member keys should carry partial truth like "pginfo failed but pgtuskmaster is up" instead of silence.

### Confidence model

Publication should be explicit about freshness and confidence:

```text
enum PublicationConfidence {
    CurrentObservation,
    RecentObservation,
    StaleObservation,
}
```

The HA decider can then distinguish "member says unknown but recently alive" from "member is fully stale."

## Where deduplication moves

### New rule

The HA decider and lowerer may assign `EpochIntentId`, but they may not decide whether a consumer should ignore a duplicate. Consumers own that decision entirely.

### Mechanism

Each consumer maintains a tiny ledger keyed by `EpochIntentId`.

For the process consumer:

1. If the same epoch intent is already active or completed successfully, ignore the duplicate.
2. If the same intent failed and the revision did not change, expose that failure back to HA instead of silently replaying.
3. If the revision increased, accept the new intent.

For the DCS consumer:

1. Compare desired publication/lease writes by semantic identity, not by sender tick.
2. If the ledger already reflects the same effective epoch write, acknowledge without re-applying.

This is safer than current sender ownership because the receiver knows whether an action is still active, completed, failed, or superseded. The sender only knows it wanted something.

## Concrete repo areas a later implementation would touch

- `src/ha/worker.rs`
  Replace the current world-to-decide-to-reconcile center with observation assembly plus epoch-story evaluation.
- `src/ha/decide.rs`
  Replace direct role selection with `infer_epoch_story(...)` and `resolve_role_contract(...)`.
- `src/ha/reconcile.rs`
  Either collapse it into epoch-intent lowering or refactor it into a pure lowering stage from `RoleContract` to consumer intents.
- `src/ha/process_dispatch.rs`
  Remove source-member discovery and startup-intent reconstruction from dispatch; require fully typed epoch-scoped inputs.
- `src/ha/decision.rs`
  Reuse or replace `DecisionFacts`, `HaDecision`, and related ADTs with epoch-story ADTs.
- `src/ha/lower.rs`
  Convert the effect model to carry `EpochRef`, `EpochIntentId`, and consumer-owned idempotency semantics.
- `src/ha/actions.rs`
  Update action identity so actions are epoch-scoped rather than tick-scoped.
- `src/dcs/state.rs`
  Extend member publication types to include observed epoch, attachment state, and freshness/confidence.
- `src/dcs/worker.rs`
  Publish richer epoch-aware member truth.
- `src/pginfo/state.rs`
  Preserve and surface lineage-relevant partial truth cleanly.
- `src/runtime/node.rs`
  Delete or quarantine stale startup-planning references and ensure runtime boot enters the unified HA epoch-story loop immediately.
- `tests/ha.rs` and HA feature fixtures
  Update or expand scenario assertions to validate epoch-story boundaries, especially around degraded-majority continuation and restart after total outage.

## Meaningful changes required by this option

### New types

- `EpochStory`
- `EpochRef`
- `EpochAuthority`
- `EpochContinuity`
- `LocalEpochRelation`
- `RoleContract`
- `EpochIntentId`
- `MemberEpochPublication`
- `AttachmentState`
- `PublicationConfidence`
- `BootstrapStory`
- `HandoffStory`
- `RecoveryEpochStory`

### Deleted or collapsed paths

- Collapse startup-only intent reconstruction out of `src/ha/process_dispatch.rs`.
- Remove any remaining special startup planner/executor path from live runtime behavior.
- Delete tick-derived process identity as the primary dedup contract.

### Responsibility moves

- HA pure logic owns epoch-story inference.
- Lowering owns translation from contracts to intents.
- Consumers own duplicate suppression and "already active/already done" checks.
- DCS publication owns truth declaration, including partial truth and epoch attachment state.

### Transition changes

- Non-full-quorum no longer means one generic degraded branch.
- Switchover becomes an explicit handoff story.
- Full-stop restart becomes either `MajorityRestartedWithoutProcessLeader` continuation or bootstrap charter, never a fuzzy startup special case.

### Lowering boundary changes

- Process lowering receives `RecoveryPath` and explicit source member ids already validated by pure logic.
- DCS lowering receives explicit publication contracts with epoch attachment information.
- Neither consumer reconstructs epoch meaning from raw cache at apply time.

### DCS publication behavior changes

- Every member publication carries richer epoch belief.
- Unknown pginfo still publishes liveness and attachment state when possible.
- Publication freshness becomes explicit.

### Startup handling changes

- Runtime starts observation and enters epoch-story evaluation immediately.
- Empty `pgdata`, existing `pgdata`, init lock, and observed leader state are all handled through the same state machine.

### Convergence handling changes

- Rejoin, rewind, and basebackup are all "attach to target epoch" flows.
- Rewind failure for an epoch becomes durable recovery state for that same epoch so the system escalates to basebackup deterministically.

### Test updates a later implementation would likely need

- Add or adjust assertions that verify `DegradedButAuthoritative` continuation instead of generic fail-safe.
- Add explicit checks that restarted old primaries recognize they are on superseded epochs.
- Add checks that full-cluster restart selects either continuation or bootstrap by explicit epoch story rather than by incidental startup ordering.
- Add receiver-idempotency tests ensuring repeated epoch intents do not produce duplicate process jobs.

## Migration sketch

1. Introduce `EpochRef` and include it in DCS publication without changing behavior yet.
2. Add observation-layer structs that assemble all lineage-relevant evidence in one place.
3. Implement `infer_epoch_story(...)` in parallel with current decision logic and compare outputs in tests/logging.
4. Move recovery source selection out of process dispatch into pure contract resolution.
5. Add consumer ledgers for idempotent epoch intents.
6. Switch the live worker from direct desired-role computation to `EpochStory -> RoleContract`.
7. Delete stale startup-specific planning remnants and any duplicate authority paths.
8. Rewrite HA tests to assert epoch-story transitions directly where that improves clarity.

The key migration principle is that no stale legacy path should survive once the epoch-story path becomes authoritative. This option is not compatible with keeping both startup reasoning systems alive indefinitely.

## Non-goals

- This option does not try to minimize type count. It intentionally adds several ADTs to make authority transitions explicit.
- This option does not propose changing the external operator model into a fully manual orchestration system.
- This option does not remove leases. It makes them more central.
- This option does not make pginfo certainty a prerequisite for publishing any member truth.

## Tradeoffs

- The design is more explicit, but it is also heavier. Engineers must understand epoch lineage, not only current role.
- Publication format becomes richer and therefore more complex to version during implementation, although this repo's greenfield rule allows aggressive cleanup.
- Some current helper functions will disappear or move, which is good for clarity but makes the refactor larger.
- If epoch publication is implemented sloppily, the model could become noisy. The implementation must preserve strong invariants around freshness and authoritativeness.

## Logical feature-test verification

### `ha_dcs_quorum_lost_enters_failsafe`

If the node truly loses authority proof and cannot classify the situation as `DegradedButAuthoritative`, the story becomes `Unauthoritative`. A primary with an old epoch transitions to `Fence`, and a follower transitions to `Wait`. This preserves the safety intent of the current scenario while narrowing it to the real condition that matters: loss of epoch authority, not merely loss of perfect quorum labels.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

When a primary loses epoch authority, the fence contract includes a cutoff derived from the last held epoch and committed LSN. That means post-cutoff writes remain blocked, but the reason is now explicit: the node is holding a superseded or unprovable epoch and must not continue as writer.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side classifies the old epoch as no longer continuable by the old holder and creates `generation + 1`. The isolated old primary sees either no authority or later a foreign higher epoch. It must fence rather than continue. This directly matches the intended single-primary behavior.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

The healed node observes that its local data belongs to the older epoch. The only valid story is `RecoveryEpochStory` targeting the higher epoch. Depending on lineage compatibility it chooses `RewindIntoEpoch` or `BaseBackupIntoEpoch`, then becomes follower.

### `ha_primary_killed_then_rejoins_as_replica`

The killed primary's lease renewal stops, a new epoch may be created, and on restart the old primary reads publications showing a higher generation holder. It therefore rejoins through recovery instead of attempting to resume primary. That is exactly what this option is designed to make structurally obvious.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

If the old leader still holds an authoritative epoch and one replica restarts, the restarted replica attaches to that same epoch as follower. The story is `ActiveEpochStory` with `DegradedButAuthoritative` or `ConfirmedMajority`, not fail-safe. Quorum restoration becomes epoch continuation, not a special restart branch.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the scenario that most directly motivates the option. On restart of two nodes after total outage, the system must explicitly decide whether one existing epoch can be continued or whether a new bootstrap-like epoch creation is needed. The decision comes from `EpochContinuity::MajorityRestartedWithoutProcessLeader` plus local lineage evidence. Once one node becomes the authoritative holder, the final node rejoins that epoch through normal recovery. The current timeout theme is addressed by forcing the system to classify restart lineage explicitly instead of waiting in ambiguous startup unknowns.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The recovery story for the old primary starts as `RewindIntoEpoch`. If rewind fails for the target epoch, the recovery state records that failure for that epoch and deterministically transitions to `BaseBackupIntoEpoch`. That preserves the intended fallback behavior.

### `ha_replica_stopped_primary_stays_primary`

Stopping a replica does not alter the active epoch if majority-backed authority still supports the current holder. The leader remains on `HoldLeader`, and the replica later rejoins through `JoinFollower` or recovery if needed.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

Rejoin attempts are local recovery stories targeting the already active epoch. They do not create new authority. A broken replica may fail recovery repeatedly without destabilizing the leader's epoch. That separation between authority story and recovery story is one of the main strengths of this option.

## Recommended future implementation shape

If this option is chosen later, the implementation should treat `EpochStory` as the single canonical HA output and force every downstream path to consume it. That means no hidden "one more helper" in runtime or dispatch that can reinterpret authority. The value of the design is lost if epoch lineage remains optional context instead of mandatory structure.

## Q1 [Should the epoch generation be stored only in leader lease records or also redundantly in member publication]

If generation exists only in the leader lease record, member publication stays smaller but follower-side diagnosis becomes more dependent on reading multiple DCS keys at once.

If generation is duplicated into member publication, restart and debug flows become easier, but publication skew becomes another case to reason about.

Should every member state echo the active generation so epoch reasoning survives partial key visibility, or should the system insist on leader-lease lookup as the sole source of truth?

## Q2 [How should full-cluster restart distinguish continuation from bootstrap]

After all nodes stop, two nodes may come back with matching lineage but no running primary.

One approach is "continue the highest provable epoch if enough lineage evidence matches." Another is "treat every total outage as bootstrap charter for a new generation."

Which rule better preserves safety and operator predictability when majority nodes restart with existing data but no current lease holder exists?

## Q3 [What evidence is sufficient for DegradedButAuthoritative]

The option depends on a narrower and more useful authority class than today's broad degraded branch.

But the exact evidence threshold matters: DCS health, member freshness, local lease memory, and peer epoch publication might all contribute.

What minimum evidence should allow degraded-majority continuation without accidentally permitting split-brain on ambiguous partial observations?

## Q4 [Should recovery failure memory be keyed by target epoch or by source member]

Rewind failure could be remembered as "rewind into epoch 42 failed" or as "rewind from node-b failed."

Epoch-keyed memory is more architectural, while source-keyed memory might avoid penalizing a whole epoch because one source member was bad.

Which memory key makes fallback to basebackup deterministic without hiding a still-viable alternate recovery source?

## Q5 [Should handoff create a brand new epoch before or after demotion completes]

The old leader and target cannot safely believe contradictory things during switchover.

Creating the new epoch too early risks overlap; creating it too late risks long no-primary windows and awkward retries.

At what exact handoff point should the system durably publish the successor epoch so that switchover is both safe and deterministic?
