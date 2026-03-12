# HA Refactor Option 6: Bootstrap Charter Machine

This is a design artifact only. It does not propose code changes in this task, it does not treat green tests as the goal of this task, and it does not authorize fixing production behavior during this run. The purpose of this document is to describe one complete redesign option in enough detail that a later implementation task can execute it without reopening chat history, repo documentation, or prior artifacts.

## Why this option exists

This option exists because bootstrap remains the least explicit authority transition in the current architecture. The live code already has several hints that bootstrap is special: DCS carries an `init_lock`, the disabled runtime startup tests still refer to `select_startup_mode(...)` and `build_startup_actions(...)`, `candidacy_kind(...)` in `src/ha/decide.rs` distinguishes bootstrap from failover, and `process_dispatch.rs` still decides how to materialize `InitDb`, `BaseBackup`, and `StartReplica` from later-stage intents. The differentiator for this option is that bootstrap itself becomes a first-class authority machine called a `BootstrapCharter`. Instead of treating init-lock as one Boolean-ish gate and then falling back into ordinary leader logic, the system explicitly proves which bootstrap charter is active, what durability claims it carries, whether existing local data can satisfy that charter, and when bootstrap authority must yield to normal lease authority. That makes this option materially different from option 1's broad regime classifier, option 2's epoch-story focus, option 3's universal recovery funnel, option 4's receiver-owned idempotency focus, and option 5's majority-proof evidence matrix.

## Ten option set decided before drafting

These are the ten materially different directions this design study will use. This document fully specifies only option 6.

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
   Replica repair, rewind, basebackup, and steady following become one graph of convergence edges with typed prerequisites.
8. `Publication-first truth model`
   Member publication is redesigned first so all other HA logic consumes richer partial-truth envelopes.
9. `Split loops with shared ADTs`
   Startup and steady-state remain separate loops, but they must consume the exact same world and intent ADTs.
10. `Authority contract DSL`
    Leadership, fencing, and switchover are encoded as typed contracts that can be model-checked independently of Postgres actions.

## Diagnostic inputs on March 12, 2026

This option uses the current repository state as input evidence.

- `make test` was run on March 12, 2026 and completed successfully: `309` tests passed, `26` were skipped by profile policy, and nextest reported `3 leaky` tests in the run summary. That matters here because the bootstrap-charter argument is not "the whole tree is red." It is that the authority boundary around initialization, first leadership, and restart after cluster-wide outage is still under-specified even in a mostly green tree.
- `make test-long` was run on March 12, 2026 and completed with `25` HA scenarios passed, `1` failed, and `4` skipped by profile policy. The failing scenario was `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, which timed out while waiting for one primary across the two restarted fixed nodes and reported the two restarted nodes as unknown. That is directly relevant to this option because a full-stop restart is exactly the situation where a bootstrap-like authority charter must say whether old durable state, visible membership evidence, and surviving majority are sufficient to start a new authoritative leader.
- `tests/ha.rs` remains the later implementation acceptance surface. A future implementation based on this option must satisfy those HA scenarios or revise them with explicit new semantics instead of accidental behavior drift.

## Current design problems

### 1. Bootstrap authority exists in pieces, not as one typed proof

The repository already contains multiple bootstrap-related fragments:

- `src/dcs/state.rs` exposes `init_lock: Option<InitLockRecord>`.
- `src/ha/decide.rs` uses `candidacy_kind(...)` to distinguish `Candidacy::Bootstrap`, `ResumeAfterOutage`, and `Failover`.
- `src/ha/lower.rs` still lowers `RecoveryStrategy::Bootstrap` as a distinct path.
- `src/ha/process_dispatch.rs` translates `ReconcileAction::InitDb` into a bootstrap process job.
- `src/runtime/node.rs` still contains disabled startup-planning references and tests around `select_startup_mode(...)`, `select_resume_start_intent(...)`, and `build_startup_actions(...)`.

Those are all real clues, but today they are not one coherent authority model. Bootstrap is partly a runtime concern, partly a DCS lock concern, partly a candidate label, and partly a process job kind. That means the system does not have one pure answer to the question: "Why is this node allowed to create or claim the first authoritative cluster state right now?"

### 2. `init_lock` is too thin to describe real bootstrap state

`InitLockRecord` currently only tells us who holds the lock. That is not enough. A valid bootstrap decision needs more than holder identity:

- whether the holder proved the cluster was empty,
- whether bootstrap already seeded DCS config,
- whether bootstrap already initialized local `pgdata`,
- whether bootstrap already started Postgres,
- whether bootstrap already published itself as primary,
- whether the charter timed out or became superseded,
- whether another node may safely resume or must wipe and follow instead.

Treating bootstrap as a single lock holder loses the difference between "bootstrap reserved but nothing durable happened yet" and "bootstrap seeded durable cluster identity and cannot be replayed blindly."

### 3. Startup reasoning is still split across runtime and HA

The live runtime path enters steady workers through `run_workers(...)`, and the older explicit startup planner survives only as disabled references in `src/runtime/node.rs`. That means the repo is already in an in-between state. The old planner is not the live path, but the architectural idea behind it has not been fully absorbed into the steady HA model either. The user explicitly wants "same newest info + same state => same actions" on startup and later ticks. As long as bootstrap authority is not derived inside the same typed observation pipeline as failover and follow decisions, startup remains a special side story.

### 4. The current decision path asks generic failover logic to answer bootstrap questions

`src/ha/decide.rs` builds leader, follower, and candidacy goals from the current `WorldView`. That works reasonably for steady-state failover, but bootstrap questions are different:

- Is there already a durable cluster identity?
- Did a previous bootstrap partially complete?
- Can this node reuse existing `pgdata` as the winning initializer?
- Does a leader record exist without a healthy leader process?
- Is the visible state a brand-new cluster, a resumed outage, or a superseded unfinished init?

Those are not merely "pick best candidate" questions. They are bootstrap charter questions. Pushing them into general candidacy selection makes the reasoning too opaque and too easy to accidentally change.

### 5. Process dispatch still recovers bootstrap semantics too late

`src/ha/process_dispatch.rs` decides how to materialize `InitDb`, `BaseBackup`, `PgRewind`, `StartPrimary`, and `StartReplica`. That is a necessary lower layer, but it still reconstructs authority details when choosing remote sources and start intents. For bootstrap, this means the system does not have an earlier pure artifact that says:

- which bootstrap charter is active,
- which node owns it,
- what step is being resumed,
- whether the current node is initializing, resuming, yielding, or following,
- whether local data should be reused, wiped, or ignored.

Under this option, lowering must consume a complete bootstrap contract rather than infer bootstrap meaning from generic actions.

### 6. Partial truth publication is good enough for DCS freshness, but not yet rich enough for charter safety

`src/pginfo/state.rs` and `src/dcs/worker.rs` already preserve useful partial truth:

- `SqlStatus::{Unknown,Healthy,Unreachable}`,
- `Readiness::{Unknown,Ready,NotReady}`,
- `PgInfoState::{Unknown,Primary,Replica}`,
- DCS member-slot publication even when Postgres is not fully healthy.

That is the correct direction. The gap is that bootstrap safety depends on more than Postgres health. The bootstrap charter needs publication of bootstrap-specific partial truth:

- local data directory shape,
- whether `initdb` has run,
- whether local cluster identity exists,
- whether a bootstrap process is in flight,
- whether bootstrap completed DCS seeding,
- whether this node is alive but blind.

Without that, one node cannot safely reason about whether another node already made durable bootstrap progress.

## Core proposal

The system must introduce a first-class `BootstrapCharter` ADT and require every startup and every bootstrap-adjacent steady-state tick to resolve bootstrap authority before ordinary lease authority.

The key idea is:

1. Every tick builds one `ObservationEnvelope`.
2. A pure `bootstrap_charter(...)` classifier determines whether the cluster is in a bootstrap-sensitive state.
3. If a bootstrap charter is active, the decider produces a `BootstrapContract`.
4. Only when bootstrap is conclusively inactive or completed does the decider proceed to normal lease/failover authority.
5. Receivers still own idempotency, but now they consume a richer contract that says exactly where bootstrap authority stands.

This means bootstrap is not just "another recovery strategy." It is a temporary authority regime with its own proof object, invariants, and completion rules.

## Proposed control flow from startup through steady state

### Startup tick

1. The node process starts workers exactly as today, but the HA worker's first observation must include:
   `DcsObservation`, `PgObservation`, `ProcessObservation`, `DataDirObservation`, and `BootstrapLocalEvidence`.
2. The pure classifier runs `bootstrap_charter(observation) -> BootstrapCharterView`.
3. If the charter says bootstrap authority is unresolved, the node may only publish truth and wait.
4. If the charter says this node owns a valid charter, the decider emits a `BootstrapContract::Execute(...)` or `BootstrapContract::Resume(...)`.
5. If the charter says another node owns the charter, the decider emits `BootstrapContract::YieldAndFollow(...)` or `BootstrapContract::ObserveOnly(...)`.
6. Once the charter reaches `Completed`, bootstrap authority is converted into ordinary leader lease authority and the node moves into the normal lease state machine.

### Steady-state tick

1. Every new observation still runs through the same `bootstrap_charter(...)` classifier first.
2. Most steady-state ticks resolve immediately to `BootstrapCharterView::Inactive`, which means "continue with ordinary lease logic."
3. Rejoin after total outage, init-lock leftovers, or partially completed init sequences reactivate the classifier and force the system back through explicit bootstrap-charter reasoning before a leader is declared.
4. The lowerer emits `IntentEnvelope`s tagged by charter revision or lease epoch, and DCS/process consumers apply idempotently at that boundary.

### ASCII diagram

```text
        startup tick / restart tick / steady-state tick / DCS refresh
                                 |
                                 v
                 +--------------------------------------+
                 | ObservationEnvelope                  |
                 | - dcs trust + member slots           |
                 | - leader lease / switchover / lock   |
                 | - pginfo snapshot                    |
                 | - process job snapshot               |
                 | - local data-dir evidence            |
                 | - bootstrap local evidence           |
                 +------------------+-------------------+
                                    |
                                    v
                 +--------------------------------------+
                 | bootstrap_charter(...)               |
                 |                                      |
                 | BootstrapCharterView                 |
                 | - inactive                           |
                 | - candidate                          |
                 | - reserved                           |
                 | - seeding                            |
                 | - initializing                       |
                 | - publishing                         |
                 | - completed                          |
                 | - aborted / superseded               |
                 +------------------+-------------------+
                                    |
                   +----------------+----------------+
                   |                                 |
                   v                                 v
    +--------------------------------+   +-------------------------------+
    | BootstrapContract              |   | LeaseAuthorityContract        |
    | execute / resume / yield /     |   | lead / follow / fail-safe /   |
    | observe / wait                 |   | recover / switchover          |
    +----------------+---------------+   +-------------------------------+
                     | 
                     v
          +--------------------------------------+
          | lower_contracts(...)                 |
          | IntentId = {authority_scope, rev}    |
          | dcs intents + process intents        |
          +------------------+-------------------+
                             |
                 +-----------+-----------+
                 |                       |
                 v                       v
        +------------------+   +----------------------+
        | DCS consumer     |   | Process consumer     |
        | owns idempotency |   | owns idempotency     |
        +------------------+   +----------------------+
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
    bootstrap_local: BootstrapLocalEvidence,
    previous_authority: Option<AuthoritySummary>,
}

enum AuthorityResolution {
    Bootstrap(BootstrapCharterView),
    Lease(LeaseAuthorityView),
}

enum BootstrapCharterView {
    Inactive(InactiveReason),
    Unresolved(UnresolvedCharter),
    Active(ActiveBootstrapCharter),
    Completed(CompletedBootstrapCharter),
    Superseded(SupersededBootstrapCharter),
}

struct ActiveBootstrapCharter {
    charter_id: BootstrapCharterId,
    owner: MemberId,
    stage: BootstrapStage,
    durability: BootstrapDurability,
    empty_cluster_proof: EmptyClusterProof,
    seed_plan: SeedPlan,
}

enum BootstrapContract {
    WaitForEvidence(WaitForEvidenceContract),
    Execute(ExecuteBootstrapContract),
    Resume(ResumeBootstrapContract),
    YieldAndFollow(YieldBootstrapContract),
    ObserveOnly(ObserveBootstrapContract),
    DeclareCompleted(CompleteBootstrapContract),
    AbandonAndFence(AbandonBootstrapContract),
}
```

### Charter identity and durability

Bootstrap needs a durable identity, not just a lock holder.

```text
struct BootstrapCharterId {
    cluster_bootstrap_nonce: String,
    proposer: MemberId,
    created_at: UnixMillis,
}

enum BootstrapDurability {
    ReservedOnly,
    DcsSeeded {
        config_seeded: bool,
        charter_published: bool,
    },
    LocalDataInitialized {
        system_id_known: Option<String>,
        timeline_hint: Option<u64>,
    },
    PostgresStarted {
        primary_visible: bool,
    },
    Completed {
        leader_lease_generation: u64,
    },
}
```

This is the heart of the option. A bootstrap owner that only reserved a charter can be replaced under stricter rules than a bootstrap owner that already seeded durable DCS state or initialized local storage.

### Bootstrap stages

```text
enum BootstrapStage {
    Candidate,          // no owner yet, evaluating empty-cluster proof
    Reserved,           // charter chosen, DCS reservation not yet durable enough
    SeedingDcs,         // config / charter metadata / init markers being written
    InitializingLocal,  // initdb or equivalent local cluster creation in progress
    StartingPrimary,    // postgres start requested, awaiting visible primary truth
    PublishingPrimary,  // publish self as authoritative initial leader
    Completed,          // ordinary lease authority now takes over
    Aborted,            // timed out, contradicted, or superseded before completion
}
```

### Empty-cluster proof

This option refuses to bootstrap without an explicit proof object.

```text
struct EmptyClusterProof {
    visible_members: Vec<MemberEvidence>,
    visible_leader_lease: Option<LeaderLeaseEvidence>,
    visible_init_lock: Option<InitLockEvidence>,
    local_data_claim: LocalDataClaim,
    decision: EmptyClusterDecision,
}

enum EmptyClusterDecision {
    ProvenEmpty,
    NotEmpty { reason: NotEmptyReason },
    Ambiguous { reason: AmbiguousBootstrapReason },
}
```

Examples:

- `ProvenEmpty`: no leader lease, no active member proof of an existing primary, no surviving charter, local data either absent or explicitly acceptable for winner reuse.
- `NotEmpty`: another primary is visible, leader lease exists, or prior bootstrap completion is already durable.
- `Ambiguous`: DCS is degraded, local data suggests an old cluster exists, or another member might have seeded config but current visibility is incomplete.

### Local bootstrap evidence

`PgInfoState` and `DataDirState` are not enough for bootstrap safety. The future design should add a local observation object like:

```text
struct BootstrapLocalEvidence {
    data_dir_kind: LocalDataDirKind,
    pg_version_file_present: bool,
    local_system_id: Option<String>,
    auto_conf_role_hint: Option<LocalRoleHint>,
    last_bootstrap_job: Option<JobOutcomeSummary>,
    bootstrap_marker: Option<BootstrapMarker>,
}
```

That lets a restarting node tell the difference between:

- empty directory,
- reserved charter but no initdb yet,
- initdb completed but Postgres never started,
- primary started and wrote cluster identity,
- replica clone data present and should not be reused as bootstrap winner.

### Invariants

This option depends on strict invariants.

1. At most one `ActiveBootstrapCharter` may exist per cluster scope.
2. A charter in `ReservedOnly` durability may be superseded only if no later durable side effects are visible.
3. Once `LocalDataInitialized` is reached, any replacement owner must treat the old owner's local durability as real evidence and must not silently rerun bootstrap semantics elsewhere.
4. A charter cannot become `Completed` until the node both:
   observes itself as a running primary, and
   has published the authoritative bootstrap completion record or equivalent lease handoff proof.
5. Lease authority may not contradict active charter authority. If a charter is still `Active`, the lease resolver must consume that fact rather than bypass it.
6. Partial truth is always publishable. A node that is alive but whose pginfo is degraded must still publish bootstrap-local evidence if available.

## Charter-state transitions

### Candidate -> Reserved

Trigger:

- majority-valid DCS visibility or equivalent bootstrap-safe proof,
- `EmptyClusterDecision::ProvenEmpty`,
- no existing active or completed charter,
- this node wins deterministic charter election.

Result:

- DCS consumer writes a charter reservation record and init lock with charter metadata.

### Reserved -> SeedingDcs

Trigger:

- reservation acknowledged in DCS,
- no conflicting owner proof,
- config and cluster bootstrap metadata need publication.

Result:

- DCS actions seed cluster config, charter metadata, and initial authority scaffolding.

### SeedingDcs -> InitializingLocal

Trigger:

- DCS seed writes acknowledged,
- local data is absent or allowed for winner reuse.

Result:

- process consumer runs bootstrap/init job.

### InitializingLocal -> StartingPrimary

Trigger:

- bootstrap process outcome successful,
- local bootstrap marker updated,
- no superseding evidence appears.

Result:

- process consumer materializes primary start config and starts Postgres.

### StartingPrimary -> PublishingPrimary

Trigger:

- pginfo reports local primary or sufficiently strong primary-start evidence,
- local system identity matches charter expectations.

Result:

- DCS consumer publishes authoritative initial primary and lease handoff seed.

### PublishingPrimary -> Completed

Trigger:

- published primary truth is visible,
- ordinary leader lease is established or bootstrap-completion record proves conversion,
- no conflicting primary evidence exists.

Result:

- the charter closes,
- authority resolution switches to the ordinary lease machine on subsequent ticks.

### Any active stage -> Superseded or Aborted

Trigger examples:

- another completed charter becomes visible,
- a durable foreign primary already exists,
- local init job fails irrecoverably,
- DCS visibility becomes ambiguous after durable side effects began,
- the owner disappears before completing and the cluster can prove a safe successor path.

Result:

- the decider emits an abandonment or yield contract,
- receivers fence or stop unsafe work,
- later ticks convert the node into follow/recover behavior instead of ad hoc retry.

## Bootstrap and ordinary lease interplay

This option does not replace leases. It orders authority layers.

1. `BootstrapCharterView::Inactive` means ordinary lease reasoning is authoritative.
2. `BootstrapCharterView::Active` means no ordinary failover or election path may ignore the charter.
3. `BootstrapCharterView::Completed` yields a typed handoff into `LeaseAuthorityView`.
4. Rejoin after total outage may reactivate bootstrap-charter reasoning even when the cluster is not literally brand new, because the correct question is "do we need chartered re-establishment of primary authority?" not only "is the data directory empty?"

This is what fixes the semantic gap behind the failing restart test: the two restarted nodes should not stay in generic unknown/awaiting-leader limbo if they can prove a valid chartered restart authority story.

## Redesigned quorum model

The current tree still has a blunt `DcsTrust::{FullQuorum,Degraded,NotTrusted}` boundary in `src/dcs/state.rs`, and earlier task research showed non-full quorum flowing too quickly toward fail-safe. Under this option:

- `DcsTrust` remains a DCS-health input, not the HA authority output.
- Bootstrap authority requires a bootstrap-specific majority proof:
  a proof that the cluster is empty, resumable, or charter-safe.
- A 2-of-3 majority may continue normal operation or form a restart charter even if one node is absent.
- The system only enters fail-safe when it lacks a valid lease authority proof and lacks a valid bootstrap charter proof.

Concrete outcomes:

- Healthy 2-of-3 majority with known leader:
  continue under lease authority.
- Healthy 2-of-3 majority after total outage with no active primary:
  derive a restart/bootstrap charter if the visible data and DCS evidence allow it.
- True ambiguity with contradictory leader or charter evidence:
  publish no primary, fence unsafe writes, and wait for a stronger proof.

This keeps degraded-but-valid majority operation alive without collapsing genuinely ambiguous states into unsafe bootstrap retries.

## Lease model

Lease thinking becomes stronger by explicitly separating bootstrap authority from lease authority.

1. Bootstrap authority owns the right to create or re-establish the first post-empty or post-blackout primary fact.
2. Lease authority owns ordinary steady-state leadership after bootstrap completion.
3. A killed primary loses ordinary authority when its lease cannot be corroborated by majority proof.
4. A bootstrap owner that dies before completion does not automatically remain authoritative forever; the charter's durability stage determines whether another node may resume, supersede, or only wait.
5. The final bootstrap stage must produce a typed handoff:
   `Completed { leader_lease_generation }`, which becomes the first ordinary lease epoch.

This prevents the system from confusing "I used to have a lease" with "I am still allowed to define cluster identity after a restart."

## Startup reasoning

This option makes startup explicit and uniform.

### Case 1: Cluster already up

- leader lease visible,
- healthy primary visible,
- no active bootstrap charter.

Outcome:

- startup resolves directly to ordinary lease authority,
- node follows or resumes according to local data.

### Case 2: Cluster not empty, but active primary not visible

- member slots or local data suggest existing cluster history,
- no current primary process visible,
- two fixed nodes restarted after outage.

Outcome:

- do not blindly initialize,
- evaluate a restart/bootstrap charter based on durable history and visible majority evidence,
- if valid, one node becomes charter owner and re-establishes primary authority,
- the other node yields and follows.

### Case 3: Truly empty cluster

- no prior leader proof,
- no surviving members with durable primary claim,
- local data absent or explicitly suitable for winner bootstrap.

Outcome:

- deterministic charter election,
- owner reserves charter, seeds DCS, initializes local data, starts primary, publishes completion.

### Case 4: Existing local `pgdata` on would-be initializer

This is where the option is deliberately stricter than a generic empty-vs-existing split.

- existing local data may be reused only if the charter explicitly classifies it as winner-valid bootstrap data,
- otherwise the node may only resume, follow, rewind, or basebackup,
- winner reuse must be encoded as a typed claim, not inferred ad hoc from filesystem presence.

### Case 5: Init lock present but leader missing

Current disabled runtime tests already highlight this shape. Under this option:

- the system does not merely say "init lock blocks initialize" or "member records suggest clone";
- it asks which charter stage the init lock represents,
- whether that charter is still durable,
- whether the holder is resumable, superseded, or failed before durable progress.

That is much clearer than treating `init_lock` as a generic veto.

## Replica convergence under this option

Bootstrap chartering does not replace the desired single convergence path. It clarifies when that path begins.

Once bootstrap authority is inactive or completed, replicas follow one unified convergence contract:

1. If local data already matches leader lineage and upstream, keep following.
2. If local data is slightly behind but compatible, start or continue streaming.
3. If local data is on the wrong timeline but rewindable, run rewind.
4. If rewind is impossible or already failed with fallback required, run basebackup.
5. If the node was the old primary, previously a replica, or freshly restored, that only changes the evidence and chosen edge, not the overall convergence framework.

What changes here is that convergence no longer needs to guess whether the leader it is following was born from bootstrap, restart charter, or normal failover. The authority model tells it explicitly.

## Partial-truth publication redesign

Member publication must stay rich and become bootstrap-aware.

Future DCS publication should include a bootstrap-aware envelope like:

```text
struct MemberTruthEnvelope {
    routing: RoutingTruth,
    postgres: PostgresTruth,
    bootstrap: BootstrapTruth,
    freshness: FreshnessTruth,
}

struct BootstrapTruth {
    charter_id: Option<BootstrapCharterId>,
    charter_stage: Option<BootstrapStage>,
    local_data_kind: LocalDataDirKind,
    last_bootstrap_outcome: Option<JobOutcomeSummary>,
    can_resume_charter: Option<bool>,
}
```

This preserves the existing principle that "pginfo failed but pgtuskmaster is up" is still useful truth, while also making bootstrap safety observable cluster-wide.

## Deduplication boundary

This option keeps the task-wide requirement that dedup must move out of sender-side HA logic.

What changes:

- HA emits `IntentEnvelope`s keyed by `AuthorityScope`.
- `AuthorityScope` is either `BootstrapCharterId` or `LeaseEpoch`.
- DCS and process consumers record last-applied revision per authority scope and intent kind.

That means the HA loop never asks "should I skip redundant process dispatch?" based on local sender heuristics. Instead:

- if the charter is still `StartingPrimary` revision 3, the process consumer knows whether `StartPrimary` revision 3 already applied;
- if the lease epoch is unchanged and the replica-follow intent revision is unchanged, the process consumer ignores the duplicate safely;
- if the charter changes, consumers naturally treat the new authority scope as distinct work.

This boundary is safer because idempotency lives where side effects are visible.

## Concrete file, module, function, and type changes a future implementation would touch

This design would later require implementation changes in at least these areas:

- `src/ha/worker.rs`
  replace ad hoc world-build ordering with explicit bootstrap-charter-first authority resolution.
- `src/ha/decide.rs`
  split bootstrap-charter classification from ordinary lease decision logic.
- `src/ha/decision.rs`
  replace generic `RecoveryStrategy::Bootstrap` with richer charter-derived contracts and outcomes.
- `src/ha/lower.rs`
  lower charter contracts into authority-scoped DCS/process intents rather than plain bootstrap recovery effects.
- `src/ha/process_dispatch.rs`
  stop reconstructing bootstrap semantics late; consume explicit charter contracts and authority-scoped source selection.
- `src/dcs/state.rs`
  expand `InitLockRecord` into richer charter metadata or add a new bootstrap-charter record family.
- `src/dcs/worker.rs`
  publish bootstrap-aware partial truth in member slots or a related charter channel.
- `src/pginfo/state.rs`
  possibly extend local observation wiring so bootstrap-local evidence can be surfaced without guessing from pginfo alone.
- `src/process/state.rs` and `src/process/worker.rs`
  track bootstrap job outcomes in a way the HA layer can consume as charter evidence, not only as generic last outcome.
- `src/runtime/node.rs`
  delete or fully absorb stale startup-planning remnants so bootstrap reasoning exists only in the shared HA observation pipeline.
- `tests/ha.rs` and HA feature files
  later implementation should add or refine scenarios around partial bootstrap progress, charter supersession, and restart-majority charter formation.

## Meaningful implementation changes required by this option

1. Introduce `BootstrapCharterId`, `BootstrapStage`, `BootstrapDurability`, and `BootstrapContract`.
2. Replace binary `init_lock` semantics with charter metadata that encodes stage and durability.
3. Add bootstrap-local evidence to the observation model.
4. Ensure the HA decider always resolves charter authority before lease authority.
5. Move bootstrap intent identity to authority-scoped consumer idempotency.
6. Replace generic bootstrap recovery lowering with explicit charter execution and completion intents.
7. Rework startup/restart handling so "empty cluster" and "restart authority re-establishment" are both charter problems, not ad hoc branches.
8. Publish bootstrap-aware partial truth through DCS.
9. Remove stale legacy startup planner paths instead of leaving dead alternatives in the tree.
10. Add test coverage for charter interruption, charter supersession, and restart-majority bootstrap proofs.

## Migration sketch

The future implementation should migrate in phases and aggressively delete stale paths.

### Step 1: Add observation types without changing behavior

- add `BootstrapLocalEvidence`,
- add richer bootstrap fields to HA snapshots,
- keep current decisions but log charter classification for comparison.

### Step 2: Introduce bootstrap-charter classifier in parallel

- implement `bootstrap_charter(...)` as a pure function,
- derive `BootstrapCharterView` from live observations,
- prove the classifier on current HA fixtures and restart scenarios.

### Step 3: Replace bootstrap-specific branches in decision logic

- remove generic `Candidacy::Bootstrap` shortcuts,
- emit `BootstrapContract` when charter authority is active,
- keep lease resolution only for `Inactive` or `Completed` charter states.

### Step 4: Move lowering to authority-scoped contracts

- revise `src/ha/lower.rs` and `src/ha/process_dispatch.rs`,
- consumers own idempotency by `(authority_scope, intent_kind, revision)`.

### Step 5: Expand DCS schema and delete the old `init_lock` semantics

- replace thin lock-only state with charter metadata,
- remove code that assumes `init_lock` is just a blocker.

### Step 6: Delete stale runtime startup planning

- remove disabled startup-planner remnants from `src/runtime/node.rs`,
- ensure startup is fully represented by the HA observation pipeline.

### Step 7: tighten tests

- update the HA feature suite for restart-majority charter behavior,
- add cases for interrupted bootstrap and charter resume/supersession,
- ensure no legacy bootstrap path remains silently callable.

## Logical feature-test verification

This section explains how the option should satisfy the key HA scenarios from `tests/ha.rs`.

### `ha_dcs_quorum_lost_enters_failsafe`

When DCS loses true quorum and neither lease authority nor bootstrap charter authority can be proven, the system emits no operator-visible primary and enters fail-safe. This option does not weaken fencing under genuine authority loss.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

Same as above, but explicitly: once authority proof disappears, the active leader cannot rely on stale bootstrap or stale lease memory. It must fence writes after the cutoff because neither authority layer remains valid.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The isolated old primary loses lease authority because majority proof moved elsewhere. The majority side uses ordinary lease failover, not bootstrap chartering, because the cluster is already established and a valid lease-successor story exists.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

The healed old primary sees a completed foreign lease story and an inactive bootstrap charter. It therefore enters ordinary convergence and rejoins as a replica rather than trying to reinterpret its own old primary data as bootstrap authority.

### `ha_primary_killed_then_rejoins_as_replica`

No bootstrap charter is needed if the majority still has clear leader continuity or elects a new leader normally. The returning old primary follows the unified convergence contract.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one healthy replica returns and re-establishes quorum with the primary, bootstrap chartering remains inactive because leader continuity still exists. Ordinary lease authority continues.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

This is the central test for this option. After a full cluster stop, the two restarted fixed nodes do not stay in generic "unknown leader" limbo. They run bootstrap-charter classification:

- if old durable cluster identity plus visible majority evidence proves restart-safe re-establishment,
- one node wins the restart charter,
- that node starts or resumes authoritative primary publication,
- the other yields and follows,
- the final node later rejoins via ordinary convergence.

If the evidence is genuinely ambiguous, the cluster waits and publishes no primary, but the desired design is to make the 2-of-3 restart case provable rather than accidentally ambiguous.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

Bootstrap chartering is inactive because the leader is already authoritative. The rejoining node follows the convergence contract and falls back from rewind to basebackup exactly as the user wants.

### `ha_replica_stopped_primary_stays_primary`

A stopped replica does not activate bootstrap chartering. The primary's lease authority remains valid, so service stays stable.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

Because bootstrap authority is explicit and separate, a broken rejoin attempt cannot accidentally destabilize the cluster by being misread as fresh bootstrap evidence. The majority leader continues; the broken replica stays in a recover/wait path.

## Tradeoffs

This option has real costs.

- It adds a substantial new ADT family and therefore implementation complexity.
- It requires DCS schema changes instead of pretending the existing `init_lock` record is sufficient.
- It forces careful thought about durable bootstrap metadata and upgrade of test fixtures.
- It may overlap with some benefits of option 2 or option 3, but it chooses to optimize first for clarity around initial authority rather than around lease lineage or recovery sequencing alone.

Those costs are acceptable if the team believes bootstrap and restart authority are the architectural blind spots most likely to keep producing ambiguous restart behavior.

## Non-goals

- This option does not advocate putting DCS IO or Postgres IO inside the pure decider.
- This option does not weaken fail-safe under genuine authority ambiguity.
- This option does not keep legacy runtime startup planners for compatibility.
- This option does not treat bootstrap as a permanent separate runtime loop; it remains one authority layer inside the shared HA reconciliation model.
- This option does not assume empty-cluster bootstrap and outage restart are identical. It gives them the same charter framework but different proof rules.

## Q1 Should restart-after-total-outage reuse the bootstrap charter machinery or only borrow its proof vocabulary?

Context:
The failing long HA scenario suggests that the cluster currently struggles to re-establish one primary when all nodes were down and only two fixed nodes return first. This document solves that by reusing the charter machine for both true empty-cluster bootstrap and majority restart authority.

Problem:
That design is coherent, but it may also blur an important distinction between "brand-new cluster creation" and "existing cluster authority re-establishment after outage."

Restated question:
Should the later implementation literally route restart-majority recovery through the same `BootstrapCharterView`, or should it create a sibling `RestartCharterView` that shares most proof logic while keeping terminology and invariants separate?

## Q2 What exact durable metadata should replace the thin `init_lock` record?

Context:
Today `InitLockRecord` only stores `holder`. This option wants charter identity, stage, and durability evidence to be durable and observable.

Problem:
Adding too little metadata recreates today's ambiguity, but adding too much may bloat DCS state or create migration overhead.

Restated question:
What is the smallest durable charter schema that still lets a future node prove whether a prior bootstrap owner may be resumed, superseded, or must be treated as the authoritative source of cluster identity?

## Q3 When may existing local `pgdata` be reused by the charter winner?

Context:
The user explicitly wants bootstrapping reconsidered, including whether existing `pgdata` can still be used when the node wins the init lock. This option introduces a typed winner-reuse claim instead of inferring reuse from directory presence.

Problem:
If the reuse rules are too strict, the system performs unnecessary destructive bootstrap work. If they are too loose, the cluster may accidentally crown stale or malformed local state as new authority.

Restated question:
What exact local evidence must exist before a charter winner is allowed to reuse existing `pgdata` as bootstrap-valid durable state instead of wiping and rebuilding?

## Q4 Should bootstrap-aware partial truth live inside member slots or in a separate charter key family?

Context:
This option proposes richer bootstrap-aware publication. The simplest path is to extend member publication. A cleaner boundary might be separate charter records owned by the DCS layer.

Problem:
Embedding bootstrap truth into member slots keeps one observation channel, but it mixes durable charter state with per-member freshness. Splitting it out clarifies ownership, but it increases join complexity in the observation builder.

Restated question:
Should the future implementation publish bootstrap charter truth as part of member-slot envelopes, or should it introduce separate charter keys and accept the extra cross-key reconciliation cost?

## Q5 What should happen when a charter owner disappears after `LocalDataInitialized` but before `Completed`?

Context:
This is the hardest safety edge. At that point some durable side effects already exist, but ordinary lease authority may not yet be valid.

Problem:
Immediate supersession risks split-brain or divergent initialization. Waiting forever risks permanent cluster unavailability. Automatic resumption by another node may require remote evidence the current system does not yet publish.

Restated question:
After bootstrap durability has crossed into local initialization, what exact proof should authorize a different node to resume or supersede the charter without unsafe duplication of bootstrap side effects?
