# Option 8: Ballot-Driven Majority Constitution

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. This document exists only to describe one complete redesign option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA architecture still reasons too weakly about one question the user explicitly called out: when does a node have a valid majority-backed mandate to keep serving, elect a new primary, or reject a risky move? The present code has lease concepts, fail-safe concepts, and member publication concepts, but the degraded-majority boundary still collapses too quickly into `FailSafe` in `src/ha/decide.rs`. That is a symptom of a missing architectural object. The system lacks an explicit, typed notion of the current voting constitution and the current election ballot.

The differentiator of Option 8 is that the pure HA decider is centered on two cluster-wide typed objects:

- `ClusterConstitution`: who currently counts as a voting member, how fresh that claim is, and what majority threshold is valid right now
- `ElectionBallot`: what leadership mandate is currently being claimed, challenged, retained, or revoked for the active constitution

Startup and steady-state are unified because every tick starts by reconstructing the newest possible constitution and ballot view from DCS publications plus local observations. The pure decider does not directly decide "start Postgres" or "become primary." It decides whether this node holds, can win, must surrender, or must reject a mandate under the presently valid constitution. Everything else is derived from that result through typed lower layers.

Option 1 centered one unified lifecycle kernel. Option 2 centered separate cluster and local state machines. Option 3 centered lease authority as the dominant abstraction. Option 4 centered a single recovery funnel. Option 5 centered generation cutovers and node contracts. Option 6 centered evidence quality and publication as the authoritative substrate. Option 7 centered explicit obligations, blockers, and satisfaction proofs. Option 8 is materially different from all seven because it treats cluster legitimacy itself as the top-level abstraction: the system should first know what majority constitution exists, then know what ballot is live inside it, and only then decide local execution.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign studies.
- The failure themes gathered earlier in this task remain directly relevant here:
  - quorum-loss scenarios still appear too tightly coupled to a blunt `fail_safe` path instead of a typed degraded-majority interpretation
  - majority-side failover scenarios did not reliably expose a replacement primary from the healthy majority partition
  - some restart and restore-service scenarios left a node answering as writable primary when the scenario expected service to remain blocked
  - targeted switchover toward a degraded replica succeeded when it should have been rejected by stronger mandate validation
  - rewind-to-basebackup fallback evidence was not consistently visible, which suggests the non-primary convergence path is still too implicit
  - proof-row convergence failures imply authority loss, demotion, or rejoin semantics remain ambiguous under disruption

These observations do not prove that a ballot/constitution model is sufficient on its own, but they strongly suggest that the current code does not model legitimacy crisply enough when the cluster is partially degraded yet still majority-capable.

## Current design problems

### Startup logic is split away from the long-running reconciliation model

`src/runtime/node.rs` still performs important startup planning before the HA loop fully takes over. That means the system can make initialization, follow, bootstrap, or startup decisions before it has passed through the same pure decision model that later steady-state ticks use. This is exactly the architectural drift the user described. Under this option, startup must not be a separate planner. Startup must be the first constitution-and-ballot evaluation tick.

### Sender-side dedup remains inside `src/ha/worker.rs`

The current HA worker still contains sender-side dedup concerns such as `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. That means the sender is partly guessing whether consumers already converged. Under this option, the sender never guesses that. It emits mandate-based effect plans tagged with constitution and ballot identifiers, and effect consumers decide whether that exact mandate/action pair is already satisfied.

### HA reasoning is spread across too many boundaries

The present flow spreads "what is safe?", "what is desired?", and "what should this node do next?" across runtime startup planning, HA decide, process dispatch bridging, and lower/effect handling. Under this option, those questions get a sharper order:

1. What constitution is currently valid?
2. What ballot or retained mandate exists within that constitution?
3. What local role or convergence duty follows from that mandate?
4. What actions do effect consumers need to take?

That ordering reduces ambiguity and gives the degraded-majority question a first-class home instead of burying it inside special-case phase logic.

### The non-full-quorum to `FailSafe` shortcut is too blunt

`src/ha/decide.rs` currently routes any non-`DcsTrust::FullQuorum` state toward `HaPhase::FailSafe`, including cases where a 2-of-3 majority can still act. This is the clearest behavioral mismatch called out by the user. Under this option, `full quorum` and `valid acting majority` are different concepts. Loss of full visibility is not automatically loss of mandate. The key question becomes whether the current constitution still has a certified majority and whether any node still holds a valid ballot-backed mandate under that majority.

### Startup and rejoin logic are still too implicit in `src/ha/process_dispatch.rs`

The current dispatch bridge derives `StartPostgres` and convergence behavior partly from prior HA decisions and local state. That forces startup/rejoin meaning to emerge indirectly. Under this option, the pure decider explicitly emits a `LocalMandate` telling the node whether it must retain primary service, enter a candidate state, remain fenced, follow an existing leader, rewind, basebackup, bootstrap, or simply wait for more evidence.

### Member publication does not yet fully embody partial truth as a first-class input

`src/dcs/worker.rs`, `src/dcs/state.rs`, and `src/pginfo/state.rs` already contain partial-information structure, but the present architecture still allows uncertainty to be flattened into absence or treated too weakly in higher layers. Under this option, constitution construction is explicitly tolerant of partial truth. A node can publish "agent alive, pginfo unknown, readiness unknown" and still matter to the constitution as an uncertain or degraded voter. Silence and uncertainty are not the same.

## Core idea

The heart of this option is that cluster authority is not a role label and not just a lease record. It is a constitution-backed mandate. A node may act as primary only when it holds a mandate that is both:

- consistent with the latest known cluster constitution
- still certified by a valid majority under freshness and lineage rules

The pure HA tick therefore becomes:

1. gather newest observations
2. build a typed constitution view
3. build a typed ballot view
4. derive the node's current local mandate
5. lower that mandate into idempotent effect plans

The constitution answers "who counts?" The ballot answers "who is authorized?" The local mandate answers "what must this node do?"

## Proposed control flow from startup through steady state

Every node, including on first process start, runs the same conceptual loop.

```text
+-----------------------------+
| newest observations arrive  |
| local pginfo                |
| local process state         |
| DCS member records          |
| DCS lease / election keys   |
| local persisted lineage     |
+-------------+---------------+
              |
              v
+-----------------------------+
| Constitution Builder        |
| - classify voters           |
| - assess freshness          |
| - form acting majority set  |
| - detect constitution split |
+-------------+---------------+
              |
              v
+-----------------------------+
| Ballot Builder              |
| - retained leader mandate   |
| - challenger ballots        |
| - mandate revocation        |
| - lease freshness           |
+-------------+---------------+
              |
              v
+-----------------------------+
| Pure HA Decider             |
| -> ClusterMandate           |
| -> LocalMandate             |
| -> SafetyInvariants         |
+-------------+---------------+
              |
              v
+-----------------------------+
| Lowering Layer              |
| - lease effects             |
| - publication effects       |
| - postgres effects          |
| - replication effects       |
| - fencing effects           |
+-------------+---------------+
              |
              v
+-----------------------------+
| Receiver-Owned Consumers    |
| - dedup by mandate token    |
| - idempotent execution      |
| - publish applied state     |
+-----------------------------+
```

The essential startup change is that there is no separate "startup planner" outside this flow. The first tick simply begins with more unknowns. Unknowns do not justify a separate architecture. They justify typed uncertainty inside the same architecture.

## Proposed typed state model

This option introduces several new pure types. The names can change in implementation, but the boundaries should remain.

### `ClusterConstitution`

This type describes the currently understood voter set and whether an acting majority exists.

```text
ClusterConstitution
- constitution_id: ConstitutionId
- voters: Vec<VoterStatus>
- majority_threshold: u16
- acting_majority: ActingMajority
- freshness_class: ConstitutionFreshness
- contradictions: Vec<ConstitutionConflict>
```

`VoterStatus` should include:

- `member_id`
- `publication_state`
- `agent_liveness`
- `pg_state_quality`
- `role_claim`
- `timeline_claim`
- `wal_position_claim`
- `last_observed_at`
- `vote_eligibility`

`ActingMajority` should be explicit:

- `Certified { voters: Vec<MemberId>, evidence_cutoff: Instant }`
- `UncertainButRecoverable { missing: Vec<MemberId>, available: Vec<MemberId> }`
- `Absent`
- `SplitView`

This type is the main fix for the current "non-full-quorum means fail-safe" error. `Certified` and `UncertainButRecoverable` are not the same as full visibility, and they must not be forced through the same path.

### `ElectionBallot`

This type expresses what leadership mandate is currently claimed or contestable.

```text
ElectionBallot
- ballot_id: BallotId
- constitution_id: ConstitutionId
- round: BallotRound
- incumbent: Option<LeaderMandate>
- challengers: Vec<ChallengerBallot>
- revocation_state: RevocationState
- lease_view: LeaseView
```

`LeaderMandate` should include:

- `leader_member_id`
- `mandate_epoch`
- `supporting_voters`
- `timeline_anchor`
- `lease_deadline`
- `writability_policy`

`ChallengerBallot` should include:

- `candidate_member_id`
- `eligibility`
- `data_fitness`
- `supporting_voters`
- `blocking_reasons`

### `ClusterMandate`

This is the pure decider's cluster-wide conclusion.

Possible variants:

- `RetainLeader { mandate: LeaderMandate }`
- `ElectLeader { mandate: LeaderMandate }`
- `PrepareElection { eligible_candidates: Vec<MemberId>, blockers: Vec<ElectionBlocker> }`
- `RevokeMandate { prior_leader: MemberId, reasons: Vec<RevocationReason> }`
- `SuspendWrites { reason: SafetyFreezeReason }`
- `BootstrapCluster { bootstrap_plan: BootstrapMandate }`
- `AwaitEvidence { missing: Vec<EvidenceGap>, safe_posture: SafePosture }`

### `LocalMandate`

This is what the node itself must do.

Possible variants:

- `ServePrimary { mandate_token: MandateToken, writability: WritabilityMode }`
- `CandidatePrimary { mandate_token: MandateToken, prerequisites: Vec<CandidatePrerequisite> }`
- `FollowLeader { leader: MemberId, follow_mode: FollowMode }`
- `ConvergeReplica { leader: MemberId, path: ReplicaConvergencePath }`
- `BootstrapNode { plan: BootstrapNodePlan }`
- `PublishOnly { reason: PublishOnlyReason }`
- `FenceAndStandDown { reason: FenceReason }`
- `WaitForBallotResolution { blockers: Vec<BallotBlocker> }`

This separation matters. `ClusterMandate` can say "a healthy majority may elect." `LocalMandate` says "this node is not that candidate, so it must publish, fence writes, and follow once the leader is certified."

### `MandateToken`

Every actionable lowered effect is tagged with a token:

```text
MandateToken
- constitution_id
- ballot_id
- mandate_epoch
- local_sequence
```

This token is the dedup mechanism. Receivers own it. If a Postgres consumer already applied a `MandateToken` for "remain fenced" or "stay primary under mandate X", the sender does not need to guess that. The receiver can compare the new token to its applied token and decide whether execution is necessary.

## Detailed phase model

Although this option is centered on constitution and ballot objects rather than one big enum, it still benefits from explicit phases for human reasoning.

### Phase A: Observation Assimilation

Inputs:

- local pginfo worker output
- local process activity snapshot
- DCS member records from `src/dcs/worker.rs`
- DCS lease/election keys
- local persisted cluster identity and timeline lineage

Output:

- `ObservationFrame`

Rules:

- partial truth must stay partial, not absent
- stale entries must be marked stale, not silently ignored
- conflicting claims must be retained for the constitution builder

### Phase B: Constitution Construction

Goal:

- determine what voter set is presently credible and whether a safe acting majority exists

Key invariants:

- no node may act as writable primary without a constitution-backed majority
- loss of full visibility is not the same as loss of acting majority
- constitution ambiguity must be represented explicitly

### Phase C: Ballot Evaluation

Goal:

- determine whether an incumbent mandate remains valid, whether it must be revoked, or whether an election may proceed

Key invariants:

- a retained leader must still satisfy lease freshness and constitution support
- a challenger must pass data-fitness gates before it can become the next leader
- a revoked leader must not continue writable service after the cutoff window

### Phase D: Local Mandate Derivation

Goal:

- translate the cluster conclusion into precise local behavior

Key invariants:

- local behavior is a consequence of mandate legitimacy, not ad hoc role guessing
- startup is not a special path; it is simply the first local mandate
- an old primary with a revoked mandate may never self-justify continued service

### Phase E: Lowering and Consumer Execution

Goal:

- lower mandate into concrete effect categories

Categories:

- `LeaseEffect`
- `PublicationEffect`
- `PostgresEffect`
- `ReplicationEffect`
- `SafetyEffect`

Key invariant:

- all effect consumers deduplicate by `MandateToken` plus consumer-local state

## Redesigned quorum model

This option deliberately separates four concepts that the current system blends too aggressively:

1. `Full Visibility`
2. `Acting Majority`
3. `Leader Mandate`
4. `Write Permission`

The current architecture behaves as though loss of full visibility often implies loss of safe operation. This option rejects that.

### Full visibility

All configured voters are publishing fresh enough information. This is the strongest case, but it should not be required for continued service in a healthy 2-of-3 majority.

### Acting majority

A certified majority subset exists with fresh enough evidence to support a constitution. This is sufficient to continue operation and elect a leader, even if one member is absent, partitioned, or publishing stale/partial truth.

### Leader mandate

Within an acting majority, exactly one leader may hold the current certified mandate. If the incumbent still satisfies support and lease conditions, it may retain. If not, the mandate is revoked and a new ballot can elect.

### Write permission

Write permission is narrower than leadership identity. A node may know it is the most likely leader candidate, but until it holds a certified mandate and has crossed local readiness gates, it must not expose writable primary service.

### Behavior in 2-of-3 style degraded clusters

The user explicitly wants a three-node cluster with two healthy members to continue operating and electing. This option handles that by allowing `ClusterConstitution::acting_majority = Certified` when two fresh, consistent voters agree even though one is missing. The decider may then either retain the current leader or elect a new one. The missing third node does not force `FailSafe`; it only reduces observability and increases scrutiny of freshness and lineage.

### When the node must still fence or suspend writes

This option still fences aggressively in the following cases:

- no acting majority can be certified
- the constitution is split and conflicting majorities appear possible from stale evidence
- the incumbent mandate lost support and no replacement mandate is yet certified
- lease freshness is beyond the maximum tolerated cutoff for writable service
- local PostgreSQL state contradicts the mandate's required lineage or role

This preserves safety while fixing the current overuse of the fail-safe path.

## Redesigned lease model

Lease semantics remain essential, but lease is not the only authority input. Lease is one component of ballot certification.

### Lease acquisition

When a node is elected or retained as leader, the `LeaderMandate` includes:

- lease holder id
- mandate epoch
- lease deadline
- constitution id that certified it
- timeline anchor and data-fitness proof

The key difference from the current shape is that a lease without constitution support is not enough. A node must hold a lease that belongs to a certified ballot under the active constitution.

### Lease retention

The incumbent may keep serving if:

- its mandate is still supported by the acting majority
- its lease deadline has not crossed revocation thresholds
- its publication and local pginfo are consistent with leader obligations

### Lease loss

If lease freshness passes the configured cutoff, the ballot builder marks the mandate as revocable. The pure decider then outputs either:

- `RevokeMandate`
- `SuspendWrites`
- `AwaitEvidence`

depending on whether a replacement election is already certifiable.

### Killed primary and lost authority

A killed primary loses authority not because another module guessed it is dead, but because the ballot can no longer certify its mandate. Once the acting majority sees the incumbent as missing, stale, or lease-expired beyond tolerance, the ballot revokes the mandate and a challenger may be elected. On restart, that old primary does not bootstrap ad hoc. It starts from the same constitution-and-ballot evaluation and receives a `LocalMandate` like `ConvergeReplica` or `FenceAndStandDown`.

## Startup reasoning

Startup must be folded into the same model. This option handles startup by requiring the first tick to answer three questions in order:

1. What constitution is currently credible?
2. What ballot or retained mandate already exists in that constitution?
3. Given local data and role evidence, what mandate can this node legally hold?

### Cluster already up with a healthy leader

The constitution builder finds an acting majority and the ballot builder finds an incumbent leader mandate. A node starting up receives either:

- `FollowLeader`
- `ConvergeReplica`
- `PublishOnly`

depending on its local data fitness. It never runs a special startup planner that bypasses the mandate model.

### Cluster leader already present but local `pgdata` exists

The node compares local lineage against the leader mandate's timeline anchor.

- If compatible and caught up enough, it receives `FollowLeader`.
- If divergent but rewindable, it receives `ConvergeReplica { path: RewindThenFollow }`.
- If divergent and not rewindable, it receives `ConvergeReplica { path: BasebackupThenFollow }`.

### Existing members already published

Existing member records contribute to the constitution even if some are degraded. This matters for deciding whether the cluster is bootstrapping, retaining leadership, or recovering after disruption.

### Empty versus existing `pgdata`

An empty data directory does not automatically imply fresh bootstrap. It implies "no local lineage." The decider still first checks whether a leader mandate already exists. If yes, the node follows or basebacks from that leader rather than attempting cluster initialization.

### Init lock behavior

The init lock becomes one input to `BootstrapCluster`, not a parallel control plane. Winning the init lock only matters if the constitution says no certified cluster exists yet and bootstrap is safe. If a node wins the lock but discovers valid existing data plus an established leader mandate, bootstrap is rejected.

### Using existing `pgdata` when the node wins init lock

This option explicitly allows reuse of existing local data if that data proves the node is already the best bootstrap candidate and no prior cluster mandate exists. The bootstrap path should check:

- cluster identity absence in DCS
- absence of certified leader mandate
- local data usability and consistency
- whether local data should seed cluster initialization or be discarded

This removes the current ambiguity where init-lock ownership can appear to overrule richer startup evidence.

## Replica convergence as one coherent path

This option keeps a unified convergence path but derives it from the local mandate rather than from scattered special cases.

`ReplicaConvergencePath` should include:

- `AlreadyHealthyFollow`
- `CatchUpFollow`
- `LagTolerantFollow`
- `RewindThenFollow`
- `BasebackupThenFollow`
- `WaitForLeaderReadiness`

The decider chooses among these using:

- leader mandate validity
- timeline compatibility
- WAL position comparison
- rewind capability evidence
- local process health

### Healthy follow

If local data matches the leader mandate and replication can proceed, the mandate is simply `FollowLeader`.

### Tolerable lag

If lag is acceptable and no rewind is needed, the node still follows. Lag alone should not destabilize quorum.

### Wrong timeline with rewind possible

The local mandate becomes `ConvergeReplica { path: RewindThenFollow }`. This is explicit and monotonic, not inferred later by a side path.

### Rewind impossible

The local mandate becomes `ConvergeReplica { path: BasebackupThenFollow }`. This is not a failure of the model. It is the final step in the convergence ladder.

### Previously-primary, previously-replica, freshly-restored nodes

All are handled with the same convergence family. What changes is the evidence:

- old primary after failover usually needs rewind or basebackup
- stopped replica often resumes direct follow
- restored node may need basebackup before follow

The architecture does not need separate conceptual subsystems for each history class.

## Partial-truth member publication

This option depends on better publication discipline.

### Publication principle

If pgtuskmaster is running and knows anything, it should publish that truth. Unknown is still a truth value. Partial truth must remain machine-readable.

### Required publication fields

Future implementation should ensure `MemberRecord` or its replacement can publish:

- agent liveness
- pginfo collection success/failure class
- SQL reachability
- readiness state
- observed local role claim
- observed timeline
- observed LSN/WAL position when known
- lease-holder belief when known
- last successful probe timestamps
- contradiction markers when local observations disagree

### Why this matters for constitutions

The constitution builder needs to classify voters as:

- fully trusted and fresh
- partial but still alive
- stale
- contradictory
- absent

That classification is impossible if missing data is encoded as silence.

## Deduplication boundary

Sender-side dedup is removed from HA logic.

### Current problem

`src/ha/worker.rs` currently contains logic that tries to determine whether the next process-dispatch action is redundant. That couples the pure decision producer to consumer-side applied state.

### Proposed replacement

Every lowered effect carries:

- `MandateToken`
- effect category
- consumer-local desired state

Consumers then store their last applied token and desired state. Dedup rules live with the consumer:

- Postgres consumer decides whether "ensure writable primary under token X" is already satisfied
- replication consumer decides whether "follow leader Y under token X" is already satisfied
- DCS publication consumer decides whether "publish member view Z under token X" is already satisfied

This is safer because consumers own the only authoritative view of what they actually applied.

## Concrete future code areas that would change

A later implementation of this option would need to touch at least the following areas:

- `src/runtime/node.rs`
  - remove or collapse separate startup planning into the unified constitution/ballot tick
  - convert startup execution into the same mandate-lowering path used later
- `src/ha/worker.rs`
  - replace sender-side dedup with mandate-token emission
  - rebuild world-snapshot handling around constitution and ballot builders
- `src/ha/decide.rs`
  - replace current phase-first logic with constitution/ballot evaluation
  - distinguish acting majority from full visibility
- `src/ha/decision.rs`
  - introduce `ClusterConstitution`, `ElectionBallot`, `ClusterMandate`, `LocalMandate`, `MandateToken`, and related enums
  - retire or reshape phase/decision types that no longer reflect the new model
- `src/ha/lower.rs`
  - lower `LocalMandate` to effect plans
  - attach mandate tokens to every effect
- `src/ha/process_dispatch.rs`
  - stop inferring startup/rejoin meaning from prior decision history
  - consume explicit local mandate variants for follow/rewind/basebackup/bootstrap
- `src/dcs/worker.rs`
  - publish richer partial-truth member records
  - read/write election and mandate keys through DCS only
- `src/dcs/state.rs`
  - add constitution-related freshness and voter classification helpers
  - model acting-majority certification separately from full quorum
- `src/pginfo/state.rs`
  - preserve partial truth needed for voter classification
- `tests/ha.rs`
  - update the behavioral expectations and fixtures to reflect mandate-driven degraded-majority behavior
- `tests/ha/features/`
  - scenario expectations, log evidence, and election transitions would need updates during later implementation

## All meaningful changes required for this option

This section deliberately spells out concrete later changes rather than saying "refactor accordingly."

### New types to add

- `ClusterConstitution`
- `ConstitutionId`
- `VoterStatus`
- `ActingMajority`
- `ConstitutionFreshness`
- `ConstitutionConflict`
- `ElectionBallot`
- `BallotId`
- `LeaderMandate`
- `ChallengerBallot`
- `RevocationState`
- `ClusterMandate`
- `LocalMandate`
- `MandateToken`
- `ReplicaConvergencePath`
- `BootstrapMandate`
- `CandidatePrerequisite`
- `BallotBlocker`

### Existing paths to remove or collapse

- separate startup planner/executor split in `src/runtime/node.rs`
- sender-side dedup heuristics in `src/ha/worker.rs`
- implicit start-intent bridging that depends too strongly on prior decision residue in `src/ha/process_dispatch.rs`
- unconditional mapping from non-full-quorum to fail-safe behavior in `src/ha/decide.rs`

### Responsibilities to move

- startup topology reasoning from runtime startup helpers into the pure decider input pipeline
- authority legitimacy from loosely combined lease/quorum code into ballot evaluation
- dedup from HA sender logic into effect consumers
- voter classification from ad hoc trust shortcuts into constitution building

### State transitions to redefine

- leader retention under degraded majority
- leader revocation after lease expiry or majority loss
- candidate promotion only after ballot certification
- old-primary restart into replica convergence instead of ambiguous startup branching
- bootstrap eligibility when init lock exists but old data or prior cluster evidence also exists

### Effect-lowering boundary changes

- lower from `LocalMandate` rather than from mixed phase/decision artifacts
- ensure every action carries a mandate token
- keep DCS IO inside DCS worker and Postgres IO inside pginfo/process consumers

### DCS publication changes

- member records must encode partial truth explicitly
- DCS election keys should represent mandate/ballot state cleanly enough for restart reconciliation
- constitution support and voter freshness should be reconstructible from DCS state

### Startup handling changes

- first tick uses the same decision model as later ticks
- no separate "pre-HA" path
- bootstrap, follow, and converge choices all arise from local mandate derivation

### Convergence handling changes

- replica convergence path becomes one explicit type family
- rewind and basebackup become planned mandate outcomes rather than fallback guesses
- convergence may proceed without destabilizing the cluster's acting majority

### Test updates a future implementation would need

- feature tests should assert acting-majority behavior separately from full-quorum behavior
- logs and evidence should show mandate acquisition, mandate revocation, and ballot resolution events
- switchover tests should fail when the target lacks ballot-eligible data fitness
- old-primary rejoin tests should assert explicit converge path selection

## Migration sketch

This task does not implement code, but the option needs an executable migration path for a future implementation.

### Step 1: Introduce typed observation and mandate types alongside current logic

Add `ClusterConstitution`, `ElectionBallot`, and `LocalMandate` as parallel pure data structures. Do not yet remove existing phases. Use adapter code to derive the new types from the existing world snapshot for comparison.

### Step 2: Build constitution and ballot builders in parallel

Add pure builder modules that classify voters and evaluate ballots from the current DCS/member/pginfo inputs. Log or test them in isolation without changing runtime behavior yet.

### Step 3: Replace non-full-quorum shortcut with acting-majority evaluation

Once the builders are trusted, route degraded-majority behavior through constitution/ballot logic instead of directly through `FailSafe`. This is the architectural heart of the option.

### Step 4: Derive `LocalMandate` and lower from it

Replace mixed phase/decision lowering with explicit local mandate lowering. Keep existing effect consumers, but make them receive tokenized mandate effects.

### Step 5: Move dedup into consumers

Delete sender-side dedup in `src/ha/worker.rs`. Add applied-token tracking to effect consumers so idempotence becomes receiver-owned.

### Step 6: Remove separate startup planner

Collapse `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and related helpers into first-tick mandate derivation. Any surviving startup-only path should be treated as leftover legacy and removed.

### Step 7: Tighten DCS publication schema

Update DCS publication to preserve partial truth and the fields needed to rebuild constitutions and ballots after restart.

### Step 8: Remove legacy compatibility paths

Once the new model owns startup, authority, convergence, and dedup boundaries, delete the old phase assumptions rather than carrying both models indefinitely. This repo is greenfield. Stale compatibility paths should not survive.

## Logical feature-test verification

This section explains how the option would logically satisfy the current HA scenarios during a later implementation. It does not claim those tests pass today.

### `ha_dcs_quorum_lost_enters_failsafe`

This scenario should be reinterpreted more precisely under this option. If the cluster truly loses any certifiable acting majority, the constitution becomes `Absent` or `SplitView`, the current mandate is revoked, and the local node receives `FenceAndStandDown` or `SuspendWrites`. If the scenario specifically models total loss of certifiable majority, the behavior still maps to fail-safe-like fencing. What changes is that the fence occurs because no constitution-backed ballot exists, not because "not full quorum" was used as a shortcut.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This option strengthens this behavior. The incumbent leader may keep serving only until mandate revocation thresholds are crossed. After the cutoff, the ballot builder marks the mandate revoked, the local mandate becomes `FenceAndStandDown`, and consumer-owned dedup ensures writability is not re-enabled by stale sender assumptions.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

This is a direct target for the option. The healthy 2-of-3 side still forms a certified acting majority even without full visibility. The constitution builder certifies that majority, the ballot revokes the isolated primary's mandate once its support and lease freshness are gone, and an eligible challenger receives a new leader mandate. The isolated old primary cannot justify continued authority because it lacks constitution support.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

On heal, the old primary starts from constitution-and-ballot evaluation. It sees a certified new leader mandate under the active constitution, recognizes its own mandate as revoked, and receives `ConvergeReplica` rather than any self-promoting startup path. Timeline compatibility determines whether the convergence path is direct follow, rewind, or basebackup.

### `ha_primary_killed_then_rejoins_as_replica`

A killed primary loses mandate certification after its lease/support cutoff. A successor can be elected by the acting majority. When the old primary returns, it no longer has any retained ballot-backed authority, so it enters replica convergence under the new leader mandate.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

When one replica returns, the constitution builder recomputes the acting majority. If the restarted node plus the healthy node form a certified majority, the ballot can retain or certify a legitimate leader mandate again. Service restoration therefore comes from constitution recovery, not from ad hoc node role assumptions.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

The first two restarted nodes rebuild the constitution from scratch. If they can form a certified majority and one has the best eligible data fitness, a ballot can re-establish leadership. The final node then rejoins as a follower or convergence replica based on lineage. Startup is unified here because no one takes a separate "cold start" branch outside the mandate model.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

The local mandate becomes `ConvergeReplica`. The path is initially `RewindThenFollow`. If rewind capability or execution proves impossible, the convergence path transitions explicitly to `BasebackupThenFollow`. This should make both behavior and log evidence clearer than the current more implicit fallback path.

### `ha_replica_stopped_primary_stays_primary`

If the primary still retains a constitution-backed mandate and the acting majority remains valid, the primary stays primary. A stopped replica does not destabilize the ballot unless its absence destroys the ability to certify the majority.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

A broken replica receives either `WaitForBallotResolution`, `PublishOnly`, or `ConvergeReplica` depending on evidence. None of those mandates allows it to destabilize the cluster's existing leader mandate. The ballot model isolates follower repair from leader legitimacy.

## Non-goals

- This option does not propose direct code changes in this task.
- This option does not claim the current tests already pass.
- This option does not replace the requirement for strong lease semantics; it embeds lease semantics inside a broader legitimacy model.
- This option does not require general-purpose distributed consensus beyond the existing DCS-backed architecture. It changes local reasoning structure, not the storage substrate.

## Tradeoffs

### Strengths

- explicitly fixes the degraded-majority boundary the user called out
- gives startup and steady-state one legitimacy model
- makes switchover and election eligibility more explainable
- gives old-primary rejoin a crisp authority story
- moves dedup to the only safe place: effect consumers

### Costs

- introduces more top-level types than the current model
- requires careful migration because constitution and ballot concepts cut across multiple existing modules
- demands better DCS publication fidelity to classify voters well
- requires disciplined log/event design so operators can understand constitution and ballot state

### Risks

- if constitution rules are too complex, the implementation could become hard to reason about
- if ballot storage in DCS is underspecified, restart behavior could become ambiguous
- if local data-fitness rules are too permissive, unsafe elections could still occur

## Open questions

## Q1 How should the constitution be versioned?

The option assumes a `ConstitutionId`, but there are several possible generation rules:

```text
inputs change -> new constitution id?
membership change -> new constitution id?
freshness degradation only -> same id, different class?
```

The implementation needs a crisp answer because mandate tokens, ballot resets, and operator logs all depend on whether constitution identity changes only on membership change or also on freshness/eligibility changes.

Restated question: should constitution identity represent configured voter membership only, or the full current voting eligibility state?

## Q2 Should a leader mandate survive brief constitution uncertainty?

There may be a short interval where publications are stale enough that the acting majority is not currently certifiable, but no contradictory evidence exists either.

```text
fresh leader evidence ages out
no challenger evidence appears
network jitter clears quickly
```

If the system revokes immediately, it may flap. If it waits too long, it may extend unsafe writability. The exact retention grace window is therefore central to safety and availability.

Restated question: how much temporary ambiguity should be tolerated before a retained leader mandate is revoked?

## Q3 Where should ballot state live in DCS?

The DCS layer must remain the only component that reads and writes etcd keys, but this option requires explicit ballot and mandate reconstruction on restart.

Possible layouts include:

- one leader-mandate key plus per-voter support records
- one election-round key plus candidate support table
- a compact certified-mandate record derived from member publication

The storage shape affects recoverability, debuggability, and split-brain resistance.

Restated question: what DCS record layout best represents ballots without introducing unnecessary write amplification or ambiguity?

## Q4 How strict should data-fitness be for challenger eligibility?

This option assumes an election should reject a degraded replica that cannot safely become primary, which directly addresses the targeted-switchover failure theme. But "safe enough" still needs a precise rule set.

Relevant factors include:

- WAL freshness
- replay lag
- timeline continuity
- local read/write readiness
- recent probe health

An over-strict rule will block legitimate failovers. An under-strict rule will promote weak candidates.

Restated question: what exact evidence threshold should make a challenger ballot-eligible for leader mandate certification?

## Q5 Should bootstrap use the same ballot machinery or a dedicated bootstrap ballot?

Bootstrap has unique semantics because there may be no incumbent leader and no established lineage yet.

```text
empty cluster
no certified constitution history
init lock winner emerges
bootstrap candidate proposes first mandate
```

Using the same ballot machinery everywhere improves conceptual unity, but bootstrap may still deserve a dedicated sub-variant with additional safety checks around existing `pgdata`, cluster identity creation, and first publication rules.

Restated question: should initial cluster formation be represented as the same election-ballot type, or as a dedicated bootstrap ballot that later upgrades into normal mandate ballots?
