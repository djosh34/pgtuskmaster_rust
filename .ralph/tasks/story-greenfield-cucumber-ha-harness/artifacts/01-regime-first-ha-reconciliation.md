# HA Refactor Option 1: Regime-First Reconciliation

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history or repo documentation.

## Why this option exists

This option exists because the HA system still has one conceptual gap even in the newer typed code: it moves quickly from observations to local desired role, while startup, authority evaluation, lease meaning, and recovery intent are still spread across separate seams. The differentiator for this option is that it inserts one explicit typed middle layer called a `ClusterRegime`. Every tick, including the first startup tick, must first answer "what cluster regime are we in?" before it answers "what should this node do?" That is different from options centered on command journals, bootstrap-only funnels, or receiver-led idempotency as the primary organizing concept.

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 1.

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

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That result matters for this study because it shows the repo is no longer in the originally expected "red by default" state, so the redesign must be justified on architecture clarity, not on pretending today's tree is broadly broken.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out waiting for one primary across the two restarted fixed nodes and reported `sampled_relevant=node-b:unknown, node-a:unknown`. That is directly relevant to this option because it points at restart-time authority classification and startup/rejoin ambiguity after a full-cluster outage.
- `tests/ha.rs` is still the acceptance contract surface that a later implementation must preserve or intentionally revise with explicit reasoning.

## Current design problems

### 1. Startup reasoning is still architecturally separate from steady-state reasoning

In the live runtime path, `run_node_from_config(...)` in `src/runtime/node.rs` validates config, derives process defaults, prepares paths, and then enters `run_workers(...)`, which wires and launches the long-running workers. The old explicit startup planner/executor names still exist only as stale references inside a disabled block, not as the live boot path. That means the repo has improved since the original task text, but the architectural complaint still stands: startup is still something the runtime does before the HA loop has established a first-class authority model. The user wanted "same newest info + same state => same actions" on the first tick as well as on later ticks, and the current runtime/worker split still does not express that principle directly.

### 2. HA reasoning is still spread across multiple "almost central" modules

The current tree is cleaner than the older task context, but it still distributes responsibility across `src/ha/worker.rs`, `src/ha/decide.rs`, `src/ha/reconcile.rs`, and `src/ha/process_dispatch.rs`. `step_once(...)` in `src/ha/worker.rs` observes, decides, reconciles, publishes, and executes. `decide(...)` in `src/ha/decide.rs` derives a `DesiredState`, `reconcile(...)` in `src/ha/reconcile.rs` turns that into `ReconcileAction`s, and `dispatch_process_action(...)` plus `start_intent_from_dcs(...)` in `src/ha/process_dispatch.rs` decide how those actions become concrete process jobs. On top of that, the repo also contains a second typed decision/effect model in `src/ha/decision.rs` and `src/ha/lower.rs` built around `DecisionFacts`, `HaDecision`, and `HaEffectPlan`, but the live worker does not drive that path. This is not untyped chaos, but it still leaves future engineers asking where cluster authority ends and local command planning begins, and which typed model is supposed to become canonical.

### 3. Sender-owned delivery identity still exists and must be pushed fully to receivers

The original task motivation called out sender-side dedup logic in an older `src/ha/worker.rs` shape. The current tree no longer exposes the same skip helper, which is good, but the sender still constructs deterministic process identities through `process_job_id(...)` in `src/ha/process_dispatch.rs` using `(scope, member, tick, action_index, action)`. That means the sender still owns the logical identity of delivered work. This is better than heuristic skipping, but it is still not the explicit receiver-owned idempotency contract the user asked for. This option completes that move by making the sender publish `IntentId`s while the process and DCS consumers become the sole owners of duplicate suppression.

### 4. The quorum and fail-safe boundary is still too compressed

In the current tree, `decide(...)` immediately routes all non-`DcsTrust::FullQuorum` states through `decide_degraded(...)`. That already distinguishes several local outcomes, but it still compresses multiple materially different situations into a single degraded branch. A three-node cluster with two healthy members and readable DCS authority is not the same problem as "the store is unavailable and no authoritative majority is knowable." The redesign must explicitly encode the difference between valid degraded-majority operation and truly unauthoritative operation.

### 5. Startup and rejoin logic still crosses a dangerous boundary in process dispatch

`src/ha/process_dispatch.rs` still contains `start_intent_from_dcs(...)`, `validate_basebackup_source(...)`, `validate_rewind_source(...)`, and `resolve_source_member(...)`. Those are reasonable helpers, but they mean the command-lowering layer still reconstructs significant topology and startup intent from DCS state at dispatch time. That is precisely the sort of boundary drift the user objected to: startup, rejoin, and authoritative source selection should be decided earlier as typed intent, not rediscovered late by a dispatcher.

### 6. Partial truth publication is richer than before, but not yet treated as a foundational contract

`src/dcs/worker.rs` writes local member state through `build_local_member_slot(...)`, and `src/dcs/state.rs` now models `MemberSlot`, `MemberPostgresView`, `LeaderLeaseRecord`, `SwitchoverIntentRecord`, and `DcsTrust`. `src/pginfo/state.rs` also preserves partial truth via `PgInfoState::{Unknown,Primary,Replica}` and `Readiness::{Unknown,Ready,NotReady}`. That is good progress. The remaining problem is conceptual: partial truth is still treated as one input to HA, not as the central publication contract that HA must respect. This option re-centers the design so member publication is not just "latest snapshot materialized to etcd" but a typed statement of observed truth, confidence, and local capability.

## The central proposal

Replace the current "world -> desired role -> reconcile actions" center with a stricter three-stage pipeline:

1. `Observe`: collect the newest local and global evidence into a single immutable `ObservationEnvelope`.
2. `ClassifyRegime`: derive a typed `ClusterRegime` that explains what authority exists, who may lead, and what kind of recovery is valid.
3. `ResolveContract`: derive a `NodeContract` for this member from the regime and the local capability model.
4. `LowerContract`: translate the contract into idempotent consumer actions with stable intent ids.

The crucial difference is that the pure decider no longer jumps straight from facts to a local role. It must first establish the cluster regime. That forces startup, failover, fencing, degraded-majority continuation, and bootstrap to use one authority model.

## Control flow from startup through steady state

### High-level method

At process boot:

1. Start the observation workers exactly as today: DCS watcher, pginfo worker, process worker, config subscriber, and local filesystem inspection support.
2. Do not introduce or preserve any startup-only planning path outside the unified HA classifier.
3. Run the HA reconciler immediately with the same loop used later for steady state.
4. The first tick sees whatever observations are currently available. If evidence is incomplete, the decider yields a `NodeContract::Wait(...)` or `NodeContract::PublishOnly(...)`, not an ad hoc startup branch.
5. Once sufficient evidence exists, the decider returns a contract that may lead to `AcquireLease`, `StartReplica`, `InitDb`, `BaseBackup`, `PgRewind`, `Promote`, `FenceAndDemote`, or "do nothing."

During steady state:

1. Every subscription change or interval tick produces a new `ObservationEnvelope`.
2. The regime classifier derives a fresh `ClusterRegime`.
3. The contract resolver compares the new regime plus the local capability to the last acknowledged contract.
4. The lowerer emits actions with deterministic `IntentId`s.
5. Receivers apply idempotently by `IntentId`, not by guessing whether a command seems redundant.

### ASCII diagram

```text
                    startup tick, steady-state tick, or subscription change
                                         |
                                         v
                            +---------------------------+
                            | ObservationEnvelope       |
                            | - dcs snapshot            |
                            | - pginfo snapshot         |
                            | - process snapshot        |
                            | - data dir evidence       |
                            | - storage evidence        |
                            +-------------+-------------+
                                          |
                                          v
                            +---------------------------+
                            | ClusterRegime             |
                            | authoritative?            |
                            | majority valid?           |
                            | leader known?             |
                            | bootstrap open?           |
                            | switchover active?        |
                            +-------------+-------------+
                                          |
                                          v
                            +---------------------------+
                            | NodeContract              |
                            | lead / follow / recover   |
                            | fence / wait / bootstrap  |
                            +-------------+-------------+
                                          |
                                          v
                            +---------------------------+
                            | LowerContract             |
                            | IntentId + action list    |
                            +------+------+-------------+
                                   |      |
                     +-------------+      +------------------+
                     v                                     v
            +-------------------+                 +-------------------+
            | DCS consumer      |                 | Process consumer  |
            | idempotent by id  |                 | idempotent by id  |
            +-------------------+                 +-------------------+
```

## Proposed typed state machine

### Core top-level ADTs

The redesign revolves around these new types.

```text
ObservationEnvelope
  = {
      observed_at,
      dcs: DcsObservation,
      local: LocalObservation,
      prior_contract: Option<NodeContractSummary>,
    }

ClusterRegime
  = NoAuthority(NoAuthorityReason)
  | MajorityAuthority(MajorityRegime)
  | BootstrapAuthority(BootstrapRegime)
  | SwitchoverAuthority(SwitchoverRegime)

NodeContract
  = Wait(WaitContract)
  | PublishOnly(PublishContract)
  | Lead(LeadContract)
  | Follow(FollowContract)
  | Recover(RecoverContract)
  | Fence(FenceContract)
  | Demote(DemoteContract)

IntentEnvelope
  = {
      intent_id,
      regime_hash,
      contract,
      dcs_actions,
      process_actions,
      publication_projection,
    }
```

### Observation ADTs

`ObservationEnvelope` must be richer than the current `WorldView`.

- `DcsObservation`
  - `store_health: StoreHealth`
  - `membership: MembershipView`
  - `lease: LeaseView`
  - `switchover: SwitchoverView`
  - `init_lock: InitLockView`
  - `published_self: Option<MemberPublication>`
- `LocalObservation`
  - `postgres: PostgresObservation`
  - `process: ProcessObservation`
  - `data_dir: DataDirObservation`
  - `storage: StorageObservation`
  - `required_roles: RolesObservation`
  - `service_liveness: SupervisorObservation`

This explicitly separates what was observed locally from what DCS says globally. That matters because "pginfo failed but the supervisor is alive" is still publishable truth.

### Cluster regime ADT

`ClusterRegime` is the center of this option.

#### `NoAuthority(NoAuthorityReason)`

Use when no authoritative majority decision can safely be made.

- `StoreUnavailable`
- `MembershipTooSparse`
- `ConflictingAuthority`
- `BootstrapUnknown`

Invariant:

- No node may newly claim leadership.
- Existing primary authority must be converted into a fencing or demotion contract once lease guarantees expire or authority is disproven.

#### `MajorityAuthority(MajorityRegime)`

Use when there is enough evidence to continue cluster operation even if not every member is healthy.

`MajorityRegime` contains:

- `quorum: QuorumAssessment`
- `lease: LeaseAuthority`
- `primary_view: PrimaryView`
- `candidate_set: CandidateSet`
- `degraded_mode: Option<DegradedMode>`

`DegradedMode` examples:

- `TwoOfThreeHealthy`
- `LeaderObservedButOneReplicaMissing`
- `RejoiningMinorityFormerPrimary`

Invariant:

- Degraded majority does not imply fail-safe by itself.
- A node may lead, continue following, or compete for leadership if and only if majority authority still exists.

#### `BootstrapAuthority(BootstrapRegime)`

Use when the cluster has not yet established an authoritative primary and bootstrap may be legal.

`BootstrapRegime` contains:

- `init_lock: InitLockStatus`
- `existing_members: ExistingMemberEvidence`
- `bootstrap_window: BootstrapWindow`

Invariant:

- Bootstrap is only legal when no authoritative leader exists and no reusable leader evidence exists.
- Existing local `pgdata` must be classified before bootstrap is allowed.

#### `SwitchoverAuthority(SwitchoverRegime)`

Use when a switchover request is active and majority authority exists.

`SwitchoverRegime` contains:

- `request: SwitchoverRequestView`
- `current_leader: LeaseEpoch`
- `target: TargetEvaluation`
- `handoff_stage: SwitchoverStage`

Invariant:

- Switchover is a contractually guided handoff, not a generic failover.
- The current primary stays authoritative until the contract transitions into a demotion-ready stage.

### Local capability ADT

The resolver needs a stronger local capability model than today's mix of `DataDirState`, `PostgresState`, and `ProcessState`.

`LocalCapability` should be derived from local observations:

- `EmptyNode`
- `PrimaryResumeCapable`
- `ReplicaResumeCapable { upstream_hint }`
- `DivergedRewindCapable { last_known_timeline }`
- `CloneRequired`
- `RunningPrimary`
- `RunningReplica { upstream }`
- `OfflineButQueryableHistory`
- `FencingInProgress`
- `DemotionInProgress`
- `RecoveryJobActive(RecoveryJobKind)`

Invariant:

- Impossible combinations are excluded by construction.
- For example, "running primary and empty data dir" is not representable.

### Contract ADTs

#### `LeadContract`

Contains:

- `epoch: LeaseEpoch`
- `startup_mode: LeadStartupMode`
- `publication: AuthorityPublication`
- `roles_requirement: RolesRequirement`

Variants of `LeadStartupMode`:

- `AlreadyPrimary`
- `StartPrimary`
- `PromoteReplica`
- `BootstrapPrimary`

#### `FollowContract`

Contains:

- `leader: MemberId`
- `publication: AuthorityPublication`
- `convergence: ConvergencePath`

`ConvergencePath`:

- `AlreadyFollowing`
- `StartStreaming`
- `RewindThenStart`
- `CloneThenStart`

#### `RecoverContract`

Contains:

- `reason: RecoveryReason`
- `source: RecoverySource`
- `publication: PublicationContract`

Use when the node is not yet safely following but recovery work can start.

#### `FenceContract`

Contains:

- `reason: FenceReason`
- `cutoff: Option<FenceCutoff>`
- `lease_release: LeaseReleasePlan`
- `publication: PublicationContract`

This separates "must fence because authority is disproven" from "must wait because authority is unknown."

#### `WaitContract`

Contains:

- `reason: WaitReason`
- `publication: PublicationContract`

Examples:

- `AwaitFreshObservation`
- `AwaitLeaseExpiry`
- `AwaitTargetDemotion`
- `AwaitBootstrapEvidence`

## Transition rules

### Startup transitions

1. `ObservationEnvelope` with empty DCS, empty data dir, and no init lock:
   Move to `BootstrapAuthority`.
2. `ObservationEnvelope` with authoritative leader and empty local data dir:
   Move to `MajorityAuthority` then resolve to `RecoverContract` with `CloneThenStart`.
3. `ObservationEnvelope` with local managed replica state and authoritative leader:
   Move to `MajorityAuthority` then resolve to `FollowContract(StartStreaming)` or `RecoverContract(RewindThenStart)` depending divergence.
4. `ObservationEnvelope` with existing local primary-capable data and authoritative self lease:
   Move to `MajorityAuthority` then resolve to `LeadContract`.
5. `ObservationEnvelope` with existing local data but no authoritative majority:
   Move to `NoAuthority` and resolve to `WaitContract` or `FenceContract`, never to bootstrap.

### Steady-state transitions

1. `MajorityAuthority` plus held lease plus local primary healthy:
   Stay in `LeadContract(AlreadyPrimary)`.
2. `MajorityAuthority` plus peer lease plus local replica healthy:
   Stay in `FollowContract(AlreadyFollowing)`.
3. `MajorityAuthority` plus no lease plus best candidate self:
   Resolve to `LeadContract` through an `AcquireLease` action first.
4. `MajorityAuthority` plus local old primary in minority after heal:
   Resolve to `RecoverContract(RewindThenStart)` or `RecoverContract(CloneThenStart)`.
5. `NoAuthority(StoreUnavailable)` plus local primary:
   Resolve to `FenceContract` once the node cannot continue to justify authority.

### Switchover transitions

1. Switchover request appears while leader is healthy and majority authority exists:
   Move to `SwitchoverAuthority`.
2. Validate target eligibility inside the pure classifier.
3. If target invalid, resolve to `PublishOnly` with `SwitchoverRejected(...)`.
4. If target valid, current leader resolves to `DemoteContract`, target resolves to `LeadContract(TargetedSwitchover)`.
5. Clear switchover only after the target has acknowledged leadership and publication has converged.

## Redesigned quorum model

The redesign deletes the idea that `DcsTrust` alone is enough to describe authority. Replace it with:

- `StoreHealth`
  - `Readable`
  - `Unavailable`
- `MembershipQuorum`
  - `AuthoritativeMajority`
  - `VisibleButNonMajority`
  - `TooSparse`
- `LeaseAuthority`
  - `HeldBy(MemberId, LeaseEpoch)`
  - `UnheldButElectable`
  - `Unknown`

This creates the following rules:

1. A three-node cluster with two members visible and a readable store is `AuthoritativeMajority`, not fail-safe by default.
2. A readable store with only one visible member in a three-node cluster is `VisibleButNonMajority`; failover is not allowed.
3. An unreadable store is `Unavailable`; publication must say no authoritative primary even if local Postgres is still running.
4. Leadership re-election in `2-of-3` cases is allowed when:
   - the lease is unheld or expired,
   - the surviving majority can still observe each other,
   - a candidate is eligible by readiness, WAL state, and reachability.
5. The old primary isolated into the minority is never accepted as authoritative once the majority has elected a new leader.

This is stricter and clearer than a single degraded branch. It preserves the user's requirement that degraded-but-valid majority operation must continue.

## Lease model

This option strengthens lease meaning substantially.

### Lease rules

1. A leader is authoritative only while both of these are true:
   - the lease epoch is valid and owned by that member;
   - the regime still says the majority can authorize that leadership.
2. A killed primary loses authority because lease renewal stops and the majority will eventually observe an expired or absent lease.
3. A partitioned old primary loses authority because the healed majority will establish a newer lease epoch held by someone else.
4. Lease loss does not directly imply immediate process demotion in every case. It implies a new regime classification. The resulting contract may be:
   - `FenceContract(lease disproven)`
   - `WaitContract(awaiting fresh confirmation)`
   - `FollowContract` if a foreign leader is authoritative

### Lease epochs and publication

Publication must include the lease epoch when advertising a primary. That makes the operator-visible primary and the fence cutoff reasoning align.

### Lease interaction with startup

Startup never bypasses lease reasoning. A node with existing `pgdata` must still observe the lease regime and classify itself accordingly:

- held-by-self plus authoritative majority -> resume leadership
- held-by-peer -> follow or recover toward peer
- unheld with bootstrap window open -> compete or bootstrap
- unavailable authority -> wait or fence, never improvise leadership

## Startup reasoning in this option

Startup is not a separate planner. It is the first observation tick.

### Cluster already up

If DCS shows an authoritative leader and valid member publications, the node does not run a startup-specific selection function. It enters `MajorityAuthority` and resolves into follow, recover, or wait based on local capability.

### Cluster leader already present

If a leader lease exists and the local node is not that leader:

- empty data dir -> clone
- consistent replica data -> start streaming
- diverged data -> rewind if possible, otherwise clone
- still-running former primary -> fence and demote first

### Existing members already published

Member publications are used as evidence for leader choice, peer eligibility, and recovery source selection, but that evidence is consumed by the pure classifier, not by process dispatch.

### Empty versus existing `pgdata`

`DataDirObservation` must distinguish:

- `Missing`
- `EmptyManaged`
- `ManagedReplicaConfigPresent`
- `ManagedPrimaryConfigPresent`
- `ExistingUnknownState`
- `ExistingDivergedState`

This is richer than a single "existing" bucket and prevents bootstrap from being incorrectly chosen over recovery or resume.

### Init lock behavior

`BootstrapAuthority` contains explicit init-lock substates:

- `Open`
- `HeldBySelf`
- `HeldByPeer`
- `HeldButContradictedByLeader`

Rules:

1. `HeldByPeer` blocks bootstrap.
2. `HeldButContradictedByLeader` means the lock is stale and must be cleaned through a controlled path, not hand-waved.
3. Existing local data may still be reusable even if the node wins the init lock later; winning the lock does not erase evidence that another cluster incarnation already exists.

### When existing local data may still be valid for initialization

Almost never. Existing local data may only support initialization if it is explicitly classified as empty bootstrap scaffolding with no authoritative cluster history. If WAL or managed recovery evidence exists, the node must enter resume or recovery logic instead.

## Replica convergence as one coherent path

This option unifies replica repair into `ConvergencePath`.

### `AlreadyFollowing`

Use when the local replica is healthy, the upstream matches the authoritative leader, and the WAL gap is acceptable.

### `StartStreaming`

Use when the node has reusable data and only needs a managed replica start pointed at the leader.

### `RewindThenStart`

Use when the node has diverged data but the divergence is still rewind-compatible.

### `CloneThenStart`

Use when:

- the data dir is missing,
- rewind is impossible,
- rewind has already failed and fallback is required,
- the node won no authority and must fully rejoin from the leader.

This makes previously-primary, previously-replica, and freshly-restored nodes all travel through the same convergence model. The difference is only which `ConvergencePath` variant is selected.

## Partial-truth publication model

This option expands DCS member publication into a proper truth envelope.

### New publication shape

Replace today's effective "routing + postgres view + lease shell" with:

- `MemberPublication`
  - `routing: RoutingView`
  - `service: SupervisorView`
  - `postgres: PostgresEvidence`
  - `capability: CapabilitySummary`
  - `observed_at`
  - `confidence: ObservationConfidence`

`PostgresEvidence` variants:

- `UnknownButSupervisorAlive`
- `PrimaryObserved`
- `ReplicaObserved`
- `ProcessOffline`

This preserves the user's requirement that "pginfo failed but pgtuskmaster is up" remains publishable truth instead of becoming silence.

### Confidence model

Add `ObservationConfidence`:

- `Fresh`
- `StaleWithinLease`
- `StaleBeyondLease`
- `LocallyAssumed`

The classifier may use stale evidence differently from fresh evidence, but DCS publication should still preserve what is known.

## Where deduplication moves

Deduplication moves entirely to receivers and effect consumers.

### New rule

The pure HA layer may emit the same logical intent many times if the regime and contract are unchanged. It is not allowed to suppress an action by reasoning about receiver state.

### Mechanism

Every lowered action carries:

- `IntentId`
- `ContractHash`
- `ActionOrdinal`
- `ConsumerKind`

Consumers persist or remember the last acknowledged `IntentId` they applied:

- the DCS mutation consumer for publication, lease, switchover clear, and init-lock changes
- the process consumer for bootstrap, basebackup, rewind, start, promote, demote

If the same `IntentId` arrives again, the consumer acks it without reapplying. That is safer than sender-side skip logic because:

1. only the receiver knows what was durably applied;
2. retries remain safe across worker restarts;
3. the pure decider stays pure and stateless with respect to delivery mechanics.

## Concrete repo areas a later implementation would touch

- `src/runtime/node.rs`
  Remove startup planner/executor orchestration and make runtime enter the unified HA loop directly.
- `src/ha/worker.rs`
  Replace the current observe-decide-reconcile center with observe-classify-resolve-lower.
- `src/ha/decide.rs`
  Replace `decide(...)` with regime classification plus contract resolution.
- `src/ha/reconcile.rs`
  Replace role-delta reconciliation with contract lowering.
- `src/ha/process_dispatch.rs`
  Remove authority reconstruction from dispatch and accept fully resolved recovery/start contracts.
- `src/ha/types.rs`
  Replace or substantially refactor `WorldView`, `DesiredState`, and `TargetRole` into `ObservationEnvelope`, `ClusterRegime`, and `NodeContract`.
- `src/ha/state.rs`
  Store last acknowledged contract and last applied intent ids per consumer.
- `src/dcs/state.rs`
  Replace `DcsTrust`-centric reasoning with richer authority and publication types.
- `src/dcs/worker.rs`
  Publish the new member truth envelope and expose richer authority evidence to HA.
- `src/pginfo/state.rs`
  Preserve the current partial-truth fidelity while adding any extra capability hints needed by the classifier.
- `tests/ha.rs`
  Keep the same acceptance corpus but potentially add explicit assertions around degraded-majority continuation and richer publication semantics.

## Meaningful changes required by this option

### New types

- `ObservationEnvelope`
- `ClusterRegime`
- `NodeContract`
- `IntentEnvelope`
- `MembershipQuorum`
- `LeaseAuthority`
- `ObservationConfidence`
- `MemberPublication`
- `ConvergencePath`
- `InitLockStatus`

### Deleted or collapsed paths

- dedicated startup planning/execution path in `src/runtime/node.rs`
- late startup intent reconstruction inside `start_intent_from_dcs(...)`
- any sender-side attempt to infer whether a command is redundant

### Responsibility moves

- startup choice moves from runtime into the unified HA classifier
- recovery source selection moves from process dispatch into pure contract resolution
- duplicate suppression moves from HA senders into DCS/process consumers
- publication semantics move from "best effort current snapshot" into an explicit truth contract

### Transition changes

- degraded-majority no longer falls through the same branch as no-authority
- switchover is represented as a special authority regime, not a flag inspected late
- bootstrap becomes legal only through `BootstrapAuthority`, never as a casual fallback

### Lowering boundary changes

- the lowerer receives a fully resolved `NodeContract`
- process dispatch no longer consults DCS to decide whether it is starting primary or replica
- DCS mutation lowering becomes explicit actions, also idempotent by `IntentId`

### DCS publication behavior changes

- partial supervisor truth becomes publishable
- confidence and freshness become explicit fields
- publication separates local observation from cluster authority

### Startup handling changes

- first tick equals startup
- runtime stops doing HA-significant prework
- local filesystem inspection becomes part of observation, not a standalone startup planner

### Convergence handling changes

- all rejoin logic is expressed through one convergence ADT
- rewind/basebackup fallback is modeled as a contract transition, not a dispatch surprise

### Test updates a later implementation would likely need

- unit tests for regime classification exhaustiveness
- unit tests for contract resolution from each regime
- acceptance assertions for continued service in degraded-majority cases
- acceptance assertions for richer partial-truth publication during pginfo degradation
- acceptance assertions for receiver-owned idempotency across repeated identical ticks

## Migration sketch

1. Introduce `ObservationEnvelope` and adapters that can be built from the current subscribers without changing behavior.
2. Add `ClusterRegime` classification in parallel with the current `decide(...)` path and compare outputs in unit tests.
3. Introduce `NodeContract` lowering behind a feature-gated or shadow-tested path.
4. Move source selection out of `src/ha/process_dispatch.rs` so dispatch consumes already resolved contracts.
5. Add receiver-side intent id handling for DCS and process consumers.
6. Delete runtime startup planner functions after the unified first-tick path is proven.
7. Remove obsolete types and helper paths immediately once the new path is active; this project explicitly should not keep legacy control paths around.

## Non-goals

- This option does not attempt to introduce distributed consensus beyond the existing DCS lease model.
- This option does not try to make every observation perfectly fresh before acting; it defines explicit confidence levels instead.
- This option does not move Postgres or etcd IO into the pure decider.
- This option does not preserve startup-specific code paths for backward compatibility.

## Tradeoffs

- The model is more explicit and therefore larger. It introduces more ADTs than the current tree.
- Engineers will need to learn the distinction between `ClusterRegime` and `NodeContract`.
- Consumer-owned idempotency requires durable or semi-durable intent tracking.
- The first implementation will feel heavier than a smaller local fix, but it will make authority and startup behavior far easier to reason about later.

## Logical feature-test verification

### `ha_dcs_quorum_lost_enters_failsafe`

This scenario maps to `NoAuthority(StoreUnavailable)` or `NoAuthority(MembershipTooSparse)` depending what the observation layer sees. The publication contract becomes `NoPrimary`, and any currently running local primary moves toward `FenceContract` once lease authority can no longer be justified. Operator-visible primary disappears even if a local process has not yet fully stopped.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This scenario maps to `FenceContract` with a concrete `FenceCutoff` derived from the last authoritative lease epoch plus local committed WAL evidence. Publication must expose the cutoff so downstream diagnostics can reason about the write boundary.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side remains in `MajorityAuthority(TwoOfThreeHealthy)` and may elect a new leader when the old primary's lease expires or becomes unauthoritative. The isolated old primary falls into `NoAuthority` from its own perspective and must never be accepted as authoritative by the healed cluster.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

After heal, the former primary observes a foreign authoritative lease epoch and resolves to `RecoverContract(RewindThenStart)` or `RecoverContract(CloneThenStart)`. It cannot return to `LeadContract` because the new regime proves a newer authority epoch.

### `ha_primary_killed_then_rejoins_as_replica`

The killed node loses lease authority through lease expiry. The surviving majority remains in `MajorityAuthority` and elects a new leader. When the old node restarts, the first tick sees an authoritative peer leader and resolves to follow/recover, not resume-primary.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

With only one healthy node remaining, authority is not enough for a new election, but the current leader may remain authoritative if the lease remains valid and the store still proves majority semantics for the surviving set. When one replica returns, the regime becomes degraded-majority-valid again and the cluster remains or becomes healthy without unnecessary fail-safe on the majority side.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes classify the cluster based on observed lease absence, init-lock state, and local data capabilities. One becomes leader only when `MajorityAuthority` or `BootstrapAuthority` authorizes it. The third node later resolves into follow/recover and rejoins cleanly. This exact scenario was the lone `make test-long` failure in the March 12, 2026 diagnostic run, which is one reason this option puts so much weight on explicit restart-time regime classification.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

`RecoverContract(RewindThenStart)` carries a typed fallback policy. A rewind failure transitions the next contract to `RecoverContract(CloneThenStart)` instead of relying on implicit dispatcher heuristics.

### `ha_replica_stopped_primary_stays_primary`

A healthy leader under majority authority remains in `LeadContract(AlreadyPrimary)` even if one replica disappears. Replica absence alone is not a reason to demote or publish no-primary.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken rejoin attempt remains a local `RecoverContract` failure on the rejoining node. The healthy majority leader stays in `LeadContract`; a failing recovery path does not mutate cluster authority.

## Recommended future implementation shape

If this option is chosen later, the first implementation task should begin with the pure ADTs and exhaustive classifier tests, not with process-dispatch rewrites. The architectural gain comes from making authority classification explicit first.

## Q1 [Should the regime classifier own every bootstrap decision]

Context:
In this option, bootstrap is not a separate runtime planner. It is one branch of `ClusterRegime`.

```text
observe -> classify bootstrap authority -> resolve contract
```

Problem:
This keeps the model coherent, but it also makes the classifier responsible for more cases, including init-lock contradictions and local data reuse.

Restated question:
Should bootstrap remain fully inside the regime classifier, or should a narrower bootstrap sub-classifier exist beneath `BootstrapAuthority`?

## Q2 [How durable must consumer intent tracking be]

Context:
This option moves deduplication to receivers by `IntentId`.

```text
same intent emitted twice -> receiver sees same id -> ack without reapply
```

Problem:
In-memory tracking is simpler but may reapply work after restarts. Durable tracking is stronger but heavier.

Restated question:
Do DCS and process consumers need persistent intent ledgers, or is restart-safe best effort enough for this system?

## Q3 [Should degraded-majority be one regime or several]

Context:
`MajorityAuthority` may contain a `DegradedMode`, such as `TwoOfThreeHealthy`.

Problem:
One regime with a mode field is compact, but separate regimes could make invariants easier to test.

Restated question:
Is degraded-majority clearer as data inside `MajorityAuthority`, or should each degraded-majority case become its own top-level regime variant?

## Q4 [How much partial truth belongs in member publication]

Context:
This option proposes `service`, `postgres`, `capability`, and `confidence` fields in DCS publication.

Problem:
Richer truth helps classification and operator clarity, but it also widens the publication schema and the surface area of compatibility-sensitive behavior.

Restated question:
Should member publication include only directly observed Postgres truth, or should it also include supervisor and capability summaries?

## Q5 [Where should rewind failure memory live]

Context:
A rewind failure in this option changes the next contract from `RewindThenStart` to `CloneThenStart`.

Problem:
That memory could live in process outcome history, in an HA-local recovery ledger, or in the classified local capability.

Restated question:
Should fallback from rewind to basebackup be derived statelessly from recent process outcomes, or should the reconciler store explicit recovery memory?
