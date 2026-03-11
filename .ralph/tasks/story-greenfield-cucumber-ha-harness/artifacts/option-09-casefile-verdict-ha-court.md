# Option 9: Casefile Verdict HA Court

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. This document exists only to describe one complete redesign option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA design still mixes three separate concerns that should be easier to reason about independently:

- what the newest evidence says
- what the system believes that evidence means
- what actions are justified by that belief

The current code already has a partial functional chain, but the meaning of the evidence still becomes distributed across startup planning in `src/runtime/node.rs`, pure-ish decision logic in `src/ha/decide.rs`, bridging in `src/ha/process_dispatch.rs`, and sender-side dedup behavior in `src/ha/worker.rs`. That spread makes it too hard to answer a simple operator question: "What exact case does the node think it is in right now, and what verdict follows from that case?"

The differentiator of Option 9 is that every HA tick, including startup, produces one explicit `HaCasefile`. The casefile is the authoritative typed summary of the newest observations, the admissible evidence quality, the cluster legitimacy assessment, the local data fitness, and the unresolved contradictions. A pure `HaCourt` then applies a deterministic rulebook to that casefile and emits one `HaVerdict`. Lower layers turn the verdict into idempotent effects.

The conceptual chain becomes:

`newest observations -> casefile -> verdict -> lowered remedies -> receiver-side convergence`

This is materially different from the earlier options:

- Option 1 centered a single reconciliation kernel as the main architectural unit.
- Option 2 centered separate cluster and local state machines.
- Option 3 centered lease authority as the dominant abstraction.
- Option 4 centered a single recovery funnel.
- Option 5 centered generation ledgers and cutover contracts.
- Option 6 centered publication quality as the main control-plane substrate.
- Option 7 centered obligations, blockers, and satisfaction proofs.
- Option 8 centered constitutions and ballots as the primary legitimacy model.

Option 9 is different because its top-level abstraction is neither a role, nor a lease, nor a ledger, nor a ballot, nor an obligation graph. Its main idea is that HA should act like an explicit adjudication system. The system should first construct the exact case being judged, then select the exact verdict for that case, then carry out remedies attached to that verdict. The user wanted "newest observations first, then a pure decide step, then a typed outcome that lower layers turn into actions." This option makes that shape literal and inspectable.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is why this design task exists.
- The failure themes collected earlier in this task remain relevant to this option:
  - degraded-majority cases still appear to fall too quickly into fail-safe behavior rather than a more precise legitimacy ruling
  - majority-side failover and re-election scenarios still suggest ambiguity around how the healthy side decides it is allowed to continue
  - some service-restoration scenarios still imply that "still running" and "still allowed to serve writes" are not cleanly separated
  - convergence from old-primary to replica still appears too implicit, especially around rewind-versus-basebackup paths
  - startup and rejoin behavior still appears overly split between pre-loop planning and steady-state reconciliation

These outcomes are inputs only. This artifact does not attempt to fix them in code. The point is to define a redesign that would make the intended reasoning explicit enough that a later implementation can satisfy those scenarios without improvising architecture case by case.

## Current design problems

### Startup logic is still outside the main HA adjudication path

`src/runtime/node.rs` still performs startup planning through functions such as `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. That means the most delicate questions, such as whether a node should initialize, continue, stay fenced, follow, promote, rewind, or basebackup, can be answered before the runtime has built the same shape of world view used later in the normal loop.

That is the opposite of the user's intended model. Startup should not be a separate planner with its own architecture. Startup should be the first casefile. If the same observations recur later, the same casefile should be generated and the same verdict should follow.

### Sender-side dedup in `src/ha/worker.rs` muddies the responsibility boundary

The current HA worker still makes sender-side judgments such as `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the sender is partially performing consumer-state interpretation. This is architecturally risky because the sender may not know whether a consumer has fully converged or only partly applied the prior intent.

In this option, the court emits verdicts with stable verdict identifiers and remedy identifiers. Consumers decide whether a remedy is already satisfied. The court never suppresses a remedy just because it looks similar to the prior one.

### HA meaning is currently spread across multiple modules with no single "case" object

Today, evidence arrives from DCS state, pginfo, process observations, and startup probes, but there is no single typed object that says, "this is what the node believes the case is." That leads to inference-by-distribution. Runtime code, decision code, lower code, and dispatch code each hold part of the story.

This is a maintainability problem and a correctness problem. If an operator or test author asks why a node fenced, promoted, or refused to bootstrap, the answer should come from one typed casefile and one typed verdict, not from mentally stitching together four modules.

### `src/ha/decide.rs` still uses a too-blunt non-full-quorum boundary

The current routing from non-`DcsTrust::FullQuorum` toward `HaPhase::FailSafe` is too coarse. It treats multiple distinct cases as though they were one case:

- the node truly cannot establish any valid acting majority
- the node has lost full visibility but can still see a valid majority partition
- the node is isolated from the majority and must demote
- the node is on the majority side and should continue or elect

Those are not the same case. This option forces them to be classified separately before any verdict can be issued.

### Startup and convergence meaning are still too indirect in `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` still bridges between HA decisions and process actions in a way that leaves startup and rejoin intent partly implicit. The code knows how to derive start intent, validate sources, and sequence rewind or basebackup, but the logic emerges from prior decisions rather than a first-class verdict about what case the node is in.

Under this option, process dispatch is no longer the place where ambiguous intent gets interpreted. It is only the place where a clear verdict gets translated into process-specific steps.

### Partial truth is still not elevated enough as adjudication input

`src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already expose partial truth such as unknown readiness, unknown SQL health, or missing pginfo while the agent is still alive. But the higher-level architecture still has places where uncertainty risks collapsing into absence, or where degraded evidence is not explicitly modeled as a separate category.

This option treats evidence quality as part of the casefile itself. The court is not allowed to silently ignore uncertainty. It must explicitly reason about admissible, weak, stale, contradictory, or missing evidence.

## Core idea

The system should operate as a deterministic court:

- Observers submit evidence.
- A pure builder assembles that evidence into one casefile.
- A pure court applies an ordered rulebook to the casefile.
- The court returns a verdict with required remedies.
- Lower layers convert remedies into effect plans.
- Consumers converge on those plans and report the next observations back.

The central types are:

- `HaCasefile`
- `EvidenceLedger`
- `ClusterLegitimacy`
- `LeadershipPosture`
- `LocalDataFitness`
- `ContradictionSet`
- `HaVerdict`
- `VerdictRemedy`
- `VerdictReason`

The key design constraint is that the court never performs side effects. It does not touch Postgres. It does not touch etcd. It does not guess what consumers already applied. It only classifies the case and returns the verdict.

The casefile is broader than a current-role snapshot. It includes:

- current DCS trust and freshness
- visible member records and their evidence quality
- visible leader claim and lease status
- local process state and pginfo partial truth
- local data lineage and timeline fitness
- startup-only evidence, such as whether local `pgdata` exists or whether the node just won init lock
- contradictions, such as "local Postgres is serving writes but leader mandate is absent"

The verdict is broader than "be primary" or "be replica." It is an adjudicated result such as:

- continue serving as certified primary
- continue serving as provisional primary under degraded-majority conditions
- demote and fence due to illegitimate authority
- hold candidate posture and await election
- bootstrap cluster with guarded bootstrap substate
- follow known leader without repair
- repair by rewind before follow
- repair by basebackup before follow
- remain observer-only because evidence is insufficient or contradictory

This produces an architecture where the most important question is always explicit: "Which case is currently being judged?"

## Proposed control flow from startup through steady state

Every runtime tick, including the very first startup tick, follows the same top-level flow:

```text
                 +------------------------+
                 | newest observations    |
                 | - DCS member records   |
                 | - leader lease state   |
                 | - pginfo partial truth |
                 | - local process state  |
                 | - local pgdata facts   |
                 | - startup lock facts   |
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | build HaCasefile       |
                 | - normalize evidence   |
                 | - score freshness      |
                 | - classify lineage     |
                 | - record contradictions|
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | HaCourt::adjudicate    |
                 | ordered rulebook       |
                 | yields HaVerdict       |
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | lower verdict remedies |
                 | to effect plans        |
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | receivers/effect       |
                 | consumers converge     |
                 | idempotently           |
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | new observations feed  |
                 | next casefile          |
                 +------------------------+
```

The startup path becomes a special case only in the evidence set, not in the architecture. On startup, the casefile builder sees evidence such as:

- "local Postgres process not yet started"
- "local `pgdata` exists and lineage is X"
- "init lock is held / not held / unknown"
- "visible leader is Y with freshness Z"
- "cluster has N published members with evidence quality distribution Q"

Later, during steady-state, the same builder may no longer include init-lock evidence, but it still builds the same kind of casefile. That unifies startup and steady-state without forcing them to share identical subcases.

The court rulebook must be ordered so that safety-critical verdicts outrank operational convenience. A reasonable order is:

1. detect self-contradictions and illegitimate serving
2. rule on cluster legitimacy and acting-majority capability
3. rule on leader mandate validity
4. rule on local eligibility to retain, challenge, or follow
5. rule on data repair path if following is required
6. rule on bootstrap path only if no existing legitimate cluster is visible

That order matters. A node must not decide between rewind and basebackup before it has decided whether a legitimate leader exists. It must not decide to bootstrap while a legitimate majority-backed leader remains visible. It must not continue serving writes simply because Postgres is already up.

## Proposed typed state model

This option still uses explicit typed states, but the primary state is the case classification rather than a long-lived role enum. The core types would look like this conceptually:

```text
HaObservationBundle
  -> HaCasefile
       - case_id: CaseId
       - observation_epoch: ObservationEpoch
       - cluster_legitimacy: ClusterLegitimacy
       - leadership_posture: LeadershipPosture
       - local_data_fitness: LocalDataFitness
       - startup_context: StartupContext
       - member_evidence: Vec<MemberEvidence>
       - contradiction_set: ContradictionSet
       - admissibility: EvidenceAdmissibility
  -> HaVerdict
       - verdict_id: VerdictId
       - class: VerdictClass
       - remedies: Vec<VerdictRemedy>
       - reasons: Vec<VerdictReason>
       - recheck_policy: RecheckPolicy
```

Suggested enums:

```text
ClusterLegitimacy
  - NoActingMajority
  - ActingMajorityProven
  - FullQuorumProven
  - ContradictoryView

LeadershipPosture
  - NoLegitimateLeaderVisible
  - LegitimateLeaderVisible { leader_id, lease_state, mandate_quality }
  - LocalNodeIsLegitimateLeader { lease_state, mandate_quality }
  - LeadershipContested { claimant_set }

LocalDataFitness
  - Unusable
  - EmptyPgData
  - BootstrapEligible
  - CanFollowDirectly { source_leader_id }
  - RequiresRewind { source_leader_id }
  - RequiresBasebackup { source_leader_id }
  - ServingWritableButIllegitimate
  - ServingReadableReplica

StartupContext
  - NotStartup
  - StartupPending
  - StartupWithInitLock
  - StartupWithoutInitLock
  - StartupWithExistingPgData

VerdictClass
  - RemainObserver
  - FenceAndDemote
  - RetainPrimary
  - RunElection
  - BootstrapCluster
  - FollowLeader
  - RepairThenFollow
  - AwaitMoreEvidence
```

Important invariant: a `VerdictClass` never hides its safety basis. If the verdict is `RetainPrimary`, the reasons must state whether that retention is backed by full quorum or by a degraded-but-valid acting majority. If the verdict is `FenceAndDemote`, the reasons must state whether the problem was loss of majority, leadership contradiction, lease expiry, illegitimate serving, or evidence collapse.

Another invariant: casefiles are immutable pure inputs. Verdicts are immutable pure outputs. Any persistence or memoization of prior verdicts happens outside the court.

## Detailed phase model

The court does not eliminate phases entirely. It replaces today's loosely distributed phases with explicit case classes and verdict phases.

### Phase A: Evidence intake

This phase gathers the newest available facts from:

- DCS member records
- leader key / lease state
- pginfo worker observations
- process lifecycle state
- local filesystem or pgdata inspection results
- startup lock / bootstrap lock visibility

No decisions happen here. The only goal is to assemble facts and retain partial truth.

### Phase B: Evidence normalization

The builder converts raw observations into a stable typed ledger:

- freshness buckets
- explicit missing-vs-unknown distinctions
- lineage interpretations
- writable/readable/unknown process capabilities
- majority visibility calculations
- contradiction detection

This phase is pure. It is where "pginfo failed but pgtuskmaster is alive" gets preserved as weak but admissible evidence instead of silence.

### Phase C: Case classification

The builder assigns the high-level case shape, for example:

- `Case::LegitimateLeaderVisibleAndLocalMustFollow`
- `Case::LocalPrimaryClaimIsNoLongerLegitimate`
- `Case::NoVisibleLeaderButActingMajorityExists`
- `Case::NoActingMajority`
- `Case::BootstrapWindowOpen`
- `Case::FollowerNeedsTimelineRepair`
- `Case::ContradictoryAuthoritySignals`

This classification does not yet issue actions. It names the case.

### Phase D: Verdict selection

The court uses a deterministic rulebook:

- safety verdicts first
- authority verdicts second
- convergence verdicts third
- optimization or convenience verdicts last

For example:

- if local node is serving writable and the casefile cannot justify legitimate authority, verdict is `FenceAndDemote`
- if a legitimate leader is visible and local data can follow directly, verdict is `FollowLeader`
- if a legitimate leader is visible and local data is on wrong timeline but rewindable, verdict is `RepairThenFollow` with `RepairMode::Rewind`
- if no leader is visible but an acting majority exists and local node is best eligible candidate, verdict is `RunElection`
- if no legitimate cluster exists and bootstrap prerequisites hold, verdict is `BootstrapCluster`

### Phase E: Remedy lowering

The verdict gets lowered into typed remedies such as:

- publish updated leader claim
- release or renew leader lease
- stop writable service
- enforce safety fence
- start Postgres with follow config
- run rewind
- run basebackup
- publish partial member truth
- stay read-only observer

Lowering is where `src/ha/lower.rs` should remain important. The option does not discard the user's preferred split between pure decision and effect planning.

### Phase F: Receiver-side convergence

Every consumer compares the remedy identity and desired terminal state with its current observed state. Consumers deduplicate at this point, not in the HA sender. A process worker decides whether Postgres is already in the required posture. A DCS worker decides whether the member record or leader claim already matches the remedy. A safety fence worker decides whether the fence is already established.

## Redesigned quorum model

This option separates three things that the current design too often conflates:

- visibility completeness
- acting-majority legitimacy
- leadership legitimacy

The cluster does not need full visibility to continue safely. It needs a provable acting majority consistent with the configured membership and freshness rules.

The casefile builder therefore computes:

- `VisibleVoterSet`
- `FreshVoterSet`
- `ActingMajorityAssessment`
- `LegitimacyConfidence`

Suggested rules:

- `FullQuorumProven` means all expected voting members are fresh enough and mutually consistent.
- `ActingMajorityProven` means a majority subset is fresh enough and sufficiently consistent to act, even if one or more members are missing or stale.
- `NoActingMajority` means no majority can be proven from admissible evidence.
- `ContradictoryView` means the visible evidence contains mutually incompatible authority claims such that the court cannot safely infer a valid actor set.

This explicitly fixes the user's complaint about the 2-of-3 case. In a three-node cluster, if two fresh nodes on the same side can still see each other and can justify a common leader lineage, they can form an `ActingMajorityProven` case even without full quorum. The verdict may therefore be `RetainPrimary` or `RunElection`, not automatic fail-safe.

The minority side gets a different casefile. It sees `NoActingMajority` or `ContradictoryView` and therefore receives `FenceAndDemote` or `RemainObserver`.

This means quorum loss is not one event. It is a family of cases:

- loss of full visibility but majority retained
- loss of majority
- contested majority evidence
- full collapse of evidence

Only the latter two categories force the majority-capable side into the same path as the isolated side.

## Redesigned lease model

Leases remain crucial, but they become one part of the casefile rather than the sole architecture anchor.

The casefile must represent:

- local belief about current lease holder
- lease freshness
- lease expiry or uncertainty
- whether lease evidence supports local authority, remote authority, or neither

The lease rules become:

- a node may retain writable primary service only if the casefile can justify both a legitimate leadership posture and a non-expired lease posture
- a killed primary loses authority because subsequent casefiles from other nodes will classify the old holder as absent, stale, or expired, making a new election admissible
- a restarted old primary cannot recover writable authority simply because its local data was previously primary; its new casefile must justify authority under current majority and lease evidence
- if lease evidence is weak but an acting majority exists, the court may issue a `RunElection` verdict rather than a blanket fail-safe verdict
- if lease evidence contradicts member evidence or lineage evidence, the contradiction must appear in `ContradictionSet`, and the verdict must prefer fencing over optimistic write service

This is important because leases should not merely say "who is primary." They should say whether continuing to act as primary is still justified. The current architecture appears to let some of that reasoning emerge indirectly. This option forces lease interpretation to be explicit in the verdict reasons.

## Startup reasoning

Startup becomes one of the strongest benefits of this option because the node no longer uses a separate startup planner. It builds a startup-flavored casefile and lets the same court rulebook decide.

The startup casefile must explicitly answer:

- is there already a legitimate leader visible?
- is there an acting majority or only isolated evidence?
- does local `pgdata` exist?
- if local `pgdata` exists, what lineage or timeline relationship does it have to the visible leader?
- did this node win the init lock?
- if it won the init lock, is bootstrap actually justified, or is there already a legitimate cluster it should not override?
- is local Postgres already running, and if so, is that legitimate under current authority evidence?

The startup subcases are:

### Cluster already up with visible legitimate leader

The verdict cannot be bootstrap. The verdict must be `FollowLeader`, `RepairThenFollow`, or `FenceAndDemote` if the node is illegitimately serving.

### No visible leader, but acting majority exists

The verdict may be `RunElection`, but only if local node meets data-fitness and eligibility rules. Startup is not special here. It is just the first opportunity to adjudicate election eligibility.

### No legitimate cluster is visible, init lock is won, and local `pgdata` is empty

The verdict may be `BootstrapCluster`, but the verdict must include guarded bootstrap substates:

- `BootstrapPrepare`
- `BootstrapInitializeDataDir`
- `BootstrapPublishGenesis`
- `BootstrapAwaitConfirmation`

This avoids the current risk of treating bootstrap as a one-shot implicit action.

### No legitimate cluster is visible, init lock is won, but local `pgdata` already exists

This is one of the subtler cases the user called out. Winning init lock is not enough to blindly wipe or ignore existing data. The casefile must evaluate whether the existing `pgdata` is:

- unusable junk
- legitimate prior cluster data that should be continued
- data from a conflicting lineage that should block bootstrap

The verdict may therefore be `BootstrapCluster`, `RunElection`, `FollowLeader`, or `RemainObserver`, depending on the evidence. The init lock is only one piece of evidence, not automatic bootstrap permission.

### All evidence is partial or contradictory

The verdict should be `AwaitMoreEvidence` or `RemainObserver`, not "do something bold and hope."

## Replica convergence as one coherent path

This option makes replica convergence part of the verdict model rather than a side path inferred later.

The casefile explicitly classifies local follower fitness:

- `CanFollowDirectly`
- `RequiresRewind`
- `RequiresBasebackup`
- `Unusable`

That allows one coherent path:

1. identify a legitimate leader
2. classify local data fitness relative to that leader
3. issue one follow-oriented verdict
4. attach the correct repair mode if direct follow is impossible

The important simplification is that old primary, old replica, or freshly restored node are not treated as fundamentally different categories. They are all simply nodes with some current `LocalDataFitness` relative to the legitimate leader.

If the node was formerly primary and now lost legitimacy:

- if its data can directly follow, the verdict is `FollowLeader`
- if it is on a wrong timeline but rewind is possible, the verdict is `RepairThenFollow { mode: Rewind }`
- if rewind cannot succeed, the verdict is `RepairThenFollow { mode: Basebackup }`

This unifies scenarios that currently risk being split among startup planning, dispatch bridging, and runtime special cases.

## Partial-truth member publication

This option requires stronger member publication, not weaker.

The DCS layer must continue to be the only place that writes etcd, but the HA casefile depends on DCS publication preserving partial truth such as:

- agent alive, pginfo unreachable
- agent alive, readiness unknown
- Postgres process observed, SQL health unknown
- local data lineage known, service state unknown

The DCS publication model should therefore represent:

- publication timestamp and freshness class
- agent liveness
- process liveness
- SQL reachability or unknown
- readiness or unknown
- lineage facts when available
- last-known role claim and confidence

Silence must mean "no publication" only. It must not be overloaded to mean "publication exists but fields are unknown."

This matters directly to adjudication. A node with weak but fresh evidence should count differently from a node with stale absence. The court can then reason about admissibility rather than inventing special cases later.

## Deduplication boundary

Deduplication must leave the sender-side HA worker completely.

The court emits:

- `verdict_id`
- `remedy_id`
- desired terminal posture
- required side-effect invariants

Consumers deduplicate based on whether the desired terminal posture is already satisfied for that remedy identity. Examples:

- the process consumer sees that Postgres is already running in the required read-only follow mode for this leader and this lineage, so it does nothing
- the DCS consumer sees that the member record already matches the required published fields, so it avoids a no-op write
- the safety consumer sees that fencing is already active, so it keeps the fence and reports the same observation

Why this boundary is safer:

- the sender no longer guesses about downstream convergence
- each consumer deduplicates using the exact semantics of its own domain
- the court remains purely about adjudication, not delivery optimization
- repeated verdict emission becomes safe and expected, which matches the user's preferred functional style

This means functions like `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)` should disappear from sender-side HA logic in a later implementation.

## Concrete future code areas that would change

This option would affect at least the following areas in a later implementation:

- `src/runtime/node.rs`
  - remove or collapse the separate startup planner/executor path
  - make startup produce an observation bundle and enter the court path immediately
- `src/ha/worker.rs`
  - replace sender-side dedup with unconditional casefile and verdict production
  - thread casefile / verdict identities through the loop
- `src/ha/decide.rs`
  - either replace it with `HaCourt` adjudication logic or shrink it into the verdict rulebook entry point
- `src/ha/decision.rs`
  - expand or replace current fact and decision types with `HaCasefile`, `HaVerdict`, `VerdictRemedy`, `VerdictReason`, `ContradictionSet`, and case classification enums
- `src/ha/lower.rs`
  - preserve the pure lowering split, but lower verdict remedies rather than today's decision shape
- `src/ha/process_dispatch.rs`
  - stop deriving implicit startup intent from prior decisions
  - consume explicit verdict remedies for follow, rewind, basebackup, bootstrap, and demotion
- `src/dcs/worker.rs`
  - strengthen publication of partial truth and possibly publish richer authority metadata
- `src/dcs/state.rs`
  - support richer evidence freshness, admissibility, and acting-majority classification helpers
- `src/pginfo/state.rs`
  - preserve and expose partial truth categories needed by the casefile builder
- `tests/ha.rs`
  - update or extend scenario expectations to align with the new degraded-majority semantics and explicit verdict classes
- `tests/ha/features/`
  - likely add or revise probes that assert not just external behavior but also more explicit authority transitions

## All meaningful changes required for this option

The later implementation would need to make all of the following changes, not just a subset:

- introduce a new pure casefile-building module
- introduce a new pure court or verdict-selection module
- define stable case, verdict, and remedy identifiers
- define contradiction detection rather than letting contradictions remain implicit
- move startup evidence gathering into the same input bundle as steady-state evidence
- delete the separate startup-planning architecture from `src/runtime/node.rs`
- redefine degraded-majority handling so that acting majority and full quorum are not treated as synonyms
- redefine leadership legitimacy so lease evidence is combined with member freshness and lineage evidence
- redefine follower convergence as one verdict family with explicit repair modes
- move all sender-side dedup out of the HA loop and into effect consumers
- ensure DCS publication preserves partial truth instead of collapsing it into absence
- ensure the court can produce `AwaitMoreEvidence` and `RemainObserver` verdicts explicitly
- make illegitimate local write service a first-class contradiction that leads to fencing
- define bootstrap substates rather than keeping bootstrap as a coarse action
- explicitly encode local data fitness relative to a visible leader
- remove stale legacy pathways once the new court path exists; this project is greenfield and should not preserve the old split indefinitely

The implementation should not try to preserve both architectures long term. A temporary migration phase may exist, but the final state should have one adjudication path.

## Migration sketch

One reasonable later migration path is:

1. Add `HaCasefile` builder alongside existing decision facts.
2. Teach the HA worker to produce casefiles for observability while still using the old decision output.
3. Introduce `HaVerdict` in parallel and compare old decisions to new verdicts in logs or tests.
4. Move startup evidence gathering into the casefile builder.
5. Switch `src/ha/lower.rs` to lower verdict remedies instead of old decision variants.
6. Move process and DCS dedup logic entirely into consumers.
7. Delete the old startup planner in `src/runtime/node.rs`.
8. Delete old decision paths and any compatibility shims once the verdict path is proven.

The key migration rule is that no "temporary forever" split should survive. The old startup planner, sender dedup, and ambiguous degraded-quorum shortcuts should be removed, not merely bypassed.

## Logical feature-test verification

This section explains how this option would logically satisfy the existing HA scenarios without implementing code in this task.

### `ha_dcs_quorum_lost_enters_failsafe`

This scenario should be reinterpreted through explicit case classification. If the node's casefile shows `NoActingMajority`, the verdict is `FenceAndDemote` or `RemainObserver`, which preserves the intended safety behavior. The point is not to remove fail-safe behavior; it is to reserve it for the right case instead of all non-full-quorum cases.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

If a node is serving writable traffic but the casefile cannot justify legitimate authority after the cutoff, the contradiction set includes "writable service without legitimate majority-backed authority." The verdict is safety-first: `FenceAndDemote`. That directly supports post-cutoff write blocking.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side builds a casefile with `ActingMajorityProven`, `NoLegitimateLeaderVisible` or `LeadershipContested`, and at least one eligible candidate. The verdict becomes `RunElection`. The isolated old primary builds `NoActingMajority` and receives `FenceAndDemote`. This is precisely the split the current architecture seems too blunt to express.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

After healing, the old primary's casefile shows a legitimate remote leader and local data fitness relative to that leader. If the timelines are compatible, verdict is `FollowLeader`; if rewind is needed, verdict is `RepairThenFollow { Rewind }`; if rewind is impossible, verdict is `RepairThenFollow { Basebackup }`. The key is that the case explicitly says the node is no longer a legitimate leader claimant.

### `ha_primary_killed_then_rejoins_as_replica`

Once the killed primary restarts, its startup casefile does not inherit legitimacy from its old role. It must justify authority from current evidence. Seeing a legitimate current leader, it receives a follow-oriented verdict. That prevents role resurrection via stale local assumptions.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica returns, the majority-side casefile changes from insufficient evidence or reduced majority confidence toward `ActingMajorityProven` or `FullQuorumProven`, depending on cluster size and freshness. The court can then re-issue a retain or election verdict without needing special recovery-only logic.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first restarted nodes produce startup casefiles, not special startup planners. If they can establish a legitimate acting majority and one node is eligible, the court can run election or retain bootstrap-generated authority. The last node later sees that legitimate leader and follows through the same casefile model.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The follow-oriented verdict includes explicit repair modes. If `RepairThenFollow { Rewind }` fails, the next observations update the casefile to classify rewind as no longer viable. The next verdict becomes `RepairThenFollow { Basebackup }`. That gives the fallback path an explicit adjudication step instead of burying it in process repair heuristics.

### `ha_replica_stopped_primary_stays_primary`

The primary's casefile still shows legitimate acting majority or full quorum support sufficient to retain authority. The stopped replica's absence affects visibility but does not automatically invalidate the primary. The verdict remains `RetainPrimary` rather than fail-safe, which is the intended behavior when majority safety remains satisfied.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica rejoining only affects its own casefile and DCS publication quality. The leader's casefile should classify that member as degraded evidence, not as a reason to destabilize legitimate authority. The returning replica receives `RepairThenFollow`, `RemainObserver`, or `AwaitMoreEvidence` depending on its data fitness. Quorum is not destabilized by a single broken follower because the case model separates local unfitness from cluster legitimacy.

## Non-goals

- This option does not try to make the court generic enough for all future cluster orchestration problems. It is specifically for HA reconciliation.
- This option does not put side effects into the decider.
- This option does not reduce everything to a single lease primitive.
- This option does not preserve sender-side dedup for convenience.
- This option does not assume that all uncertainty can be resolved optimistically.

## Tradeoffs

- The casefile model adds more explicit types and more up-front modeling work than a lighter refactor.
- The ordered rulebook can become large if the team is not disciplined about keeping case classification coherent.
- Some engineers may find a court metaphor unfamiliar compared with role/phase language.
- The implementation will need careful observability so casefiles and verdicts are inspectable, otherwise the benefits are partly lost.
- This option depends on strong contradiction modeling. If contradictions are under-modeled, the court may still appear deterministic while hiding ambiguity.

## Open questions

## Q1 How many case classes should exist?

The strength of this option comes from naming the exact case being adjudicated. Too few case classes and the design collapses back into vague phases. Too many case classes and the rulebook becomes brittle or unreadable.

The design tension is between explicitness and maintainability. A small set of broad cases keeps the system simpler but may hide important distinctions such as "acting majority retained" versus "authority contested within a majority-visible cluster."

Restated plainly: should the future implementation keep case classification coarse and push detail into verdict reasons, or should it create many explicit case variants so the architecture documents itself more directly?

## Q2 Where should contradictions be surfaced to operators and tests?

This option assumes contradiction tracking is first-class:

```text
evidence A: local writes still accepted
evidence B: no legitimate authority remains
-----------------------------------------
contradiction: illegitimate writable service
verdict: fence and demote
```

The problem is not whether contradictions exist; the problem is where they should appear. They could live only in logs, in metrics, in DCS publication, in test-only debug output, or in the public runtime state model.

Restated another way: how visible should the contradiction model be outside the pure court, and what minimal external surface is needed so future engineers can diagnose verdicts without reading code?

## Q3 Should the court remember prior casefiles explicitly?

This design is intentionally observation-first, but some judgments may benefit from a typed notion of recent history, such as "rewind was already attempted and failed" or "this provisional acting majority persisted for three ticks."

The problem is that too much remembered history can quietly reintroduce hidden statefulness into the pure adjudication path. Too little memory can make the court unable to distinguish a transient blip from a sustained condition.

Restated directly: should prior casefile summaries become explicit inputs to the next casefile, and if so, which history is legitimate state versus accidental temporal coupling?

## Q4 How strict should local data fitness be for election eligibility?

This option says the court should not run elections merely because a node is present. It should consider whether the node's data is fit enough to become leader.

The tension is in the degraded cases. If the best-visible node is slightly stale but still the safest available candidate, a strict fitness bar may block recovery. A loose bar may elect a node that later forces avoidable repair.

Restated: when the court sees an acting majority but imperfect candidates, how conservative should election eligibility be, and which data-fitness signals are mandatory versus preferred?

## Q5 Should bootstrap be judged by the same court or by a specialized bootstrap court?

This option keeps bootstrap inside the same casefile and verdict architecture because the user wants startup unified with steady-state reconciliation.

The open question is whether bootstrap complexity will overload the general rulebook. A specialized bootstrap sub-court could keep rules clearer, but it risks recreating the startup split that the user wants removed.

Restated one last way: can one unified court handle bootstrap cleanly enough, or will bootstrap need a separate internal rule layer without regressing to a separate architecture?
