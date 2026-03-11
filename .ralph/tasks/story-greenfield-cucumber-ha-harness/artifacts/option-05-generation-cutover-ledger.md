# Option 5: Generation-Cutover Ledger

This is a design artifact only. It does not change production code, tests, configuration, documentation, or runtime behavior in this run. It does not attempt to make `make check`, `make test`, `make test-long`, or `make lint` green. Green repository gates are explicitly outside the scope of this task. The purpose of this document is to describe one complete refactor option in enough detail that a later implementer can execute it without chat history, prior task files, or anything under `docs/`.

## Why this option exists

This option exists because the current HA design still tends to answer the question "what should this node do next?" before it answers the more structural question "what cluster generation is currently authoritative, and what contract does this node hold within that generation?" The differentiator of Option 5 is that every startup decision, steady-state decision, failover decision, rejoin decision, rewind decision, and bootstrap decision is made relative to an explicit generation ledger.

In this design, the HA loop does not primarily think in terms of "role plus special-case recovery." It thinks in terms of "authoritative cluster generation plus per-node execution contract." A generation is the current cluster authority epoch: it names the elected leader candidate, its lease state, the expected timeline lineage, the committed voter set, and the convergence obligations of other members. A contract is this node's assignment inside that generation: serve, follow, fence, bootstrap, converge, or wait for more evidence.

Option 1 centered a unified reconciliation kernel. Option 2 centered the split between cluster intent and local execution. Option 3 centered lease authority as the dominant abstraction. Option 4 centered a unified non-primary recovery funnel. Option 5 is materially different from all four because it treats topology change itself as a first-class typed object. Instead of letting failover, restart, rejoin, and bootstrap infer their behavior from local role and a handful of booleans, this option records an explicit cluster-generation cutover and then derives node behavior from the node's contract within that cutover.

## Current run diagnostic evidence

This design uses the observed repo state on March 11, 2026 as evidence only.

- `make test` passed in the repo root.
- `make test-long` failed in HA-oriented scenarios, which is the exact domain this redesign studies.
- The failing themes observed earlier in this task remain relevant here:
  - quorum-loss behavior is currently too closely tied to the `fail_safe` path
  - majority-side failover and re-election do not become crisp enough when one node is partitioned
  - rejoin behavior for an old primary remains too ambiguous during timeline divergence
  - some service-restoration scenarios still let a node answer as writable primary when it should instead be fenced, waiting, or following

Those observations are diagnostic inputs only. They are not bugs to fix in this run.

## Current design problems

The current architecture has several structural mismatches with the user's intended direction.

### Startup logic is still split away from the main HA model

`src/runtime/node.rs` still contains dedicated startup planning and execution entrypoints such as `plan_startup(...)`, `plan_startup_with_probe(...)`, `execute_startup(...)`, and `build_startup_actions(...)`. That means the system makes some of its highest-impact HA decisions before the long-running reconcile loop gets a chance to apply the same model. This split creates two problems:

- the startup path can drift semantically from steady-state reconciliation
- a later implementer has no single typed source of truth for node lifecycle

### Sender-side dedup remains in the HA worker

`src/ha/worker.rs` follows a mostly good functional chain, but it still contains dispatch suppression logic such as `should_skip_redundant_process_dispatch(...)` and `decision_is_already_active(...)`. Those checks mean the sender is deciding whether a downstream consumer "really needs" an action. The user explicitly wants that concern removed from sender-side HA logic and moved to effect consumers or receivers.

### HA responsibility is still spread across too many boundaries

The current runtime/startup layer, HA decider layer, process dispatch layer, and lower/effect layer each own part of the lifecycle truth. The result is that "startup," "rejoin," "keep leading," and "converge replica" are not visibly different contracts inside one model. They are instead scattered across separate modules that answer overlapping questions.

### The degraded-quorum boundary is too blunt

`src/ha/decide.rs` currently routes any non-`DcsTrust::FullQuorum` state toward `HaPhase::FailSafe`. That is too blunt for three-node majority cases. A 2-of-3 majority with a healthy candidate leader should not be modeled the same way as total loss of authoritative cluster state. The user explicitly called out this boundary as wrong.

### Startup and rejoin intent still pass through ambiguous dispatch bridging

`src/ha/process_dispatch.rs` is still an authority bridge for local process start intent and recovery-source validation. That means startup, follow, rewind, and basebackup behavior can be derived from previous HA decisions instead of from one direct typed contract built from current observations.

### Member publication still risks collapsing partial truth into silence

`src/dcs/worker.rs` publishes member state from local pginfo snapshots, while `src/pginfo/state.rs` already has partial-truth modeling such as `Unknown`, `Unreachable`, and readiness uncertainty. The user wants those partial truths to stay visible in DCS even when SQL reads fail or are incomplete. The current boundaries make it too easy for local uncertainty to become missing publication instead of explicit degraded publication.

## Design summary

Option 5 introduces an explicit generation ledger into the pure HA decision path. Every tick begins by deriving a `GenerationView` from the newest observations. The pure decider then answers two linked questions:

1. Which cluster generation is authoritative, or at least the strongest currently defensible candidate?
2. What execution contract does this node hold inside that generation?

The decider returns a typed `GenerationDecision` that contains:

- the selected generation interpretation
- the local execution contract
- the publication contract for DCS
- the effect plan identifier that receivers use for idempotence

The key architectural shift is that failover is not "node A stops being primary and node B becomes primary." It is "generation N is no longer authoritative enough to continue; generation N+1 is proposed or confirmed; every node receives a new contract in relation to that cutover." That makes startup, rejoin, old-primary repair, and bootstrap variations of the same question: "what contract do I hold in the latest authoritative generation?"

## Proposed control flow

The full lifecycle uses one repeated chain from the first startup tick onward.

```text
pginfo worker ----\
process observer --+--> newest observations --> GenerationView builder --> pure decide
dcs worker -------/                                                |
startup probe ----/                                                v
                                                        GenerationDecision
                                                                |
                           +------------------------------------+----------------------------------+
                           |                                    |                                  |
                           v                                    v                                  v
                    DCS publication plan                process effect plan                 fencing/safety plan
                           |                                    |                                  |
                           v                                    v                                  v
                   DCS worker owns IO                  receivers own idempotence           effect consumers own IO
```

The `GenerationView` builder is still pure. It does not read etcd or Postgres directly. It only consumes the newest observations already gathered by workers. Startup is unified because the first startup probe simply contributes observations into the same builder rather than invoking a special startup-only planner.

## Core concepts

### Cluster generation

A cluster generation is the typed unit of cluster authority. It answers:

- which member is authorized to lead, if any
- which voter set currently defines majority
- which lease epoch and lease health are attached to that authority
- which data lineage and timeline are expected for followers
- which convergence obligations each non-leader must complete

Proposed future types:

```text
ClusterGenerationId
GenerationLedger
GenerationAuthority
GenerationMembership
GenerationTimeline
GenerationLease
GenerationCutoverReason
```

The ledger is not an imperative event log inside the HA worker. It is a pure interpreted structure derived from observed DCS state plus local observations. If the implementation later chooses to materialize more of it in DCS, that still happens through DCS-owned keys and DCS-owned IO. The decider itself only reasons over typed observations.

### Node contract

A node contract is the authoritative answer to "what must this node do now in the current generation?" Proposed future contracts:

```text
NodeContract
  - ServePrimary
  - HoldPrimaryButFenceWrites
  - FollowLeader
  - ConvergeReplica
  - BootstrapCluster
  - BootstrapReplicaFromLeader
  - AwaitGenerationEvidence
  - FenceAndAdvertise
  - StandDownAndRepair
```

Every contract is explicit about:

- whether the node may accept writes
- whether the node may keep Postgres running
- whether timeline alignment is expected
- whether recovery work is required before it can join the generation
- what publication truth must be emitted to DCS

### Effect idempotence token

Sender-side dedup is removed by attaching a stable idempotence token to each effect plan:

```text
EffectPlanToken {
  generation_id,
  contract_kind,
  contract_revision,
  convergence_stage,
}
```

The HA worker always emits the token. Receivers use it to determine whether a start, stop, promote, rewind, basebackup, or fencing action is already in the desired state. This is the same overall functional shape the user wants, but with idempotence moved to the consumer side.

## Detailed state machine

This option uses one lifecycle machine that is parameterized by generation context rather than by ad hoc role flags.

### Top-level phases

```text
LifecyclePhase
  - DiscoverObservations
  - SelectGeneration
  - ExecuteContract
  - VerifyContract
  - PublishTruth
```

These top-level phases happen conceptually on every tick, but the persistent node lifecycle state is carried in the contract and convergence substate.

### Persistent node execution substates

```text
ExecutionState
  - AwaitingEvidence
  - PrimaryServing
  - PrimaryFenced
  - ReplicaFollowing
  - ReplicaConverging(ConvergenceStage)
  - Bootstrapping(BootstrapStage)
  - RepairBlocked(RepairBlockReason)
```

### Convergence stages

```text
ConvergenceStage
  - ValidateLeader
  - ValidateLocalPgData
  - FollowAsIs
  - FastForwardIfAllowed
  - RewindFromLeader
  - BasebackupFromLeader
  - StartReplica
  - VerifyStreaming
```

The purpose of explicit convergence stages is to prevent the current architecture from encoding replica repair as a set of loosely connected booleans spread across startup and process dispatch code.

### Bootstrap stages

```text
BootstrapStage
  - AwaitInitLock
  - InspectExistingPgData
  - SelectBootstrapMode
  - InitializePrimaryData
  - PublishClusterBirth
  - VerifyLeaderLease
```

This keeps bootstrapping explicit and allows the design to answer whether existing local `pgdata` can still be reused when the node wins the init lock.

## Generation selection model

The generation selector is pure. It consumes observations and returns one of these high-level judgments:

```text
GenerationSelection
  - ConfirmedActiveGeneration(GenerationLedger)
  - MajorityCanCutover(NewGenerationProposal)
  - EvidenceInsufficientButExistingGenerationStillDefensible(GenerationLedger)
  - NoAuthoritativeGeneration
```

This is where the degraded-quorum redesign becomes concrete.

### Quorum interpretation

This option stops treating "not full quorum" as one category. Instead it distinguishes:

- `FullQuorumHealthy`: all expected voter observations are sufficiently fresh
- `MajorityHealthy`: a majority of the authoritative voter set is fresh and capable of continuing or electing
- `MajorityHealthyButVisibilityPartial`: a majority can still decide, but some member data is partial or stale
- `NoMajority`: there is no authoritative basis to elect or continue

Only `NoMajority` forces a full authority stand-down. The others still allow a generation to continue or a new generation to be cut over.

### Why 2-of-3 keeps working

In a three-node cluster where one node is partitioned away but two nodes can still see each other, the selector returns either:

- `ConfirmedActiveGeneration` if the current leader remains lease-valid and observed by the majority, or
- `MajorityCanCutover` if the current leader has lost authority and a new leader can be elected by the majority

That means the primary-side majority does not blindly enter the current `FailSafe` bucket simply because full visibility was lost. Fencing remains required on the isolated side, but the majority side may continue or re-elect.

## Lease model

Option 5 keeps lease authority central, but it binds lease semantics directly to generation cutovers.

### Lease rules

- every active generation has exactly one leader lease authority record
- a lease is interpreted relative to a generation id and leader member id
- a node may only serve writes if its local contract is `ServePrimary` for the current active generation and the lease is locally considered alive
- if lease evidence becomes ambiguous but majority evidence still supports a new leader, the selector prefers a generation cutover over indefinite degraded continuation
- if neither continued authority nor a safe cutover can be established, the node contract becomes fencing or waiting

### Killed primary behavior

A killed primary loses practical authority because it stops renewing the generation lease. The majority side can observe lease expiry, cut over to a new generation, and issue follower/repair contracts to the old primary for when it returns. When the old primary comes back, startup does not ask "was I primary before?" It asks "what contract do I hold in the latest authoritative generation?" That naturally leads to rewind, basebackup, or follow behavior instead of accidental old-primary resurrection.

## Startup reasoning

Startup is not a separate planner. It is the first opportunity to assemble observations and enter generation selection.

### Startup observation set

The initial tick should gather or consume:

- latest DCS cluster membership view
- latest observed leader lease view
- latest member publications, including partial truths
- local process activity snapshot
- local pgdata metadata and timeline facts
- local readiness and SQL reachability state

### Startup outcomes

The same decision machinery then chooses one of these broad outcomes:

- join an already active generation as leader because this node is confirmed authoritative
- join an already active generation as follower because another leader is authoritative
- enter convergence because local data is not aligned to the active generation
- bootstrap a new generation because no cluster exists and bootstrap preconditions are satisfied
- wait or fence because evidence is insufficient

No special startup-only imperative branch is required. The difference between startup and steady-state is only the freshness and completeness of the first observation set.

### Existing `pgdata` handling

When the node wins the init lock, the decider does not assume that empty-cluster bootstrap means "always wipe and initdb." Instead the bootstrap stage explicitly checks:

- is local `pgdata` empty?
- is local `pgdata` already on a valid lineage for the to-be-born generation?
- is the existing data a recoverable remainder from a previously interrupted bootstrap?
- would reusing it violate cluster-birth safety?

This is the main benefit of making bootstrap a contract with substates rather than a one-shot action.

## Replica convergence model

Option 5 treats all non-primary data repair as contract fulfillment inside a generation.

### Convergence contract rules

If a valid leader exists, a non-leader node gets one of two broad contracts:

- `FollowLeader` if its data and process state already satisfy the generation
- `ConvergeReplica` if it must complete ordered repair before following

The ordered repair sequence is always:

1. validate leader health and authority
2. validate local data lineage and timeline
3. keep following as-is if already aligned
4. tolerate minor lag if allowed by policy
5. rewind if timeline divergence is repairable
6. fall back to basebackup if rewind is impossible or fails
7. start replica and verify streaming

That sequence is explicit in the contract stage rather than implicitly reconstructed from scattered helper decisions.

### Old primary rejoin

An old primary is not special-cased by role memory. It is just a node whose local data lineage may conflict with the active generation. The convergence contract drives it through validation, rewind, or basebackup in exactly the same ordered pipeline as any other divergent node. That gives the implementation one repair ladder instead of separate "old primary" and "bad replica" code paths.

## Partial-truth member publication

Only the DCS layer still reads and writes etcd keys. This option does not change that boundary. What changes is the publication contract the HA loop hands to DCS.

### Publication contract

The pure decider emits a `PublicationTruth` value that explicitly separates:

- process liveness truth
- SQL reachability truth
- readiness truth
- generation contract truth
- convergence truth
- fencing truth

That allows DCS publication to represent states such as:

- pgtuskmaster is alive, SQL is unreachable, local data lineage unknown, node is fenced from serving
- pgtuskmaster is alive, SQL is healthy, node is converging via rewind, not yet eligible to vote as healthy follower
- pgtuskmaster is alive, process stopped by contract, waiting for majority evidence

This directly addresses the user requirement that "pginfo failed but pgtuskmaster is up" must still produce publishable truth instead of absence.

## Deduplication boundary

This option removes sender-side action suppression from `src/ha/worker.rs`.

### Current problem

The HA sender currently tries to decide whether an effect is redundant before handing it off. That means the sender implicitly owns both the desired state and the receiver's current action memory.

### Proposed replacement

The sender always emits the full typed plan:

- target contract
- desired effect set
- `EffectPlanToken`

Receivers own idempotence by comparing the incoming token and desired payload with their last applied token and actual observed local state. This is safer because:

- receivers know whether an action is already completed
- receivers know whether an in-flight action is still valid
- the HA decider no longer needs intimate knowledge of execution-side retries
- state drift between sender memory and actual process state becomes less dangerous

## Future code areas affected

A later implementation of this option would need to touch, at minimum:

- `src/runtime/node.rs`
  - remove or radically shrink `plan_startup(...)`
  - remove or radically shrink `plan_startup_with_probe(...)`
  - remove or radically shrink `execute_startup(...)`
  - make startup observation gathering feed the same HA reconcile entrypoint as steady-state
- `src/ha/worker.rs`
  - replace current decision/publish/apply framing with `GenerationView -> GenerationDecision -> lower`
  - delete sender-side dedup checks such as `should_skip_redundant_process_dispatch(...)`
  - delete sender memory assumptions such as `decision_is_already_active(...)`
- `src/ha/decide.rs`
  - replace the non-full-quorum shortcut with multi-tier majority interpretation
  - add generation selection logic and node contract generation
- `src/ha/decision.rs`
  - add generation ledger types, node contracts, convergence stages, bootstrap stages, publication truth, and effect plan tokens
- `src/ha/lower.rs`
  - lower contracts and contract stages into effect plans without re-deciding cluster policy
- `src/ha/process_dispatch.rs`
  - stop deriving startup/follow intent from previous HA decisions
  - consume explicit contract stages instead
- `src/dcs/worker.rs`
  - accept richer publication truth contracts
  - possibly publish generation metadata more explicitly
- `src/dcs/state.rs`
  - teach trust evaluation about majority-capable operation versus no-majority shutdown
  - expose generation-relevant membership and freshness structures
- `src/pginfo/state.rs`
  - preserve and surface partial truth fields that publication will serialize
- `tests/ha.rs`
  - revise expectations to align with majority-capable degraded operation where appropriate
- `tests/ha/features/`
  - update feature fixtures and assertions to reflect generation cutover semantics and clearer rejoin contracts

## Meaningful changes required for this option

The later implementation would need the following concrete changes.

### New types

- `GenerationView`
- `GenerationLedger`
- `GenerationDecision`
- `GenerationSelection`
- `NodeContract`
- `ExecutionState`
- `ConvergenceStage`
- `BootstrapStage`
- `PublicationTruth`
- `EffectPlanToken`
- `GenerationCutoverReason`

### Deleted or heavily reduced legacy paths

- dedicated startup-planning branches in `src/runtime/node.rs`
- sender-owned dispatch dedup in `src/ha/worker.rs`
- policy inference from previous process-dispatch output in `src/ha/process_dispatch.rs`
- direct mapping from "not full quorum" to "enter fail safe"

### Responsibility moves

- startup lifecycle ownership moves from runtime startup helpers into the HA contract model
- convergence sequencing moves from scattered helpers into typed convergence stages
- idempotence ownership moves from HA sender logic to effect consumers
- publication semantics move from "best-effort member record assembly" to explicit publication contracts

### Transition changes

- failover becomes generation cutover rather than role flip
- old-primary return becomes contract validation plus convergence rather than startup special case
- bootstrap becomes staged contract fulfillment rather than singular branch
- degraded-majority operation continues when majority evidence is sufficient

## Migration sketch

This option requires an aggressive cleanup path because the repo is greenfield and should not preserve stale architecture.

### Step 1

Define the new typed model in `src/ha/decision.rs` without wiring behavior changes yet:

- generation selection enums
- node contracts
- convergence and bootstrap stages
- publication truth
- effect plan token

### Step 2

Refactor `src/ha/decide.rs` to return `GenerationDecision` from current observation inputs, even if the lower layer still temporarily translates back into existing effect types.

### Step 3

Delete sender-side dedup from `src/ha/worker.rs` and move idempotence checks into the relevant receivers or effect executors.

### Step 4

Delete the separate startup planning path from `src/runtime/node.rs` and replace it with startup observation collection followed by the same HA reconcile entrypoint used during steady-state.

### Step 5

Replace `src/ha/process_dispatch.rs` role inference with direct consumption of node contracts and convergence stages.

### Step 6

Expand DCS publication to include explicit partial-truth and contract-truth fields, then delete any stale bridging logic that reconstructs those meanings indirectly.

### Step 7

Update HA feature tests to assert generation cutover behavior, majority-capable degraded operation, and explicit old-primary convergence semantics.

The critical discipline in this migration is deletion. Once a new typed contract path exists, the old startup planner, old sender dedup, and old ambiguous dispatch bridge should be removed rather than kept as fallback.

## Non-goals

- This option does not propose adding direct Postgres or etcd IO to the pure HA decider.
- This option does not propose preserving backwards-compatibility with the existing startup planner shape.
- This option does not propose letting minority-isolated nodes continue writable service.
- This option does not propose hiding uncertainty. Partial truth should stay visible.
- This option does not implement any of these changes in this task.

## Tradeoffs

### Strengths

- topology changes become explicit and auditable in the type system
- startup and steady-state use one model
- old-primary rejoin semantics become cleaner because they are just generation-contract fulfillment
- degraded-majority behavior becomes more precise than the current all-or-nothing full-quorum shortcut
- receiver-owned idempotence aligns better with actual execution state

### Costs

- this is a large type-system and module-boundary rewrite
- introducing generation vocabulary everywhere may feel heavier than simpler role-based designs
- test fixtures will need careful updating because cutover semantics become more explicit
- if implemented sloppily, there is a risk of creating both generation terms and old role terms at the same time, which would be worse than either model alone

## Logical feature-test verification

This section explains, scenario by scenario, how this option would logically satisfy the HA feature suite. This is design reasoning only, not an implementation claim.

### `ha_dcs_quorum_lost_enters_failsafe`

Under this design, the scenario should be reinterpreted through generation authority. If the cluster truly loses authoritative majority and cannot defend the current generation or create a new one, the node contract becomes `FenceAndAdvertise` or `AwaitGenerationEvidence`, which corresponds to the spirit of fail-safe. The design still honors the need to stop unsafe write service, but it does so because no generation is authoritative, not because full quorum was absent by itself.

### `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`

This becomes cleaner because write permission is part of the node contract. Once the node loses a writable `ServePrimary` contract for the active generation, fencing is mandatory. The contract itself carries "writes forbidden," and the lower layer enforces it. There is no ambiguity about a node continuing to answer writes after authority is lost.

### `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`

The majority side observes that the old leader can no longer defend its generation lease. The selector returns `MajorityCanCutover`, creates generation `N+1`, and assigns `ServePrimary` to the new leader. The isolated old primary, when it can no longer verify its authority, loses its writable contract and becomes fenced. This directly supports majority election without requiring full quorum.

### `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover`

When the old primary heals, startup and rejoin both run through generation selection. The active generation is now the majority-created generation `N+1`, so the old primary receives `ConvergeReplica`. It validates lineage, rewinds if possible, falls back to basebackup if necessary, then starts as follower. There is no separate old-primary resurrection path.

### `ha_primary_killed_then_rejoins_as_replica`

A killed primary stops renewing the active generation lease. The majority cuts over to a new generation. When the killed node returns, it sees that its previous generation is obsolete and receives a non-primary contract. That drives it into the convergence sequence and then into follower service.

### `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`

The remaining healthy replica can help re-establish majority visibility. Once majority evidence exists again, the selector can defend the active generation or cut over to a safe one. The restarted node does not need special startup logic; it simply consumes the newest generation and gets either `FollowLeader` or `ConvergeReplica`. Service restoration becomes a contract outcome, not an accident of timing.

### `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`

When two nodes restart, they rebuild the newest observable generation view from DCS records, local pgdata, and lease evidence. If an existing generation is defensible, they resume it; if not, they cut over safely. The final node later rejoins by receiving a contract in the established generation. This avoids split startup logic and reduces the risk that late arrivals make conflicting decisions.

### `ha_rewind_fails_then_basebackup_rejoins_old_primary`

This is directly modeled in the explicit convergence ladder. `ConvergeReplica` moves from `RewindFromLeader` to `BasebackupFromLeader` when rewind is impossible or fails. Because that fallback is a declared stage transition, the architecture does not need separate ad hoc repair logic.

### `ha_replica_stopped_primary_stays_primary`

The primary's `ServePrimary` contract remains valid as long as the current generation and lease remain authoritative. A single replica stopping does not invalidate the generation if majority and lease conditions remain satisfied. The design therefore preserves stable leadership instead of overreacting to one replica failure.

### `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum`

The broken replica only receives a follower or convergence contract. It cannot destabilize cluster authority because generation selection happens from majority evidence, not from the broken node's self-report. The worst case is that the node remains in `StandDownAndRepair` or `ConvergeReplica` longer; it does not rewrite cluster authority.

## Evaluation against the user's required themes

### Unified startup and HA loop

Satisfied. Startup becomes the first observation-gathering tick of the same generation decision model.

### Sender-side dedup removal

Satisfied. Idempotence moves to receivers through `EffectPlanToken`.

### Reduced HA spread and more unified state-machine shape

Satisfied, but with a generation-oriented vocabulary rather than a simple role machine.

### Corrected degraded-quorum boundary

Satisfied. Majority-capable operation is separated from no-majority shutdown.

### Stronger lease semantics

Satisfied. Lease validity is interpreted relative to generation authority and cutover.

### Authoritative startup observation

Satisfied. Startup decisions are made only after generation view assembly from newest observations.

### Partial-truth member publication

Satisfied. Publication truth explicitly encodes degraded but meaningful member state.

### Bootstrap rethink

Satisfied. Bootstrap becomes a staged contract with explicit `pgdata` reuse checks.

### Simplified replica convergence

Satisfied. All follower repair becomes one declared convergence sequence inside the generation contract.

## Implementation risks to watch later

- The implementation must avoid persisting stale role-based shortcuts next to the new generation model.
- The lower layer must not re-decide cluster policy from effect plans.
- DCS schema changes must remain DCS-owned and should not leak etcd concerns into the HA decider.
- The meaning of "generation" must stay crisp; if it becomes a loose synonym for "term" or "epoch" without precise invariants, the design loses value.

## Q1 Should the generation ledger be purely interpreted or partly published?

Context: this design is strongest when the HA decider can reason over one explicit generation model. One implementation path is to keep the ledger purely interpreted from existing DCS records. Another is to publish richer generation metadata into DCS so every node can read the same cutover framing.

Problem or decision point: a purely interpreted ledger reduces DCS schema churn, but it may leave too much room for nodes to interpret partial evidence differently. A partly published ledger improves convergence around one cluster story, but it introduces more DCS schema and migration work.

Restating the question in different way: should a future implementation treat the generation ledger as only an internal type-level lens over existing records, or should it make generation cutover itself an explicit DCS-published object?

## Q2 How much contract detail should DCS publication expose?

Context: this option proposes publishing not only member liveness and SQL truth, but also contract truth such as converging, fenced, waiting, or serving. That improves observability and operator clarity.

Problem or decision point: exposing too much contract detail in DCS may blur the boundary between cluster coordination data and local execution telemetry. Exposing too little may leave operators and other nodes unable to distinguish "healthy follower" from "alive but fenced and repairing."

Restating the question in different way: what is the minimal publication contract that preserves partial truth and useful coordination without turning DCS member records into an oversized execution log?

## Q3 Should generation cutover require an explicit handoff state?

Context: a cutover may be modeled as an instantaneous selection from generation `N` to generation `N+1`, or it may include an intermediate handoff state where the old generation is formally draining while the new one is being activated.

Problem or decision point: an explicit handoff state may help explain fencing and lease transitions, but it may also add complexity and delay when the safer behavior is simply "old contract invalid, new contract active."

Restating the question in different way: is the model clearer and safer if cutover is atomic from the decider's perspective, or should there be a first-class transitional handoff generation state?

## Q4 How should bootstrap reuse of existing `pgdata` be constrained?

Context: this design wants bootstrap to consider whether existing local data can be safely reused rather than assuming wipe-and-init or assuming reuse is fine.

Problem or decision point: permissive reuse may save time and recover from interrupted bootstrap, but it also risks silently accepting data that does not belong to the intended birth of the cluster. Overly strict rejection may force unnecessary reinitialization.

Restating the question in different way: what evidence must be present before bootstrap is allowed to reuse existing local `pgdata` under a newly won init lock?

## Q5 Should contract tokens be persisted across restarts?

Context: receivers use `EffectPlanToken` for idempotence. If the process restarts, the system can either rebuild idempotence from fresh observation and actual process state, or it can persist the last applied token to accelerate recovery.

Problem or decision point: persistence may improve clarity for in-flight actions, but it also creates another state store that can become stale or misleading. Pure rebuild from observation is conceptually cleaner, but may make some long-running repair steps harder to resume.

Restating the question in different way: should effect-plan idempotence be restart-stable through persisted tokens, or should the implementation rely only on real observed state plus newly emitted tokens after restart?
