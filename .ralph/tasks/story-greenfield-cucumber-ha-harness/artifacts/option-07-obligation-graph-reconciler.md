# Option 7: Obligation Graph Reconciler

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA architecture still thinks too much in terms of role labels and special-case branches, and not enough in terms of explicit obligations that must be satisfied before a node may do something dangerous. The user's complaints all point at missing obligations:

- startup decides too early, before cluster obligations are fully checked
- the HA worker deduplicates sender-side instead of letting effect consumers prove whether an obligation is already satisfied
- degraded-majority cases are incorrectly treated like "no safe path exists" because the code does not model the exact obligations required to continue
- replica recovery paths are fragmented because "follow", "rewind", and "basebackup" are not expressed as one ordered sequence of obligations

The differentiator of Option 7 is that the pure decider does not primarily emit a role, a phase, or a generation contract. It emits a graph of obligations, blockers, and satisfaction proofs. Startup and steady-state become the same operation:

`from newest observations, compute which safety and convergence obligations exist, which are already satisfied, which are blocked, and which effect consumers must act next`

Option 1 centered one unified reconciliation kernel. Option 2 centered separate cluster and local state machines. Option 3 centered lease authority. Option 4 centered one recovery funnel. Option 5 centered generation cutovers and contracts. Option 6 centered publication evidence quality. Option 7 is materially different from all of them because its primary abstraction is a typed obligation graph: every leader action, follower action, fencing step, startup decision, bootstrap path, rewind path, and publication refresh is treated as an obligation with prerequisites and completion proofs.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign studies.
- The previously captured failing themes that matter most for this option are:
  - degraded-quorum scenarios currently collapse too quickly into `FailSafe` rather than evaluating whether the remaining majority can still satisfy leader-election obligations
  - old-primary recovery scenarios do not converge cleanly because post-failover rejoin obligations are spread across startup, dispatch, and recovery branches
  - service restoration scenarios show that the current architecture does not clearly separate "must stop unsafe writes now" from "may continue serving because enough obligations are still satisfied"

This document treats those failures as planning inputs only. No production or test-code fixes are proposed or performed in this run.

## Current design problems

### Startup logic is split away from the main HA reasoning path

`src/runtime/node.rs` still contains separate startup planning and execution paths (`plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, `build_startup_actions(...)`). That means the system decides bootstrap/follow/init/reuse behavior before the long-running HA reconciliation model has a chance to apply its full safety logic. From an obligation point of view, this is backwards. Startup is just the first moment when obligations are checked. It should not be a separate planner.

### Sender-side dedup in `src/ha/worker.rs` hides the real safety question

`should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)` are sender-side attempts to infer that nothing needs doing. That is fragile because the sender does not own the truth of whether an effect was completed, partially completed, superseded, or contradicted by newer evidence. In an obligation architecture, dedup belongs to the consumer that can prove "obligation X with proof preconditions Y is already satisfied" or "this is still pending".

### HA logic is spread across multiple boundaries that each make partial decisions

The repo currently spreads authority across runtime startup code, `src/ha/decide.rs`, `src/ha/process_dispatch.rs`, and effect lowering. Each layer answers a slightly different version of "what should happen now?" That spread is exactly what an obligation graph is meant to fix. There should be one pure place that enumerates obligations and blockers, one lowerer that maps obligations to effects, and effect consumers that report completion proofs back into observations.

### Non-full-quorum currently shortcuts into `FailSafe`

`src/ha/decide.rs` currently routes non-`DcsTrust::FullQuorum` states into `HaPhase::FailSafe`, including cases where a degraded but valid majority still exists. That means the code asks "is quorum perfect?" instead of "are the obligations for safe leader service still satisfiable?" A 2-of-3 majority can satisfy leader-election and replication obligations even if some members are absent. This option makes that reasoning explicit.

### Startup and rejoin decisions are ambiguous in `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` currently acts as a bridge from HA decision to process start intent, including leader start intent, rewind, and basebackup validation. That creates a mixed boundary: process dispatch is no longer a pure consumer of typed intent, because it still contains parts of the policy. Under this redesign, dispatch consumes already-resolved obligations such as `NeedWritableLeader`, `NeedReplicaConvergence`, or `NeedDataReset`, rather than inferring intent from previous decisions.

### Member publication does not yet fully serve as obligation proof material

`src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already contain the beginnings of partial truth modeling. But the rest of the HA path still treats published truth as general context rather than as proof material. In this option, member records become explicit evidence for or against obligation satisfaction: for example, "leader lease visible", "local Postgres unreachable", "timeline mismatch", or "agent alive but SQL unknown". Partial truth is still valuable because it changes which obligations are satisfiable.

## Core design summary

The pure HA decider is replaced by a pure `ObligationPlanner` that consumes the newest cluster observation set and produces an `ObligationGraphPlan`.

The plan contains:

- cluster-level obligations that must hold for any node to consider the cluster healthy
- node-local obligations that this specific node must satisfy
- blockers that make an obligation temporarily unsatisfied but potentially resolvable
- hard violations that require fencing, demotion, or refusal to promote
- proof requirements that effect consumers can later confirm via observations

The pure planner never says only "be primary" or "be replica". Instead it says things like:

- `NeedSingleAuthoritativeLeader`
- `NeedLeaderLeaseForGeneration`
- `NeedLocalWriteFence`
- `NeedLeaderPublicationFreshness`
- `NeedReplicaConvergenceFromLeader`
- `NeedTimelineRepairViaRewind`
- `NeedReplicaRecloneViaBasebackup`
- `NeedBootstrapAuthority`
- `NeedBootstrapDataDecision`
- `NeedPublicationRefreshWithPartialTruth`

The lower layer then translates unsatisfied obligations into typed effects. Effect consumers execute those effects and later publish observations that prove the obligations are now satisfied or still blocked.

## Proposed control flow from startup through steady-state

Startup and steady-state use the same flow. The only difference is that on startup more obligations begin in the unsatisfied state.

```text
 newest local observations        newest DCS publications        durable local markers
            |                                |                             |
            +--------------- ObservationAssembler ------------------------+
                                            |
                                            v
                               ClusterObservationSnapshot
                                            |
                                            v
                                   ObligationPlanner
                                            |
             +------------------------------+-----------------------------+
             |                              |                             |
             v                              v                             v
      Cluster obligations            Local obligations                Blockers/violations
             |                              |                             |
             +------------------------------+-----------------------------+
                                            |
                                            v
                                   ObligationLowerer
                                            |
                                            v
                                   Typed effect batches
                                            |
            +-----------------------+----------------------+----------------------+
            |                       |                      |                      |
            v                       v                      v                      v
     Lease consumer         Postgres consumer       Replication consumer     DCS consumer
            |                       |                      |                      |
            +-----------------------+----------------------+----------------------+
                                            |
                                            v
                                   New observations/proofs
                                            |
                                            +------ next tick ------+
```

The key architectural rule is that only consumers can claim completion. The planner can require an obligation; it cannot assert that the obligation is already fulfilled unless current observations prove it.

## Typed state model

This option still uses typed states, but the primary type is not a role enum. It is a graph of obligations and proofs.

### Core types

The future implementation should introduce a family like this in `src/ha/decision.rs` or a new sibling module:

```text
struct ObligationGraphPlan {
    observation_epoch: ObservationEpoch,
    cluster_mode: ClusterMode,
    cluster_obligations: Vec<ClusterObligationState>,
    local_obligations: Vec<LocalObligationState>,
    blockers: Vec<ObligationBlocker>,
    violations: Vec<SafetyViolation>,
    preferred_service_posture: ServicePosture,
}

enum ClusterMode {
    BootstrapDiscovery,
    MajorityOperating,
    MajorityElecting,
    MajorityRecovering,
    NoSafeMajority,
}

enum ObligationStatus {
    Satisfied(ProofRef),
    Pending,
    Blocked(ObligationBlockerId),
    Violated(SafetyViolationId),
}

enum ClusterObligation {
    NeedSingleAuthoritativeLeader,
    NeedElectionCandidateWithMajority,
    NeedLeaderLeaseVisibility,
    NeedFreshLeaderPublication,
    NeedWriteFenceOutsideAuthority,
    NeedBootstrapAuthority,
}

enum LocalObligation {
    NeedWritableLeaderPostgres,
    NeedReadOnlyOrStoppedPostgres,
    NeedReplicaAttachedToLeader,
    NeedTimelineAlignment,
    NeedBasebackupFromLeader,
    NeedBootstrapDataDecision,
    NeedPublicationRefresh,
    NeedLocalLeaseAcknowledge,
}
```

This model is deliberately explicit:

- cluster obligations express what the cluster must satisfy before any service posture is safe
- local obligations express what this node must do to match the cluster outcome
- blockers express why an obligation is not yet satisfiable
- violations express unsafe conditions that require fencing, demotion, or refusal to serve

### Service posture

Instead of deriving all behavior from a single HA phase, the planner emits a service posture:

- `ServeWritable`
- `ServeReadOnlyReplica`
- `Converging`
- `Fenced`
- `WaitingForEvidence`
- `Bootstrapping`

This posture is descriptive, not authoritative by itself. It summarizes the graph outcome for operators and tests, but the actual action path still comes from the obligations.

### Transition model

State transitions happen whenever proofs or blockers change.

- If `NeedSingleAuthoritativeLeader` changes from `Pending` to `Satisfied`, the node may move from `WaitingForEvidence` to `ServeWritable` or `ServeReadOnlyReplica` depending on its local obligations.
- If `NeedLeaderLeaseVisibility` becomes `Violated`, a node currently serving writable must immediately acquire `NeedReadOnlyOrStoppedPostgres` and `NeedWriteFenceOutsideAuthority`.
- If `NeedTimelineAlignment` is `Blocked(WrongTimelineButRewindPossible)`, the node posture is `Converging`; when rewound successfully, the blocker is replaced by `Satisfied`.
- If `NeedTimelineAlignment` is `Violated(RewindImpossible)` and `NeedBasebackupFromLeader` becomes `Pending`, the node transitions within `Converging` without requiring separate startup-only branching.

## Quorum model redesign

This option redesigns quorum reasoning around obligation satisfiability rather than full-quorum perfection.

### Key rule

The cluster may continue writable service if and only if the planner can prove all obligations required for authoritative writable leadership are satisfied by a valid majority-compatible evidence set.

That means:

- full quorum is sufficient but not necessary
- majority evidence plus fresh leader authority may still satisfy the writable-service obligations
- missing minority members do not automatically imply `FailSafe`
- contradictory evidence or inability to prove majority authority does imply fencing or read-only posture

### Practical 2-of-3 behavior

In a three-node cluster with one node missing:

- if two members still form a valid voter majority
- and one candidate can satisfy `NeedElectionCandidateWithMajority`
- and `NeedLeaderLeaseVisibility` plus `NeedWriteFenceOutsideAuthority` remain provable
- then the cluster remains in `MajorityOperating` or `MajorityElecting`, not `NoSafeMajority`

The old primary on the minority side becomes subject to a local obligation graph that cannot satisfy `NeedWritableLeaderPostgres`. Its plan instead contains `NeedReadOnlyOrStoppedPostgres` plus `NeedWriteFenceOutsideAuthority`.

### When to fence

Fencing is required when the planner cannot prove the obligations needed for safe writable authority. That includes:

- no majority-compatible candidate exists
- lease authority cannot be observed or renewed within the cutoff model
- contradictory publications imply possible split-brain
- local node remains isolated from the authority proof set

This is sharper than the current "non-full quorum means failsafe" shortcut.

## Lease model redesign

This option treats lease facts as proofs attached to obligations, not just inputs to role choice.

### Lease obligations

The planner introduces explicit obligations:

- `NeedLeaderLeaseForGeneration`
- `NeedLeaseRenewalBeforeCutoff`
- `NeedLeaseLossFence`

For a leader candidate to serve writable, the plan must show:

- majority-compatible election proof
- active lease proof
- no newer conflicting authority proof
- publication freshness sufficient for peers to observe the leader

### Killed primary and lost lease behavior

A killed primary loses authority not merely because its process is down, but because the obligation graph for that node can no longer satisfy the lease and publication proof chain. On restart, the first startup tick observes:

- prior local data may exist
- prior role may have been primary
- current cluster authority proof belongs elsewhere

The resulting plan does not need a bespoke "old primary restart" branch. It simply cannot satisfy `NeedWritableLeaderPostgres`, but it can satisfy convergence obligations toward replica service.

### Lease and startup interaction

Startup uses the same lease obligations as steady state. A node never starts writable Postgres from local assumptions alone. It must satisfy the leader obligations from current observations. That eliminates the current startup/steady-state mismatch.

## Startup reasoning redesign

On startup, the node begins with many unsatisfied obligations and asks the same planner to resolve them.

### Cases the planner must handle

- cluster already has a healthy leader
- cluster has enough members to elect a new leader
- cluster is empty and needs bootstrap
- cluster appears empty but local `pgdata` exists
- local node holds init lock opportunity
- local data exists on a node that previously served primary
- local node is alive but pginfo is partial or unavailable

### Startup path

```text
startup tick
  -> assemble newest observations
  -> build obligation graph
  -> if bootstrap obligations are satisfied, lower bootstrap effects
  -> else if leader obligations are satisfied locally, lower leader-start effects
  -> else if replica convergence obligations are satisfiable, lower recovery effects
  -> else lower safe waiting/fencing/publication effects
```

The key change is that startup does not have a separate planner. It has only a larger set of initially unsatisfied obligations.

### Bootstrap rethink

Bootstrap should be modeled as explicit substates of `NeedBootstrapAuthority` and `NeedBootstrapDataDecision`.

Example bootstrap substates:

- `BootstrapAuthorityPending`
- `BootstrapAuthorityHeld`
- `BootstrapDataEmpty`
- `BootstrapDataReusable`
- `BootstrapDataConflicting`
- `BootstrapInitInProgress`
- `BootstrapPublishPending`

This is important because a node that wins init authority but already has reusable valid `pgdata` should not automatically discard it. The obligation graph can carry the proof question explicitly: is existing data compatible with safe bootstrap, or must the node reinitialize?

## Replica convergence as one coherent obligation sequence

Replica convergence becomes an ordered chain of local obligations:

1. `NeedReplicaAttachedToLeader`
2. `NeedTimelineAlignment`
3. `NeedCatchupWithinTolerance`
4. `NeedSteadyFollow`

If the node is already aligned and following, the obligations are satisfied and the node can serve as replica.

If the node is on the wrong timeline but rewind is possible, `NeedTimelineAlignment` is `Blocked(RewindRequired)`, which lowers to rewind effects.

If rewind is impossible, `NeedTimelineAlignment` becomes `Violated(RewindImpossible)` and `NeedBasebackupFromLeader` becomes pending. That moves the node to a different convergence branch without leaving the obligation framework.

The same sequence applies whether the node was:

- a healthy replica that briefly stopped
- an old primary after failover
- a freshly restarted node with stale data
- a node restored from previous failed recovery attempts

This removes the current special-case fragmentation.

## Partial-truth member publication

Member publication remains the responsibility of the DCS worker, but the publication schema should explicitly preserve proof-relevant partial truth.

Examples:

- agent alive, SQL unreachable
- SQL reachable, readiness unknown
- lease last seen locally, DCS publish delayed
- Postgres stopped intentionally due to fencing obligation
- timeline unknown because pginfo probe failed

The planner should treat these not as absence but as proof qualifiers. For example:

- "agent alive, SQL unreachable" is enough to prove the node is not silently gone
- "Postgres intentionally stopped due to fencing obligation" is stronger than mere absence and helps peers reason about split-brain risk

## Deduplication moves to effect consumers

This option is especially strict on dedup.

The planner emits obligation identifiers and proof requirements. The lowerer maps them to effect requests that carry:

- obligation id
- observation epoch
- requested operation
- expected completion proof shape

Consumers deduplicate by checking whether the proof already exists or whether an equivalent in-flight request already owns the same obligation id.

That means:

- the HA worker never decides "probably already active"
- the Postgres consumer decides whether the requested process state already satisfies `NeedWritableLeaderPostgres` or `NeedReadOnlyOrStoppedPostgres`
- the replication consumer decides whether a rewind or basebackup obligation is already in progress or complete
- the DCS consumer decides whether the required publication proof is already present

This is safer because the consumer owns the side effect and the evidence of completion.

## Concrete future code areas affected

A later implementation of this option would need to touch at least:

- `src/runtime/node.rs`
- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/ha/decision.rs`
- `src/ha/lower.rs`
- `src/ha/process_dispatch.rs`
- `src/dcs/worker.rs`
- `src/dcs/state.rs`
- `src/pginfo/state.rs`
- `tests/ha.rs`
- `tests/ha/features/`

Likely concrete changes include:

- remove startup-only planning from `src/runtime/node.rs` and route startup through the same observation-to-plan path
- replace current high-level `HaDecision` shape or extend it with obligation-graph output types in `src/ha/decision.rs`
- replace the current shortcut-heavy logic in `src/ha/decide.rs` with obligation satisfaction evaluation
- shrink `src/ha/process_dispatch.rs` so it becomes a consumer/lowering boundary, not a hidden policy layer
- update `src/ha/lower.rs` to lower obligation states into effect families
- remove sender-side dedup from `src/ha/worker.rs`
- enrich DCS publication fields and freshness classification in `src/dcs/worker.rs` and `src/dcs/state.rs`
- preserve proof-relevant partial state from `src/pginfo/state.rs`
- rewrite HA feature expectations where necessary to align them to explicit obligations and service postures rather than legacy internal phase names

## Meaningful changes required for this option

The future implementation would need all of the following, not just a subset:

- introduce typed obligation graph plan structs and enums
- define proof references and blocker ids as stable typed objects
- remove or drastically shrink startup-specific planning paths
- move leader safety reasoning from role branching to obligation satisfaction
- replace non-full-quorum shortcut logic with majority-compatible obligation evaluation
- express bootstrap authority and bootstrap data decisions as explicit obligations
- unify rewind and basebackup under one convergence obligation chain
- move dedup from HA sender logic into consumer-owned proof checks
- enrich member publication so proof-relevant partial truth survives publication
- define operator-facing service posture separately from the internal obligation graph
- remove stale legacy paths that infer intent from previous decisions instead of current obligations

## Migration sketch

A later implementation could migrate in this sequence:

1. Introduce obligation graph types alongside the current `HaDecision` model.
2. Add observation-to-obligation planning in parallel, initially for diagnostics only.
3. Teach `src/ha/lower.rs` to lower a small subset of obligations while preserving current behavior.
4. Remove sender-side dedup once consumers can prove completion for those obligations.
5. Route startup through the planner behind a feature branch or staged refactor, deleting the old startup planner instead of keeping compatibility paths.
6. Expand obligation coverage to leader authority, fencing, replica convergence, and bootstrap.
7. Delete the legacy role/dispatch inference paths once all major scenarios use obligation planning.

The critical migration discipline is to delete obsolete branching as soon as the new path owns that concern. This repo is greenfield and should not keep dual systems around.

## Logical feature-test verification

This section explains how the option should logically satisfy the current HA scenarios once implemented. It does not claim those tests pass today.

### `ha_dcs_quorum_lost_enters_failsafe`

If the cluster truly loses the ability to satisfy majority-compatible leader obligations, the planner emits violated authority obligations and pending fencing obligations. The node posture becomes `Fenced` or `WaitingForEvidence`, depending on the exact cutoff state. This still preserves the spirit of failsafe, but the trigger is obligation failure rather than lack of full quorum alone.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

Once lease proof or majority authority proof becomes violated beyond the cutoff model, the local graph includes `NeedReadOnlyOrStoppedPostgres` and `NeedWriteFenceOutsideAuthority`. Those obligations lower to fencing effects. Because consumers own completion proofs, write-blocking is tied to actual fenced state, not optimistic sender inference.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side can satisfy election and lease obligations even without the old primary, so it remains in `MajorityElecting` then `MajorityOperating`. The isolated old primary cannot satisfy writable-leader obligations and must transition to fenced/converging posture.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

When healed, the old primary observes that cluster authority obligations are already satisfied elsewhere. Its local graph acquires convergence obligations toward replica service. If timeline mismatch is rewindable, rewind is selected; otherwise basebackup is required. No bespoke old-primary branch is needed.

### `ha_primary_killed_then_rejoins_as_replica`

The killed primary loses its local proof chain. On restart, the startup tick builds the same obligation graph as any other node. Since leader authority is already satisfied elsewhere, the node acquires replica convergence obligations and rejoins as replica.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica returns, the planner reevaluates majority-compatible obligations. If enough proof exists to restore majority operation, the cluster posture changes from unsafe waiting/fencing toward operating. This avoids the current over-broad "still primary without enough obligations" behavior because writable service depends on explicit satisfaction, not stale role memory.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes evaluate bootstrap-versus-election obligations from current evidence. Once a safe authority path is satisfied, they operate. The final node later restarts, sees existing authority proof, and enters the convergence chain as replica.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The obligation chain handles this directly. `NeedTimelineAlignment` first lowers to rewind. If consumer feedback proves rewind impossible, that obligation is marked violated and `NeedBasebackupFromLeader` becomes pending. The node stays in `Converging` and does not destabilize cluster authority.

### `ha_replica_stopped_primary_stays_primary`

A replica loss does not by itself violate leader obligations if majority-compatible authority remains satisfied. The primary remains writable because the relevant obligations are still satisfied.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica has local unsatisfied convergence obligations, but those do not invalidate already-satisfied leader obligations on the healthy majority side. The obligation graph isolates local repair failures from cluster authority unless the failures actually remove majority safety.

## Non-goals

- This option does not try to preserve existing internal phase names.
- This option does not promise a minimal patch. It would require a structural refactor.
- This option does not put DCS or Postgres IO inside the pure planner.
- This option does not use absence of data as a substitute for proof.

## Tradeoffs

- The obligation graph is more verbose than a simple role enum, which increases type volume.
- Debugging may initially feel heavier because operators will see multiple simultaneous obligations instead of one phase string.
- The planner needs disciplined proof typing or it could become another vague state bag.
- Migration must be strict about deleting legacy branches, or the repo could end up with obligations layered on top of the old architecture instead of replacing it.

## Q1 Should service posture be derived or explicitly stored?

Context: this option keeps `ServicePosture` as a human-facing summary derived from the obligation graph.

Problem: deriving posture avoids duplicated truth, but explicit storage may improve logging and test assertions.

Restated question: should the future implementation compute operator posture from obligations each tick, or should it persist posture as a first-class field for observability and compatibility?

## Q2 How fine-grained should obligation ids be?

Context: consumers deduplicate by obligation id plus proof shape.

Problem: if ids are too coarse, independent work items collide. If ids are too fine, consumers cannot recognize semantically equivalent requests.

Restated question: what is the right granularity for obligation identity so consumers can deduplicate safely without masking distinct work?

## Q3 Should bootstrap data reuse require a separate proof family?

Context: the user explicitly wants bootstrap reconsidered, including whether existing `pgdata` may still be used after winning init authority.

Problem: data reuse is safety-critical and can easily become an implicit shortcut if modeled as only another branch of bootstrap.

Restated question: should reusable-bootstrap-data validation be its own typed proof family, separate from general bootstrap authority, so the planner cannot blur "may initialize" with "may reuse local state"?

## Q4 How much of the current `HaDecision` vocabulary should survive?

Context: this option would likely replace much of the current role/phase-centered vocabulary with obligation-centered types.

Problem: reusing too much old vocabulary risks carrying old assumptions forward, but replacing everything at once increases migration cost.

Restated question: should a future implementation preserve a thin compatibility layer around `HaDecision`, or should it delete the current vocabulary aggressively once obligation planning owns the main loop?

## Q5 Where should proof history live?

Context: the planner consumes current observations, but some obligations benefit from short proof history such as repeated lease loss, failed rewind attempts, or stale publication streaks.

Problem: storing history in the wrong layer could reintroduce hidden policy in consumers or runtime wrappers.

Restated question: should short-lived proof history be part of the pure HA planner input model, part of durable local state, or split between the two depending on proof type?
