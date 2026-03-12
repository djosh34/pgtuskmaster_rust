# HA Refactor Option 4: Receiver-Owned Command Ledger

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because the current HA architecture is already fairly disciplined about computing actions before applying them, but it still lets the HA sender path own too much of command identity, late source selection, and duplicate suppression meaning. The differentiator for this option is that it keeps the high-level chain `newest info -> decide -> typed outcome -> actions`, but it moves command identity, application progress, and idempotency authority out of `src/ha/worker.rs` / `src/ha/process_dispatch.rs` and into explicit receiver-owned ledgers. Instead of the HA loop saying "run this exact job id now," it says "here is the cluster-approved intent revision," and each consumer decides whether that revision is new, superseded, already durable, still running, or safely ignorable. That makes this option materially different from option 1, which centers cluster regime ADTs, option 2, which centers lease epochs and handoff stories, and option 3, which centers recovery-stage classification.

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 4.

1. `Regime-first reconciliation`
   The system first derives a cluster regime ADT, then derives a local contract from that regime.
2. `Lease-epoch story machine`
   The system is organized around explicit lease epochs and handoff stories, with every transition anchored to epoch ownership.
3. `Startup-as-recovery funnel`
   Startup is deleted as a special case and replaced by one recovery funnel that handles empty, existing, diverged, and stale data uniformly.
4. `Receiver-owned command ledger`
   HA produces stable intent revisions while DCS and process consumers own all idempotency, duplicate suppression, and apply-progress truth.
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

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because this artifact is not claiming that receiver-owned idempotency is required simply because the tree is broadly red. The claim is architectural: the current sender/receiver split still leaves too much command authority in the sender.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out waiting for one primary across the two restarted fixed nodes and sampled both restarted nodes as unknown. That failure matters here because restart authority and rejoin recovery are exactly the kinds of flows that become fragile when the sender has to rediscover intent and reissue late-bound jobs instead of handing receivers a durable, cluster-approved command revision.
- `tests/ha.rs` remains the acceptance contract surface for later implementation work. A future implementation based on this option must either satisfy those scenarios or revise them with explicit new semantics.

## Current design problems

### 1. Startup reasoning is still not expressed as durable intent with observable apply progress

The live runtime path starts in `src/runtime/node.rs`, enters `run_workers(...)`, and then the HA worker begins deciding on actions. That is already cleaner than the older startup planner split described in the task file, but the first startup-related process work is still emitted as ordinary process requests rather than as durable cluster-approved intent revisions with receiver-owned apply state.

That distinction matters because startup is where the system most needs to answer:

- which authority story is active,
- which node is supposed to converge toward which role,
- whether the intended work is already in flight,
- whether that work completed, failed, or was superseded,
- whether a restart is a retry of the same intended operation or a genuinely new command.

Today those questions are only partially captured in sender-side action choice plus process worker outcomes. This option makes them explicit ledger questions.

### 2. The HA loop still owns too much of process command identity

`src/ha/worker.rs` currently drives `observe -> decide -> reconcile -> publish -> apply`. When a process-oriented `ReconcileAction` appears, `execute_action(...)` calls `dispatch_process_action(...)` in `src/ha/process_dispatch.rs`. That function constructs a `ProcessJobRequest` immediately and gives it an id derived by `process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick)`.

That is a clean implementation detail for a small system, but it still means the HA sender owns:

- the logical identity of the command,
- the deduplication key shape,
- the timing of when a new request should exist,
- the action-specific distinction between "same intent restated" and "new intent revision,"
- the boundary where source resolution must occur.

The user explicitly wanted deduplication away from sender-side HA logic. This option completes that move by making the HA layer publish typed intent revisions and by making the receiving workers decide whether they need to do anything.

### 3. Late source selection in `src/ha/process_dispatch.rs` still hides decision meaning in the lowering layer

`start_intent_from_dcs(...)`, `resolve_source_member(...)`, `validate_rewind_source(...)`, and `validate_basebackup_source(...)` are all coherent helpers, but together they prove that important recovery meaning still lives in the dispatch layer. At the moment the sender says "start replica" or "basebackup from member X," and the dispatcher then consults DCS state to materialize the exact startup shape.

That is too late. The dispatch layer should not be reinterpreting cluster state in order to figure out what the command really meant. The pure HA decision should already have produced a typed convergence intent whose receiver can execute or reject based on local feasibility and durable intent revision ordering.

### 4. Quorum and fail-safe boundaries still combine command planning and command application too tightly

`src/ha/decide.rs` already has a readable decision surface, but it still produces outputs that are interpreted almost immediately into side-effect requests. That encourages a design where "this tick wants promotion" turns too quickly into "send promote now." In degraded-majority, fail-safe, and restart scenarios, that is exactly where systems become noisy or ambiguous.

A ledger-based design inserts a deliberate seam:

- the decider produces an authoritative intent revision,
- the publication layer records that revision,
- consumers reconcile their own local apply state against that revision,
- only then are subprocesses or DCS writes emitted.

This does not weaken safety. It improves auditability and allows progress tracking to survive worker restarts.

### 5. Receiver state already exists, but it is not yet treated as the source of idempotency truth

The process subsystem already has the beginning of a receiver-owned state model:

- `src/process/state.rs` defines `ProcessState::{Idle,Running}`,
- `ProcessJobRequest` carries an id and a kind,
- `JobOutcome` records success, failure, or timeout,
- `src/process/worker.rs` rejects busy requests and reports outcomes.

Those are real foundations. The missing architectural step is to stop treating them as merely the aftermath of sender-issued jobs and instead treat them as the authoritative progress ledger for process-intent consumption. The same design pressure applies to DCS-side actions such as lease acquisition, lease release, and switchover clearing. Consumers should own whether a specific intent revision has been applied.

### 6. Member publication carries partial truth, but action progress is not published with the same rigor

`src/dcs/worker.rs` and `src/dcs/state.rs` already preserve partial truth in member keys through `MemberSlot`, `MemberPostgresView`, `Readiness`, and timeline/WAL information. That is good and must remain. The gap is that command application progress is still mostly local worker state. A cluster that wants deterministic recovery after restart should be able to tell not only "node-b last looked like a replica" but also "node-b has observed intent revision 42 for convergence-to-follower and is currently applying step 2 of 3" or "node-b rejected revision 42 because the source member disappeared."

This option therefore extends the idea of partial truth from data-state publication into command-progress publication.

## The central proposal

Replace direct sender-issued process/DCS jobs with a two-layer intent system:

1. `Observe`: collect the newest local and global evidence into an immutable `ObservationEnvelope`.
2. `Decide`: derive a pure `ClusterIntentRevision` that includes cluster authority, startup/recovery meaning, and per-member intended contracts.
3. `PublishIntent`: publish the latest intended contract revision for this member to receiver-owned ledgers.
4. `ConsumeIntent`: let the process and DCS consumers compare their local apply state with the latest revision and determine whether work is required, already complete, still running, failed, or superseded.
5. `ReportProgress`: publish receiver-owned apply progress back into state so the next HA tick reasons over durable progress, not just inferred sender history.

The decisive design rule is this: the HA worker owns desired meaning, but it does not own command identity or duplicate suppression. Receivers own those responsibilities because receivers are the only components that can truthfully know whether a command has already been accepted, is still active, became obsolete, or completed durably.

## Full proposed control flow from startup through steady state

This option keeps one HA reconciliation loop, but it inserts explicit intent publication and receiver-owned application ledgers.

```text
          +---------------------------+
          | pginfo / dcs / process /  |
          | config newest observations|
          +-------------+-------------+
                        |
                        v
              +-------------------+
              | ObservationEnvelope|
              +---------+---------+
                        |
                        v
              +-------------------+
              | Pure HA Decider    |
              | decide_revision()  |
              +---------+---------+
                        |
                        v
          +-------------------------------+
          | ClusterIntentRevision         |
          | - authority verdict           |
          | - startup/recovery contract   |
          | - lease contract              |
          | - publication contract        |
          | - process intent revision     |
          +---------------+---------------+
                          |
                          v
         +--------------------------------------+
         | Intent Publication Layer             |
         | writes receiver-readable ledger rows |
         +----------------+---------------------+
                          |
              +-----------+-----------+
              |                       |
              v                       v
   +---------------------+   +---------------------+
   | DCS Intent Consumer |   | Process Consumer    |
   | owns apply ledger   |   | owns apply ledger   |
   +----------+----------+   +----------+----------+
              |                         |
              v                         v
   +---------------------+   +---------------------+
   | etcd read/write only|   | postgres subprocess |
   | side effects        |   | side effects        |
   +----------+----------+   +----------+----------+
              |                         |
              +-----------+-------------+
                          |
                          v
               +------------------------+
               | published apply truth  |
               | seen on next HA tick   |
               +------------------------+
```

### Startup tick behavior

On the first tick after `run_workers(...)` starts, the HA worker does not immediately send bootstrap/basebackup/start-postgres jobs. Instead it:

1. reads the newest `pginfo`, `dcs`, `process`, and local config facts,
2. computes a `ClusterIntentRevision`,
3. publishes a member-scoped intended contract,
4. lets the process consumer decide whether that revision is already satisfied, needs execution, or is blocked by missing prerequisites.

That means startup becomes recoverable and inspectable. If the node restarts mid-bootstrap, the next run reads the last published intent revision and the last receiver-owned apply state instead of inferring everything from sender history.

### Steady-state tick behavior

Later ticks behave the same way:

1. observe newest facts,
2. derive the next intended revision,
3. compare with the last published revision,
4. publish a new revision only when intended meaning changed,
5. let receivers consume or ignore the revision based on their local ledgers.

This makes repeated "stay primary" or "stay replica" conclusions cheap. They become stable intent revisions rather than repeated imperative job sends.

### Switchover and failover behavior

Switchover and failover are represented as intent-revision changes:

- a new authority contract is published,
- the old process intent revision becomes superseded,
- the target node's ledger sees a promotion intent revision,
- the old leader's ledger sees a demotion or fence revision,
- DCS lease actions are driven by a DCS consumer that reasons from the same authority revision.

The important benefit is that the sender no longer needs to decide "did I already ask for this exact demote?" The demote intent is durable, versioned, and receiver-consumed.

## Proposed typed state machine

This option requires a more explicit split between intent meaning and apply progress.

### Core pure-decision ADTs

```text
ObservationEnvelope
  = {
      local_member: MemberId,
      dcs_view: DcsObservation,
      pg_view: LocalPgObservation,
      process_view: ProcessApplyObservation,
      config_view: RuntimeConfigSnapshot,
      now: UnixMillis,
    }

ClusterIntentRevision
  = {
      revision: IntentRevision,
      authority: AuthorityContract,
      publication: PublicationContract,
      local_contract: LocalNodeContract,
    }

LocalNodeContract
  = {
      role_contract: RoleContract,
      recovery_contract: RecoveryContract,
      lease_contract: LeaseContract,
      process_contract: ProcessIntentContract,
      dcs_contract: DcsIntentContract,
    }
```

### Intent revision identity

`IntentRevision` must be a semantic revision, not a tick-based job id. It should change only when intended meaning changes. A later implementation could define it as:

```text
IntentRevision
  = {
      authority_epoch: AuthorityEpoch,
      member_revision: u64,
      contract_hash: IntentFingerprint,
    }
```

The crucial point is that it is derived from intended semantics, not from `(ha_tick, action_index)`. Two consecutive ticks that want the same behavior should produce the same revision.

### Role and recovery ADTs

```text
RoleContract
  = RemainPrimary
  | BecomePrimary
  | FollowLeader { leader: MemberId }
  | StayIdle
  | FenceWrites
  | StopPostgres

RecoveryContract
  = NoRecoveryNeeded
  | BootstrapCluster { charter: BootstrapCharter }
  | ReattachReplica { leader: MemberId, method: ReattachMethod }
  | RecoverFormerPrimary { leader: MemberId, method: ConvergenceMethod }
  | WaitForAuthority { reason: WaitReason }

ReattachMethod
  = HealthyFollow
  | RestartFollow
  | RewindThenFollow
  | BasebackupThenFollow

ConvergenceMethod
  = RewindPreferred
  | BasebackupRequired
```

### Receiver-owned apply ADTs

The new state model needs a durable description of what receivers have done with an intent revision.

```text
ReceiverApplyState
  = NotSeen
  | Accepted { accepted_at: UnixMillis }
  | Running { started_at: UnixMillis, step: ApplyStep }
  | Succeeded { finished_at: UnixMillis, evidence: ApplyEvidence }
  | Failed { finished_at: UnixMillis, error: ApplyError }
  | Rejected { finished_at: UnixMillis, reason: RejectionReason }
  | Superseded { superseded_by: IntentRevision }

ApplyStep
  = DcsLeaseAcquire
  | DcsLeaseRelease
  | ClearSwitchover
  | MaterializeManagedConfig
  | StartPostgres
  | PromotePostgres
  | DemotePostgres
  | RunPgRewind
  | RunBasebackup
  | BootstrapDataDir
```

There should be one receiver-owned ledger per consumer domain:

- `ProcessIntentLedgerState`
- `DcsIntentLedgerState`

The HA worker may read those states, but it may not mutate them directly except by publishing a new intended revision.

### State invariants

This design depends on several invariants.

1. A receiver may only transition from one apply state to another for the latest known intent revision or while marking an older revision superseded.
2. A receiver must never execute side effects for a revision it already marked `Succeeded`, unless the specific action type is explicitly defined as repeat-safe and the evidence requires replay.
3. A new HA tick may reuse the same `IntentRevision` when intended meaning is unchanged.
4. Startup and steady-state use the same revision model. There is no "special startup job id" concept.
5. Lease changes and process changes must both reference the same `AuthorityEpoch` so a node cannot process a promotion intent that belongs to a different authority story than the lease it thinks it is serving.

## How this redesign changes the authority model

This option is not primarily a new quorum algorithm, but it still changes how authority becomes actionable.

### Authority first, side effects second

The pure decider must still determine:

- whether there is full quorum,
- whether degraded but valid majority authority exists,
- whether the current leader lease is valid,
- whether a new primary may be elected,
- whether the local node must fence or stop,
- whether startup should become bootstrap, follow, rewind, basebackup, or wait.

The difference is that the decider now publishes those conclusions as an authority contract revision instead of directly triggering imperative actions.

### Degraded-majority continuation

The current `DcsTrust::{FullQuorum,Degraded,NotTrusted}` shape in `src/dcs/state.rs` is too compressed for this option if it remains the main authority input. A later implementation should preserve the broad trust signal but derive a richer HA authority contract, for example:

```text
AuthorityContract
  = AuthoritativePrimary { holder: MemberId, epoch: AuthorityEpoch }
  | AuthoritativeElection { eligible_voters: VoterSet, epoch: AuthorityEpoch }
  | DegradedButValidMajority { holder: Option<MemberId>, epoch: AuthorityEpoch }
  | Unauthoritative { reason: AuthorityLossReason }
```

The reason this matters for a ledger design is simple: receivers must know whether the revision they are consuming came from an authoritative source. A promotion revision under `Unauthoritative` is invalid by construction. A fence or stop revision under `Unauthoritative` is allowed because it is conservative.

### Lease semantics

Lease semantics become part of the authority contract rather than an implementation detail of a single `AcquireLease` or `ReleaseLease` action. The DCS consumer owns the idempotent application of lease changes, but the pure decider must still derive:

- whether the node should hold a lease,
- whether a held lease is stale relative to the current authority epoch,
- whether a former primary must release or be ignored,
- whether restart after full outage should begin a fresh authority epoch.

A killed or isolated former primary loses authority not because the sender happens to stop reissuing "stay primary," but because later authority revisions no longer validate that node's lease or role contract.

## Startup reasoning under the command-ledger model

Startup must first reason about the cluster story before any receiver executes work.

### Cluster already running with an active leader

If DCS and member observations show an authoritative leader and the local node has healthy replica data, the first revision published for the local member should be:

- `RoleContract::FollowLeader { leader }`
- `RecoveryContract::ReattachReplica { leader, method: HealthyFollow | RestartFollow }`
- `LeaseContract` that explicitly forbids local lease acquisition
- a process contract whose latest intended revision is "ensure replica postgres is running with this upstream identity"

The process consumer can then compare that revision to its own apply ledger and determine whether it already has a matching active or successful runtime.

### Cluster already running but local data diverged

If local timeline or WAL evidence shows the node is a former primary on the wrong branch, the first revision should publish a convergence contract such as:

- `RecoverFormerPrimary { leader, method: RewindPreferred }`

The process consumer then owns the stepwise application:

1. accept revision,
2. stop postgres if needed,
3. run rewind,
4. if rewind fails with a classified irrecoverable reason, mark failure evidence,
5. wait for the next HA revision to either preserve basebackup fallback inside the same semantic contract or promote a new `BasebackupRequired` revision.

The sender does not need to construct a new tick-specific rewind job id each time.

### Empty cluster and bootstrap

If there is no existing leader, no durable authoritative cluster state, and the node wins the init lock under a valid authority contract, the first startup revision can be:

- `BootstrapCluster { charter }`
- `RoleContract::BecomePrimary`
- `LeaseContract::AcquireAfterBootstrap`

The process consumer owns bootstrap progress. The DCS consumer owns init-lock and lease application. The next HA tick reasons from their published progress rather than from assumptions.

### Existing `pgdata` when init lock is won

This option does not assume that winning the init lock always means wiping and bootstrapping blindly. The pure decision should classify existing local data explicitly:

- valid candidate to continue as authoritative primary,
- acceptable seed for cluster bootstrap,
- stale branch that requires rewind/basebackup before service,
- unusable data that must be replaced.

Those classifications become part of the `BootstrapCharter` or `RecoveryContract`, not hidden inside the process dispatcher.

## Replica convergence as one coherent path

This option keeps the user's desired simplified convergence story, but expresses it as receiver-consumable intent revisions.

### Convergence order

When a valid healthy leader exists, a node should move through this semantic order:

1. `HealthyFollow`
   If local data is already on the correct timeline and sufficiently aligned, simply ensure follower runtime.
2. `RestartFollow`
   If data is valid but postgres is offline, start it under replica config.
3. `RewindThenFollow`
   If the node is on the wrong timeline but rewind is possible, publish a rewind revision.
4. `BasebackupThenFollow`
   If rewind is impossible or has failed irrecoverably, publish a basebackup revision.

The important part is that all of these are variations of one convergence contract family. The receiver ledger records which revision and which step were actually applied.

### Why this is better than sender-issued retries

With sender-issued job ids, the HA loop is tempted to keep rediscovering and resending operations. With receiver-owned ledgers, retries become deliberate:

- same semantic contract can retain the same revision while a receiver remains `Running`,
- a receiver can publish `Failed { error }` with classified evidence,
- the pure decider can then decide whether the same revision should remain, whether the contract should escalate, or whether authority changed enough to supersede the contract.

That produces cleaner reasoning for `ha_rewind_fails_then_basebackup_rejoins_old_primary` and similar scenarios.

## Member publication and partial truth in this design

Partial truth must remain publishable even when command application is uncertain.

### Member slot truth stays in DCS

`src/dcs/worker.rs` should continue to publish local member truth derived from `PgInfoState` into `MemberSlot`. That includes:

- `Unknown` postgres status with readiness and timeline if available,
- `Primary` committed WAL truth,
- `Replica` upstream and replay/follow WAL truth.

This artifact does not propose removing that model.

### Add intent-progress truth alongside member truth

A later implementation should add separate published progress records, not overload `MemberSlot` into a giant everything-document. For example:

```text
MemberIntentProgressRecord
  = {
      member_id: MemberId,
      latest_seen_revision: Option<IntentRevision>,
      process_apply: ReceiverApplyState,
      dcs_apply: ReceiverApplyState,
      observed_at: UnixMillis,
    }
```

That record should preserve partial truth too. Examples:

- "pginfo unknown, process consumer accepted replica-follow revision but has not started postgres yet"
- "process consumer failed rewind revision due to source validation"
- "DCS consumer succeeded lease release for authority epoch 18"

This is useful because a supervisor looking at the cluster can then distinguish "node is silent" from "node is alive, has partial database truth, and is blocked on a specific convergence step."

## Deduplication boundary in this option

This section is the heart of the option.

### Current boundary

Today `src/ha/process_dispatch.rs` constructs `ProcessJobRequest` objects and generates `JobId` values from sender-side context such as tick and action index. That is deterministic, but it is still sender-owned identity.

### New boundary

The new boundary is:

- HA publishes semantic intent revisions.
- Consumers record whether they have seen and applied those revisions.
- Consumers suppress duplicates because they have the authoritative view of what they have already accepted or completed.

### Why receivers are safer owners of idempotency

Receivers are safer because only receivers know:

- whether an earlier request is still running,
- whether the effect already completed durably,
- whether a failure was transient or terminal,
- whether local prerequisites are still satisfied,
- whether a new revision supersedes the current one.

The sender can infer these things, but inference is weaker than direct knowledge. This is especially relevant after worker restart, partial network failure, or log replay.

### DCS consumer also needs a ledger

This option is not only about process jobs. DCS-side actions such as lease acquisition and switchover clearing should also move through a receiver-owned intent consumer. That means the DCS layer becomes the sole owner of "lease acquire revision 21 already applied" truth instead of the HA loop treating DCS calls as immediate imperative side effects.

## Concrete repo areas a future implementation would touch

This option would later require changes in at least these code areas:

- `src/runtime/node.rs`
  Startup wiring would need to initialize the new intent-ledger publishers/subscribers and ensure first tick observes prior apply state.
- `src/ha/worker.rs`
  `step_once(...)` would stop directly executing `ReconcileAction`s and would instead publish `ClusterIntentRevision`s plus member-scoped intended contracts.
- `src/ha/decide.rs`
  The pure decision surface would need to derive richer authority and convergence contracts that are stable across equivalent ticks.
- `src/ha/decision.rs`
  This file is a natural place to become the canonical home for intent-contract ADTs and semantic revision shaping.
- `src/ha/lower.rs`
  Lowering would shift from "emit imperative effects" toward "render intent publication payloads for DCS and process consumers."
- `src/ha/process_dispatch.rs`
  Much of the current process request construction would either move into the process consumer or become pure conversion from a process intent contract into receiver-executable local steps.
- `src/dcs/worker.rs`
  The DCS worker would need a receiver-owned intent-consumption path for lease and switchover actions, plus publication of DCS apply progress.
- `src/dcs/state.rs`
  New records would be needed for intent progress and possibly richer trust/authority inputs.
- `src/process/state.rs`
  `ProcessJobRequest`, `ProcessState`, and `JobOutcome` would need to become revision-aware ledger records instead of one-shot job envelopes.
- `src/process/worker.rs`
  The process worker would become the authoritative consumer of process intent revisions and publish apply-state transitions explicitly.
- `tests/ha.rs` and `tests/ha/features/`
  The HA suite would need to verify not only resulting roles but also that revised authority and convergence contracts lead to the expected single-primary and rejoin outcomes.

## Meaningful changes required by this option

This option is large. A later implementation would need to make all of the following changes deliberately.

1. Replace tick-derived process job ids with semantic `IntentRevision` identities.
2. Introduce explicit `AuthorityContract`, `LocalNodeContract`, `ProcessIntentContract`, and `DcsIntentContract` ADTs.
3. Remove late DCS/source reinterpretation from `src/ha/process_dispatch.rs` by moving it into earlier typed contracts or into receiver-local execution planning that no longer changes cluster meaning.
4. Add receiver-owned apply ledgers for the process and DCS consumers.
5. Publish apply-progress truth so future HA ticks reason over durable progress, not only over current `ProcessState`.
6. Treat startup, rejoin, failover, and switchover as ordinary intent-revision changes, not as special sender retries.
7. Keep all etcd reads/writes in the DCS layer and all Postgres subprocess interactions in the process layer.
8. Preserve partial member truth in DCS even when command application is failing or blocked.
9. Refine trust interpretation so degraded-majority authority can still publish valid revisions while truly unauthoritative states can publish only conservative stop/fence/wait revisions.
10. Add clear supersession rules so a receiver can abandon obsolete revisions without ambiguous duplicate handling.
11. Define how multiple ledgers stay aligned on the same authority epoch so a node does not process a process revision from one authority story and a DCS revision from another.
12. Update operator-facing observability so intent revision, apply state, and rejection reasons are visible during recovery.

## Migration sketch

A later implementation should not attempt to rewrite everything in one opaque jump. A reasonable migration could be:

1. Introduce pure intent-contract ADTs alongside existing `DesiredState` / `ReconcileAction` flows.
2. Make the process worker capable of tracking receiver-owned apply state for a semantic revision while still accepting legacy job requests.
3. Route one class of operations first, likely process operations, through semantic revisions.
4. Add DCS-side apply ledgers for lease actions.
5. Remove tick/action-index job id generation from HA.
6. Delete or drastically shrink `src/ha/process_dispatch.rs` once all cluster meaning is decided earlier and all remaining lowering is receiver-local.
7. Finally remove any stale legacy path so the system has one canonical intent-publication model.

The key migration rule is that once a semantic revision path becomes authoritative for a class of action, the old imperative path for that action must be deleted, not left around as fallback legacy.

## Non-goals

This option is not trying to:

- turn the HA decider into an effectful workflow engine,
- replace DCS member publication with command logs,
- eliminate the need for lease reasoning,
- hide failures behind automatic retries,
- make all command application distributed or globally serialized.

## Tradeoffs

This option improves auditability and idempotency ownership, but it adds more published state and more moving parts. It also requires sharper semantic revision rules than the current tick-derived request ids. If the revision model is weak, the system could replace one kind of ambiguity with another. The value only appears if the implementation is disciplined about semantic identity and supersession.

## Logical feature-test verification

This section explains how a later implementation of this option would logically satisfy the key HA scenarios without changing code in this task.

- `ha_dcs_quorum_lost_enters_failsafe`
  Under true authority loss, the pure decider publishes an `Unauthoritative` revision whose local contract permits only conservative fence/stop/wait behavior. Receivers may mark any previous primary-serving revision superseded and apply the new conservative revision. Operator-visible primary disappears because publication follows the authority contract, not stale process state.
- `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`
  The DCS and process consumers both receive the same fence-oriented authority revision. The DCS consumer can record lease loss application, and the process consumer can record stop or write-fencing progress. Because the revision is durable, later ticks do not need to rediscover whether fencing was already requested.
- `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`
  The majority publishes a new authoritative election or primary-serving revision. The isolated old primary cannot produce a valid competing authoritative revision because it lacks the required authority contract. Its local receiver ledger can only continue the last valid revision until superseded or fenced, preventing sender-side duplicate confusion from creating a second authoritative story.
- `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`
  After heal, the old primary observes a newer authority revision and publishes a convergence contract for itself as a former primary rejoining the winning leader. Its process consumer then applies rewind-or-basebackup progression under one receiver-owned revision family until it rejoins.
- `ha_primary_killed_then_rejoins_as_replica`
  On restart, the node reads existing intent-progress state and the current authoritative revision. It does not need the sender to rediscover a fresh ad hoc rejoin command id. It simply consumes the latest rejoin revision for its member and converges as a replica.
- `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`
  The restarted replica can consume the current follower revision directly. Because process consumers own apply state, a restart can safely tell whether the intended follower runtime is already satisfied, needs restart, or needs deeper recovery.
- `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`
  This is the currently failing long test. Under this option, the two restarted nodes do not need to rely on sender-issued transient process ids to recover cluster service. They derive or consume a fresh authority revision, publish durable apply progress, and one of them can become the authoritative primary for the new epoch while the other follows. When the final node returns, it sees the same revision family and consumes a convergence contract instead of waiting on ambiguous "unknown" sender state.
- `ha_rewind_fails_then_basebackup_rejoins_old_primary`
  The process consumer records the rewind revision as failed with classified evidence. The pure decider then publishes a superseding basebackup convergence revision. Because failure evidence is durable, the system does not keep blindly restating the same rewind command.
- `ha_replica_stopped_primary_stays_primary`
  The primary's authoritative revision remains stable. The stopped replica's receiver-owned process ledger simply shows that its follower revision is not currently satisfied. That does not require any leadership change.
- `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`
  A broken replica's process consumer can mark its rejoin revision as failed or rejected without changing the primary-serving revision on healthy majority nodes. Receiver-owned failure truth stays local to the blocked node unless the authority contract itself changes.

More broadly, this option logically improves the entire HA feature corpus because the acceptance scenarios are mostly about stable authority, safe failover, deterministic rejoin, and absence of duplicate authority. A receiver-owned ledger directly supports those goals by making command application progress durable and observable.

## Q1 Should `IntentRevision` be purely semantic, or should it include a monotonic store-backed sequence?

Context: a purely semantic revision is attractive because identical intended meaning reuses the same revision. A monotonic sequence is attractive because it makes ordering explicit even when contracts differ only subtly. The two ideas can coexist, but they optimize for different failure modes.

Problem / decision point: if the revision is too semantic, comparing supersession across store restarts or rare hash collisions may become awkward. If the revision is too sequence-driven, the system may recreate the same dedup problem under a fancier name by generating needless "new" revisions for unchanged meaning.

Restated question: should the canonical receiver key be "same meaning means same revision," or should the system always mint a new ordered revision and then separately describe semantic equivalence?

## Q2 How much of source resolution should be inside the pure contract versus inside receiver-local planning?

Context: this option says `src/ha/process_dispatch.rs` should stop rediscovering cluster meaning late. But some details, such as exact connection materialization, naturally belong near the process consumer because only it knows local filesystem and config realities.

Problem / decision point: if the pure contract includes too much source detail, it becomes brittle and over-specific. If it includes too little, the receiver may silently reconstruct important cluster meaning and recreate the very boundary drift this design is trying to remove.

Restated question: where is the precise line between "cluster meaning that must be decided centrally" and "receiver-local execution planning that may remain local"?

## Q3 Should receiver-owned progress be published into DCS, local state only, or both?

Context: publishing progress into DCS improves cluster-wide inspectability and restart recovery. Keeping some progress local is simpler and may avoid excessive DCS churn. The system already publishes member truth into DCS, so there is precedent for cluster-visible state.

Problem / decision point: cluster-wide progress publication could become noisy or stale if not designed carefully. Purely local progress could make post-restart recovery and operator diagnosis weaker, especially in multi-fault scenarios.

Restated question: which apply-progress facts are valuable enough to make cluster-visible, and which should remain receiver-local implementation detail?

## Q4 Should lease application use the same generic ledger shape as process work, or a distinct DCS-specific ledger?

Context: using one generic ledger model for both process and DCS consumers would simplify the conceptual design. A distinct DCS ledger may be clearer because lease operations have different semantics, lower latency, and different failure evidence than subprocess work.

Problem / decision point: over-generalizing may hide important differences between "subprocess still running" and "lease compare-and-swap already committed." Over-specializing may fragment the model and make it harder to reason about one authority revision spanning both consumers.

Restated question: do process and DCS consumers need one shared apply-state abstraction, or should they share only high-level concepts while keeping different concrete ledger types?

## Q5 How should the HA loop react when receiver apply state is permanently failed but authority still demands the same role outcome?

Context: a replica may repeatedly fail basebackup, or a demotion may fail because the local process is already gone. The authority contract might still be correct even when the current receiver cannot satisfy it. The next HA tick needs a deterministic response.

Problem / decision point: automatically restating the same revision forever would be noisy and misleading. Immediately minting a new revision for every retry would also be noisy and could destroy idempotency value. The design needs a principled policy for persistent failure, operator intervention, and deliberate retries.

Restated question: when the desired semantic contract is unchanged but the receiver cannot complete it, should the system keep the same revision with richer failure evidence, mint an explicit retry revision, or transition into a separate blocked state that requires human or higher-level policy action?
