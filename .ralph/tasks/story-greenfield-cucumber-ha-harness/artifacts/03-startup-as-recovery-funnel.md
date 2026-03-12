# HA Refactor Option 3: Startup-As-Recovery Funnel

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because the current HA architecture still makes startup and rejoin feel like special cases that later merge back into steady-state HA. The differentiator for this option is that it deletes "startup" as a first-class control-flow concept and replaces it with one typed recovery funnel. The funnel begins on the first tick and stays valid forever. Every node action, including bootstrap, continue-primary, follow-primary, rewind, basebackup, demote, and wait, must pass through the same recovery classification stages. That makes this option materially different from option 1, which first classifies a broad regime, and option 2, which first classifies lease epochs and handoff stories. Here the organizing question is narrower and more operational: "Given the latest observations, what recovery stage is this node in relative to the currently authoritative cluster story?"

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 3.

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

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because this artifact is not arguing that the repo is unusable today. It is arguing that the architecture is still too split across startup, recovery, and steady-state concerns.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out while waiting for one primary across the two restarted fixed nodes and reported both restarted nodes as unknown. That failure is directly relevant to this option because a full-cluster restart should not be handled as a strange "startup exception"; it should be expressed as a normal pass through the same recovery funnel that also covers rejoin, failover, rewind, and basebackup.
- `tests/ha.rs` remains the acceptance contract surface that a later implementation must satisfy or intentionally revise with explicit reasoning.

## Current design problems

### 1. Runtime startup still prepares the world before HA has a first-class recovery model

The live path now enters through `run_node_from_config(...)` and `run_workers(...)` in `src/runtime/node.rs`. That is better than the older explicit startup planner/executor split described in the task background, but it still leaves startup as something the runtime does before the HA model has fully explained what this node is trying to recover into. `ensure_start_paths(...)` materializes directories, `run_workers(...)` wires subscriptions, and only then does the live HA loop begin. This is a clean runtime bootstrap, but it is not yet a unified recovery model.

The practical problem is not that path creation happens outside HA. The practical problem is that the first meaningful cluster interpretation still happens only after startup wiring has already been mentally separated from HA. That separation encourages future engineers to keep inventing special startup behavior when the real requirement is simpler: the first tick should just be recovery stage zero.

### 2. Current HA still jumps too quickly from world facts to local role goals

`src/ha/worker.rs` drives `step_once(...)`, which observes state, decides, reconciles, publishes, and applies actions. `src/ha/decide.rs` converts a `WorldView` into `DesiredState`. `src/ha/reconcile.rs` turns the target state into `ReconcileAction`s. `src/ha/process_dispatch.rs` lowers process-oriented actions into process job requests.

That pipeline is substantially better than ad hoc imperative code, but it still tends to answer "what role should I be?" before fully answering "what recovery stage am I in?" For example, `decide(...)` branches early on DCS trust, storage stall state, and lease state, then directly selects leader, follower, candidate, fenced, idle, or fail-safe outputs. Recovery intent exists inside those branches, but it is not the primary organizing abstraction. That matters because startup, rejoin, divergence repair, and authority recovery are all recovery questions before they are role questions.

### 3. Degraded quorum handling still compresses several recovery situations into one branch

`decide_degraded(...)` in `src/ha/decide.rs` makes the current safety boundary readable, but it still compresses fundamentally different recovery situations:

- a primary that has lost store quorum while still holding the latest locally observed WAL,
- a replica that still has a reachable upstream and can continue streaming safely,
- a node that is fully offline and waiting for authority evidence,
- a two-of-three majority that could still form authoritative service if enough evidence survives,
- a full-cluster restart where no current holder is active but valid data still exists on disk.

These are all recovery situations, but they are not the same recovery situation. A recovery-funnel design should force the system to classify them explicitly before it chooses fencing, continuation, bootstrap, or convergence work.

### 4. Startup and rejoin meaning is still reconstructed too late inside process dispatch

`src/ha/process_dispatch.rs` still contains `start_intent_from_dcs(...)`, `resolve_source_member(...)`, `validate_rewind_source(...)`, and `validate_basebackup_source(...)`. Those helpers are coherent, but they still mean the dispatch layer is deriving critical rejoin information from DCS state at the moment commands are emitted. That is too late in the pipeline.

The dispatcher should not have to rediscover whether local data is still valid, whether a node is rejoining an existing cluster, whether it should rewind versus clone, or whether it is in a bootstrap-adjacent state. Those should already be encoded in a typed recovery decision. Dispatch should only lower a fully determined recovery step into concrete jobs.

### 5. Delivery identity still originates from the HA sender path

The current code is improved relative to the original task background, but `process_job_id(...)` in `src/ha/process_dispatch.rs` still manufactures the concrete process job id on the HA side using `(scope, self, tick, action_index, action)`. That means the sender still owns the logical identity of dispatched work. For a recovery-funnel design, the sender should instead emit stable recovery-step identities, while consumers own deduplication and idempotent application.

### 6. Member publication already preserves partial truth, but the recovery model does not yet fully build around that fact

`build_local_member_slot(...)` in `src/dcs/state.rs` maps `PgInfoState::{Unknown,Primary,Replica}` into `MemberPostgresView::{Unknown,Primary,Replica}`. `src/pginfo/state.rs` also preserves `SqlStatus::{Unknown,Healthy,Unreachable}` and `Readiness::{Unknown,Ready,NotReady}`. This is strong groundwork. The missing architectural step is to make every recovery stage consume and preserve those partial truths explicitly.

A node with process liveness, unknown SQL health, and a present data directory is not "missing." It is "recoverable with uncertain local PostgreSQL facts." The system should publish that truth and classify recovery from it rather than silently collapsing uncertainty into absence.

## The central proposal

Replace the current role-first center with a recovery funnel that every tick must pass through:

1. `ObserveWorld`
   Gather the newest local and global evidence into one immutable `RecoveryObservation`.
2. `EstablishAuthority`
   Determine whether there is an authoritative cluster story, what leader evidence exists, and whether degraded-majority continuation is valid.
3. `ClassifyLocalRecovery`
   Determine what kind of local data situation this node is in relative to that authority story.
4. `SelectRecoveryStage`
   Produce a typed `RecoveryStage` that completely describes this node's next safe progression.
5. `LowerRecoveryStage`
   Translate the stage into receiver-owned intents and concrete actions.

The critical rule is that there is no separate startup planner. The first post-boot tick is just the earliest entry into the same recovery funnel. If the node is empty and the cluster is empty, that becomes bootstrap recovery. If the node has old primary data and a healthy leader exists elsewhere, that becomes convergence recovery. If the node was the leader and still has valid authority, that becomes continue-service recovery. If authority is ambiguous, that becomes publication-and-wait recovery. One funnel covers them all.

## Full proposed control flow from startup through steady state

### High-level method

At process boot:

1. Runtime still performs only the mechanical work that must exist outside HA:
   directory existence, channel wiring, worker spawn, log bootstrap, and subscription setup.
2. No startup planner, startup probe, startup executor, or startup-only command builder is allowed to exist as a competing logic path.
3. As soon as the first combined observation snapshot exists, HA runs the exact same funnel used later in steady state.
4. The node may initially lack DCS, pginfo, or process evidence. In that case the funnel yields a typed waiting stage such as `RecoveryStage::ObserveOnly` or `RecoveryStage::PublishAndAwaitAuthority`. It does not invent a startup shortcut.
5. Once enough evidence arrives, the node moves into one of the normal recovery stages:
   `ContinuePrimary`, `ContinueReplica`, `BootstrapCluster`, `AcquireLeadership`, `FollowLeader`, `RewindThenFollow`, `CloneThenFollow`, `FenceAndStop`, or `WaitForAuthority`.

During steady state:

1. Every new observation snapshot reruns the funnel from the beginning.
2. The node does not preserve a distinct "boot mode." It preserves only the last acknowledged recovery stage.
3. Stage transitions are triggered by changed evidence, not by which phase of process lifetime the node is in.
4. The lowerer emits intents keyed by stable recovery-step ids.
5. Consumers own duplicate suppression and acknowledgement.
6. The next tick compares observed consumer acknowledgements against the desired recovery stage and either advances, remains stable, or retreats safely to a more conservative stage.

### ASCII diagram

```text
          runtime wiring only
     (paths, channels, worker spawn)
                    |
                    v
     +-----------------------------------+
     | RecoveryObservation               |
     | - dcs trust + cache               |
     | - lease / leader evidence         |
     | - local pginfo                    |
     | - local process state             |
     | - local data-dir evidence         |
     | - storage status                  |
     | - last acknowledged intents       |
     +----------------+------------------+
                      |
                      v
     +-----------------------------------+
     | EstablishAuthority                |
     | - authoritative leader?           |
     | - valid degraded majority?        |
     | - bootstrap possible?             |
     | - authority ambiguous?            |
     +----------------+------------------+
                      |
                      v
     +-----------------------------------+
     | ClassifyLocalRecovery             |
     | - empty data?                     |
     | - consistent replica?             |
     | - old primary branch?             |
     | - unknown but salvageable?        |
     | - storage fenced?                 |
     +----------------+------------------+
                      |
                      v
     +-----------------------------------+
     | RecoveryStage                     |
     | ObserveOnly                       |
     | PublishAndAwaitAuthority          |
     | BootstrapCluster                  |
     | ContinuePrimary                   |
     | FollowLeader                      |
     | RewindThenFollow                  |
     | CloneThenFollow                   |
     | FenceAndStop                      |
     +----------------+------------------+
                      |
                      v
     +-----------------------------------+
     | LowerRecoveryStage                |
     | - publish member truth            |
     | - publish primary/no-primary      |
     | - issue lease intent              |
     | - issue process intent            |
     +----------------+------------------+
                      |
                      v
            receiver-owned idempotent
                DCS and process work
```

## Proposed typed state machine

The heart of this option is a new group of ADTs that make recovery state explicit and remove startup as a separate conceptual path.

### `RecoveryObservation`

This is the single pure input to the funnel. It should replace the informal spread of world facts across live decision and dispatch code.

Suggested shape:

```text
RecoveryObservation
  authority: AuthorityObservation
  local: LocalRecoveryObservation
  peers: PeerRecoveryObservations
  commands: ConsumerAckState
```

Where:

- `AuthorityObservation` contains DCS trust, current leader lease record if any, observed primary publication if any, switchover intent if any, and init-lock state if any.
- `LocalRecoveryObservation` contains local postgres truth, process worker truth, data-dir evidence, storage health, and last durable recovery marker if one exists.
- `PeerRecoveryObservations` contains member-slot truth from DCS for all peers, including partial-truth records.
- `ConsumerAckState` contains the last acknowledged recovery intents from DCS and process consumers.

### `AuthorityMode`

This is not the final stage. It is the first recovery gate.

```text
AuthorityMode
  UnknownStore
  NoAuthoritativeLeaderButBootstrapPossible
  AuthoritativeLeaderPresent { leader, lease_epoch }
  LeaseOpenButElectionPossible { eligible_voters }
  DegradedMajorityCanContinue { likely_leader, quorum_basis }
  NoSafeAuthority
```

Invariants:

- `UnknownStore` means DCS truth is too weak to make authority claims.
- `NoAuthoritativeLeaderButBootstrapPossible` is valid only when cluster initialization is still legitimately open.
- `AuthoritativeLeaderPresent` means follow or fence decisions can be made against a concrete leader.
- `LeaseOpenButElectionPossible` means there is no current holder, but enough evidence exists for an election or takeover.
- `DegradedMajorityCanContinue` is distinct from `NoSafeAuthority`. This is the key correction to the current compressed degraded path.
- `NoSafeAuthority` means the only correct outcomes are fence, wait, or publication-only truth.

### `LocalRecoveryClass`

This classifies local state relative to the authority mode.

```text
LocalRecoveryClass
  EmptyDataDir
  BootstrapEmptyDataDir
  HealthyPrimaryBranch { committed_wal }
  HealthyReplicaBranch { upstream, replay_wal }
  DivergedFormerPrimary { last_known_timeline }
  UncertainLocalState { reason }
  OfflineButDataPresent
  StorageFenced { cutoff }
```

Invariants:

- `EmptyDataDir` and `BootstrapEmptyDataDir` are distinct because the latter means initialized-but-empty local metadata exists.
- `HealthyPrimaryBranch` means local data is consistent enough to continue only if authority still allows it.
- `HealthyReplicaBranch` means following might continue without repair.
- `DivergedFormerPrimary` means the node previously had writable authority or conflicting lineage and must not be treated as a normal follower restart.
- `UncertainLocalState` is explicit and safe. It prevents the design from erasing partial truth.
- `StorageFenced` is terminal until evidence changes materially.

### `RecoveryStage`

This is the canonical output of the pure decider.

```text
RecoveryStage
  ObserveOnly
  PublishAndAwaitAuthority
  BootstrapCluster(BootstrapPlan)
  ContinuePrimary(PrimaryContinuationPlan)
  AcquireLeadership(ElectionPlan)
  FollowLeader(FollowPlan)
  RewindThenFollow(RewindPlan)
  CloneThenFollow(ClonePlan)
  DemoteAndReclassify(DemotePlan)
  FenceAndStop(FencePlan)
```

Every stage must be total. That means a future implementation cannot "also inspect DCS in the dispatcher just in case." If the stage is `RewindThenFollow`, it already contains the source member, timeline assumptions, and post-rewind next stage. If the stage is `BootstrapCluster`, it already contains init-lock claims, data-dir usage rules, and publication requirements.

### Stage transition rules

1. `ObserveOnly`
   Entered when evidence is too incomplete to claim authority or local recovery shape.
   Exits once member truth, DCS truth, or local process/data-dir truth becomes sufficient.

2. `PublishAndAwaitAuthority`
   Entered when the node can publish partial truth but must not yet act on leadership or convergence.
   Exits to election, continue, follow, bootstrap, or fencing once authority is clear.

3. `BootstrapCluster`
   Entered only when initialization remains legitimately open and bootstrap preconditions hold.
   Exits to `ContinuePrimary` only after bootstrap effects are acknowledged.

4. `ContinuePrimary`
   Entered when the node has valid authority to remain or become primary and local data is usable.
   Exits to `DemoteAndReclassify` or `FenceAndStop` if authority is lost or storage stalls.

5. `AcquireLeadership`
   Entered when lease is open or degraded-majority continuation allows a safe candidate election.
   Exits to `ContinuePrimary` after leadership acquisition is confirmed.
   Exits to `PublishAndAwaitAuthority` if a better candidate or stronger evidence appears.

6. `FollowLeader`
   Entered when leader authority is clear and local data is already compatible.
   Exits to `RewindThenFollow` or `CloneThenFollow` if compatibility checks fail.

7. `RewindThenFollow`
   Entered when local data is salvageable by rewind.
   Exits to `FollowLeader` after successful rewind acknowledgement.
   Exits to `CloneThenFollow` if rewind is impossible or previously failed in a way that requires basebackup.

8. `CloneThenFollow`
   Entered when local data must be replaced.
   Exits to `FollowLeader` after clone acknowledgement and startup acknowledgement.

9. `DemoteAndReclassify`
   Entered when the node must stop being primary or must stop its current role before reevaluating.
   Exits by rerunning the funnel against updated process and DCS truth.

10. `FenceAndStop`
    Entered when authority is unsafe or storage fencing requires a hard boundary.
    Exits only when changed evidence justifies a more permissive stage.

## How this redesign changes the authority model

This option intentionally does not make regime or epoch the top-level abstraction. It still needs explicit authority handling, but authority exists to feed the recovery funnel, not the other way around.

### Degraded-but-valid majority operation

The current tree routes all non-`FullQuorum` worlds through the degraded branch. This option instead forces the decider to ask whether a degraded but still authoritative majority can continue service.

For a three-node cluster:

- 3 of 3 with healthy DCS is normal authoritative operation.
- 2 of 3 with one node missing but DCS truth still usable is not automatically fail-safe. If the surviving pair can prove authoritative membership and a valid leader contract, they may continue or elect.
- 1 of 3 with no external authority is not valid majority operation and must not continue as operator-visible primary.
- DCS totally unavailable and no reliable authority proof means `NoSafeAuthority`, which leads to `FenceAndStop` or `PublishAndAwaitAuthority`.

This is how the design makes "degraded but healthy enough" an explicit state instead of an accidental side effect.

### Lease handling inside the funnel

Lease semantics still matter, but they are consumed by the authority stage rather than standing apart from recovery:

- if a valid lease is held by this node and local storage is not fenced, the node can enter `ContinuePrimary`;
- if a valid lease is held by a peer, this node can enter `FollowLeader`, `RewindThenFollow`, or `CloneThenFollow` depending on local classification;
- if no lease is held but election is safe, the node can enter `AcquireLeadership`;
- if lease evidence is stale or contradictory, the node must publish truth and wait or fence.

### Killed-primary and lost-lease handling

The design explicitly treats "old primary is dead" and "old primary lost authoritative lease" as recovery transitions, not special failover code paths:

- a killed primary that returns with old writable state becomes `DivergedFormerPrimary`;
- if a healthy leader exists elsewhere, that class can only go to `RewindThenFollow` or `CloneThenFollow`;
- if no authoritative leader exists but election is possible, the node is re-evaluated from scratch rather than assuming prior authority still matters.

## Startup reasoning under the recovery funnel

Startup is now a pure recovery case analysis.

### Case 1: cluster already has a leader and local node is empty

- `AuthorityMode` becomes `AuthoritativeLeaderPresent`.
- `LocalRecoveryClass` becomes `EmptyDataDir`.
- `RecoveryStage` becomes `CloneThenFollow`.

No startup-specific helper is needed.

### Case 2: cluster already has a leader and local node has consistent replica data

- authority says follow,
- local class says compatible replica,
- stage becomes `FollowLeader`.

The node simply starts as a follower using leader-derived managed config that was already decided earlier in the funnel.

### Case 3: cluster already has a leader and local node has old primary data

- authority says foreign leader exists,
- local class says `DivergedFormerPrimary`,
- stage becomes `RewindThenFollow` if rewind prerequisites hold, otherwise `CloneThenFollow`.

This is a rejoin case, not a startup special case.

### Case 4: cluster has no leader and bootstrap is still open

- authority says `NoAuthoritativeLeaderButBootstrapPossible`,
- local class distinguishes empty versus reusable initialized data,
- stage becomes `BootstrapCluster` only if bootstrap invariants hold.

The bootstrap plan must explicitly answer whether existing `pgdata` can be reused or must be wiped before initialization.

### Case 5: full-cluster restart with surviving data on multiple nodes

This is the scenario highlighted by the long test failure. The funnel handles it like this:

- observations first classify what data each visible member claims to have,
- authority stage determines whether any node can safely claim continuation or election,
- local classification distinguishes consistent primary branch, replica branch, or uncertain branch,
- the chosen stage is either `AcquireLeadership`, `ContinuePrimary`, or `PublishAndAwaitAuthority`.

The critical rule is that the system must not stall in "unknown startup" simply because all nodes restarted. Restart is not special. Recovery evidence is what matters.

## Replica convergence as one coherent path

This option makes convergence a sub-problem of the recovery funnel rather than a patchwork of dispatch-time decisions.

### Desired convergence order

1. If a healthy leader exists and local data is already aligned, use `FollowLeader`.
2. If a healthy leader exists and local data is plausibly salvageable, use `RewindThenFollow`.
3. If rewind is impossible, has already failed irrecoverably, or source validation fails, use `CloneThenFollow`.
4. After clone or rewind completes, the node re-enters the funnel and should naturally classify as `FollowLeader`.

### Why this is better than the current boundary

Today `follow_goal(...)` in `src/ha/decide.rs` reasons about some recovery choices, but `start_intent_from_dcs(...)` plus source validation in `src/ha/process_dispatch.rs` still performs important rejoin interpretation later in the stack. The funnel pulls those decisions upward:

- `decide` becomes responsible for selecting a complete convergence stage,
- `reconcile` becomes responsible for decomposing that stage into ordered intents,
- `process_dispatch` becomes a pure lowerer of already-complete recovery instructions.

## Member publication and partial truth in this design

This option depends on richer publication remaining first-class.

### Publication rules

1. Every node always attempts to publish member truth even when it cannot yet take a recovery action.
2. `PgInfoState::Unknown` still becomes a DCS member record, not silence.
3. A node may publish:
   process worker healthy,
   SQL unreachable,
   readiness unknown,
   timeline unknown,
   data dir present,
   last acknowledged recovery stage unknown.
4. HA must consume that publication as evidence for recovery classification.
5. Publication failure is itself evidence and must not be silently ignored.

### Why partial truth matters specifically for the funnel

The funnel is only safe if it can distinguish:

- "I know nothing about this peer,"
- "this peer is alive but pginfo is degraded,"
- "this peer is alive and was recently primary,"
- "this peer is alive with empty or missing data,"
- "this peer is alive but storage is fenced."

Those are distinct recovery inputs. The design falls apart if they collapse into generic absence.

## Deduplication boundary in this option

Deduplication moves out of sender-owned HA logic and into consumers.

### New rule

The pure HA path produces stable recovery intent descriptors such as:

- `publish-member-truth(member, version)`
- `publish-primary-view(epoch_ref or no_primary_reason)`
- `acquire-lease(recovery_stage_ref)`
- `start-postgres-as-primary(recovery_stage_ref)`
- `rewind-from(source_member, recovery_stage_ref)`
- `clone-from(source_member, recovery_stage_ref)`

The HA sender does not create the final consumer job ids. Instead:

- the process worker assigns or validates process-command identity based on a stable stage reference plus command kind,
- the DCS worker does the same for lease/publication writes,
- acknowledgements flow back as consumer-owned receipts.

### Why this is safer

Sender-side tick-based identity is fragile because retries, reordered deliveries, or internal stage splits can accidentally create different job ids for the same logical work. Consumer-owned idempotency is safer because the receiver is the only component that knows whether the requested effect has already been applied, superseded, or is still in flight.

## Concrete repo areas a future implementation would touch

The later implementation for this option would need to reshape the following areas.

- `src/ha/types.rs`
  Add `RecoveryObservation`, `AuthorityMode`, `LocalRecoveryClass`, `RecoveryStage`, recovery plans, and consumer acknowledgement types.
- `src/ha/decide.rs`
  Replace the current direct `DesiredState` selection with the recovery funnel stages.
- `src/ha/reconcile.rs`
  Lower `RecoveryStage` into ordered DCS and process intents instead of lowering a role-shaped target.
- `src/ha/worker.rs`
  Keep the top-level loop but rename the internal conceptual steps around recovery observation and recovery-stage application.
- `src/ha/process_dispatch.rs`
  Delete `start_intent_from_dcs(...)` and source rediscovery logic from the dispatch layer; lower only typed recovery actions that already name their source and semantics.
- `src/ha/decision.rs`
  Either absorb this file into the new canonical recovery ADTs or delete/migrate overlapping decision vocabulary so there is one live typed center.
- `src/ha/lower.rs`
  Reuse whatever remains useful for effect-lowering, but align it to `RecoveryStage` instead of a parallel dormant decision model.
- `src/dcs/state.rs`
  Extend member publication and trust inputs so the recovery funnel gets richer authority evidence and explicit partial-truth markers.
- `src/dcs/worker.rs`
  Preserve publication-first behavior and add consumer acknowledgements for DCS-owned intents.
- `src/pginfo/state.rs`
  Possibly add explicit local recovery evidence such as durable control-file facts, last-known upstream evidence, or data-dir classification helpers.
- `src/runtime/node.rs`
  Remove any stale startup-planning remnants and keep runtime responsibilities strictly mechanical.
- `tests/ha.rs` and HA feature fixtures
  Update scenario expectations only where the redesign intentionally changes authority and degraded-majority semantics.

## Meaningful changes required by this option

### New types

- `RecoveryObservation`
- `AuthorityMode`
- `LocalRecoveryClass`
- `RecoveryStage`
- `RecoveryStageRef`
- `RecoveryPlan::{Bootstrap,PrimaryContinuation,Election,Follow,Rewind,Clone,Fence}`
- `ConsumerAckState`

### Deleted or collapsed paths

- any startup-only planning abstractions that duplicate later recovery reasoning,
- process-dispatch logic that re-derives source authority from DCS,
- sender-owned final job ids based on HA tick counters.

### Moved responsibilities

- authority interpretation moves earlier and becomes explicit,
- local data interpretation moves out of late dispatch helpers and into pure classification,
- deduplication moves into DCS/process consumers,
- startup reasoning stops being a separate architectural domain.

### Changed transitions

- degraded-majority continuation becomes distinct from total authority loss,
- old-primary return becomes a typed recovery class instead of a branch hidden inside follow logic,
- full-cluster restart becomes normal recovery evaluation rather than "startup weirdness,"
- bootstrap becomes one recovery stage among others, gated by explicit invariants.

### Changed DCS publication behavior

- publication stays continuous even when action is impossible,
- member truth may include explicit uncertainty and recovery-stage hints,
- primary publication becomes an output of the stage, not an opportunistic side effect.

### Changed startup handling

- runtime boot performs no startup-only HA planning,
- the first combined observation immediately enters the same funnel used for all later ticks,
- durable local data evidence is part of recovery classification from the first tick.

### Changed convergence handling

- rewind and basebackup become explicit stages,
- source-member selection is decided before lowering,
- successful repair always returns to the same `FollowLeader` stage.

### Required future test updates

Most existing HA scenarios should remain conceptually valid. Test changes would be needed mainly where the repo currently assumes all degraded trust implies fail-safe visibility loss. A later implementation must update those scenarios explicitly and only where the new recovery semantics intentionally redefine valid degraded-majority service.

## Migration sketch

This option requires a disciplined migration so stale parallel paths do not survive.

1. Introduce the new recovery ADTs alongside the current `DesiredState` types.
2. Add a pure adapter layer that constructs `RecoveryObservation` from the current world inputs.
3. Implement `AuthorityMode` and `LocalRecoveryClass` derivation first, without changing dispatch behavior.
4. Add `RecoveryStage` selection and temporarily map it back into today's `DesiredState` plus `ReconcileAction` outputs for equivalence testing.
5. Once stable, rewrite `reconcile` to consume `RecoveryStage` directly.
6. Move source-selection details out of `process_dispatch` into earlier recovery plans.
7. Add consumer acknowledgement plumbing so DCS and process workers own idempotency.
8. Delete the old role-first shortcuts and any stale startup-only planning artifacts.
9. Remove parallel or redundant HA decision vocabularies so there is one canonical center.

The migration must delete old paths when the new ones become authoritative. The project is greenfield. There is no reason to keep legacy branches alive "just in case."

## Non-goals

- This option does not attempt to preserve every current type name.
- This option does not keep startup as a separately optimized planner.
- This option does not treat DCS dispatch helpers as an acceptable place to rediscover recovery meaning.
- This option does not promise zero test updates; it promises explicit semantic updates where behavior intentionally changes.
- This option does not authorize any production-code or test-code fixes in this task.

## Tradeoffs

- The recovery funnel is more operationally concrete than the current role-first design, but it introduces additional ADTs that future contributors must learn.
- Some engineers may find regime-first or epoch-first designs more elegant for reasoning about authority at cluster scope. This option intentionally favors implementer clarity around recovery transitions over top-level conceptual purity.
- The decider becomes responsible for more complete plans, which raises the bar for type design. That is acceptable in this repo because impossible states should be made unrepresentable rather than pushed into late dispatch branches.

## Logical feature-test verification

This section does not claim that the repository already behaves this way. It explains how a later implementation of this option should map onto the HA feature suite.

### `ha_dcs_quorum_lost_enters_failsafe`

If DCS truth becomes too weak to establish safe authority, the funnel yields `FenceAndStop` or `PublishAndAwaitAuthority` depending on local role and evidence. Operator-visible primary publication disappears because authority is no longer safe enough to present a primary.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

A primary that loses safe authority enters `FenceAndStop` with an explicit fence plan carrying the cutoff evidence. Post-cutoff writes are rejected because the stage itself requires demotion or fencing before service can be considered healthy again.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority partition classifies as either `LeaseOpenButElectionPossible` or `DegradedMajorityCanContinue`, depending on surviving authority evidence. The isolated old primary classifies as `NoSafeAuthority` or foreign-to-majority. The majority side can enter `AcquireLeadership` or `ContinuePrimary`, while the isolated node can only publish limited truth or fence.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

When healed, the old primary is classified as `DivergedFormerPrimary`. If rewind is possible, the stage is `RewindThenFollow`; otherwise it becomes `CloneThenFollow`. It must not return directly to `ContinuePrimary`.

### `ha_primary_killed_then_rejoins_as_replica`

The restarted old primary re-enters the funnel from scratch. If another authoritative leader exists, it follows the same `DivergedFormerPrimary -> RewindThenFollow/CloneThenFollow -> FollowLeader` path.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

The surviving primary initially may fall into degraded-majority or no-safe-authority classification depending on DCS evidence. Once one replica returns and the authority stage becomes sufficient again, the primary can remain or become operator-visible and the restarted node enters follower convergence.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the key scenario for this option. The first two restarted nodes must not wait in a vague startup limbo. They classify their surviving data, determine whether authoritative continuation or election is possible, and choose `AcquireLeadership` or `ContinuePrimary`. The final node later returns through the normal rejoin stages and becomes `FollowLeader` after any required rewind or clone.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The funnel explicitly models this transition. `RewindThenFollow` carries a next-stage fallback to `CloneThenFollow` when rewind is impossible or fails in a way marked non-retriable.

### `ha_replica_stopped_primary_stays_primary`

The primary remains in `ContinuePrimary` because authority and local health remain valid. The stopped replica simply disappears from current peer truth until it returns.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

The broken node can remain stuck in `CloneThenFollow`, `RewindThenFollow`, or `PublishAndAwaitAuthority` without affecting the healthy leader's `ContinuePrimary` stage. Rejoin failure stays localized to the recovering member.

## Q1 Should `RecoveryStage` carry full source-member details or only abstract recovery plans?

Context: this option wants to remove `resolve_source_member(...)`, `validate_rewind_source(...)`, and `validate_basebackup_source(...)` from late dispatch logic in `src/ha/process_dispatch.rs`.

Problem: if `RecoveryStage::RewindThenFollow` and `RecoveryStage::CloneThenFollow` carry only abstract intent, the dispatcher will still need to rediscover source members and will recreate the current boundary drift. If they carry full concrete source details, the pure decider or an earlier pure planning layer must absorb more topology material.

Restating the question: should the stage be fully concrete enough that dispatch never looks back into DCS for recovery meaning, or is there a narrower middle shape that still keeps late dispatch honest?

## Q2 How should degraded-majority continuation be proven without reintroducing unsafe optimism?

Context: this option explicitly distinguishes `DegradedMajorityCanContinue` from `NoSafeAuthority`, which is a deliberate departure from the current compressed degraded branch.

Problem: the implementation must define what evidence is sufficient to claim safe degraded continuation. Too strict and the design collapses back into current fail-safe behavior. Too loose and the system risks admitting dual-primary windows after authority fractures.

Restating the question: what exact peer, lease, and publication evidence should be required before the funnel may choose `ContinuePrimary` or `AcquireLeadership` under degraded trust?

## Q3 Should local data classification live entirely inside HA or be precomputed by a dedicated local-state worker?

Context: `LocalRecoveryClass` needs durable evidence such as data-dir presence, likely lineage hints, prior role hints, storage fencing state, and pginfo truth.

Problem: computing that inside HA keeps one pure decision center, but it may lead to heavier HA-specific filesystem and control-file parsing responsibilities. Moving it to a dedicated local-state worker could keep HA pure, but it introduces another channel and another typed boundary that can drift.

Restating the question: is the right architecture to make HA own all local recovery classification from raw evidence, or should a subordinate worker publish a typed local recovery snapshot that HA consumes?

## Q4 How should bootstrap reuse existing `pgdata` without making stale states silently acceptable?

Context: the user explicitly wants bootstrap reconsidered, including whether existing `pgdata` can still be used when the node wins the init lock.

Problem: some initialized directories may be safely reusable, while others may represent partial bootstrap, stale primary history, or ambiguous failed initialization. The funnel must avoid both extremes: unconditional wipe and dangerous reuse.

Restating the question: what exact local-data proofs should permit `BootstrapCluster` to reuse existing data, and when must the stage force a wipe before initialization?

## Q5 How should consumer acknowledgements be represented so the funnel can progress deterministically?

Context: this option moves deduplication ownership into DCS and process consumers. That means the HA loop needs reliable acknowledgement inputs for issued recovery intents.

Problem: if acknowledgements are too coarse, HA cannot safely tell whether a stage is complete, still in progress, or partially applied. If they are too detailed, the system may become cumbersome and overfit to current worker internals.

Restating the question: what is the minimal acknowledgement model that still lets the funnel advance from `BootstrapCluster`, `RewindThenFollow`, and `CloneThenFollow` without sender-side guesswork?
