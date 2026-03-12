# HA Refactor Option 5: Peer-Evidence Vote Matrix

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because the current HA code already collects a surprising amount of peer evidence, but then compresses that evidence too early into `DcsTrust` plus a small `PeerKnowledge` ranking. The differentiator for this option is that leadership is not chosen by "full quorum yes/no, then pick best candidate." Instead, every tick builds an explicit peer-evidence vote matrix that records what each member claims, what each observer can corroborate, what freshness window applies, which lease lineage is visible, and whether a majority proof actually exists for continuing or replacing authority. That makes this option materially different from option 1, which first classifies a broad regime, option 2, which first classifies lease epochs and handoff stories, option 3, which first classifies recovery stages, and option 4, which first recenters command identity and idempotency.

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 5.

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

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because this artifact is not arguing that a vote matrix is required due to broad instability. It is arguing that the evidence pipeline is still too compressed and too easy to misread when restart, failover, or degraded-majority behavior becomes subtle.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out while waiting for one primary across the two restarted fixed nodes and sampled both restarted nodes as unknown. That matters here because a full-cluster restart is exactly the kind of situation where "full quorum or fail-safe" is too blunt. The system needs a typed way to say, "two members have enough corroborated evidence to prove a safe majority authority story even though no current primary is visible."
- `tests/ha.rs` remains the acceptance contract surface for later implementation work. A future implementation based on this option must either satisfy those scenarios or revise them with explicit new semantics.

## Current design problems

### 1. The live runtime path still enters HA with a generic trust enum instead of a majority-proof object

The live startup path now goes through `run_node_from_config(...)`, `ensure_start_paths(...)`, and `run_workers(...)` in `src/runtime/node.rs`. That is better than the older explicit startup planner/executor split described in the task background, but once the HA worker starts it still receives a `DcsState` whose authority view is primarily expressed as `DcsTrust::{FullQuorum,Degraded,NotTrusted}` from `src/dcs/state.rs`.

That is a useful low-level store-health signal, but it is not rich enough to express the HA question the user cares about. HA does not merely need "store healthy or unhealthy." It needs:

- which members are currently observable,
- which observations are fresh enough to count,
- whether a majority of members agrees on a leader lineage,
- whether any minority claim contradicts that lineage,
- whether a majority is still operational even if not every member is visible,
- whether the cluster is in a restart window with no current primary but enough evidence to safely elect.

`DcsTrust` should remain a DCS-layer input, not the top-level HA authority decision.

### 2. The current decision path compresses evidence too early into `PeerKnowledge` and ranking helpers

`src/ha/worker.rs` builds `GlobalKnowledge` by collecting:

- `dcs_trust`,
- `LeaseState`,
- `observed_lease`,
- `observed_primary`,
- peer rows mapped from `MemberSlot` into `PeerKnowledge`,
- self evidence mapped by `build_self_peer(...)`.

That is already an evidence-gathering step, but it collapses each peer into a small `PeerKnowledge` object with only:

- `api` visibility,
- `ElectionEligibility`.

Then `src/ha/decide.rs` uses `best_failover_candidate(...)` and `compare_candidate_rank(...)` to choose a promotion winner when the lease is open. This is too much compression. It reduces a multi-dimensional question into a sortable candidate rank before the system has first established whether the cluster possesses a valid majority authority proof.

The practical result is a mental model like:

1. Is `DcsTrust` full?
2. If yes, do we have a lease?
3. If no lease, who is the best candidate?

That is not the same as:

1. What evidence exists per member?
2. Which pieces of evidence corroborate each other?
3. Does a valid majority proof exist for continuing authority, replacing authority, or waiting?
4. Which node contract follows from that proof?

### 3. Degraded-majority operation and unauthoritative blackout still share too much code-path shape

`decide(...)` immediately routes every non-`DcsTrust::FullQuorum` case into `decide_degraded(...)`. Inside that branch:

- a primary becomes `FailSafeGoal::PrimaryMustStop(...)` if an active or observed epoch exists,
- replicas become `FailSafeGoal::ReplicaKeepFollowing(...)`,
- offline nodes wait for quorum.

This is safe-biased, but it conflates multiple materially different situations:

- total store blackout,
- minority-isolated old primary,
- healthy majority that has lost one member,
- full-cluster restart where two nodes return first,
- store recovered but observations are temporarily stale,
- switchover request active during partial visibility.

Those situations should not all begin from the same degraded branch. A vote matrix design should explicitly prove or fail to prove majority authority before deciding whether fail-safe, re-election, or wait is correct.

### 4. Startup and rejoin reasoning is still partially rediscovered in process dispatch instead of being proved in authority resolution

`src/ha/process_dispatch.rs` still contains `start_intent_from_dcs(...)`, `resolve_source_member(...)`, `validate_rewind_source(...)`, and `validate_basebackup_source(...)`. That means command lowering still asks DCS for authoritative source details while materializing process jobs.

The problem is not those helpers themselves. The problem is that if the system has not already constructed a majority-backed authority proof, then dispatch is being asked to recover semantic meaning too late:

- who is the current authoritative leader,
- which source member is valid,
- whether this node should bootstrap, follow, rewind, or clone,
- whether a full-stop restart permits a new primary election.

Under this option, those questions must be answered by the vote matrix and its proof result before lowering happens.

### 5. Sender-owned command identity still leaks authority semantics into the dispatch layer

`process_job_id(...)` in `src/ha/process_dispatch.rs` still derives command identity in the HA sender path using `(scope, self_id, action, action_index, ha_tick)`. The earlier sender-side skip helper discussed in the task background is no longer the live dominant shape, but command identity is still born on the sender side.

That is specifically relevant to this option because a vote matrix decision should produce stable authority-proof revisions, not tick-scoped imperative jobs. Receivers should decide whether a proof-backed intent revision is already satisfied, already running, superseded, or conflicting.

### 6. Partial-truth publication already exists, but it is not yet promoted into first-class voting evidence

`src/dcs/state.rs` already models:

- `MemberPostgresView::{Unknown,Primary,Replica}`,
- `UnknownPostgresObservation`,
- `PrimaryObservation`,
- `ReplicaObservation`,
- `LeaderLeaseRecord`,
- `InitLockRecord`.

`src/pginfo/state.rs` already preserves `Readiness::{Unknown,Ready,NotReady}` and `SqlStatus::{Unknown,Healthy,Unreachable}`. This is strong groundwork. The missing architectural move is to stop treating those fields as merely "inputs to later heuristics" and instead treat them as entries in an explicit evidence matrix from which majority proofs are derived.

## The central proposal

Replace the current top-level HA decision framing with a four-stage proof pipeline:

1. `ObserveWorld`
   Collect the newest local and global facts into an immutable `ObservationEnvelope`.
2. `BuildVoteMatrix`
   Expand those facts into one `VoteMatrix` row per known member plus one row for self-observed local truth.
3. `ResolveAuthorityProof`
   Derive a typed `AuthorityProof` that says whether the cluster has a majority-backed proof for continuing current leadership, electing a replacement, bootstrapping, or waiting.
4. `ResolveNodeContract`
   Map the proof into a local `NodeContract`, then lower that contract into receiver-owned intents and effect plans.

The decisive change is this: no node may promote, continue as primary, demote, or choose a rejoin source until the system can point to the explicit majority proof row-set that authorized that decision.

## Proposed control flow from startup through steady state

The control flow should look like this for every tick, including the first tick after process startup:

```text
runtime startup
    |
    v
run_workers(...)
    |
    v
ObserveWorld
    |
    v
BuildVoteMatrix
    |
    v
ResolveAuthorityProof
    |
    +--> AuthorityProof::MajorityContinue(existing leader lineage)
    |         |
    |         v
    |     ResolveNodeContract
    |
    +--> AuthorityProof::MajorityElect(new winner proof)
    |         |
    |         v
    |     ResolveNodeContract
    |
    +--> AuthorityProof::BootstrapAuthorized(init charter proof)
    |         |
    |         v
    |     ResolveNodeContract
    |
    +--> AuthorityProof::NoValidProof(wait or fail-safe)
              |
              v
         ResolveNodeContract
              |
              v
         Lower receiver-owned intents
              |
              v
         DCS / process consumers apply idempotently
```

The same pipeline must run on:

- cold start,
- restart after total outage,
- steady-state primary operation,
- minority partition,
- majority failover,
- healed old-primary rejoin,
- switchover,
- rewind and basebackup recovery.

There is no separate startup planner. Startup is just the first moment the matrix has to be built from sparse evidence.

## The vote matrix model

### Concept

The vote matrix is a typed table, not a free-form diagnostic blob. Every row must represent one member and every column must encode one kind of evidence with explicit freshness and contradiction semantics.

### Proposed types

```text
ObservationEnvelope
  - observed_at
  - local_member_id
  - dcs_snapshot
  - pginfo_snapshot
  - process_snapshot
  - config_snapshot

VoteMatrix
  - cluster_size_hint
  - rows: BTreeMap<MemberId, VoteRow>
  - local_row: VoteRow
  - contradictions: Vec<MatrixContradiction>
  - freshness_window_ms

VoteRow
  - member_id
  - membership: MembershipEvidence
  - api: ApiEvidence
  - pg: PostgresEvidence
  - wal: WalEvidence
  - data_dir: DataDirEvidence
  - lease_claim: LeaseClaimEvidence
  - init_claim: InitClaimEvidence
  - observer_confidence: ObserverConfidence

AuthorityProof
  - MajorityContinue(ContinueProof)
  - MajorityElect(ElectionProof)
  - BootstrapAuthorized(BootstrapProof)
  - FailSafeRequired(FailSafeProof)
  - WaitForEvidence(WaitProof)
```

### Evidence columns

Each `VoteRow` should answer at least the following questions:

- Is the member present in DCS right now?
- Is that presence fresh enough to count as a vote?
- Does the member advertise API reachability or only local process liveness?
- Is its Postgres view `Unknown`, `Primary`, or `Replica`?
- What timeline and WAL position does it claim?
- If replica, who does it claim to follow?
- Does it hold or cite a lease generation?
- Does it hold or cite init-lock or bootstrap intent?
- Is its data dir missing, bootstrap-empty, consistent, diverged, or unknown from the observer's perspective?
- Is the row self-observed, peer-published, or inferred from stale cache?

This explicit structure is the heart of the option. The system must stop asking "who looks best?" and start asking "what is actually proved?"

## Majority proof resolution

### Principle

An authority decision is valid only if the matrix can produce a row-set that satisfies the cluster's majority rule and does not contain unresolved contradictions strong enough to invalidate the proof.

### Proposed proof classes

#### `ContinueProof`

Used when a current leader lineage is still majority-backed.

Requirements:

- A majority of members are currently present and fresh enough.
- The majority corroborates the same leader lineage or lease generation.
- No conflicting majority-backed lineage exists.
- The proposed continuing primary is either self or a peer with compatible WAL authority.

#### `ElectionProof`

Used when no valid current leader proof exists, but a majority can agree that one candidate should become leader.

Requirements:

- No surviving `ContinueProof` exists.
- A majority of fresh members exists.
- The elected candidate has the strongest WAL-backed candidacy among that majority.
- Minority claims for another leader do not carry matching majority evidence.
- Any switchover request is either incorporated into the proof or explicitly rejected.

#### `BootstrapProof`

Used when the matrix proves the cluster is legitimately in bootstrap formation rather than recovery from prior authority.

Requirements:

- No existing valid leader lineage is corroborated.
- No existing consistent cluster history is visible that should be followed instead.
- Init-lock evidence and member data-dir evidence say bootstrap is still permitted.
- The chosen bootstrap owner is majority-backed or init-lock-authorized under the chosen bootstrap policy.

#### `FailSafeProof`

Used when a node that was primary loses enough authority that continued writes would be unsafe.

Requirements:

- The node previously held authority or still has locally running primary state.
- The matrix cannot prove continuing majority authority.
- A stronger conflicting authority or an authority blackout exists.
- A fence cutoff can be derived when locally committed WAL is known.

#### `WaitProof`

Used when no safe majority action is yet provable, but immediate fencing or promotion would be premature.

Requirements:

- Evidence is too sparse, stale, or contradictory.
- No valid bootstrap or election proof exists.
- The node must wait, continue read-only behavior, or stay idle depending on local role.

## Why this is different from the current `DcsTrust` boundary

Today the main boundary is roughly:

```text
if dcs_trust != FullQuorum:
    degraded path
else if lease exists:
    lease-holder or follower path
else:
    best_failover_candidate(...)
```

This option changes the logic to:

```text
build vote matrix
derive proof candidates
select strongest valid proof
derive local contract from that proof
```

That is materially different because:

- `Degraded` can still yield `MajorityContinue` or `MajorityElect` if the matrix proves enough healthy members exist.
- `FullQuorum` alone is no longer sufficient if the visible rows contradict each other or reveal stale authority.
- election selection is proof-based first, candidate ranking second.
- restart-time decisions become explicit proof questions rather than accidental fall-through cases.

## Typed state machine

The state machine should be centered on proof resolution rather than role names.

### Proposed top-level ADT

```text
HaProofState
  - GatheringEvidence
  - ProofResolved(AuthorityProof)
  - ContractLowered(NodeContract)
  - IntentPublished(IntentRevision)
```

### Proposed node contracts

```text
NodeContract
  - ContinueLeader(ContinueLeaderContract)
  - FollowLeader(FollowLeaderContract)
  - ElectSelf(ElectSelfContract)
  - YieldToPeer(YieldToPeerContract)
  - BootstrapCluster(BootstrapClusterContract)
  - RepairAndRejoin(RepairAndRejoinContract)
  - FenceAndDemote(FenceAndDemoteContract)
  - Wait(WaitContract)
```

### Key invariants

- No `ContinueLeader` contract may exist without a `ContinueProof`.
- No `ElectSelf` contract may exist without an `ElectionProof` naming self.
- No `BootstrapCluster` contract may exist without a `BootstrapProof`.
- No `FollowLeader` or `RepairAndRejoin` contract may name a source leader that is not part of the winning proof.
- No process-dispatch path may re-resolve leader identity after proof resolution.
- Contradictions in the matrix must be preserved as typed facts until a proof explicitly overrides them.

### Important transitions

```text
GatheringEvidence
    -> ProofResolved(MajorityContinue)
    -> ProofResolved(MajorityElect)
    -> ProofResolved(BootstrapAuthorized)
    -> ProofResolved(FailSafeRequired)
    -> ProofResolved(WaitForEvidence)

ProofResolved(...)
    -> ContractLowered(ContinueLeader | FollowLeader | ElectSelf | YieldToPeer | BootstrapCluster | RepairAndRejoin | FenceAndDemote | Wait)

ContractLowered(...)
    -> IntentPublished(...)

IntentPublished(...)
    -> next tick returns to GatheringEvidence
```

## Quorum model redesign

### Core rule

The system should reason in terms of `majority proof` instead of `full quorum only`.

### Proposed quorum classes

```text
AuthorityQuorum
  - FullVisibleMajority
  - DegradedVisibleMajority
  - MinorityOnly
  - NoReliableMajority
```

Meaning:

- `FullVisibleMajority`
  All expected members are visible or enough evidence exists to treat visibility as complete.
- `DegradedVisibleMajority`
  A legal majority is visible and internally corroborating, but one or more members are absent.
- `MinorityOnly`
  The local node can see too few members to prove authority.
- `NoReliableMajority`
  Raw count might look sufficient, but freshness or contradictions invalidate authority proof.

### Why degraded-majority must continue

A three-node cluster with two healthy members is still a majority. If those two nodes:

- agree on visible membership,
- agree that the old primary is absent or minority-isolated,
- agree on the same best WAL-backed candidate,
- and do not see a stronger conflicting majority lineage,

then the system should be able to produce `AuthorityProof::MajorityContinue` or `AuthorityProof::MajorityElect`. It should not be forced into the current "non-full quorum means degraded branch" simplification.

### When fail-safe is still required

Fail-safe is still mandatory when:

- the local node is primary but only a minority is visible,
- store health is insufficient and no majority evidence can be recovered,
- a conflicting leader lineage is visible and the local node cannot prove it still owns majority authority,
- fence cutoff semantics require write rejection after lease-loss or authority blackout.

## Lease model redesign

The lease remains important, but it becomes one column in the vote matrix, not the entire authority story.

### Lease principles

- A lease claim is evidence of authority, not authority by itself.
- A lease generation is valid only if enough fresh rows corroborate the holder's continued legitimacy.
- Lease loss must be interpreted alongside WAL evidence, member visibility, and contradiction evidence.
- A killed primary loses authority when the matrix can no longer produce a continuing majority proof for it.

### Proposed lease-related types

```text
LeaseClaimEvidence
  - NoLeaseObserved
  - LeaseObserved { holder, generation, freshness }
  - LeaseClaimContradicted { holder, generation, contradiction }

LeaseContinuationRule
  - ContinueLeaderIfMajorityCorroborates
  - FenceIfMinorityOrContradicted
  - ReElectIfLeaseGoneButMajorityStillExists
```

### Interaction with startup and failover

- On startup, a stored or observed lease is only one input to the matrix.
- If the lease holder is absent but a majority can corroborate a new candidate, the cluster may elect without pretending the old lease is still authoritative forever.
- If an old primary restarts after the majority has elected a new leader, its local row should resolve to `RepairAndRejoin`, not to ambiguous "leader maybe" behavior.

## Startup reasoning

Startup must become "first matrix build" rather than a special planner.

### Startup cases that the matrix must distinguish

#### Cluster already up

Evidence:

- majority rows visible,
- existing leader lineage corroborated,
- self may be offline or replica.

Result:

- `ContinueProof` for the active leader,
- self gets `FollowLeader` or `Wait` depending on local readiness.

#### Cluster leader already present but local node was offline

Evidence:

- peer rows show one valid leader proof,
- local data dir exists,
- local pg is offline or stale.

Result:

- `RepairAndRejoin` or `FollowLeader` contract.

#### Full-cluster restart with two nodes returning first

Evidence:

- no current primary visible,
- lease may be absent or stale,
- two rows form a legal majority,
- WAL evidence shows which member has the strongest continuity claim.

Result:

- `ElectionProof` for the highest-proof candidate,
- one node gets `ElectSelf`,
- the other gets `YieldToPeer` or `FollowLeader`,
- final returning node later gets `RepairAndRejoin`.

#### Empty cluster bootstrap

Evidence:

- no credible prior leader lineage,
- init-lock allows bootstrap,
- data dirs are missing or bootstrap-empty,
- no member advertises recoverable prior cluster state.

Result:

- `BootstrapProof`,
- only authorized bootstrap owner may initialize.

#### Existing pgdata when init lock is held

Evidence:

- init lock exists,
- local data dir is initialized,
- matrix shows whether that data is part of a real prior history or disposable bootstrap residue.

Result:

- if data is consistent with prior cluster proof, rejoin instead of wiping;
- if data is bootstrap-empty and no prior history exists, bootstrap may continue;
- if data is conflicting and unrecoverable, transition to repair path.

## ASCII diagram: matrix-driven authority resolution

```text
             +---------------------------+
             |   ObservationEnvelope     |
             | pg + dcs + process + cfg  |
             +-------------+-------------+
                           |
                           v
             +---------------------------+
             |       VoteMatrix          |
             | row/member, column/fact   |
             +-------------+-------------+
                           |
         +-----------------+------------------+
         |                 |                  |
         v                 v                  v
 +---------------+ +---------------+ +----------------+
 | ContinueProof | | ElectionProof | | BootstrapProof |
 +-------+-------+ +-------+-------+ +--------+-------+
         |                 |                  |
         +--------+--------+------------------+
                  |
                  v
         +----------------------+
         |     NodeContract     |
         +----------+-----------+
                    |
                    v
         +----------------------+
         | Receiver-owned intent|
         +----------------------+
```

## Replica convergence redesign

Replica convergence should be represented as a proof-selected repair contract, not as late dispatch heuristics.

### One coherent convergence sequence

Once a proof names the authoritative leader, every non-leader node should enter one of these ordered paths:

1. `HealthyFollow`
   Local data already matches the winning lineage and upstream.
2. `LagTolerantCatchUp`
   Local data is behind but compatible.
3. `RewindRequired`
   Local data diverged on timeline but rewind is still possible.
4. `BasebackupRequired`
   Rewind is impossible or already failed.
5. `BlockedRepair`
   Required source exists but local or remote prerequisites are missing.

### Proposed convergence ADT

```text
RepairPlan
  - None
  - Follow { leader }
  - CatchUp { leader }
  - Rewind { leader, source }
  - Basebackup { leader, source }
  - WaitForRepairPrereq { reason }
```

### Important rule

`src/ha/process_dispatch.rs` must not rediscover source leader through DCS at dispatch time. The winning proof must already have named the leader and the repair plan source.

## Partial-truth publication

This option depends on richer member publication, even though publication is not the primary organizing idea here.

### Publication rule

If `pgtuskmaster` is alive but pginfo is degraded, the node must still publish:

- that the member exists,
- whether API is reachable,
- whether SQL status is `Unknown`, `Healthy`, or `Unreachable`,
- last known timeline if any,
- last known replay or commit position if any,
- local readiness,
- any local lease or bootstrap claim visible to this node,
- freshness timestamp.

Silence is worse than partial truth. The vote matrix can down-rank weak evidence, but it cannot reason about omitted evidence.

### Proposed publication envelope

```text
PublishedMemberEvidence
  - member_id
  - api_visibility
  - pg_observation
  - observation_freshness
  - process_liveness_hint
  - lease_claim_hint
  - bootstrap_claim_hint
```

## Deduplication and receiver ownership

This option keeps the requirement from the task context: deduplication must move out of sender-side HA logic.

### What changes

- The vote matrix and authority proof produce an `IntentRevision`.
- The sender no longer defines final process-job identity from `(tick, action_index, action)`.
- Process and DCS consumers persist or derive receiver-owned application ledgers keyed by proof-backed intent revision.

### Why this is safer

The authority proof is stable for as long as the matrix meaning remains stable. Tick-local sender ids are not. A receiver-owned ledger can safely answer:

- this promotion intent is already satisfied,
- this basebackup intent is still in progress,
- this demotion was superseded by a new proof revision,
- this command belongs to an older majority proof and must not be replayed.

## Concrete future code areas and types to change

A later implementation for this option would need to touch at least these areas:

- `src/runtime/node.rs`
  Startup wiring must stop implying a separate conceptual startup planner and instead feed the first proof tick cleanly.
- `src/ha/worker.rs`
  Replace current `GlobalKnowledge` compression with matrix building and proof resolution.
- `src/ha/decide.rs`
  Replace `DcsTrust` gating plus `best_failover_candidate(...)` with matrix-to-proof logic.
- `src/ha/reconcile.rs`
  Reconcile from `NodeContract` instead of directly from current role goals.
- `src/ha/decision.rs`
  Likely becomes the natural home for `AuthorityProof`, `VoteMatrix`, contradiction types, and proof ranking rules.
- `src/ha/lower.rs`
  Lower proof-backed contracts into receiver-owned intents and effect plans.
- `src/ha/process_dispatch.rs`
  Remove authority rediscovery and sender-owned job identity; accept fully typed repair/start/demote intents.
- `src/dcs/state.rs`
  Expand publication types so evidence rows retain freshness, lease-claim, and contradiction-relevant data.
- `src/dcs/worker.rs`
  Publish the richer evidence envelope every tick and preserve partial truth.
- `src/pginfo/state.rs`
  Potentially extend local evidence surfaces used by the matrix without collapsing to absence.
- `tests/ha.rs` and `tests/ha/features/**`
  Later implementation must verify that proof-based majority semantics satisfy the HA feature corpus.

## Detailed list of meaningful implementation changes for a later task

The later implementation would need to make all of these changes, explicitly and without leaving stale legacy paths behind:

- Introduce `VoteMatrix`, `VoteRow`, `MatrixContradiction`, `AuthorityProof`, and proof-detail structs.
- Remove direct top-level dependence on `DcsTrust::FullQuorum` as the decisive HA branch point.
- Delete or demote `best_failover_candidate(...)` as the primary leader-election center.
- Replace `PeerKnowledge` as the main election abstraction with richer row-level evidence.
- Carry freshness and contradiction data explicitly through the pure decision path.
- Encode degraded-majority continuation and degraded-majority election as first-class proof outcomes.
- Encode minority isolation and authority blackout as distinct proof failures.
- Move source-leader selection for follow, rewind, and basebackup into proof resolution or contract lowering.
- Change reconciliation to operate on proof-derived contracts, not directly on current desired role alone.
- Introduce receiver-owned intent revision ids and delete sender-owned final job-id semantics from HA.
- Preserve partial-truth member publication even when pginfo cannot deliver a fully healthy local view.
- Rework bootstrap authorization so it is derived from matrix evidence and init-lock proof, not from ad hoc empty-cluster assumptions.
- Rework switchover handling so a requested target is evaluated inside the same proof system rather than beside it.
- Remove stale paths that still imply startup is special or that dispatch may reconstruct authority late.

## Migration sketch

A later implementation could migrate in this order:

1. Introduce `VoteRow` and `VoteMatrix` as an additional pure layer while preserving current behavior.
2. Teach `src/dcs/worker.rs` and `src/dcs/state.rs` to publish richer evidence without changing HA actions yet.
3. Build proof diagnostics alongside existing `DesiredState` selection and compare them in logs or tests.
4. Replace `best_failover_candidate(...)` and `decide_degraded(...)` with proof selection once parity is understood.
5. Shift reconciliation from role-goal driven to contract driven.
6. Replace sender-owned process ids with proof-backed intent revisions and receiver-owned ledgers.
7. Delete stale ranking helpers and authority shortcuts once proof logic is fully canonical.

This order matters because it lets the team validate evidence richness before changing failover semantics.

## Non-goals

- This option does not propose implementing the redesign in this task.
- This option does not claim that leases are unimportant; it claims they are insufficient as the only top-level authority primitive.
- This option does not require turning the pure decider into an effectful subsystem.
- This option does not require reading `docs/` or depending on prior chat history.
- This option does not treat green tests as the goal of this task; test outcomes are evidence inputs only.

## Tradeoffs

- The proof system is more verbose than the current ranking helper approach.
- Engineers will need to reason about contradictions explicitly instead of relying on a small number of branch cases.
- Publication payloads may become larger because they must preserve vote-relevant partial truth.
- Debugging will improve if proof objects are logged well, but worsen if they are implemented opaquely.
- This option may feel heavier than the regime-first or lease-epoch options because it chooses explicit evidence accounting over conceptual elegance.

## Logical feature-test verification

This section explains how a later implementation of this option would logically satisfy the key HA scenarios without implementing code in this task.

### `ha_dcs_quorum_lost_enters_failsafe`

When a DCS quorum majority is stopped, the matrix loses enough fresh rows that no majority authority proof survives. A running primary therefore resolves to `FailSafeProof`, publishes no operator-visible primary, and transitions to `FenceAndDemote` or equivalent fail-safe behavior. This remains safe and consistent with the current acceptance intent.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

The moment a formerly valid leader loses its majority proof, the matrix resolves a `FailSafeProof` with a fence cutoff tied to the local committed WAL. That cutoff becomes part of the publication and effect-lowering path so post-cutoff writes are rejected while pre-cutoff commits remain valid evidence.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The isolated old primary can no longer form a majority proof. The two-node majority can. Their matrix rows corroborate a new election candidate, so they produce `ElectionProof` for the majority winner. The minority old primary cannot produce an equivalent proof and therefore must not remain operator-accepted as primary.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

After healing, the old primary's local row conflicts with the majority-backed lineage. The matrix resolves that contradiction in favor of the already established majority proof, so the healed node receives `RepairAndRejoin` rather than any leadership contract. Rewind or basebackup is then selected by the repair plan.

### `ha_primary_killed_then_rejoins_as_replica`

Once the original primary is down, the surviving majority builds an `ElectionProof` for a replacement. When the killed node returns, its row is no longer part of the winning proof, so it cannot resume leadership. It must rejoin under the current authority proof as a replica.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When only the primary remains, no majority proof exists for continued cluster authority. Once one healthy replica restarts, the matrix regains a degraded but valid majority. That permits `ContinueProof` for the primary or a valid election flow if the old primary is absent. This is exactly the kind of case the current `FullQuorum` boundary compresses too harshly.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the most direct motivation for the option. Two restarted fixed nodes should be able to build a degraded-majority matrix, compare WAL evidence, and elect exactly one primary through `ElectionProof`, even before the third node returns. The final node then sees the established proof and enters `RepairAndRejoin`.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The proof layer names the leader and expected lineage. The repair contract first selects `RewindRequired` when divergence is repairable. If process feedback marks rewind as impossible or already failed, the receiver-owned repair ledger advances to `BasebackupRequired` without destabilizing cluster authority.

### `ha_replica_stopped_primary_stays_primary`

A single replica loss in a three-node cluster still leaves a majority with the primary plus one healthy replica. The matrix therefore keeps a `ContinueProof` for the existing leader rather than treating the situation as generic degraded uncertainty.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

The broken rejoining replica contributes weak or contradictory evidence but not a valid competing majority proof. The healthy majority continues to corroborate the existing leader, so the broken node's repair failure remains localized and does not destabilize primary authority.

## Q1 Should matrix rows represent direct evidence only, or also inferred evidence?

Context:
The more inferred evidence the matrix allows, the more expressive it becomes. For example, the system might infer "member is probably stale" from timestamps, "leader likely dead" from lease expiration plus missing member slot, or "timeline divergence probable" from replay WAL mismatch.

Problem:
If inference is embedded directly into rows, proof resolution gets richer but harder to audit. If rows are limited to direct evidence only, proof resolution becomes clearer but may need more passes and more helper types.

Restated question:
Should `VoteRow` contain only directly observed facts, with inference moved into proof derivation, or should the matrix itself store both raw and inferred evidence columns?

## Q2 How should cluster size be determined during partial visibility?

Context:
A majority proof depends on cluster size. During outages or reconfiguration-like futures, the system may know the configured cluster size, the currently visible DCS membership count, and a historically observed count.

Problem:
If cluster size is taken only from current visibility, a two-node minority could accidentally treat itself as complete. If cluster size is taken only from static config, genuine bootstrap or one-node deployments can become awkward.

Restated question:
What is the authoritative source for majority math inside the matrix, and how should the system behave when configured size, visible size, and historical size disagree?

## Q3 Should lease claims and WAL evidence have separate proof priority ladders?

Context:
Sometimes lease generation is clean but WAL evidence is sparse. Other times WAL evidence is strong while lease evidence is stale or absent, especially after full-cluster restarts.

Problem:
A single total ordering may be too crude. The system may need to say "lease evidence wins for continuity proofs, but WAL evidence wins for restart elections once lease continuity is broken."

Restated question:
Should `ResolveAuthorityProof` use one unified priority ladder for all evidence, or distinct priority ladders for continuity, failover, restart, and bootstrap proofs?

## Q4 How much contradiction should force immediate wait instead of best-effort election?

Context:
The matrix will inevitably see contradictions: two members may both claim primary, one row may be stale, one node may publish `Unknown`, and a healed old primary may briefly advertise outdated lineage.

Problem:
If every contradiction forces `WaitForEvidence`, recovery may become too slow. If contradictions are tolerated too aggressively, the system risks electing on stale or ambiguous inputs.

Restated question:
Which contradiction classes should be automatically survivable inside a majority proof, and which must always block election or continuation?

## Q5 Should the matrix itself include process feedback, or should process feedback only affect repair contracts?

Context:
Process feedback such as "rewind failed," "basebackup blocked," or "promote succeeded" strongly affects what the node should do next. The current code already observes process state in `src/ha/worker.rs`.

Problem:
Including process feedback directly inside vote rows could make authority proofs sensitive to local job state in surprising ways. Excluding it entirely could make repair reasoning feel artificially detached from the proof pipeline.

Restated question:
Should local process outcomes participate directly in `VoteMatrix` authority proof derivation, or should they only refine the already-selected `NodeContract` and `RepairPlan` after authority has been proved?
