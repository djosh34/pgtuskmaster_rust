# Option 3: Lease-First Authority Core

This is a design artifact only. It does not change production code, tests, configuration, or documentation in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repo gates are explicitly outside the scope of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA design still treats lease handling as one concern among many, when in practice lease authority is the central question behind nearly every HA failure the user called out. The differentiator of Option 3 is that lease authority becomes the primary model and every other node action is derived from it. Startup, steady-state, failover, rejoin, fencing, and publication all begin from the same question:

`Who is currently authorized to act as leader for this cluster generation, and how certain is that answer?`

Option 1 centered a single lifecycle kernel. Option 2 centered a split between cluster intent and local execution. Option 3 centers an explicit authority core that reasons about lease epochs first, then derives lifecycle behavior second. The functional direction stays the same: newest info first, then a pure decide step, then typed outcomes, then lowered actions owned by effect consumers.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as input evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign is meant to clarify.
- Relevant failure themes observed previously from `target/nextest/ultra-long/junit.xml` and exported logs:
  - quorum-loss scenarios did not reliably surface the expected `fail_safe` evidence
  - degraded-majority scenarios did not reliably expose a replacement primary from the healthy majority
  - some restart and restore-service scenarios left a node writable when it should have remained blocked
  - targeted switchover toward a degraded replica was accepted when it should have been rejected
  - rewind-to-basebackup fallback evidence was not reliably visible
  - old-primary loss-of-authority timing remained too weak in storage-stall and kill scenarios
  - rejoin behavior after failover remained ambiguous rather than following one clean authority transfer path

These failures do not prove that this option is correct. They do reinforce the user's claim that the current authority, quorum, startup, and convergence rules are not yet expressed through one coherent model.

## Option differentiator

The core difference in this option is that HA phases are no longer the primary truth. Lease authority is.

In Option 3, the pure decider does not begin with "am I in Primary or Replica or FailSafe?" It begins with "what is the lease situation?" and only then derives node behavior from that answer. The pure output of the decider is an `AuthorityOutcome` that encodes:

- whether a valid leader lease exists
- whether this node holds that authority, may compete for it, or must stand down
- what cluster generation and lease epoch the node believes it is in
- whether local writes must be fenced
- whether local data may be reused, rewound, or discarded

That is what makes this option materially different from the first two:

- Option 1 organized around one lifecycle kernel and one broad node-phase machine.
- Option 2 organized around a cluster-intent machine plus a local-execution machine.
- Option 3 organizes around an authority ledger with lease epochs, claim validity, expiry reasoning, and explicit lease transfer semantics.

## Ten option set for the overall task

For completeness, this artifact remains one member of the already-fixed ten-option set:

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

Option 3 is the most authority-centric proposal in the set. It is intended for a future implementation that wants a very explicit answer to the question "who may write right now?" before it answers "what role should the process take?"

## Current design problems

### 1. Startup logic is still outside the main authority model

`src/runtime/node.rs` still contains `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. Those functions make decisions that are not operational details. They are authority decisions:

- may the node initialize?
- may the node continue using local `pgdata`?
- should it follow an existing leader?
- is it allowed to become a primary candidate?

The problem is not only duplication. The problem is that lease authority is being reasoned about before the main HA loop exists, and with a different vocabulary.

Option 3 removes the special startup planner entirely. Startup is just the first authority-resolution tick, where some observations are unknown and some lease evidence may still be stale or absent.

### 2. Sender-side dedup in `src/ha/worker.rs` is hiding receiver truth

`should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)` currently ask the HA sender to infer whether a process request would be redundant. That is an ownership error. The sender cannot know whether the previous request:

- was accepted
- is still executing
- partially executed
- failed and must be retried
- became obsolete because the authority epoch changed

In a lease-first design, sender-side dedup is even more dangerous because every action must be interpreted relative to a lease epoch. A request that was redundant in epoch 41 may be mandatory in epoch 42. Dedup belongs with receivers that compare the incoming epoch-keyed envelope against their own last-applied state.

### 3. Authority rules are spread across too many modules

The current architecture distributes leadership truth across:

- runtime startup logic in `src/runtime/node.rs`
- HA phase selection in `src/ha/decide.rs`
- effect planning in `src/ha/lower.rs`
- local transition selection in `src/ha/process_dispatch.rs`
- member publication and trust rules in `src/dcs/worker.rs` and `src/dcs/state.rs`

The result is that no single type answers these questions cleanly:

- who is leader now?
- who may become leader next?
- when is an old leader definitely unauthorized?
- when is degraded-majority operation still valid?
- when must a node refuse writes even if PostgreSQL still runs?

Option 3 introduces one explicit authority model so those answers are encoded first and consumed everywhere else.

### 4. The current non-full-quorum shortcut is not lease-aware enough

`src/ha/decide.rs` currently treats non-`FullQuorum` as a near-direct path into fail-safe behavior. That approach conflates:

- no full membership visibility
- no majority
- no lease renewal path
- stale leader evidence
- minority isolation

Those are not equivalent.

A healthy 2-of-3 majority can still sustain cluster authority if it can confirm current leadership or elect a replacement. A minority-isolated former primary cannot. The real safety question is lease authority under majority-confirmed visibility, not merely whether all configured members are visible.

### 5. Startup and rejoin decisions are still partially hidden in process dispatch

`src/ha/process_dispatch.rs` derives authoritative start and recovery intent through helpers such as `start_intent_from_dcs(...)`, `start_postgres_leader_member_id(...)`, and rewind/basebackup source validation. That means the layer that should only translate typed intent into concrete work is still choosing parts of the node's authority path.

Option 3 changes that: process dispatch becomes mechanical. The pure authority core decides whether the node:

- may claim or renew leadership
- must remain fenced
- may follow
- must rewind
- must basebackup
- may bootstrap a fresh generation

The dispatch layer only lowers those typed decisions into worker-specific commands.

### 6. Partial truth publication is still too weak

`src/dcs/worker.rs` and `src/pginfo/state.rs` already show that the codebase understands partial truth, but the control plane still does not treat it as a first-class input to authority reasoning. The user requirement is explicit: "pginfo failed but pgtuskmaster is up" is useful truth and should stay publishable.

In a lease-first system this is critical. Lease authority should not depend on pretending unknown states are absent. A node may be:

- process alive
- SQL unknown
- role last known as replica
- local data present
- unable to prove readiness

That is still actionable information for authority and recovery decisions.

## Core thesis

The core thesis of this option is:

1. Cluster safety should be defined in terms of lease authority, not coarse role phases.
2. Every tick should produce a pure authority answer before any local role or recovery answer.
3. Startup is not special. It is simply the earliest authority tick with incomplete evidence.
4. Local actions must be tagged with the authority epoch that caused them.
5. Receivers, not senders, must reject stale or duplicate work.
6. A node that loses authority must be able to prove that writes are fenced before the system considers the cluster healthy.

## Proposed control flow

Every reconciliation tick, including the first startup tick, follows the same path.

1. Gather newest observations from DCS, pginfo, local process state, local data-dir inspection, init-lock state, and node runtime facts.
2. Normalize those observations into one immutable `AuthorityObservation`.
3. Feed `AuthorityObservation + prior AuthorityLedgerState` into a pure `resolve_authority(...)` function.
4. Receive an `AuthorityOutcome` that answers:
   - what lease epoch the cluster is in
   - whether a leader lease is active, expiring, expired, or absent
   - whether this node holds, may compete for, or must yield authority
   - what safety stance applies to local writes
   - what convergence path applies to local data
5. Feed `AuthorityOutcome` into pure lowerers that create typed envelopes for process, lease, and DCS workers.
6. Publish the latest member truth and authority summary through DCS.
7. Let effect consumers accept or reject envelopes based on receiver-owned epoch-aware idempotency.

### High-level flow diagram

```text
                 +---------------------------+
pginfo --------->|                           |
dcs ------------>| AuthorityObservation      |
process -------->| assembler                 |
data-dir ------->| newest facts + partial    |
init-lock ------>| truth preserved           |
runtime -------->|                           |
                 +---------------------------+
                               |
                               v
                 +---------------------------+
                 | resolve_authority()       |
                 | pure lease-first decider  |
                 +---------------------------+
                               |
                               v
                 +---------------------------+
                 | AuthorityOutcome          |
                 | - lease epoch             |
                 | - authority stance        |
                 | - write safety            |
                 | - local convergence path  |
                 +---------------------------+
                               |
                               v
                 +---------------------------+
                 | pure lowerers             |
                 | epoch-tagged envelopes    |
                 +---------------------------+
                    |            |           |
                    v            v           v
              Process worker  Lease worker  DCS worker
              owns dedup      owns dedup    owns dedup
```

### Changed responsibility boundary

```text
CURRENT
-------
startup planner + HA phase selection + process dispatch
share authority logic indirectly

OPTION 3
--------
AuthorityObservation
  -> resolve_authority()
  -> typed epoch-tagged outcome
  -> lowerers
  -> receivers validate epoch and dedup locally
```

### Control flow from startup through steady state

Startup uses the same path as steady state. The only difference is that the initial `AuthorityObservation` will often contain:

- no confirmed local SQL status yet
- potentially stale DCS membership
- unknown process state while probes warm up
- fresh data-dir inspection
- init-lock evidence
- no prior in-memory authority ledger

The decider explicitly tolerates those unknowns. It does not branch out to a separate startup planner. That means the same input rules determine:

- whether a cluster already exists
- whether a lease is valid
- whether this node may claim authority
- whether this node must stay fenced until more evidence arrives
- whether the node should follow, rewind, basebackup, or bootstrap

Steady-state ticks then continue from the same model but with richer observations and a non-empty authority ledger.

## Proposed typed state model

### Main observation type

```text
struct AuthorityObservation {
    tick: u64,
    now: TimeSnapshot,
    cluster: ClusterObservation,
    lease: LeaseObservation,
    membership: MembershipObservation,
    local_postgres: LocalPostgresObservation,
    local_data: LocalDataObservation,
    init_lock: InitLockObservation,
    process: LocalProcessObservation,
    prior_ledger: Option<AuthorityLedgerState>,
}
```

The critical point is that `AuthorityObservation` does not erase uncertainty. It carries explicit unknown and stale states so the pure decider can reason about them.

### Authority ledger state

```text
struct AuthorityLedgerState {
    generation: ClusterGeneration,
    last_resolved_epoch: LeaseEpoch,
    last_known_leader: Option<MemberId>,
    local_authority: LocalAuthorityStatus,
    write_safety: WriteSafetyState,
    convergence: ConvergenceState,
}
```

### Lease and authority primitives

```text
struct ClusterGeneration(u64);
struct LeaseEpoch(u64);

enum LeaseStatus {
    Active {
        holder: MemberId,
        epoch: LeaseEpoch,
        renew_deadline: Timestamp,
    },
    GraceWindow {
        holder: MemberId,
        epoch: LeaseEpoch,
        renew_deadline: Timestamp,
        cutoff_deadline: Timestamp,
    },
    Expired {
        prior_holder: Option<MemberId>,
        prior_epoch: Option<LeaseEpoch>,
    },
    Unknown {
        reason: LeaseUnknownReason,
    },
}

enum LocalAuthorityStatus {
    HoldsLease {
        epoch: LeaseEpoch,
    },
    EligibleCandidate {
        next_epoch: LeaseEpoch,
    },
    MustFollow {
        leader: MemberId,
        epoch: LeaseEpoch,
    },
    MustFence {
        reason: FenceReason,
    },
    ObservationInsufficient {
        reason: InsufficientReason,
    },
}
```

### Write safety state

```text
enum WriteSafetyState {
    WritesAllowed {
        epoch: LeaseEpoch,
        proof: WriteAuthorityProof,
    },
    ReadOnlyGrace {
        epoch: LeaseEpoch,
        cutoff_deadline: Timestamp,
    },
    WritesBlocked {
        reason: WritesBlockedReason,
    },
}
```

This explicit safety type is important. A PostgreSQL process can keep running while writes are blocked. That lets the design distinguish "service process alive" from "leader authority valid."

### Convergence state

```text
enum ConvergenceState {
    NoActionNeeded,
    FollowLeader {
        leader: MemberId,
        epoch: LeaseEpoch,
    },
    RewindThenFollow {
        leader: MemberId,
        epoch: LeaseEpoch,
        source: RewindSource,
    },
    BasebackupThenFollow {
        leader: MemberId,
        epoch: LeaseEpoch,
        source: BasebackupSource,
    },
    BootstrapGeneration {
        generation: ClusterGeneration,
    },
    HoldForEvidence {
        reason: EvidenceHoldReason,
    },
}
```

The decider chooses one convergence path keyed to authority. It does not leave that choice implicit inside process dispatch.

### Final pure output

```text
struct AuthorityOutcome {
    next_ledger: AuthorityLedgerState,
    lease_status: LeaseStatus,
    local_authority: LocalAuthorityStatus,
    write_safety: WriteSafetyState,
    convergence: ConvergenceState,
    publication: MemberPublicationView,
    effects: AuthorityEffectPlan,
}
```

### Member publication view

```text
struct MemberPublicationView {
    member_id: MemberId,
    generation: Option<ClusterGeneration>,
    last_seen_epoch: Option<LeaseEpoch>,
    authority_claim: PublishedAuthorityClaim,
    local_role_observation: PublishedLocalRole,
    sql_status: PublishedSqlStatus,
    readiness: PublishedReadiness,
    data_state: PublishedDataState,
    process_state: PublishedProcessState,
    observation_quality: ObservationQuality,
}
```

This view guarantees that the DCS layer publishes the best obtainable truth rather than deleting detail whenever probes are degraded.

## How `resolve_authority(...)` works

The pure authority resolution sequence should be explicit and stable.

### Step 1: Resolve cluster generation

The decider first determines whether the node is observing:

- an existing cluster generation with a known history
- no existing generation but a possible bootstrap opportunity
- conflicting generation evidence that requires holding

Bootstrap is impossible unless generation resolution is clear enough to avoid accidental split-brain.

### Step 2: Resolve lease status

The decider then answers whether there is:

- an active renewable leader lease
- a lease in grace but not yet fully expired
- an expired lease that may be replaced
- insufficient lease evidence

This step is where degraded-majority reasoning belongs. Missing one member is not automatically failure. The important questions are:

- can a majority still confirm the current epoch?
- can a majority safely renew or replace the lease?
- is this node in the majority or in an isolated minority?

### Step 3: Resolve local authority stance

Based on generation plus lease status, the node classifies itself as:

- current lease holder
- eligible candidate for the next lease epoch
- required follower of another leader
- fenced due to minority or ambiguity
- held pending more evidence

This is the core answer that later layers consume.

### Step 4: Resolve write safety

Write safety is derived from authority, not inferred from the current PostgreSQL role alone.

- A lease holder in a confirmed majority gets `WritesAllowed`.
- A holder in a temporary grace window gets `ReadOnlyGrace` if policy allows probe-driven wind-down before full fencing.
- A node without valid authority gets `WritesBlocked` even if local PostgreSQL still reports primary.

### Step 5: Resolve local convergence path

Only after authority and safety are known does the decider choose local data convergence:

- healthy follower can continue following
- stale timeline with usable source becomes `RewindThenFollow`
- incompatible or unrecoverable timeline becomes `BasebackupThenFollow`
- valid bootstrap candidate becomes `BootstrapGeneration`
- insufficient evidence becomes `HoldForEvidence`

### Step 6: Build publication and effects

The decider packages:

- the next ledger state
- the publication view
- the epoch-tagged effect plan

Lowerers turn those into receiver envelopes without adding new authority logic.

## Quorum and degraded-majority model

This option explicitly rejects the idea that "not full quorum" should map directly to fail-safe.

### Core quorum rules

1. Majority matters more than unanimity.
2. Lease renewal requires confirmation from a valid majority, not necessarily every configured member.
3. A minority-isolated node must assume it is unauthorized even if it still runs PostgreSQL.
4. A majority partition may continue or elect a new leader if it can prove the prior lease is not renewable by the old holder.
5. Ambiguous evidence is not the same as lost majority. Ambiguity may require temporary hold, but not always full fail-safe.

### 2-of-3 behavior

In a three-node cluster:

- if two nodes remain healthy and can observe an expired or non-renewable lease, they may elect or confirm a leader
- if the old primary is the isolated one, it must fence once it can no longer prove renewable majority-backed authority
- if the current leader remains within the healthy majority, it may keep renewing and keep serving

This directly addresses the user's complaint that the current boundary is too blunt.

### When fail-safe still exists

Fail-safe still exists, but it becomes a narrower state:

- DCS majority cannot be confirmed
- lease evidence is stale or contradictory
- local process cannot prove fencing after authority loss
- generation ownership is conflicting or corrupted

That is different from simple degraded-majority operation.

## Lease model

### Lease authority becomes the first-class safety contract

Every leader action is tied to a `LeaseEpoch` within a `ClusterGeneration`. A node is not leader because PostgreSQL says primary. It is leader because the authority core resolved that it currently holds a renewable lease for the current generation.

### Lease acquisition

Lease acquisition requires all of the following:

- a resolved cluster generation
- majority-backed DCS visibility
- no still-valid competing lease
- local eligibility to serve
- no unresolved local data contradiction that would make serving unsafe

The decider then emits an epoch-tagged `AcquireLease` effect envelope.

### Lease renewal

Lease renewal is periodic and explicit. If renewal succeeds, the ledger remains in the same epoch. If renewal enters a grace window due to temporary communication issues, the authority core may choose `ReadOnlyGrace` while preparing fencing unless majority-backed renewal is re-established quickly.

### Lease expiry and loss

If the node cannot renew within the allowed window, the pure authority answer changes before local process commands run:

- `local_authority` becomes `MustFence`
- `write_safety` becomes `WritesBlocked` or, at most, a very short `ReadOnlyGrace`
- outgoing process/DCS envelopes are tagged with the loss-of-authority epoch

That makes "killed primary loses authority" an explicit model rule rather than an emergent behavior.

### Killed primary / lost-lease scenarios

A killed or stalled primary loses meaningful authority as soon as a healthy majority proves the old lease is not renewable and installs the next epoch. When that old primary later restarts, its first authority tick sees:

- newer generation or epoch evidence
- absence of a valid local leadership claim
- requirement to follow or converge, not to resume serving

That sharply reduces the ambiguity seen in old-primary rejoin scenarios.

## Startup reasoning

Startup must use the same authority core. This option makes startup a lease-evidence problem first and a process-start problem second.

### Startup cases the decider must handle

#### Cluster already exists with a valid leader

If DCS shows a current generation and a valid leader lease:

- this node may not bootstrap
- this node may not claim leadership
- this node resolves to follower or convergence behavior depending on local data

#### Cluster exists but leader lease expired

If generation exists but the lease is expired:

- a majority-backed node may become a candidate for the next epoch
- a minority node must hold or fence

#### No cluster exists yet

If there is no generation, no leader, and the init lock is available:

- an eligible node may resolve `BootstrapGeneration`
- existing `pgdata` must still be evaluated, not discarded by default

#### Existing local `pgdata`

Existing `pgdata` is not automatically disqualifying. The decider must classify it:

- reusable for continued service
- reusable for follow after rewind
- too divergent and requiring basebackup
- inconsistent enough that bootstrap cannot safely use it

This is one of the user's direct requests: bootstrap should reconsider whether existing data may still be valid if the node wins the init lock.

### Startup diagram

```text
startup observations
    |
    v
resolve generation?
    |
    +--> existing generation + valid lease --> follow/converge path
    |
    +--> existing generation + expired lease --> candidate or hold
    |
    +--> no generation + init lock + valid local data? --> bootstrap generation
    |
    +--> insufficient evidence --> hold fenced, publish partial truth
```

## Replica convergence as one coherent path

This option treats replica convergence as one authority-derived pipeline rather than scattered branching.

### Unified convergence rules

1. If a valid leader lease exists and local data is already aligned, `FollowLeader`.
2. If the node is close enough and timeline-compatible, continue following without extra work.
3. If the node has the wrong timeline but rewind prerequisites are valid, `RewindThenFollow`.
4. If rewind is impossible or failed, `BasebackupThenFollow`.
5. If no valid leader exists, do not start convergence work that assumes one.

The point is that previously-primary, previously-replica, and freshly-restored nodes use the same pipeline. Their differences are input facts, not separate architectures.

### Why this is simpler than the current shape

The current design still hides too much rejoin truth in `src/ha/process_dispatch.rs`. Option 3 makes convergence a typed part of authority resolution. That means the future implementation can inspect one output and know exactly why the node is rewinding, basebacking up, holding, or following.

## Partial-truth member publication

Member publication must preserve best-known truth even when PostgreSQL probes are degraded.

### Publication rules

1. Never turn "unknown" into silence.
2. Publish observation quality explicitly.
3. Publish the last resolved lease epoch if known.
4. Publish whether the node believes itself authorized, follower-only, fenced, or waiting for evidence.
5. Publish local process state independently from SQL health.

### Example publication states

- `process=alive, sql=unknown, readiness=unknown, authority_claim=must_follow, data_state=present`
- `process=alive, sql=unreachable, readiness=not_ready, authority_claim=must_fence, last_seen_epoch=42`
- `process=stopped, sql=unknown, readiness=unknown, authority_claim=eligible_candidate`

These are better than omission because other nodes can reason about degraded but real information.

## Where deduplication moves

Deduplication must move from sender-side HA logic into effect consumers.

### Sender behavior

The authority core emits epoch-tagged envelopes such as:

- `AcquireLease { generation, epoch, member_id }`
- `FenceWrites { generation, epoch, reason }`
- `StartAsReplica { generation, epoch, leader }`
- `RunRewind { generation, epoch, source }`

The sender emits them whenever the pure outcome says they are desired. It does not attempt to infer whether the receiver is already busy.

### Receiver behavior

Each receiver stores the last accepted semantic key:

- process worker keys on `(generation, epoch, action_kind, source_hash)`
- lease worker keys on `(generation, epoch, lease_action)`
- DCS worker keys on `(generation, epoch, publication_version)`

If the same envelope arrives twice, the receiver no-ops safely. If an older epoch arrives after a newer one, the receiver rejects it as stale. This is much safer than sender-side suppression because only the receiver knows what has actually been applied.

## Concrete future code areas affected

A future implementation of this option would need to touch at least these areas:

- `src/runtime/node.rs`
  - remove or collapse `plan_startup(...)`
  - remove or collapse `plan_startup_with_probe(...)`
  - remove or collapse `execute_startup(...)`
  - remove or collapse `build_startup_actions(...)`
- `src/ha/worker.rs`
  - replace the current phase-driven reconciliation entrypoint with authority-first resolution
  - remove sender-side dedup helpers such as `should_skip_redundant_process_dispatch(...)`
  - stop using `decision_is_already_active(...)` as an HA-sender filter
- `src/ha/decide.rs`
  - replace non-full-quorum shortcut logic with majority-and-lease-aware authority resolution
  - encode lease expiry, grace, renewal, and replacement rules
- `src/ha/decision.rs`
  - define `AuthorityObservation`
  - define `AuthorityLedgerState`
  - define `AuthorityOutcome`
  - define lease, safety, and convergence enums
- `src/ha/lower.rs`
  - lower epoch-tagged authority outcomes into process, lease, and DCS effect envelopes
- `src/ha/process_dispatch.rs`
  - stop deriving lifecycle truth
  - become a mechanical translator from typed convergence or fencing actions into process jobs
- `src/dcs/worker.rs`
  - publish richer `MemberPublicationView`
  - publish authority and observation-quality details
- `src/dcs/state.rs`
  - extend trust and membership structures to preserve partial truth and majority-backed lease reasoning
- `src/pginfo/state.rs`
  - preserve partial states as authority inputs rather than treating them as absence
- `tests/ha.rs`
  - align feature expectations with explicit lease-epoch and fencing semantics
- `tests/ha/features/`
  - update scenario harness expectations where log or state evidence now reflects authority epochs and write-safety transitions

## All meaningful implementation changes required by this option

The future implementation would need to make all of these categories of changes, not just some of them:

- introduce new types for cluster generation, lease epoch, local authority status, write safety state, convergence state, and publication view
- delete the separate startup planning path in runtime and fold startup into the authority reconciliation path
- remove sender-side dedup logic from the HA worker
- move idempotency and stale-action rejection into receivers
- replace broad non-full-quorum fail-safe shortcuts with explicit majority-and-lease rules
- add explicit lease grace and lease-expired handling
- make local write safety an explicit state instead of an implied consequence of role
- move rejoin and recovery choice out of `process_dispatch` into pure authority resolution
- extend DCS publication to include authority claim, last known epoch, process truth, and observation quality
- unify previously-primary and previously-replica rejoin behavior under one convergence pipeline
- redefine bootstrap eligibility so existing `pgdata` can be evaluated rather than automatically ignored
- update tests to validate authority loss, fencing, and lease-epoch handoff directly

If any of those are skipped in a later implementation, the option is only partially implemented.

## Migration sketch

This option would require a deliberate migration that removes stale legacy paths rather than leaving the old startup model in place.

### Stage 1: Introduce authority types beside current behavior

Add the new authority types in `src/ha/decision.rs` and build a passive `resolve_authority(...)` function that can be exercised in parallel with current logic. The goal of this stage is not dual-running forever. It is only to make the future model concrete.

### Stage 2: Convert startup into authority observations

Replace `plan_startup(...)` and related runtime helpers with observation assembly that feeds the authority core. At this stage the old startup functions should be removed, not kept behind flags.

### Stage 3: Move convergence choice out of process dispatch

Refactor `src/ha/process_dispatch.rs` so it becomes a lowering layer only. The authority core should fully choose follow, rewind, basebackup, fence, bootstrap, or hold.

### Stage 4: Move dedup to receivers

Delete sender-side dedup in `src/ha/worker.rs`. Add receiver-owned epoch-aware idempotency keys in process, lease, and DCS consumers.

### Stage 5: Rework quorum and test semantics

Reinterpret HA behavior in terms of lease epochs and write safety. Update tests so they assert the new authority contract directly rather than inferring it from weaker side effects.

### Stage 6: Remove legacy paths entirely

Delete any remaining phase or dispatch shortcuts that bypass authority resolution. This repo is greenfield. Backward-compatible dual paths should not remain.

## Non-goals

- This option does not attempt to preserve the existing startup planner.
- This option does not put etcd or PostgreSQL IO into the pure decider.
- This option does not depend on sender-side dedup remaining in the HA worker.
- This option does not claim that every degraded condition should remain writable.
- This option does not solve the failing tests in this run; it only explains a future architecture that could.

## Tradeoffs

- This design introduces more explicit types than the current code, especially around authority and lease state.
- The model is conceptually stricter, which means future implementations will need disciplined epoch handling in receivers.
- Some existing tests and log expectations will likely need to be updated because authority transitions will become more explicit.
- The design may feel heavier than a phase-only approach, but that extra structure is the point: authority ambiguity is the current core problem.

## Logical feature-test verification

This section explains, scenario by scenario, how this option would logically satisfy the important HA behaviors. It is not implementation proof. It is architectural intent.

### `ha_dcs_quorum_lost_enters_failsafe`

Under this design, the node enters fail-safe only when it cannot prove majority-backed lease authority or cannot prove fencing after authority loss. The test should pass because quorum loss is interpreted through lease renewability, not just membership count. If a node truly cannot confirm majority-backed authority, `write_safety` becomes `WritesBlocked` and the authority claim published to DCS reflects fenced or insufficient-authority status.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This design makes write blocking an explicit state. Once the grace or renewal deadline passes without authority, the primary is no longer merely "probably unsafe." It is explicitly in `WritesBlocked`. That gives a clear architectural path to blocking post-cutoff writes.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

A partitioned old primary becomes a minority node that cannot prove renewable lease authority. The healthy majority can observe that the old lease is not renewable and install the next epoch. The old primary publishes or infers `MustFence`; the majority leader resolves `HoldsLease { epoch: next }`.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

When healed, the old primary sees a newer valid epoch and no valid local leadership claim. The convergence path becomes `FollowLeader`, `RewindThenFollow`, or `BasebackupThenFollow` depending on local timeline compatibility. It does not resume leadership because authority is epoch-based, not role-memory-based.

### `ha_primary_killed_then_rejoins_as_replica`

After restart, the killed node's first authority tick sees the current leader lease and resolves follower-only behavior. If local data is divergent, it enters rewind or basebackup. The important point is that restart does not restore authority; only current epoch evidence can do that.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

A restarted replica that rejoins a majority-backed leader does not need a special ad hoc path. The authority core sees an active leader epoch and chooses the appropriate convergence plan. Service restoration occurs because lease-majority authority becomes valid again, not because a role flag happened to flip.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two nodes re-establish generation and authority using bootstrap or epoch renewal rules, depending on existing cluster evidence. The final node then joins through the same follower convergence pipeline. This is exactly the kind of scenario improved by startup using the same authority core as steady-state.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The convergence state machine explicitly orders recovery as:

1. follow if already aligned
2. rewind if possible
3. basebackup if rewind is impossible or failed

That means a rewind failure is not a weird exception path hidden in worker state. It is an expected authority-derived fallback.

### `ha_replica_stopped_primary_stays_primary`

If the leader still has renewable majority-backed authority, the loss of a replica alone does not force fail-safe. The primary remains primary because authority and write safety are still valid.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica is only a local convergence problem unless it changes majority-backed lease authority. The leader keeps serving if its lease remains valid. The broken replica resolves to `HoldForEvidence`, `RewindThenFollow`, or `BasebackupThenFollow` without destabilizing the cluster-wide authority epoch.

## Summary judgment on this option

Option 3 is strongest when the team wants the architecture to answer authority questions unambiguously and early. It is likely a good fit if the root cause of current HA drift is that too many modules are making implicit claims about leadership, safety, and write authority. It is a weaker fit if the team prefers a more topology-first or minimal-change design, because this option is unapologetically centered on explicit lease semantics.

## Q1 Should lease grace permit read-only continuation?

Context:
The design includes `ReadOnlyGrace` as a possible intermediate state between fully valid authority and fully blocked writes. That can help when a leader temporarily loses renewal visibility but should still stop accepting writes before demotion is fully complete.

Problem / decision point:
Should the future implementation preserve a short read-only grace period, or should it move directly from `WritesAllowed` to `WritesBlocked` whenever majority-backed lease renewal cannot be proven?

Restated question:
Is there enough operational value in a read-only grace mode to justify the added state, or is strict immediate fencing the cleaner contract?

## Q2 Should lease epochs advance only on leader replacement or also on renewal failures?

Context:
This design assumes lease epochs are the main authority timeline. Epoch inflation can make debugging and stale-action rejection easier, but too much epoch churn can also make the model noisy.

Problem / decision point:
When a leader temporarily loses renewal and then regains it without an actual replacement, should that stay in the same epoch with a degraded safety state, or should any lost-renewal event force a new epoch?

Restated question:
What is the right balance between a stable epoch model and a strict one where every serious authority wobble becomes a new epoch?

## Q3 How much authority detail should member publication expose?

Context:
The publication view in this option includes authority claim, last known epoch, process truth, SQL truth, and observation quality.

Problem / decision point:
Publishing richer authority details helps debugging and better decider inputs, but it also risks creating too many externally visible states for operators and tests to depend on.

Restated question:
Should DCS member publication expose the full authority shape, or only a smaller summary plus enough metadata for deciders?

## Q4 Should bootstrap reuse existing `pgdata` under an init-lock win?

Context:
The user explicitly asked for bootstrap to reconsider whether existing local data may still be valid when a node wins the init lock.

Problem / decision point:
Reusing data can make restarts and cold-cluster recovery faster, but it also raises the risk of bootstrapping from data that is locally consistent yet globally wrong for the intended generation.

Restated question:
When a node wins the init lock with existing `pgdata`, should the default be "reuse if safety checks pass" or "force fresh bootstrap unless a strong proof of compatibility exists"?

## Q5 Should process receivers fence stale epochs before or after observing local PostgreSQL role?

Context:
This design puts stale-action rejection and idempotency in receivers. A process receiver may receive an epoch-tagged fencing or role-change request while PostgreSQL still locally reports an older role state.

Problem / decision point:
Should receivers treat epoch freshness as absolute and fence first, or should they consult current local process role before applying epoch-driven commands?

Restated question:
Is epoch ordering alone the safe enough source of truth for receivers, or should receivers combine epoch ordering with live local process confirmation before executing disruptive actions?
