# Option 6: Member-Publication-First Control Plane

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA architecture still treats DCS member publication mostly as an output of the HA loop, when in practice the quality and shape of published member truth is one of the biggest upstream causes of the loop's ambiguity. The user explicitly wants member keys to contain the newest obtainable truth, even when that truth is partial, degraded, or awkward. That requirement is not cosmetic. It changes what the cluster can safely conclude.

The differentiator of Option 6 is that the redesign starts by making the control plane publication model explicit and authoritative. The pure HA decider no longer reasons from a vaguely assembled world snapshot that may silently flatten uncertainty into absence. Instead, every tick first constructs a typed `ClusterEvidenceTable` from best-known member publications plus local observations. The decider then reasons from evidence quality, freshness, and contradiction explicitly.

Option 1 centered one unified reconciliation kernel. Option 2 centered a split between cluster intent and local execution. Option 3 centered lease authority first. Option 4 centered one recovery funnel for all non-primary behavior. Option 5 centered explicit generation cutovers and per-node contracts. Option 6 is materially different from all five because it says the deepest architectural fix is to strengthen the evidence substrate itself: publish better truth, classify it better, reason from it directly, and let startup plus steady-state share the same evidence-first model.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign studies.
- The failure themes already gathered earlier in this task remain directly relevant:
  - quorum-loss scenarios did not consistently surface the expected `fail_safe` evidence
  - degraded-majority scenarios did not consistently expose a new primary from the healthy majority
  - some restore-service and restart scenarios left a node writable when the scenario expected it to remain blocked
  - targeted switchover toward a degraded replica succeeded when it should have been rejected
  - rewind-to-basebackup fallback evidence was not consistently visible
  - old-primary loss-of-authority timing remained too weak in kill and storage-stall scenarios
  - rejoin convergence still appeared ambiguous instead of following one crisp path

Those observations are inputs only. This document does not propose fixing them in this run. It proposes one architecture that could later make the relevant behavior more coherent.

## Ten option set for the overall task

This document remains one member of the ten-option redesign set for the larger task:

1. `Option 1: Unified Observation-Reconciliation Kernel`
2. `Option 2: Dual-Layer Cluster/Local State Machine`
3. `Option 3: Lease-First Authority Core`
4. `Option 4: Recovery Funnel Architecture`
5. `Option 5: Generation-Cutover Ledger`
6. `Option 6: Member-Publication-First Control Plane`
7. `Option 7: Receiver-Owned Work Queue`
8. `Option 8: Intent Ledger With Reconciliation Snapshots`
9. `Option 9: Safety Gate Plus Role Machine`
10. `Option 10: Startup-As-Synthetic-Ticks`

Option 6 is the most evidence-centric proposal in the set. Its main design bet is that the HA loop becomes much simpler if the system publishes richer truth and attaches first-class quality semantics to that truth before any role or authority decision happens.

## Option differentiator

The specific differentiator of this option is:

- DCS member publication is no longer treated as a passive mirror of local pginfo
- every member record becomes an explicit evidence envelope with freshness, quality, and contradiction markers
- the pure HA decider consumes a `ClusterEvidenceTable` derived from those envelopes
- startup uses the exact same evidence path as steady-state, because the first startup tick simply begins with weaker and more partial evidence
- lease, quorum, recovery, and fencing decisions become downstream of evidence quality instead of downstream of role inference alone

In other words, this design moves the first architectural question from:

`What role should this node try to be right now?`

to:

`What does the cluster most credibly know right now, how trustworthy is that knowledge, and what behavior is safe given that evidence?`

That is why this option is not just a minor DCS publication tweak. It reorders the whole control surface.

## Current design problems

### 1. Startup logic is split across `src/runtime/node.rs`

`src/runtime/node.rs` still contains `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. Those functions are not only process boot helpers. They decide whether existing local data may be trusted, whether cluster birth should proceed, whether the node should wait, and whether it should follow.

That means startup reasons from a different information path than steady-state HA. Some of the most important lifecycle decisions happen before the normal HA worker can apply the standard `newest info -> decide -> lower -> actions` model.

Option 6 removes that split by treating startup as the earliest evidence-construction phase. The node starts by publishing and consuming best-known truth, including partial truth, on the very first tick. There is no separate architectural family called "startup planning."

### 2. Sender-side dedup still lives in `src/ha/worker.rs`

`src/ha/worker.rs` currently includes suppression logic such as `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the sender is trying to infer whether downstream workers are already aligned with the desired state.

That boundary is especially weak in an evidence-centric architecture. A sender cannot know whether a previous request is still valid after evidence quality changes, a lease epoch changes, or a contradiction is resolved. Deduplication must belong to the effect receivers that see their own durable in-flight state, not to the pure HA sender.

### 3. HA reasoning is spread across runtime, decide, lower, and process-dispatch boundaries

The current architecture distributes lifecycle truth across:

- `src/runtime/node.rs` for startup planning
- `src/ha/decide.rs` for phase selection
- `src/ha/lower.rs` for effect selection
- `src/ha/process_dispatch.rs` for process transition derivation
- `src/dcs/worker.rs` and `src/dcs/state.rs` for member publication and trust interpretation

The result is that no single typed artifact answers:

- what the cluster credibly knows
- which uncertainty is tolerable
- when degraded-majority operation should continue
- when leadership evidence is strong enough to promote or keep serving
- when a node must fence because evidence quality is too weak

Option 6 creates one explicit substrate for those answers: `ClusterEvidenceTable`.

### 4. Non-full-quorum currently shortcuts too quickly toward fail-safe

`src/ha/decide.rs` currently treats non-`DcsTrust::FullQuorum` as a near-direct path into `FailSafe` logic. That is too blunt because "not full quorum" can mean several different evidence states:

- one node is missing but a healthy majority remains
- a majority remains but one member's publication is stale
- a leader lease is fresh but one follower has contradictory local facts
- the local node is isolated and only sees a minority
- DCS visibility is stale enough that no safe authority claim exists

Those are different safety boundaries. Option 6 models them separately by reasoning about evidence quality and majority credibility, not only raw membership count.

### 5. Startup and rejoin intent is still partially hidden inside `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` currently still helps determine whether a node should start PostgreSQL, follow, rewind, or basebackup. That is too much authority for a dispatch bridge. It means the shape of local lifecycle is partly hidden in translation logic rather than in one explicit decision artifact.

Option 6 changes the boundary so the pure decider produces a typed `NodeIntentFromEvidence`, and `process_dispatch` only translates it into concrete receiver requests.

### 6. Member publication does not yet fully embrace partial truth as first-class control-plane input

`src/dcs/worker.rs` publishes member state, and `src/pginfo/state.rs` already contains useful partial states such as `PgInfoState::Unknown`, `SqlStatus::Unknown`, `SqlStatus::Unreachable`, and `Readiness::Unknown`. The user wants that partial truth retained and published instead of collapsed into silence.

The current architecture still risks treating incomplete local knowledge as "member absent" or "no useful signal." That weakens cluster reasoning. Silence should mean "no observation arrived." It should not mean "pginfo degraded." Those are different things.

Option 6 makes published partial truth central. A node should say, in effect:

- agent alive
- DCS connected or not
- postgres process observed or not
- SQL reachable or not
- readiness known or unknown
- last known role and source of that belief
- local data lineage facts known or not

That richer truth then becomes the cluster's actual decision substrate.

## Design summary

Option 6 introduces a publication-first HA control plane. Every tick, including the first startup tick, has three conceptual stages:

1. Build and publish best-known local truth as an evidence envelope.
2. Assemble newest cluster-wide evidence into a `ClusterEvidenceTable`.
3. Run a pure evidence interpreter that derives authority, local role intent, recovery intent, and publication updates from that table.

The system still preserves the user-preferred functional chain:

`newest info -> decide -> typed outcome -> actions`

The change is that "newest info" is no longer an implicit, lossy bundle. It is a typed evidence table whose quality is explicit and whose uncertainty is preserved rather than hidden.

## Proposed control flow

Every tick uses the same control flow from startup onward.

```text
local process inspector ----\
pginfo worker --------------+--> LocalEvidenceBuilder --> LocalEvidenceEnvelope
data-dir inspector ---------/                                   |
init-lock inspector --------/                                   v
                                                           DCS publish
                                                               |
                                                               v
                     remote member envelopes ---> ClusterEvidenceTableBuilder
                                                               |
                                                               v
                                                     PureEvidenceInterpreter
                                                               |
                        +------------------------------+-------+------------------------------+
                        |                              |                                      |
                        v                              v                                      v
                 AuthorityConclusion            LocalNodeIntent                        PublicationRevision
                        |                              |                                      |
                        v                              v                                      v
                  effect lowering             receiver-owned actions                 next member publication
```

The ordering matters.

- First, the node contributes its best-known local truth.
- Second, it reasons from the richest cluster evidence available, including remote partial truth.
- Third, it chooses authority and local behavior.

Startup is unified because the initial tick is just the earliest pass through the same evidence builder. The difference is only that some fields begin as `Unknown`.

## Core concepts

### Local evidence envelope

Each node publishes a structured evidence envelope instead of a thin role/status record.

Proposed future type sketch:

```text
LocalEvidenceEnvelope {
  member_id,
  observation_time,
  publication_revision,
  agent_status,
  dcs_reachability,
  postgres_process_state,
  sql_probe_state,
  readiness_state,
  last_known_role,
  role_evidence_source,
  local_timeline_state,
  local_wal_position_state,
  recovery_source_state,
  init_lock_state,
  data_dir_state,
  publication_quality,
  contradiction_flags,
}
```

The important shift is that publication explicitly carries both facts and the quality of those facts.

### Publication quality

Every envelope includes a typed quality classification. Proposed values:

```text
PublicationQuality
  - DirectFresh
  - DirectButPartial
  - IndirectLastKnown
  - StaleButUsable
  - Contradictory
  - Expired
```

This lets the decider distinguish "I know the process is running but SQL is unreachable" from "I have no idea what this node is doing."

### Cluster evidence table

The pure decision path does not read raw DCS entries ad hoc. It reasons from a normalized evidence table:

```text
ClusterEvidenceTable {
  cluster_time_basis,
  local_member_id,
  expected_voter_set,
  member_evidence: Vec<MemberEvidenceView>,
  cluster_contradictions,
  authority_candidates,
  freshness_summary,
  lease_summary,
  topology_summary,
}
```

Each `MemberEvidenceView` is a normalized interpretation of one envelope plus any local corroborating signals.

### Authority conclusion

The interpreter produces a typed authority answer before it produces any process action. Proposed values:

```text
AuthorityConclusion
  - ConfirmedLeader(MemberId, AuthorityBasis)
  - MajorityCanElect(Vec<MemberId>, ElectionBasis)
  - MajorityHealthyButLeaderUnclear(UncertaintyBasis)
  - MinorityIsolated(IsolationBasis)
  - EvidenceTooWeakForAuthority(WeakEvidenceReason)
```

This is how the design replaces the blunt non-full-quorum shortcut.

### Local node intent

The pure decider then derives the local node's intent from the cluster evidence and authority conclusion:

```text
LocalNodeIntent
  - ServePrimary(ServePrimaryPlan)
  - ServeReplica(ServeReplicaPlan)
  - ConvergeReplica(ConvergeReplicaPlan)
  - BootstrapCluster(BootstrapPlan)
  - WaitForEvidence(WaitPlan)
  - FenceAndHold(FencePlan)
  - StopPostgresAndAdvertise(StopPlan)
```

The intent is pure and explicit. It does not encode dispatch suppression or downstream worker state guesses.

## Detailed state machine

Option 6 uses an evidence-centric lifecycle machine. The persistent state is mostly about evidence interpretation and local convergence progress, not about raw role labels.

### Top-level phases

```text
EvidenceLifecyclePhase
  - GatherLocalFacts
  - PublishLocalEvidence
  - RebuildClusterEvidence
  - InterpretAuthority
  - SelectLocalIntent
  - LowerEffects
  - VerifyAndRepublish
```

These phases conceptually happen on every tick. The long-lived state lives in the evidence and convergence substates described below.

### Persistent substates

```text
NodeEvidenceState
  - ColdStart
  - PublishingPartialTruth
  - AwaitingCredibleClusterEvidence
  - ActingOnConfirmedAuthority
  - ActingOnMajorityElectionPath
  - ConvergingToLeader
  - LocallyFenced
  - RepairBlocked
```

### Convergence substates

```text
ConvergenceSubstate
  - VerifyLeaderEvidence
  - ValidateLocalLineage
  - FollowWithoutRepair
  - RewindToLeader
  - BasebackupFromLeader
  - StartReplica
  - VerifyStreaming
```

### Publication substates

```text
PublicationSubstate
  - PublishFreshDirectFacts
  - PublishPartialFacts
  - PublishContradiction
  - PublishFencedStatus
  - PublishRepairInProgress
```

### Key invariants

- A node may only claim `ServePrimary` intent if the current evidence table says either `ConfirmedLeader(self, ...)` or `MajorityCanElect(...)` with this node chosen by deterministic election rules.
- A node with `PublicationQuality::Contradictory` may keep publishing but may not be promoted until the contradiction is resolved or explicitly outweighed by stronger direct evidence.
- A node never disappears from control-plane truth merely because SQL probing failed. It publishes weaker truth instead.
- Effect receivers, not the decider, own idempotence and deduplication.
- Startup and steady-state both use `ClusterEvidenceTable`; there is no separate startup-only planning language.

## Evidence interpretation model

This option depends on a strong, explicit evidence interpreter.

### Evidence categories

Each member is interpreted along these axes:

- liveness evidence
- DCS write/read reachability evidence
- process evidence
- SQL evidence
- readiness evidence
- role evidence
- data-lineage evidence
- lease evidence
- topology membership evidence

Each axis can be:

```text
EvidenceStrength
  - FreshDirect
  - FreshPartial
  - LastKnownRecent
  - Stale
  - Contradictory
  - Absent
```

The interpreter does not flatten these axes into one boolean "healthy." It carries them forward into authority reasoning.

### Contradiction handling

Contradictions are first-class:

- process says Postgres running, SQL says unreachable, readiness last known ready
- DCS says member advertises primary, but lease evidence points elsewhere
- local timeline evidence conflicts with expected generation lineage
- node claims replica follow target that no longer exists

Rather than silently picking one source, the interpreter records contradictions and adjusts the authority conclusion accordingly.

### Deterministic precedence rules

The design requires explicit precedence rules so that later implementation does not reintroduce ambiguity:

1. Fresh local direct facts outrank older remote publications about the same local member.
2. Lease evidence outranks stale role advertisement.
3. Majority-supported authority outranks isolated self-assertion.
4. Contradictions downgrade confidence; they do not silently vanish.
5. Absence of SQL reachability does not erase process evidence or local-data evidence.

## Quorum model

The quorum redesign in Option 6 is evidence-aware rather than membership-count-only.

### Quorum classes

```text
EvidenceQuorum
  - FullVisibleMajority
  - MajorityWithPartialTruth
  - MajorityWithoutLeaderConfidence
  - MinorityOnly
  - NoCredibleQuorum
```

### Meaning of each class

- `FullVisibleMajority`: enough fresh evidence exists to continue or confirm leadership with strong confidence.
- `MajorityWithPartialTruth`: a voting majority is still credibly visible, but some members are only partially known. This still permits continued service or re-election if the leader basis is strong enough.
- `MajorityWithoutLeaderConfidence`: a majority is visible, but leader evidence is too contradictory or stale to safely continue writes. This requires fencing and a deterministic re-election path instead of blind fail-safe or blind continuation.
- `MinorityOnly`: the local node only sees minority evidence and must not act as primary.
- `NoCredibleQuorum`: evidence is so stale or absent that no safe authority claim exists.

### Why degraded-but-valid majority keeps working

In a 3-node cluster with one partitioned member and two healthy members, the evidence table can still produce `MajorityWithPartialTruth` or `FullVisibleMajority`. If lease and role evidence align on one healthy leader, leadership continues. If the old leader is gone or lease-invalid, the majority may elect a new leader. This is exactly the boundary the user requested.

The isolated minority side should instead produce `MinorityOnly` and fence writes, even if PostgreSQL locally still runs.

### When fail-safe still exists

Fail-safe behavior is still needed, but it becomes evidence-specific. It applies when:

- local evidence is minority-only
- lease evidence is expired or contradictory
- no credible majority exists
- cluster contradictions make leadership unsafe to infer

That is narrower and more precise than "not full quorum."

## Lease model

Option 6 keeps lease handling explicit, but it interprets lease through the evidence table.

### Lease principles

- lease state is published as evidence, not inferred only from local assumptions
- a primary may continue serving only while lease evidence and majority evidence remain compatible
- an isolated old primary that cannot refresh its authority evidence must demote or fence
- a killed primary loses authority because its lease cannot continue to be corroborated by fresh evidence

### Proposed lease types

```text
LeaseEvidenceView {
  holder,
  epoch,
  freshness,
  renewal_path_status,
  corroborating_members,
}

LeaseJudgement
  - ValidAndCorroborated
  - LocallyObservedButUncorroborated
  - Expired
  - Contradictory
  - Unknown
```

### Lease interaction with startup

Startup does not get a lease exception. On startup, the node publishes whatever evidence it has and then reads cluster evidence:

- if a credible leader lease is already visible, the node plans replica convergence or wait behavior
- if no leader exists but a credible majority and bootstrap conditions exist, the node may participate in election or cluster birth
- if lease evidence is contradictory or too weak, the node waits or fences rather than guessing

### Lease loss and killed primary behavior

If a primary is killed, its publication freshness decays and lease corroboration disappears. The cluster evidence table then shifts away from `ConfirmedLeader(old_primary)` and toward `MajorityCanElect(...)` or a new `ConfirmedLeader(new_primary)`.

If the old primary later returns, it does not regain authority from local role memory. It publishes partial truth about its process and data lineage, then the evidence interpreter assigns it a recovery or follower intent.

## Startup reasoning

Option 6 explicitly folds startup into the normal evidence pipeline.

### Startup cases the design must handle

#### Cluster already up and leader already present

The local node begins by publishing:

- agent alive
- local postgres process observed or not
- SQL reachability or failure
- local data-dir state
- any locally known timeline facts

The evidence interpreter sees a credible remote leader and selects:

- `ServeReplica` if the node is already a healthy follower
- `ConvergeReplica` if rewind or basebackup is needed
- `WaitForEvidence` if the local node's own state is too uncertain to safely act yet

#### Cluster leader already present but local publications are partial

The design still works because partial truth is allowed. The local node can say:

- process present
- SQL unreachable
- timeline unknown
- data-dir exists

That is enough to choose a conservative convergence plan instead of silence or accidental bootstrap.

#### Existing members already published

This is the normal case. The node consumes existing envelopes, grades their quality, and determines whether a stable leader exists or whether majority re-election is needed.

#### Empty `pgdata`

If local `pgdata` is absent and a credible leader exists, the node gets a bootstrap-replica or basebackup path. If no leader exists but cluster birth conditions are satisfied and init-lock rules permit it, the node may participate in `BootstrapCluster`.

#### Existing `pgdata`

Existing `pgdata` is not automatically valid or invalid. The evidence interpreter combines:

- local timeline facts
- local control-file facts
- last known role
- expected leader lineage
- generation/lease evidence

That yields one of:

- safe to continue following
- rewind required
- basebackup required
- hold and wait for stronger evidence

#### Init lock behavior

The init lock becomes part of the evidence table, not a startup-only side path. A node can observe:

- init lock held elsewhere
- init lock free
- init lock locally held
- init lock state unknown

That allows cluster birth decisions to be made in the same architecture as all other decisions.

### Existing local data may still be valid for initialization decisions

The design specifically keeps room for "existing local data is acceptable for cluster birth or reuse" instead of assuming empty directory semantics only. The evidence interpreter classifies local data state as:

```text
DataDirEvidence
  - Empty
  - ExistingAndCompatible
  - ExistingButTimelineDiverged
  - ExistingButUnknown
  - CorruptOrUnreadable
```

That classification then drives bootstrap or convergence intent without special startup code.

## Replica convergence as one coherent path

Option 6 does not make recovery the sole primary abstraction, but it still requires one coherent replica convergence path after authority is determined.

### Ordered convergence path

```text
1. verify leader evidence
2. verify local lineage
3. follow as-is if already compatible
4. tolerate acceptable lag if lineage is compatible
5. rewind if timeline diverged but rewind is possible
6. basebackup if rewind cannot succeed
7. start replica
8. verify streaming and publication quality
```

### Healthy follow

If evidence says the node already follows the credible leader on the correct lineage and lag is acceptable, the decider chooses `ServeReplica` without extra churn.

### Tolerable lag

Lag by itself is not a reason to destabilize role assignment. If lineage and leader evidence remain compatible, the node keeps following and publishes degraded-but-valid lag information.

### Wrong timeline rewind

If local lineage diverges but rewind preconditions are satisfied, the intent becomes `ConvergeReplica(RewindToLeader)`. The decider does not directly run rewind. It emits the typed plan and lets the receiver own execution.

### Basebackup fallback

If rewind cannot succeed, the typed convergence plan explicitly becomes `BasebackupFromLeader`. This preserves the desired fallback ordering without leaving the choice implicit inside process-dispatch helpers.

## Partial information publication

This option requires partial truth to be published always, not only when everything is healthy.

### Example envelopes

#### Agent alive, SQL probe failing, Postgres process observed

```text
agent_status = alive
postgres_process_state = running
sql_probe_state = unreachable
readiness_state = unknown
last_known_role = replica
publication_quality = DirectButPartial
```

That is valuable control-plane truth. It should not be replaced with absence.

#### Agent alive, postgres not running, data dir present, leader exists

```text
agent_status = alive
postgres_process_state = not_running
sql_probe_state = absent
data_dir_state = ExistingAndCompatible
publication_quality = DirectButPartial
```

This tells the cluster that the node exists, has reusable data, and may be recoverable without a full bootstrap.

#### Contradictory local state

```text
postgres_process_state = running
sql_probe_state = healthy
last_known_role = primary
lease_judgement = expired
contradiction_flags = [role_without_valid_authority]
publication_quality = Contradictory
```

This lets other members reason safely about a returning old primary.

## Deduplication boundary

Option 6 explicitly moves deduplication out of sender-side HA logic.

### Why sender-side dedup is unsafe here

The sender cannot know:

- whether a previous start request completed
- whether a rewind is still in progress
- whether a new lease epoch superseded the old intent
- whether a publication revision already reflected a newer fact set

### Receiver-owned dedup model

Each lowered effect includes a stable semantic key:

```text
EffectSemanticKey {
  authority_epoch,
  evidence_revision,
  intent_kind,
  convergence_substate,
}
```

Receivers compare the incoming key with their own last-applied or in-flight key. If it matches, they suppress replay. If it differs in a superseding way, they replace or cancel work. That matches the user's requested ownership boundary much better than `should_skip_redundant_process_dispatch(...)`.

## Concrete repo files, modules, functions, and types a future implementation would touch

This design would require a later implementation to touch at least these areas:

- `src/runtime/node.rs`
  - remove or collapse `plan_startup(...)`
  - remove or collapse `plan_startup_with_probe(...)`
  - remove or collapse `execute_startup(...)`
  - remove or collapse `build_startup_actions(...)`
  - replace them with startup entry into the evidence-first HA loop
- `src/ha/worker.rs`
  - change world-snapshot construction
  - remove sender-side dedup decisions
  - add evidence-table construction and interpretation pipeline
- `src/ha/decide.rs`
  - replace direct phase logic with authority-from-evidence interpretation
  - stop collapsing all non-full-quorum cases into one fail-safe branch
- `src/ha/decision.rs`
  - add `ClusterEvidenceTable`
  - add `MemberEvidenceView`
  - add `AuthorityConclusion`
  - add `LocalNodeIntent`
  - add publication-quality and contradiction types
- `src/ha/lower.rs`
  - lower evidence-derived intents into effect envelopes
  - attach receiver-owned idempotence keys
- `src/ha/process_dispatch.rs`
  - stop deriving authority or startup truth
  - translate typed intent to process commands only
- `src/dcs/worker.rs`
  - publish richer local evidence envelopes
  - preserve partial truth instead of flattening to absence
- `src/dcs/state.rs`
  - extend `MemberRecord` or replace it with richer evidence types
  - represent freshness and contradiction explicitly
- `src/pginfo/state.rs`
  - keep and extend partial information modeling as first-class evidence input
- `tests/ha.rs`
  - later implementation would need scenario updates to reflect clearer evidence and authority boundaries

## Meaningful implementation changes this option would require

If a future implementation chose this option, it would need to make at least these changes:

- add new evidence envelope types for DCS publication
- add new publication quality and contradiction enums
- add a pure `ClusterEvidenceTableBuilder`
- add a pure `PureEvidenceInterpreter`
- delete legacy startup-only planning paths
- move startup into the same HA reconciliation entrypoint as steady-state
- remove sender-side deduplication from HA worker logic
- change `process_dispatch` so it no longer derives authority-sensitive local intent
- change DCS publication behavior to always emit best-known partial truth
- change quorum interpretation to distinguish majority-with-partial-truth from true no-authority states
- change lease handling so it is corroborated by evidence quality rather than local role memory
- change convergence handling so rewind versus basebackup is chosen by a typed convergence plan
- remove stale legacy paths after migration rather than leaving two competing architectures alive

## Logical feature-test verification

This section maps the design logically, not as a promise that the current repo already behaves this way.

### `ha_dcs_quorum_lost_enters_failsafe`

Under Option 6, this scenario should only enter fail-safe if the evidence table classifies the local node as `MinorityOnly` or `NoCredibleQuorum`. The key clarification is that fail-safe is driven by evidence class, not by the mere loss of full visibility.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

If evidence no longer supports confirmed leadership or credible majority authority, the local node transitions to `FenceAndHold`. Publication should explicitly advertise that fencing state, and local writes must be blocked even if PostgreSQL remains locally running for a short time.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side can still produce `MajorityWithPartialTruth` or `FullVisibleMajority`, then deterministically elect a new leader if the old leader lease is no longer corroborated. The isolated old primary produces `MinorityOnly` and must fence instead of continuing writes.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

When the old primary returns, it publishes partial or contradictory truth about its local state. The cluster evidence interpreter sees that the authoritative leader is now elsewhere and assigns `ConvergeReplica`, with `RewindToLeader` or `BasebackupFromLeader` depending on lineage evidence.

### `ha_primary_killed_then_rejoins_as_replica`

The killed primary loses authority as its evidence freshness and lease corroboration expire. On rejoin, it does not self-restore to primary. It republishes local evidence and is assigned a follower or repair path based on the current evidence table.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica returns, its partial publication immediately improves evidence quality even before SQL becomes healthy. That allows the cluster to move from `MinorityOnly` toward `MajorityWithPartialTruth`, then restore service once leadership and follower evidence become credible enough.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes publish partial truth, establish majority evidence, and can deterministically decide whether cluster birth, leadership confirmation, or replica recovery should happen. The final node later joins by publishing its local evidence and receiving a convergence plan.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The convergence path is explicit. If `RewindToLeader` fails or preconditions show rewind is impossible, the plan transitions to `BasebackupFromLeader`. Because this is encoded as a typed convergence stage, the fallback should no longer be hidden inside ad hoc dispatch logic.

### `ha_replica_stopped_primary_stays_primary`

As long as majority evidence and lease corroboration remain valid, a single stopped replica does not destabilize the primary. The primary continues serving while the stopped replica publishes degraded or absent process truth when it comes back.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A returning broken replica publishes partial or contradictory evidence, but that does not override majority-supported leadership. It receives a repair or wait intent without destabilizing the already-healthy authority conclusion.

## Migration sketch

This option requires a later implementation to migrate aggressively rather than leave stale architectural leftovers.

### Stage 1: Introduce evidence types without changing authority behavior

- add richer local evidence envelope types
- extend DCS publication to carry explicit quality and contradiction fields
- build `ClusterEvidenceTable` alongside the current snapshot path
- keep existing HA decisions for one short transitional window

### Stage 2: Switch the pure decider to evidence-first authority interpretation

- introduce `AuthorityConclusion`
- drive quorum and fail-safe logic from evidence classes
- preserve current effect interfaces temporarily while switching decision inputs

### Stage 3: Fold startup into the same entrypoint

- remove startup-only planners from `src/runtime/node.rs`
- route startup through the same evidence build and decision flow as steady-state
- ensure init-lock and data-dir facts appear in evidence types instead of startup-only helpers

### Stage 4: Remove sender-side dedup and push it to receivers

- add semantic idempotence keys to lowered effects
- update process, DCS, lease, and fencing receivers to own replay suppression
- delete `should_skip_redundant_process_dispatch(...)`-style logic from HA worker code

### Stage 5: Delete legacy paths completely

- remove obsolete startup path helpers
- remove obsolete authority bridging in `process_dispatch`
- remove duplicate truth representations if `MemberRecord` becomes insufficient
- keep only one evidence-first decision architecture alive

The important migration rule is that the repo must not keep both "old startup planner" and "new evidence-first loop" permanently. This project is greenfield and should not preserve legacy structure.

## Non-goals

- This option does not claim that DCS publication alone solves every HA problem.
- This option does not move Postgres or etcd side effects into the pure decider.
- This option does not try to preserve legacy API shapes for backwards compatibility.
- This option does not keep sender-side dedup "for convenience."
- This option does not require full observability before any action can happen; it explicitly supports partial truth.

## Tradeoffs

- The publication schema becomes larger and more explicit.
- The control plane must handle contradiction semantics carefully or it will become noisy.
- More explicit evidence categories mean more up-front type design.
- If implemented poorly, richer publication could create operator confusion unless logs and diagnostics clearly explain which evidence fields are driving decisions.
- This option puts more architectural weight on DCS publication shape than the earlier options do.

## ASCII diagram for authority and startup unification

```text
             startup tick / steady-state tick
                       |
                       v
              [ gather local facts ]
                       |
                       v
            [ publish best-known local truth ]
                       |
                       v
           [ read cluster member evidence table ]
                       |
                       v
         [ interpret evidence quality + contradictions ]
                       |
          +------------+------------+
          |                         |
          v                         v
 [ credible majority / leader ]   [ minority / weak evidence ]
          |                         |
          v                         v
 [ serve / converge / bootstrap ] [ fence / wait / repair ]
```

## Q1 How much evidence should publication carry directly

The design assumes member publication becomes a richer evidence envelope rather than a thin role/status summary. That makes the decider simpler, but it raises a schema-boundary question.

```text
thin member record  --->  richer evidence envelope  --->  simpler HA interpretation
```

Should the DCS publication carry all first-class evidence fields directly, or should it carry only a compact summary plus references to local-only detail that the decider reconstructs elsewhere?

In other words, how far should this architecture push "publication-first" before the member key becomes too overloaded?

## Q2 How should contradictions be surfaced to operators

This option treats contradictions as first-class instead of silently choosing one source of truth. That is safer, but it affects operator ergonomics.

```text
process=running
sql=healthy
lease=expired
role=primary
=> contradiction: role_without_valid_authority
```

Should contradictions appear directly in member publication and logs as explicit machine-readable flags, or should they be collapsed into a smaller set of operator-facing reason codes?

What is the best representation so the system stays transparent without becoming too verbose?

## Q3 How strict should majority-with-partial-truth be

This option allows continued service or re-election when a majority remains visible even if some evidence is partial. That is the desired direction, but the exact boundary still matters.

```text
node A = fresh direct
node B = fresh partial
node C = absent
=> majority with partial truth
```

Which evidence fields must remain fresh and direct before a majority is strong enough to keep serving writes, and which fields may safely remain partial without over-risking split-brain or stale-primary behavior?

Put differently, what is the minimum evidence set for "degraded but still authoritative"?

## Q4 Should publication revision participate in effect idempotence

The design proposes effect semantic keys that include authority epoch, evidence revision, intent kind, and convergence substate. That helps make actions sensitive to newly published truth, but it may also increase churn.

```text
effect key = authority_epoch + evidence_revision + intent_kind + convergence_substate
```

Should publication revision be part of every receiver-owned idempotence key, or should only authority-meaningful evidence changes produce new effect identities?

How do we keep receivers safe and precise without turning harmless evidence refreshes into unnecessary work replacement?

## Q5 How much of the evidence model should be materialized in DCS versus kept local

This design leans hard on DCS publication as the cluster's authoritative evidence substrate. That raises a boundary question about what must be persisted centrally and what may remain a local interpretation detail.

```text
local facts --> local evidence envelope --> DCS --> cluster evidence table
```

Which evidence fields truly need to be materialized in DCS for cross-node reasoning, and which ones should stay local to avoid overcomplicating the shared control plane?

In another phrasing: where is the right line between "shared truth needed for HA correctness" and "local debugging detail that should not shape cluster-wide contracts"?
