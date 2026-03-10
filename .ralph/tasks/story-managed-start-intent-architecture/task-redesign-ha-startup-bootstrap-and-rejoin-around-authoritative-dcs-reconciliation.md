## Task: Redesign HA Startup Bootstrap And Rejoin Around Authoritative DCS Reconciliation <status>retry-tests</status> <passes>false</passes> <priority>ultra high</priority>


### Note from you PO:
This task has been partially done outside the ralph loop, your first action must be to check which acceptance checkboxes can be ticked off via verification inside the code, and which are still 'TODO'

<description>

**Goal:** Replace the current split startup/rejoin/follow-leader architecture with one authoritative reconciliation model that derives node behavior from DCS authority plus local physical facts, rather than from mixed local heuristics and phase-specific patches. The higher-order goal is to guarantee that ephemeral node restarts, cold restarts, preserved-PGDATA rejoins, and leader-loss reactions all converge through the same control rules and therefore produce the same safe behavior.

The redesign must explicitly eliminate the current class of confusing and unsafe behavior where startup can infer local primary intent without authoritative DCS confirmation, while HA steady-state follow behavior performs a different partial sequence than cold startup. The new model must make bootstrap, rejoin, election participation, rewind, basebackup, and replica retargeting parts of one coherent state machine.

This task is not a bug patch. It is an architecture replacement. The finished code must not leave the old overlapping startup/HA/follow logic in place behind compatibility shims or dead branches. Remove the old state overlap instead of layering new logic on top of it.

The old design being replaced here must **go away completely**. In particular, the implementation must delete, not preserve, all startup or HA behavior that:
- infers writable-primary intent because DCS was unavailable or stale
- infers follower source from any “foreign healthy primary” member record when there is no authoritative leader lease
- preserves separate startup-only and steady-state authority rules
- treats multiple visible primaries as a normal source-selection input instead of a bug or fencing condition

**Target architecture to implement:**
- Bootstrap is a **cluster-level** concept, not a node-level `PGDATA` concept.
- Rejoin is an **initialized-cluster** concept, regardless of whether the local node was fully stopped, runtime-restarted, or all nodes were offline.
- A node never becomes writable primary purely because DCS was unreachable and local disk shape looked “primary enough”.
- In both cold startup and steady-state HA, if DCS is unavailable or there is no `FreshQuorum`, an initialized node must remain non-writable in `Quiescent` or `Fence` / fail-safe style behavior until authoritative DCS visibility returns. There must be no startup-only escape hatch that becomes primary before that point.
- Replica convergence is one logical reconcile flow with ordered substeps:
- `DirectFollow`
- `RewindThenFollow`
- `BasebackupThenFollow`
- These are subplans of one replica-reconcile decision, not independent top-level HA state machines that drift apart across ticks.
- Replica source authority comes from the authoritative leader lease only. If there is no authoritative leader, nodes do not “just follow some healthy primary-looking member”; they wait for leader election. Any visible non-leader primary is a bug signal, not a valid fallback source.

**Required DCS model after redesign:**
- `cluster_initialized`: durable cluster fact; not leased; distinguishes bootstrap vs rejoin even after all leases expire
- durable cluster record: must exist alongside `cluster_initialized` and must include at minimum the authoritative cluster `system_identifier` once bootstrap succeeds; the task must define the exact schema and path
- `bootstrap_lock`: leased; held only while one node is actively responsible for first-cluster bootstrap
- `leader_lock`: leased; held only by the current elected leader
- member records remain the live cluster observation surface
- Existing plain `init_lock` semantics must be removed or replaced, not left in parallel with the new model

**Required trust naming and semantics after redesign:**
- Rename `DcsTrust::FullQuorum` to a name that reflects the real semantics, preferably `FreshQuorum`
- Preserve the actual meaning: healthy DCS plus fresh quorum, not “all nodes present”
- Keep separate states for:
- DCS unavailable / not trusted
- DCS reachable but no fresh quorum
- DCS reachable with fresh quorum
- Startup and HA must obey the same trust rule. `DcsUnavailable` or `NoFreshQuorum` is never enough authority to start or remain writable primary in an initialized cluster.

**Required per-tick reconciliation model after redesign:**
- Replace the current phase/decision split that makes startup, follow, and recovery partially overlap with a smaller set of high-level desired states and nested subplans.
- `ClusterMode` and `DesiredNodeState` are **derived per tick** from authoritative facts. They are not persisted in DCS, not written as extra cluster keys, and not treated as durable state sources.
- DCS remains the source of truth only for durable/leased cluster facts and published member descriptors. `ClusterMode` is a local computed summary over those facts.
- The design target is:
- `DesiredClusterState::Bootstrap(BootstrapPlan)`
- `DesiredClusterState::Primary(PrimaryPlan)`
- `DesiredClusterState::Replica(ReplicaPlan)`
- `DesiredClusterState::Quiescent(QuiescentReason)`
- `DesiredClusterState::Fence(FencePlan)`
- `ReplicaPlan` must contain the ordered follow method:
- `DirectFollow`
- `RewindThenFollow`
- `BasebackupThenFollow`
- `PrimaryPlan` must distinguish:
- retain-primary under owned leader lease
- acquire-leader-and-promote
- acquire-leader-and-start-primary
- `QuiescentReason` must distinguish:
- waiting for bootstrap winner
- waiting for fresh quorum
- waiting for authoritative leader
- waiting for recovery preconditions
- startup blocked because local state is unsafe to interpret without authority

The task must turn that shape into concrete Rust enums close to the following form:

```rust
enum ClusterMode {
    UninitializedNoBootstrapOwner,
    UninitializedBootstrapInProgress { holder: MemberId },
    InitializedLeaderPresent { leader: MemberId },
    InitializedNoLeaderFreshQuorum,
    InitializedNoLeaderNoFreshQuorum,
    DcsUnavailable,
}

enum DesiredNodeState {
    Bootstrap(BootstrapPlan),
    Primary(PrimaryPlan),
    Replica(ReplicaPlan),
    Quiescent(QuiescentReason),
    Fence(FencePlan),
}

enum BootstrapPlan {
    InitDb,
}

enum PrimaryPlan {
    KeepLeader,
    AcquireLeaderThenStartOrPromote,
}

enum ReplicaPlan {
    DirectFollow { leader: MemberId },
    RewindThenFollow { leader: MemberId },
    BasebackupThenFollow { leader: MemberId },
}

enum QuiescentReason {
    WaitingForBootstrapWinner,
    WaitingForAuthoritativeLeader,
    WaitingForFreshQuorum,
    UnsafeUninitializedPgData,
}

enum FencePlan {
    StopAndStayNonWritable,
}
```

The implementation may rename these, but it must preserve this degree of compression:
- few top-level node states
- replica reconciliation represented as one enum with ordered subplans
- no separate overlapping “follow”, “wait to start”, and “start requested” concepts left behind as primary mental models
- `ClusterMode` itself must be a computed enum, not a stored DCS record

**Required bootstrap/rejoin distinction after redesign:**
- “Bootstrap” means `cluster_initialized = false`
- “Rejoin” means `cluster_initialized = true`
- Whether local `PGDATA` exists only decides *how* the node fulfills the chosen cluster-level plan:
- bootstrap path with missing/empty `PGDATA` => initdb/bootstrap
- replica rejoin with compatible `PGDATA` => direct follow
- replica rejoin with divergent but rewindable `PGDATA` => rewind then follow
- replica rejoin with non-rewindable `PGDATA` => basebackup then follow
- If `cluster_initialized = false` but a node has non-empty unexpected `PGDATA`, the redesign must treat this as `Quiescent(UnsafeUninitializedPgData)` plus an explicit hard error surfaced to the operator. It must not silently reinterpret that case as ordinary bootstrap, and it must not delete local data.

**Required local `PGDATA` inspection contract after redesign:**
- Add one authoritative local-physical-facts inspection path for startup/election/rejoin. It must be reusable across startup and HA, not reimplemented ad hoc in multiple files.
- The inspection path must answer at minimum:
- `data_dir_kind`:
- `Missing`
- `Empty`
- `Initialized`
- `InvalidNonEmptyWithoutPgVersion`
- `system_identifier`
- `pg_version`
- `control_file_state`
- `timeline_id`
- `durable_end_lsn`
- whether local control data indicates the node was in recovery / standby state
- whether local signal files indicate standby/recovery mode
- whether managed signal files conflict or local state is physically inconsistent
- whether local state is even eligible to be considered for:
- initdb bootstrap
- direct follower start
- rewind
- basebackup
- The inspection path must use durable physical metadata such as `pg_control` / `pg_controldata`-equivalent facts. It must not rely on heuristics like “newest WAL file name in the directory”.
- The inspection path may use an external binary or a Rust parser, but the task must make one authoritative implementation choice and use it consistently.

**Required cross-node comparison model after redesign:**
- Add a published per-node pre-election descriptor that can be compared across nodes without requiring every cold node to start writable PostgreSQL first.
- Cold nodes must be able to publish this descriptor before leader election completes. The task must define when startup publishes or refreshes it and how its freshness is maintained while PostgreSQL is still stopped.
- This descriptor must include enough data for deterministic ranking:
- `system_identifier`
- `timeline_id`
- `durable_end_lsn`
- `state_class`
- `postgres_runtime_class`
- `updated_at`
- `member_id`
- `state_class` must at minimum distinguish:
- empty/missing `PGDATA`
- initialized inspectable `PGDATA`
- initialized but inconsistent / invalid `PGDATA`
- replica-only local state that requires a recovery source
- promotable local state
- `postgres_runtime_class` must distinguish:
- postgres already running and healthy
- postgres stopped but `PGDATA` inspectable
- postgres unavailable and local state unsafe
- The task must define exactly where this descriptor lives in DCS/member state and update all relevant types, APIs, and tests accordingly.

**Required leader authority semantics after redesign:**
- `leader_lock` is the only authoritative proof of the current leader.
- If `leader_lock` is absent, the cluster is in “no authoritative leader” state even if some member records still claim `role = primary`.
- Nodes must not follow, clone from, or defer to a non-leader “foreign healthy primary” as an authority fallback.
- Multiple visible primaries without a single authoritative `leader_lock` are a safety bug signal to be fenced/quiesced around, not a valid steady-state topology.
- If the local node owns `leader_lock` and then detects that PostgreSQL is unhealthy, unreachable, or no longer a valid primary candidate, it must proactively stop serving and release leadership through the proper owned-lease path when possible.
- If the node is hard-killed or cannot execute release, lease expiry is the cleanup path. Followers still rely on lease-backed leader truth rather than inventing a second “healthy primary fallback” rule.

**Required deterministic winner ordering after redesign:**
- When choosing which node may race for leader in an initialized cluster with no current leader, compare candidates in this exact order:
1. matching expected cluster `system_identifier` beats non-matching or unknown `system_identifier`
2. eligible/promotable state beats ineligible state
3. higher `timeline_id` wins
4. higher `durable_end_lsn` wins
5. `postgres_runtime_class = running healthy` beats `postgres_runtime_class = offline inspectable`
6. lowest lexical `member_id` wins as final deterministic tie-break
- A node may attempt leader acquisition only if its own descriptor is maximal among the fresh-quorum-visible candidate set under the above ordering.
- All other nodes must remain non-writable and wait for the elected leader to appear, then reconcile as replicas.
- The leased `leader_lock` remains the final serialization point. The ranking rules reduce ambiguity and race churn; they do not replace the leader lease.

**Required bootstrap winner ordering after redesign:**
- When `cluster_initialized = false`:
- only nodes with `data_dir_kind = Missing | Empty` may participate in bootstrap race
- any node with non-empty initialized `PGDATA` must surface `UnsafeUninitializedPgData` and must not participate
- bootstrap winner is chosen solely by leased `bootstrap_lock`
- the task must encode this rule explicitly in code and tests rather than leaving “healthier” as an informal idea
- `cluster_initialized` must be written only after bootstrap succeeds. There must be no durable plain `init_lock` or equivalent durable “bootstrap started” marker written before successful bootstrap completion.
- `bootstrap_lock` alone represents bootstrap-in-progress authority. If bootstrap fails before success is recorded, the cluster remains uninitialized and another eligible node may retry only after the lease-backed bootstrap authority is gone.

**Required no-dual-primary election rule after redesign:**
- The redesign must not depend on starting multiple writable primaries to learn who is freshest.
- Cold election must be driven by:
- DCS fresh-quorum visibility
- published pre-election descriptors
- local `PGDATA` inspection
- leader lease acquisition
- If the implementation starts PostgreSQL before final leader acquisition for any inspection or readiness reason, that start must be non-writable/fenced by design and must be documented explicitly. The default implementation should avoid requiring this.

**Required detailed per-tick state machine after redesign:**
- The task must implement the following logical algorithm and keep the code close to it:
1. Gather DCS facts:
- DCS store health
- trust (`NotTrusted`, `NoFreshQuorum`, `FreshQuorum` or equivalent final names)
- durable cluster record
- bootstrap lock record
- leader lock record
- fresh member/pre-election descriptors
2. Gather local facts:
- local postgres runtime state if postgres is already running
- local physical `PGDATA` inspection descriptor
3. Derive cluster mode:
- `UninitializedNoBootstrapOwner`
- `UninitializedBootstrapInProgress`
- `InitializedLeaderPresent`
- `InitializedNoLeaderFreshQuorum`
- `InitializedNoLeaderNoFreshQuorum`
- `DcsUnavailable`
4. Derive local desired output:
- `Bootstrap(BootstrapPlan::InitDb)`
- `Primary(PrimaryPlan::KeepLeader)`
- `Primary(PrimaryPlan::AcquireLeaderThenStartOrPromote)`
- `Replica(ReplicaPlan::DirectFollow)`
- `Replica(ReplicaPlan::RewindThenFollow)`
- `Replica(ReplicaPlan::BasebackupThenFollow)`
- `Quiescent(QuiescentReason::WaitingForBootstrapWinner)`
- `Quiescent(QuiescentReason::WaitingForAuthoritativeLeader)`
- `Quiescent(QuiescentReason::WaitingForFreshQuorum)`
- `Quiescent(QuiescentReason::UnsafeUninitializedPgData)`
- `Fence(FencePlan::StopAndStayNonWritable)`
5. Execute only the next required step for that plan, but keep the plan identity stable across ticks until it is complete or invalidated by new authoritative facts

**Required transition rules after redesign:**
- `UninitializedNoBootstrapOwner`:
- eligible initdb candidate may try `bootstrap_lock`
- non-winners stay `Quiescent(WaitingForBootstrapWinner)` with postgres stopped
- `UninitializedBootstrapInProgress`:
- bootstrap owner executes bootstrap plan
- all other nodes stay non-writable and do not start ordinary postgres
- if bootstrap lock expires before `cluster_initialized` is written, return to `UninitializedNoBootstrapOwner`
- on bootstrap success, the task must define the exact ordered handoff: bootstrap success, durable cluster record write including cluster identity, `cluster_initialized` write, publication of leader-eligible local facts, and leader acquisition/startup sequencing
- `InitializedLeaderPresent`:
- leader owner goes `Primary(KeepLeader)` or `Primary(AcquireLeaderThenStartOrPromote)` depending on local runtime
- all non-leaders go to one `Replica(...)` plan chosen by direct-follow/rewind/basebackup ordering
- `InitializedNoLeaderFreshQuorum`:
- only the maximal ranked candidate may try `leader_lock`
- all others stay quiescent until a leader appears
- once leader appears, non-leaders immediately switch to one `Replica(...)` plan
- in a 3-node cluster, any healthy fresh 2-of-3 quorum must be able to elect exactly one new primary and restore service **before** the third node returns; this requirement is mandatory and must not be weakened in code, tests, or Ralph task rewrites
- `InitializedNoLeaderNoFreshQuorum`:
- nobody may become writable primary
- all nodes stay quiescent or fenced according to local safety state
- `DcsUnavailable`:
- initialized clusters may not infer writable primary from local disk shape alone
- nodes remain quiescent/fenced until DCS authority returns

**Scope:**
- Redesign startup planning in `src/runtime/node.rs`
- Redesign HA decision sequencing in `src/ha/decide.rs`, `src/ha/decision.rs`, and `src/ha/lower.rs`
- Redesign HA process dispatch/reconciliation in `src/ha/process_dispatch.rs` and `src/ha/worker.rs`
- Redesign DCS bootstrap/cluster-authority semantics in `src/dcs/state.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, and `src/dcs/etcd_store.rs`
- Redesign HA/shared state shapes in `src/ha/state.rs`
- Redesign process job interfaces in `src/process/jobs.rs`, `src/process/state.rs`, and `src/process/worker.rs` if needed so one reconcile plan can own a full follower transition without cross-tick ambiguity
- Revisit managed postgres config/recovery contract in `src/postgres_managed.rs` and `src/postgres_managed_conf.rs`
- Revisit source-member / recovery-source selection in `src/ha/source_conn.rs`
- Revisit API/debug/status surfaces in `src/api/mod.rs`, `src/api/controller.rs`, `src/cli/status.rs`, and `src/debug_api/view.rs` so the exposed HA states match the new model exactly
- Audit and rewrite the source-level HA/DCS/runtime tests in:
- `src/runtime/node.rs`
- `src/dcs/state.rs`
- `src/ha/decision.rs`
- `src/ha/decide.rs`
- `src/ha/process_dispatch.rs`
- `src/ha/worker.rs`
- Update or replace HA E2E scenarios in `tests/ha/support/multi_node.rs` and `tests/ha_multi_node_failover.rs` so they validate the new unified behavior rather than patched scenario-specific fallbacks
- Audit and update HA partition tests that overlap the new authority semantics:
- `tests/ha_partition_isolation.rs`
- `tests/ha/support/partition.rs`
- Reconcile or supersede existing Ralph HA test/bug task narratives if the redesign changes their semantics:
- `.ralph/tasks/story-parallel-ha-test-hardening/02-task-add-ha-restart-and-leadership-churn-e2e-coverage.md`
- `.ralph/tasks/story-parallel-ha-test-hardening/03-task-add-clone-and-rewind-failure-ha-e2e-coverage.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/01-task-fix-leader-liveness-and-majority-election-after-hard-node-loss.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/02-task-add-whole-node-kill-and-partial-recovery-ha-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/03-task-add-full-1-to-2-network-partition-quorum-survival-ha-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/04-task-add-primary-storage-stall-and-wal-full-failover-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/05-task-add-broken-returning-node-and-single-good-recovery-ha-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/06-task-add-full-failsafe-recovery-when-quorum-returns-ha-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/07-task-add-old-primary-returns-as-replica-only-after-majority-failover-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/08-task-add-lagging-or-stale-replica-is-never-promoted-over-healthier-candidate-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/09-task-add-node-flapping-with-healthy-majority-does-not-cause-leadership-thrash-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/10-task-add-minority-old-primary-returns-with-stale-view-and-is-forced-to-rejoin-safely-e2e.md`
- `.ralph/tasks/story-ha-quorum-survival-under-real-failures/11-task-add-broken-replica-rejoin-does-not-block-healthy-quorum-availability-e2e.md`
- `.ralph/tasks/bugs/preserved-replica-rejoin-stalls-after-runtime-stop-failover.md`
- `.ralph/tasks/bugs/runtime-restart-replica-can-stall-before-replaying-post-restart-writes.md`
- `.ralph/tasks/bugs/rapid-repeated-failovers-can-drop-intermediate-writes.md`
- `.ralph/tasks/bugs/targeted-switchover-request-can-promote-wrong-node.md`
- Update docs that currently describe startup/follow behavior in:
- `docs/src/reference/ha-decisions.md`
- `docs/src/explanation/ha-decision-engine.md`
- `docs/src/how-to/handle-primary-failure.md`

**Context from research:**
- Current startup first inspects local `PGDATA`, then probes DCS, then selects startup mode in `src/runtime/node.rs`.
- Current `select_resume_start_intent(...)` still falls back to local `Primary` intent when DCS is unavailable and local managed replica residue is absent; this is too permissive for a safe HA design.
- Current startup also contains a `foreign healthy primary` fallback shape that treats any visible primary-looking member as enough authority for follow/reclone in some cases. This fallback must be deleted; follower authority comes from the elected leader only.
- Current bootstrap semantics are tied to a plain `init_lock` and local startup action sequencing in `src/runtime/node.rs`, instead of being modeled as a leased cluster-level bootstrap authority flow with a separate durable initialized marker.
- Current cold startup and live HA rejoin are split:
- startup can now choose `CloneReplica { reset_data_dir: true }` when preserved `PGDATA` exists but DCS shows a foreign healthy leader
- live HA `FollowLeader` rewrites config and demotes, but relies on a later `WaitForPostgres(start_requested=true)` path to actually start postgres again
- Current `WaitForPostgres { start_requested: bool }` and `FollowLeader` split one logical reconcile action into multiple partial actions, with redundant-dispatch/latching behavior in `src/ha/worker.rs` that can suppress retries and create timing-sensitive bugs.
- Current HA/public state space is too broad and overlapping:
- `HaPhase` in `src/ha/state.rs`
- `HaDecision` in `src/ha/decision.rs`
- lowered effects in `src/ha/lower.rs`
- process jobs in `src/process/state.rs` / `src/process/jobs.rs`
- These should be collapsed so there are fewer concepts to hold in mind and fewer invalid intermediate combinations.
- Current runtime startup already renders managed config before postgres starts, which is the right property to preserve, but the authority model and sequencing around when to start primary vs replica need redesign.
- Current DCS has a leader lease, but the init/bootstrap lock is still plain presence semantics rather than a leased bootstrap authority flow, and there is no durable “cluster initialized” fact distinct from ephemeral leases.
- Current HA decision engine hard-gates on `DcsTrust::FullQuorum`; redesign must explicitly clarify and preserve the distinction between:
- store unavailable / not trusted
- quorum not present
- quorum present with authoritative leader election allowed
- Current recovery behavior is inconsistent across paths:
- HA recovery can choose `pg_rewind` before basebackup in some live paths
- startup does not run the same reconcile choice and can jump directly to reclone
- Current test coverage and some scenario assertions validate the wrong granularity of behavior, including cases where a restart test can pass while only checking a subset of expected replica convergence nodes.

**Expected outcome:**
- Bootstrap is modeled as a first-class leased DCS authority flow and is cleanly separated from initialized-cluster rejoin by a durable `cluster_initialized` fact.
- The redesign defines an explicit safe policy for `cluster_initialized = false` with existing non-empty local `PGDATA`:
- no silent deletion
- no implicit bootstrap over existing data
- explicit hard error only
- A node never starts writable primary merely because DCS was unavailable, stale, or lacking fresh quorum and local state “looked primary”.
- In an initialized cluster, the absence of `FreshQuorum` always means no writable primary authority. Cold startup and HA steady-state obey that same rule.
- If an authoritative leader exists in DCS, any joining/restarting node derives replica intent from that leader and starts only as replica, unless it first proves it needs rewind or reclone.
- If no authoritative leader exists in DCS, nodes do leader election or wait; they do not follow arbitrary non-leader primary member records.
- Rejoin after restart and steady-state follow use the same authoritative reconciliation rules instead of separate partial sequences.
- Replica following is one reconcile decision with nested ordered subplans, not separate top-level states for “follow”, “wait to start”, “rewind”, and “bootstrap” that obscure one logical flow.
- Recovery ordering is explicit and consistent: determine whether local state is directly followable, then whether rewind is possible, and fall back to basebackup only when required.
- HA startup, bootstrap, leader loss, old-primary return, preserved replica return, and all-nodes-offline restart behavior are specified as one cohesive model with deterministic test coverage.
- The final codebase contains no stale legacy branches that still encode the old startup-mode vs live-follow split in parallel.
- The final test suite and Ralph task ledger tell the same story as the new architecture; no outdated test/task wording may remain that describes the superseded behavior.

</description>


## Detailed implementation plan

The implementation will be done in explicit phases so the architecture is replaced coherently instead of layering new logic on top of the current startup/HA split.

Skeptical review amendment:
- Do not rewrite shipped user docs first. The repo currently has broad user-visible `FullQuorum`, `WaitForPostgres`, and `FollowLeader` wording across docs, CLI, API, and debug surfaces. Updating those pages before the code lands would create a long stale-doc window during the refactor. Freeze the target architecture early in one internal architecture note and code comments, then update shipped docs only after the new behavior is actually passing.
- Do not extend the published member descriptor with placeholder offline-election facts before the local physical inspection path exists. The DCS worker currently builds `MemberRecord` only from `PgInfoState`, so the authoritative offline descriptor must be implemented first and then wired into member publication; otherwise the plan would temporarily invent ranking fields from incomplete facts.

### Phase 1: Freeze the target model in a narrow internal note and characterization tests before the large refactor

1. Add a short internal architecture note that becomes the canonical reference for the redesign during the refactor. Keep it close to code or under docs as an implementation note, but do not rewrite the main user-facing HA pages yet.
2. In that note, define the exact DCS paths and payloads to implement:
   - `/{scope}/cluster/initialized` -> durable record, written only after successful first bootstrap
   - `/{scope}/cluster/identity` -> durable record containing at minimum `system_identifier`, plus the bootstrap winner and bootstrap completion timestamp if useful
   - `/{scope}/bootstrap` -> leased bootstrap authority record replacing the plain `/{scope}/init`
   - `/{scope}/leader` -> leased leader authority record, preserved as the only authoritative leader proof
   - `/{scope}/member/{member_id}` -> live member record, extended to include the pre-election/startup descriptor rather than introducing a second parallel member key family
3. In the same note, lock the precise trust vocabulary:
   - `NotTrusted` means DCS unavailable/unhealthy
   - `NoFreshQuorum` means DCS reachable but insufficient fresh quorum for authority
   - `FreshQuorum` means DCS reachable and authoritative enough for leader/bootstrap decisions
4. Write the exact desired-state vocabulary that the refactor will implement:
   - `ClusterMode`
   - `DesiredNodeState`
   - `BootstrapPlan`
   - `PrimaryPlan`
   - `ReplicaPlan`
   - `QuiescentReason`
   - `FencePlan`
5. Add or update characterization tests before deleting legacy paths so the refactor keeps the few behaviors that must survive the redesign:
   - managed config is still rendered before PostgreSQL is started
   - a healthy 2-of-3 quorum can still restore exactly one primary without waiting for the third node
   - the leader lease remains the only authoritative leader proof
6. Add source comments near the new enums explaining that they are derived each tick from DCS facts plus local physical facts and are never themselves persisted into DCS.

### Phase 2: Introduce the new DCS schema and remove `init_lock`

1. Replace `DcsKey::InitLock` in [src/dcs/keys.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/keys.rs) with explicit keys for:
   - `ClusterInitialized`
   - `ClusterIdentity`
   - `BootstrapLock`
2. Replace `InitLockRecord` in [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) with:
   - `BootstrapLockRecord { holder: MemberId }`
   - `ClusterInitializedRecord`
   - `ClusterIdentityRecord { system_identifier, ... }`
3. Extend `DcsView` so the authoritative cluster facts are present together:
   - `cluster_initialized: Option<ClusterInitializedRecord>`
   - `cluster_identity: Option<ClusterIdentityRecord>`
   - `bootstrap_lock: Option<BootstrapLockRecord>`
   - existing `leader` and `members`
4. Update watch decoding and cache refresh in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) and [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs) so reset/apply/delete semantics handle the new keys and no `init_lock` code remains.
5. Extend the etcd store in [src/dcs/etcd_store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/etcd_store.rs) with leased bootstrap-lock operations, parallel to the existing leader lease support.
6. Add an explicit bootstrap-lease API surface in the store traits so runtime/HA code cannot keep using raw `put_path_if_absent` as an escape hatch.
7. Remove `claim_dcs_init_lock_and_seed_config(...)` from [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) and replace it later with a bootstrap-authority step that:
   - acquires bootstrap lease first
   - performs bootstrap
   - writes cluster identity on success
   - writes cluster initialized on success
   - releases bootstrap lease
8. Add unit tests for DCS key parsing, watch decoding, cache reset, and lease APIs so the schema migration is pinned before the larger HA rewrite.

### Phase 3: Add one authoritative local physical inspection module

1. Introduce a dedicated local-inspection module instead of leaving physical state inference split across `inspect_data_dir`, managed recovery signal inspection, and HA runtime facts.
2. The module will expose one authoritative result struct that includes:
   - `data_dir_kind`: `Missing`, `Empty`, `Initialized`, `InvalidNonEmptyWithoutPgVersion`
   - `system_identifier`
   - `pg_version`
   - `timeline_id`
   - `durable_end_lsn`
   - control-file state
   - whether control data says the cluster was in recovery
   - signal-file mode and signal-file conflicts
   - whether the local state is eligible for bootstrap, direct follow, rewind, or basebackup
3. The authoritative implementation choice will be: parse facts from `pg_controldata` output, with signal-file inspection used only as a supplemental local-managed-state input. The plan deliberately does not use WAL filename heuristics.
4. Put this logic in one reusable module consumed by both startup planning and HA per-tick reconciliation, so there is no second startup-only interpretation path left in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs).
5. Add focused tests for:
   - missing and empty directories
   - non-empty without `PG_VERSION`
   - valid inspectable `PGDATA`
   - conflicting signal files
   - parsing of system identifier, timeline, and durable end LSN
   - eligibility decisions for direct follow vs rewind vs basebackup

### Phase 4: Extend the member record with the startup/election descriptor after offline inspection exists

1. Extend `MemberRecord` in [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) with a nested descriptor published even when PostgreSQL is stopped:
   - `system_identifier`
   - `timeline_id`
   - `durable_end_lsn`
   - `state_class`
   - `postgres_runtime_class`
   - `updated_at`
   - `member_id`
2. Define `state_class` so it distinguishes at minimum:
   - missing/empty `PGDATA`
   - initialized inspectable `PGDATA`
   - initialized but invalid/inconsistent `PGDATA`
   - replica-only state that still requires a source
   - promotable state
3. Define `postgres_runtime_class` so it distinguishes:
   - postgres running healthy
   - postgres stopped but local state inspectable
   - postgres unavailable and local state unsafe
4. Update the DCS worker to publish this descriptor from the new local-inspection module on every tick, including cold startup ticks before PostgreSQL is started. Do not publish partially synthesized offline descriptor values from `PgInfoState` alone.
5. Update trust evaluation only to classify DCS authority; do not fold candidate ranking into trust itself.
6. Add deterministic comparator functions over the descriptor with tests proving the required ordering:
   - matching cluster `system_identifier`
   - promotable over ineligible
   - higher timeline
   - higher durable LSN
   - running healthy over offline inspectable
   - lexical `member_id` tie-break

### Phase 5: Replace the HA state model with derived cluster/node plans

1. Replace the broad phase model in [src/ha/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/state.rs) with a smaller derived state surface centered on:
   - `ClusterMode`
   - `DesiredNodeState`
2. Collapse the current decision vocabulary in [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs) so it mirrors the target architecture instead of encoding overlapping transport steps like `WaitForPostgres` and `FollowLeader` as separate mental models.
3. Keep the decision/effect separation only if it still serves dispatch clarity. If it becomes redundant under the new model, remove the extra layer rather than preserving it for familiarity.
4. Rename `DcsTrust::FullQuorum` to `FreshQuorum` through product code, tests, API responses, CLI output, debug API, docs, and generated docs.
5. Ensure the top-level decision function in [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) computes:
   - one cluster mode from DCS health, quorum, bootstrap lock, initialized fact, and leader lease
   - one local node state from cluster mode plus local physical inspection plus local PostgreSQL runtime facts
6. Preserve the strict authority rule:
   - initialized cluster + `NotTrusted` => not writable primary
   - initialized cluster + `NoFreshQuorum` => not writable primary
   - no leader lease => no authoritative leader, even if member records still claim primary
7. Make the cold-restart/all-leases-expired path explicit:
   - only members with matching cluster identity and promotable descriptor may join election
   - non-promotable members stay quiescent and wait
   - candidate ranking uses the deterministic comparator from Phase 4
8. Make the unsafe-uninitialized-data path explicit:
   - `cluster_initialized = false` + non-empty unexpected `PGDATA` => `Quiescent(UnsafeUninitializedPgData)` plus surfaced hard error

### Phase 6: Replace startup planning in `src/runtime/node.rs` with the same reconcile model

1. Delete `StartupMode::{InitializePrimary, CloneReplica, ResumeExisting}` and the separate startup-only mode selection logic in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs).
2. Replace `inspect_data_dir(...)` and startup DCS probing with a single startup reconcile input builder that uses:
   - the new local physical inspection module
   - a startup DCS snapshot using the new schema
   - the same cluster-mode and desired-node-state derivation used by HA
3. Preserve the useful current property that managed config is rendered before PostgreSQL starts, but make the intent entirely derived from the authoritative plan.
4. Explicitly remove the following startup fallbacks:
   - DCS probe failure + preserved local primary-ish state => start primary
   - `foreign healthy primary` as a fallback follow source
   - `init_lock` absence as sufficient bootstrap proof
5. Implement startup execution against the new desired states:
   - `Bootstrap(InitDb)` => acquire bootstrap lease, run bootstrap, write durable cluster identity, write initialized fact, start PostgreSQL as primary
   - `Primary(KeepLeader)` => retain leadership only if the local node already owns the authoritative leader lease
   - `Primary(AcquireLeaderThenStartOrPromote)` => attempt leader lease only if the local descriptor wins the initialized-cluster election
   - `Replica(DirectFollow/RewindThenFollow/BasebackupThenFollow)` => materialize replica config and run the ordered subplan
   - `Quiescent(...)` => do not start writable PostgreSQL
   - `Fence(...)` => stop and remain non-writable

### Phase 7: Refactor process dispatch so one replica plan owns the whole transition

1. Rework [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs), [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs), [src/process/jobs.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/jobs.rs), [src/process/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/state.rs), and [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) so a follower reconcile plan is not split across unrelated ticks.
2. Replace the current `WaitForPostgres(start_requested)` plus `FollowLeader` dance with a single follower convergence plan that can own:
   - demote if needed
   - stop PostgreSQL if required
   - rewrite managed config
   - run rewind if required
   - run basebackup if required
   - restart PostgreSQL as replica
3. Keep plan progress in explicit process state so retries are idempotent and do not depend on latching hacks in the HA worker.
4. Make basebackup, rewind, and direct follow subplans mutually exclusive ordered outputs of one reconcile decision, not top-level competing states.
5. Ensure leader self-failure is handled through one path:
   - fence immediately
   - release owned leader lease when possible
   - otherwise rely on lease expiry

### Phase 8: Remove foreign-primary fallback and retarget source selection

1. Rewrite source selection in [src/ha/source_conn.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/source_conn.rs) and related helpers so replication source authority comes only from the authoritative leader lease holder.
2. Treat visible non-leader primaries as bug/fence signals rather than normal source candidates.
3. Update any ranking or follow helper in [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs) and [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) that currently uses arbitrary healthy primary records.

### Phase 9: Update API, debug, CLI, and documentation surfaces

1. Update [src/api/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/mod.rs), [src/api/controller.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/controller.rs), [src/cli/status.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs), and [src/debug_api/view.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/debug_api/view.rs) so exposed state names exactly match the new model.
2. Remove stale references to `FullQuorum`, `init_lock`, `WaitForPostgres`, `FollowLeader`, and startup-mode terminology from user-visible output.
3. Update docs with the `k2-docs-loop` skill after code behavior is finalized:
   - [docs/src/reference/ha-decisions.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/ha-decisions.md)
   - [docs/src/explanation/ha-decision-engine.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/explanation/ha-decision-engine.md)
   - [docs/src/how-to/handle-primary-failure.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/handle-primary-failure.md)
4. Remove or regenerate stale built docs artifacts only after the source docs are correct.

### Phase 10: Rewrite tests around the new authority model

1. Replace source-level tests that currently pin legacy phases and decisions in:
   - [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)
   - [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs)
   - [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs)
   - [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs)
   - [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs)
   - [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs)
2. Rewrite the multi-node HA scenarios in:
   - [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs)
   - [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs)
3. Add or update scenarios for:
   - concurrent first bootstrap with one leased bootstrap winner
   - bootstrap lease expiry before initialization completes
   - all-nodes-offline restart with expired leases and preserved `PGDATA`
   - 2-of-3 quorum restoring exactly one primary while the third node is still offline
   - healthy follower restart while a leader lease exists
   - old primary restart after failover for both rewindable and reclone-required cases
   - preserved replica restart after failover
   - unsafe uninitialized cluster with unexpected non-empty `PGDATA`
   - leader lease loss with fresh quorum
   - leader loss without fresh quorum
4. Tighten assertions so restart/rejoin tests validate the full final topology rather than only a subset of nodes.
5. Update partition tests in:
   - [tests/ha_partition_isolation.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_partition_isolation.rs)
   - [tests/ha/support/partition.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/partition.rs)
6. Reconcile or supersede the Ralph HA tasks and bug files listed in the Scope section once the final semantics are stable, so the task ledger stops describing obsolete behavior.

### Phase 11: Final verification and completion

1. Run the full required validation set with the implementation complete:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
2. Fix every failing test rather than downgrading coverage or skipping suites.
3. Confirm docs match the shipped behavior, including new trust naming and startup/bootstrap/rejoin semantics.
4. Only after all of the above passes:
   - set `<passes>true</passes>`
   - run `/bin/bash .ralph/task_switch.sh`
   - commit all changes, including `.ralph` updates
   - `git push`


<acceptance_criteria>
- [ ] Write and commit a concrete architecture note in docs and code comments that defines:
- [ ] durable DCS facts (`cluster_initialized`)
- [ ] leased DCS facts (`bootstrap_lock`, `leader_lock`)
- [ ] per-tick input facts (DCS authority, trust, member freshness, local postgres runtime facts, local physical `PGDATA` facts, recovery compatibility facts)
- [ ] per-tick desired outputs (`Bootstrap`, `Primary`, `Replica`, `Quiescent`, `Fence`) and the nested subplans for each
- [ ] the exact deterministic bootstrap and initialized-cluster leader-election ordering
- [ ] the exact local `PGDATA` inspection fields and which implementation source is authoritative (`pg_control` / `pg_controldata`-equivalent)
- [ ] Replace the current startup-mode selection in `src/runtime/node.rs` with the new reconcile model and delete the old `InitializePrimary` / `CloneReplica` / `ResumeExisting` split if it is no longer the direct architecture.
- [ ] Remove the current “initialized cluster + no DCS authority + no managed replica residue => start as primary” fallback from `src/runtime/node.rs`.
- [ ] Remove every startup or HA path that treats DCS probe failure, stale DCS, or no-fresh-quorum state as enough reason to become writable primary in an initialized cluster.
- [x] Replace plain `init_lock` semantics across `src/dcs/state.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/dcs/etcd_store.rs`, and `src/runtime/node.rs` with the new durable/leased bootstrap model. No old init-lock-only bootstrap path may remain reachable after completion.
- [x] Rename `DcsTrust::FullQuorum` everywhere it is surfaced in product code, tests, debug API, and CLI output to a more truthful name such as `FreshQuorum`, and update all labels/docs accordingly.
- [x] Delete the current `foreign healthy primary` fallback model from startup and HA. After completion, follower/rejoin authority comes from the authoritative leader only, not from arbitrary member records that claim to be primary.
- [x] Implement the explicit safe policy for `cluster_initialized = false` plus non-empty local `PGDATA`:
- [x] this is always `Quiescent(UnsafeUninitializedPgData)` plus a hard surfaced error
- [x] no adoption mode exists
- [x] no silent deletion exists
- [x] Redesign `src/ha/state.rs`, `src/ha/decision.rs`, and `src/ha/lower.rs` so the public/internal HA state space reflects the new smaller set of desired states and nested plans rather than the current overlapping phases and decisions.
- [x] Redesign `src/ha/decide.rs` so replica reconciliation is one ordered flow that chooses:
- [x] direct follow
- [x] rewind then follow
- [x] basebackup then follow
- [x] without splitting that one logical flow across unrelated top-level states
- [x] Redesign `src/ha/process_dispatch.rs`, `src/ha/worker.rs`, `src/process/jobs.rs`, `src/process/state.rs`, and `src/process/worker.rs` so a follower reconcile plan can own the full transition it needs without cross-tick ambiguity or latching hacks that suppress necessary retries.
- [x] Remove the current semantic overlap between `WaitForPostgres`, `StartPostgres`, and `FollowLeader`; after completion there must be one clearly authoritative path for “this node should converge to follower-of-X”.
- [ ] Define explicitly what happens when `cluster_initialized = true` and all nodes were offline long enough for all leases to expire:
- [ ] which nodes may join election
- [ ] which nodes stay quiescent
- [ ] how no dual-primary is preserved while re-electing a leader
- [ ] how the per-node published startup/election descriptor is compared across nodes and which node is allowed to attempt the leader lease
- [ ] how cold nodes publish their pre-election descriptors before a leader exists and before writable PostgreSQL starts
- [ ] explicitly preserve the requirement that a healthy 2-of-3 quorum elects and restores one primary before the third node returns; no task rewrite or test rewrite may weaken this into “wait for all three nodes”
- [ ] Define explicitly what happens when `cluster_initialized = false` but unexpected non-empty local `PGDATA` exists; the task must choose and implement one safe policy, and test it:
- [ ] explicit hard error requiring operator action
- [ ] silent deletion or silent bootstrap-over-existing-data is forbidden
- [ ] Define explicitly what happens when the local node owns `leader_lock` and then self-detects PostgreSQL unhealth or loss of local primary validity:
- [ ] stop serving and fence immediately
- [ ] release the owned leader lease when possible through the proper owner path
- [ ] rely on lease expiry when hard death prevents active release
- [ ] do not introduce any non-leader “healthy primary fallback” for followers while waiting for lease cleanup
- [ ] Define explicitly whether cold leader election requires starting postgres before final leader acquisition:
- [ ] if no, implement the full offline inspection + published descriptor path
- [ ] if yes, the task must implement and document a non-writable/fenced pre-election postgres mode and prove it cannot create dual-primary service exposure
- [ ] Define explicitly the success ordering for first bootstrap:
- [ ] bootstrap lease acquired first
- [ ] bootstrap performed
- [ ] durable cluster identity record written on success
- [ ] `cluster_initialized` written only on success
- [ ] no durable plain `init_lock` or equivalent bootstrap-start marker remains
- [ ] Ensure `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` remain pure render/output layers:
- [ ] they may receive an authoritative reconcile plan
- [ ] they must not derive authority or role intent from local managed files
- [ ] Add or rewrite tests in `tests/ha/support/multi_node.rs`, `tests/ha_multi_node_failover.rs`, and any necessary unit/integration suites to cover at minimum:
- [ ] concurrent first bootstrap with one bootstrap winner and all other nodes staying stopped until leadership exists
- [ ] bootstrap lock expiry and bootstrap retry before cluster initialization completes
- [ ] initialized cluster restart after all nodes were offline and all leases expired
- [ ] initialized cluster restart after all leases expired but existing local `PGDATA` is preserved on every node
- [ ] in a 3-node cluster, primary hard loss with one healthy replica still online and the third node still offline must restore exactly one primary on the healthy 2-node quorum before the third node returns
- [ ] follower restart while healthy leader exists
- [ ] old primary restart after failover, including rewind-eligible and reclone-required variants
- [ ] preserved replica restart after failover
- [ ] `cluster_initialized = false` with unexpected non-empty `PGDATA`, proving the chosen safe policy
- [ ] deterministic winner-selection unit tests for the published descriptor ordering:
- [ ] higher timeline beats lower timeline
- [ ] same timeline higher durable LSN beats lower durable LSN
- [ ] running healthy beats offline inspectable on otherwise equal facts
- [ ] lexical `member_id` tie-break is stable and deterministic
- [ ] leader lease loss while fresh quorum remains
- [ ] leader loss while fresh quorum does not remain
- [ ] strict multi-node post-restart convergence assertions that validate the full expected follower set rather than only a subset of nodes
- [x] Audit and update the source-level tests in:
- [x] `src/runtime/node.rs`
- [x] `src/dcs/state.rs`
- [x] `src/ha/decision.rs`
- [x] `src/ha/decide.rs`
- [x] `src/ha/process_dispatch.rs`
- [x] `src/ha/worker.rs`
- [x] so their asserted phases/decisions/trust labels match the new architecture instead of the removed overlap
- [ ] Audit and update HA partition coverage in `tests/ha_partition_isolation.rs` and `tests/ha/support/partition.rs` wherever the new leader/bootstrap/rejoin rules change expected behavior.
- [ ] Remove or rewrite stale legacy tests, helper logic, and API/debug labels that still encode the old startup/follow split. No dead or shadow logic may remain in the final tree.
- [ ] Reconcile the Ralph HA task and bug files listed in the Scope section so completed/superseded task narratives no longer contradict the redesigned product behavior and test suite.
- [ ] Update docs in `docs/src/reference/ha-decisions.md`, `docs/src/explanation/ha-decision-engine.md`, and `docs/src/how-to/handle-primary-failure.md` so the shipped behavior matches the redesigned architecture exactly.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
- [ ] Additional remaining work discovered during retry verification:
- [ ] Replace the remaining startup-only `InitializePrimary` / `CloneReplica` / `ResumeExisting` planning split in `src/runtime/node.rs` with an authoritative reconcile-first startup path that can safely defer to HA when leader authority is not yet available.
- [ ] Add an explicit startup mode for “authoritative facts insufficient to start postgres yet” so runtime startup does not exit hard while the cluster is initialized but there is temporarily no authoritative leader or no basebackup source.
- [ ] Fix initialized-cluster empty-`PGDATA` join/rejoin so follower nodes wait for authoritative leader election instead of aborting with `cluster is already initialized but no healthy primary is available for basebackup`.
- [ ] Fix initialized-cluster preserved-`PGDATA` restart/rejoin so nodes do not abort with `cluster is already initialized but no authoritative leader is available to derive startup intent`; they must come up non-writable and reconcile through HA.
- [ ] Reconcile runtime restart semantics in `src/test_harness/ha_e2e/handle.rs` and runtime startup so `restart_node` no longer fails with `runtime restart requested for running node` in the redesigned control flow.
- [ ] Re-run and make green every failing HA multi-node e2e discovered during deep verification:
- [ ] `e2e_multi_node_partial_recovery_restores_quorum_before_full_heal`
- [ ] `e2e_multi_node_rejects_targeted_switchover_to_ineligible_member`
- [ ] `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain`
- [ ] `e2e_multi_node_repeated_failovers_preserve_intermediate_writes`
- [ ] `e2e_multi_node_repeated_leadership_changes_preserve_single_primary`
- [ ] `e2e_multi_node_rewind_failure_falls_back_to_basebackup`
- [ ] `e2e_multi_node_stress_planned_switchover_concurrent_sql`
- [ ] `e2e_multi_node_stress_unassisted_failover_concurrent_sql`
- [ ] `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`
- [ ] `e2e_multi_node_targeted_switchover_promotes_requested_replica`
- [ ] `e2e_multi_node_primary_whole_node_hard_kill_failover_recovers_after_heal`
- [ ] `e2e_multi_node_cli_primary_and_replicas_follow_switchover`
- [ ] Re-run `tests/ha_partition_isolation.rs` after the runtime startup/rejoin fixes and confirm the updated partition expectations still hold.
- [ ] Legacy-code cleanup verification still required before this task can be marked passing:
- [ ] repo-wide grep proves no `HaPhase`, `HaDecision`, `ha_phase`, `ha_decision`, `WaitForPostgres`, `FollowLeader`, `WaitingSwitchoverSuccessor`, `FullQuorum`, or old fail-safe phase labels remain in `src/` or `tests/`
- [ ] runtime startup code no longer contains the old architecture split as the controlling mental model, even if helper names survive temporarily during refactor
- [ ] multi-node and partition helpers/assertions no longer encode old phase semantics; they must assert against `DesiredNodeState` / new safe-state outcomes only
- [ ] CLI/debug/test fixtures no longer serialize or assert the removed legacy HA API fields
- [ ] source-level and e2e tests together prove that restart, cold rejoin, switchover, no-quorum fencing, and post-failover rejoin all converge through the same authoritative reconciliation rules
  </acceptance_criteria>

NOW EXECUTE
