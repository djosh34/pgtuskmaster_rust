## Task: Refactor The HA Acceptance Suite Around Typed Invariants And `NodeState`-First Assertions <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Redesign `tests/ha` so the ultra-long HA suite is organized around a small, explicit set of safety and liveness invariants instead of a large collection of ad hoc scenario-specific steps and hidden bookkeeping. The suite must become simpler to read, simpler to maintain, and stricter about what counts as truth. Cluster-state assertions must be driven directly by `NodeState`; SQL assertions must be driven directly by SQL; product-surface checks for `pgtm primary` / `pgtm replicas` must be separated from core cluster-state invariants instead of being mixed into every scenario as if they were a second source of truth.

**Higher-order goal:** The user explicitly questioned why `tests/ha` is so complicated when the apparent intent is "execute scenario actions, then check status". Research showed that a large part of the current complexity is real black-box HA harness work, but a significant part is accidental complexity caused by a giant `steps/mod.rs`, stringly and stale scenario bookkeeping, partial duplication between step families, mixed use of semantic aliases and physical node names, and fallback logic that hides bugs in `NodeState` instead of failing loudly. The refactor must keep the black-box nature of the HA tests while aggressively simplifying the DSL, internal structure, and assertion model.

**Decisions already made from research and user discussion:**
- The suite is allowed to change expected assertions and scenario shapes if the new assertions are more coherent and more directly express the intended invariants.
- Feature files may be merged if they are materially the same scenario class and differ only by repetitive wording or thin variants. Do not preserve one-file-per-current-scenario just because that is how the current tree is laid out.
- Mid-scenario assertions are still required. This is not a "setup, do many actions, assert once at the end" redesign. Some invariants must be asserted during the transition window, immediately after a fault, before healing, or before a stopped node is restarted. The new design must preserve those checkpoints.
- `NodeState` must be the sole source of truth for cluster-state assertions. If the suite expects a node to have known state and `NodeState` still reports `Unknown`, that must fail. Do not fall back to SQL or `pgtm` helper behavior to "recover" from an unknown `NodeState` role.
- SQL is the source of truth for data-plane assertions only: write success/rejection, proof-row visibility, replication convergence, fencing cutoffs, and split-brain write evidence.
- When a scenario intentionally checks a node's published authority or writability, add a separate SQL corroboration step that confirms the data plane agrees with the published `NodeState`. This corroboration is a second explicit assertion, not a fallback and not a reinterpretation of `NodeState` when `NodeState` is wrong.
- `pgtm primary` / `pgtm replicas` are not a second cluster-state oracle. They are product-surface checks. They should remain covered, but by a smaller set of explicit product-surface assertions or tests rather than being repeated everywhere as though they independently establish who the primary is.
- Log scraping is not an acceptable correctness oracle for the HA acceptance suite. Logs may still be captured as artifacts for debugging, but pass/fail assertions must be based on current typed state or real observation surfaces (`NodeState`, SQL, explicit CLI/API result surfaces).
- Fault injection itself is part of the refactor contract. The suite must stop relying on ad hoc marker-file copying into stopped containers. Replace that mechanism with one clean, typed fault-control path, such as a dedicated bind-mounted fault volume or an equivalently explicit harness-controlled mechanism that works the same way for running and stopped nodes.
- A replica is not "never primary" in the absolute sense. Any replica that is part of the healthy majority and eligible for leadership may become primary when the prior primary loses authority or disappears. The actual invariant is: a node that is isolated into the minority, ineligible, or explicitly degraded must never become authoritative primary; an eligible majority-side replica may and should race to become primary when liveness requires it.
- Switchover eligibility must not be a weaker, parallel concept. Planned and targeted switchover admission must use the exact same typed eligibility contract that HA failover uses to decide whether a node may become authoritative primary.
- The suite should prefer semantic role aliases over physical node names. Physical names like `node-a` / `node-b` / `node-c` should appear only when a scenario truly depends on fixed identities, fixed configs, or a specific given. Otherwise, scenarios should name the relevant participants at the beginning in semantic terms such as `old_primary`, `target_replica`, `healthy_replica`, `isolated_node`, `majority_primary`, etc.
- There should not be magic "default aliases" silently assumed by the harness. If a scenario wants semantic names, it should declare them explicitly near the top. That keeps the feature text honest and avoids hidden state. The one exception is that the harness can still expose fixed topology constants internally; the feature DSL itself should remain explicit.
- Recovery-path scenarios must assert the typed recovery policy rather than merely eventual success: if a node can simply resume following/streaming, that path should be chosen before destructive recovery; if divergence is rewind-eligible, `pg_rewind` must be tried before `pg_basebackup`; `pg_basebackup` is only correct when the typed state says it is required or after an explicit rewind failure that marks fallback to basebackup.

**Target invariant model for the HA suite:**

Safety invariants that must hold whenever applicable:
- Never more than one authoritative primary at any point in the observed transition window.
- A minority-isolated or otherwise ineligible node never becomes authoritative primary.
- A rejected switchover never changes cluster authority.
- If DCS quorum is lost, the cluster must not expose an operator-visible writable primary until quorum and authority are properly restored.
- After a fencing cutoff is observed, no later writes may commit.
- `pgtm primary` / `pgtm replicas` must never contradict the authoritative information derived from the same seed `NodeState` when those product-surface commands are explicitly tested.
- When a scenario claims a node is authoritative, non-authoritative, writable, or non-writable, the separately asserted SQL behavior must agree with the published `NodeState`; disagreement is a failure, not a cue to reinterpret one surface through the other.

Liveness invariants that must eventually hold under the required preconditions:
- With DCS quorum and enough healthy members, the cluster eventually converges to exactly one authoritative primary.
- A restarted old primary eventually rejoins as a replica when the cluster is healthy enough to recover it.
- Healed replication paths eventually lead replicas to converge to the primary's committed rows.
- A planned or targeted switchover eventually reaches the intended single-primary state when the target is eligible.
- A healthy majority eventually restores service after losing one node or recovering quorum.

**Required conceptual split between truth surfaces:**
- `NodeState`: cluster-state / authority / role / quorum / fail-safe / visible-member assertions.
- SQL: proof rows, replication convergence, write success/failure, split-brain evidence, fencing cutoff evidence.
- Typed `ha` / `process` state exposed through `NodeState`: recovery-path assertions such as follow/start-streaming vs rewind vs basebackup, and explicit process-failure recovery semantics.
- `pgtm primary` and `pgtm replicas`: narrow product-surface validation only, in their own assertions or smaller dedicated tests, not as a fallback and not as a pervasive co-assertion in every HA scenario.
- Cross-surface corroboration: explicit separate checks that SQL behavior agrees with the published `NodeState` for authority/writability claims. These corroboration checks must never be used to "repair" or reinterpret incorrect `NodeState`.

**Scope:**
- Redesign the structure of `tests/ha/support/steps/`, `tests/ha/support/world/`, and adjacent support modules so step files become thin adapters over typed harness/query/assertion layers rather than giant mixed-concern modules.
- Rewrite or merge the `.feature` files under `tests/ha/features/` around a smaller invariant-oriented DSL. Preserve scenario coverage, but change scenario wording, grouping, and assertions wherever that simplifies the suite while still proving the intended invariants.
- Replace the current stopped-container fault-marker mechanism with a cleaner harness-level fault-control design. The refactor must not keep `docker cp`-style direct file injection into stopped container filesystems as the way blockers and restart-time faults are expressed.
- Remove fallback role/state behavior from the step/assertion layer. In particular, eliminate the current behavior where `NodeState::Unknown` can be masked by checking direct SQL connectivity or other helper surfaces to decide whether a node "is really a replica anyway".
- Remove stale or misleading scenario bookkeeping such as `unsampled_nodes` if the new assertion model can express reachability and observer scope explicitly.
- Centralize cluster topology, member names, and observer config selection instead of hardcoding topology in multiple places.
- Unify switchover-target admission with failover eligibility. The acceptance suite, helper layers, and scenario expectations must encode one typed notion of "promotion-eligible", not separate weaker API-admission and stronger HA-execution predicates.
- Keep the suite black-box and end-to-end. This is not a rewrite into unit tests. The harness must still start real Docker stacks, inject real faults, and validate eventual behavior through black-box observation surfaces.
- It is acceptable and preferred to reduce total feature count if several current files are thin variants of the same invariant class and can be merged without losing coverage.
- Remove log-content assertions from HA correctness checks. If a scenario currently proves behavior only by matching log text, rewrite it to assert typed current state and real external behavior instead.

**Out of scope:**
- Do not weaken the suite by deleting hard scenarios solely because they are difficult. If a scenario proves a distinct invariant or fault class, keep that coverage even if the file layout changes.
- Do not reintroduce sampled/debug-era semantics, hidden "best effort" fallbacks, or new broad linter ignores.
- Do not turn the HA suite into direct unit tests of internal functions. The target remains acceptance-style black-box testing.

**Context from research:**
- `tests/ha/support/steps/mod.rs` is currently about 2.3k lines and mixes step registration, polling, SQL/proof logic, DSN construction, status formatting, direct fallback behavior, and fault semantics.
- `tests/ha/support/world/mod.rs` is currently about 1k lines and combines per-scenario state, harness bootstrap, Docker actions, network fault injection, artifact capture, timeline recording, and helper functions.
- `tests/ha/support/docker/cli.rs` is a justified layer because it provides the real control plane for multi-container HA tests, but it repeats Compose plumbing and should likely be split or wrapped by a typed Compose context.
- `tests/ha/support/faults/mod.rs` is one of the cleaner parts of the suite because it models failure modes with enums such as `TrafficPath` and `BlockerKind`; keep that typed direction.
- The current fault-marker plumbing for blocked rejoin and blocked basebackup scenarios is not acceptable as the long-term mechanism. It currently depends on copying marker files into stopped containers and on restart-time cleanup behavior that can drift by ownership or permissions.
- `tests/ha/support/observer/pgtm.rs` already uses `NodeState` directly as the cluster status view and reuses production connection DTOs from `src/cli/connect.rs`. The current problem is no longer duplicated local status DTOs; it is the orchestration, fallback, and DSL layers around them.
- `tests/ha/support/steps/mod.rs` still contains current fallback behavior that undermines `NodeState` as truth. Example: `assert_member_is_replica_via_member(...)` in `tests/ha/support/steps/mod.rs` accepts `MemberPostgresView::Unknown(_)` and then falls back to `pg_is_in_recovery()` via SQL. This masks bugs where `NodeState` should have already reported a known role.
- `tests/ha/support/steps/mod.rs` also contains DSN fallback behavior in `sql_target_for_member(...)`, where `pgtm` connection helper failures are replaced with direct member DSNs. This can remain only for SQL/data-plane assertions if explicitly justified, but it must not be used to infer cluster role when `NodeState` is unknown.
- The current scenario state stores `aliases`, `markers`, `stopped_nodes`, `wedged_nodes`, `unsampled_nodes`, `proof_convergence_blocked_nodes`, `last_command_output`, `proof_rows`, `proof_table`, and `observed_primaries` in one mutable bag. This is practical but not well-typed.
- `unsampled_nodes` currently affects `online_expected_count(...)` and therefore many assertions. It is stale terminology from older semantics and should be removed if assertion scope becomes explicit.
- The current feature corpus under `tests/ha/features/` contains 26 feature files, 345 step lines, and 183 unique step texts. Highly repeated steps include:
  - `the 3 online nodes contain exactly the recorded proof rows`
  - `the "three_node_plain" harness is running`
  - `I create a proof table for this feature`
  - `I wait for exactly one stable primary as "initial_primary"`
  - `I start tracking primary history`
  - `I heal all network faults`
- Current features also reveal repeated scenario classes:
  - primary killed / old primary rejoins
  - replica stopped / replica flapped / two replicas stopped
  - DCS quorum lost with and without workload fencing
  - minority partition of old primary or isolated replica
  - targeted and planned switchover variants
  - blocked rejoin / blocked basebackup / blocked rewind
  - replication-path lag and convergence
- This repetition is a strong signal that the feature DSL and file layout can be simplified without losing coverage.

**Target design direction for features and steps:**
- Features should be written around named participants and invariants. Example opening pattern:
  - the harness/given is running
  - I label the current primary as `"old_primary"`
  - I label one eligible replica as `"target_replica"`
  - I label the remaining replica as `"other_replica"`
- Avoid mixing semantic aliases and physical node names unless the scenario genuinely depends on a fixed physical member.
- Mid-scenario assertions must remain first-class. Examples:
  - after fault, before heal, assert no operator-visible primary
  - after majority failover, before old primary heal, assert old primary never became authoritative
  - during replication isolation, assert replicas do not yet contain a new row
  - during switchover window, assert no dual-primary evidence
- Product-surface assertions for `pgtm primary` / `pgtm replicas` should be kept only where the product surface itself is what is being tested. Cluster-authority assertions should come from `NodeState`.
- Switchover scenarios must encode failover-grade eligibility directly. If a target would be ineligible for failover because it is isolated, lagging, API-unreachable, or otherwise not promotion-eligible, the switchover path must reject it under the same contract instead of using a weaker admission check.
- Recovery-path scenarios should assert the typed `ha` / `process` state that is already exposed via `NodeState`, instead of using compose-log snippets as proof that `pg_rewind`, `pg_basebackup`, or follower/start-streaming recovery happened.

**Concrete canonical step set to end up with:**

Setup / participant-labeling steps:
- `the "<given>" harness is running`
- `I label the current primary as "<alias>"`
- `I label one eligible replica as "<alias>"`
- `I label the two replicas as "<alias_a>" and "<alias_b>"`
- `I label the remaining replica as "<alias>"`
- `I create proof tracking`
- `I create workload tracking`
- `I record transition window "<marker>"`

Action steps:
- `I kill "<alias>"`
- `I kill "<alias_a>" and "<alias_b>"`
- `I start "<alias>"`
- `I restart "<alias>"`
- `I isolate "<alias>" on "<api|dcs|postgres|all>"`
- `I isolate "<alias_a>" and "<alias_b>" on "<api|dcs|postgres|all>"`
- `I heal "<alias>"`
- `I heal the cluster`
- `I enable blocker "<pg_basebackup|pg_rewind|postgres_start>" on "<alias>"`
- `I disable blocker "<pg_basebackup|pg_rewind|postgres_start>" on "<alias>"`
- `I wipe the data directory on "<alias>"`
- `I wedge postgres on "<alias>"`
- `I unwedge postgres on "<alias>"`
- `I stop DCS quorum`
- `I restore DCS quorum`
- `I request planned switchover`
- `I request switchover to "<alias>"`
- `I request switchover to "<alias>" and expect rejection`
- `I write proof token "<token>" through "<alias>"`
- `I start the write workload`
- `I stop the write workload`

Cluster-state assertion steps, all driven by `NodeState` only:
- `eventually exactly one primary exists across <n> reachable members as "<alias>"`
- `eventually no authoritative primary exists across <n> reachable members`
- `"<alias>" remains the only primary`
- `eventually "<alias>" is a replica`
- `"<alias>" never becomes primary during window "<marker>"`
- `always no dual primary occurs during window "<marker>"`
- `eventually "<alias>" enters fail-safe or loses authority safely`
- `every reachable node reports fail-safe`
- `the lone reachable node is not a writable primary`
- `the cluster is degraded but operational across <n> reachable members`

Recovery-path assertion steps, driven by typed `ha` / `process` state exposed through `NodeState`:
- `eventually "<alias>" follows "<leader>" with recovery plan "<none|start_streaming|rewind|basebackup>"`
- `eventually "<alias>" records process failure "<pg_rewind|pg_basebackup|postgres_start>" with recovery "<retry_same_job|fallback_to_basebackup|wait_for_operator>"`

Data-plane assertion steps, all driven by SQL/workload evidence only:
- `the healthy members contain exactly the recorded proof tokens`
- `"<alias>" does not yet contain proof token "<token>"`
- `proof writes through "<alias>" succeed`
- `proof writes through "<alias>" are rejected`
- `the workload records at least one commit`
- `the workload establishes a fencing cutoff with no later commits`
- `there is no split-brain write evidence during window "<marker>"`

Explicit product-surface assertion steps, used narrowly and intentionally:
- `pgtm primary resolves to "<alias>"`
- `pgtm replicas resolves to every healthy replica except "<alias>"`
- `direct API observation to "<alias>" fails`
- `the last operator-visible error is recorded`

**Concrete step merges from the current suite into that canonical set:**
- Merge `the current primary container crashes` and `I kill the node named "..."` into `I kill "<alias>"`. The scenario must label the primary explicitly first.
- Merge `I start the killed node container again`, `I restart the node named "..."`, and `I start only the fixed nodes "..." and "..."` into `I start "<alias>"`, `I restart "<alias>"`, and `I kill/start "<alias_a>" and "<alias_b>"`.
- Merge `I choose one non-primary node as "..."`, `I choose the two non-primary nodes as "..." and "..."`, and `I record the remaining replica as "..."` into the explicit participant-labeling family.
- Merge `the cluster reaches one stable primary`, `I wait for exactly one stable primary as "..."`, `after the configured HA lease deadline a different node becomes the only primary`, `I wait for a different stable primary than "..." as "..."`, and `I wait for the primary named "..." to become the only primary` into the single canonical `eventually exactly one primary exists across <n> reachable members as "<alias>"`.
- Merge `there is no operator-visible primary across ...`, `the lone online node is not treated as a writable primary`, and the DCS fail-safe expectations into the `eventually no authoritative primary exists`, `the lone reachable node is not a writable primary`, and `every reachable node reports fail-safe` steps.
- Merge `the node named "..." rejoins as a replica`, `the node named "..." remains online as a replica`, and the recovery-deadline variants into `eventually "<alias>" is a replica`.
- Merge `I enable the "..." blocker on the node named "..."` and `I disable the "..." blocker on the node named "..."` into the blocker on/off pair with typed blocker parameters.
- Rebuild the blocker on/off implementation around the new harness-level fault-control mechanism so the same typed operation works for running nodes, stopped nodes, and repeated restart sequences without ownership or cleanup drift.
- Merge all path-specific isolation wording into `I isolate "<alias>" on "<path>"` and `I isolate "<alias_a>" and "<alias_b>" on "<path>"`.
- Merge `I heal all network faults` and `I heal network faults on the node named "..."` into `I heal the cluster` and `I heal "<alias>"`.
- Merge `I wedge the node named "..."` and `I unwedge the node named "..."` into `I wedge postgres on "<alias>"` and `I unwedge postgres on "<alias>"`.
- Merge `I stop the DCS service`, `I stop a DCS quorum majority`, `I start the DCS service`, and `I restore DCS quorum` into `I stop DCS quorum` and `I restore DCS quorum`.
- Merge `I create a proof table for this feature` and `I create one workload table for this feature` into separate but canonical `I create proof tracking` and `I create workload tracking`.
- Merge `I start tracking primary history` and `I record marker "..."` into the explicit `I record transition window "<marker>"` model. The old implicit "primary history" wording should disappear.
- Merge `the primary history never included "..."` and `the node named "..." never becomes primary after marker "..."` into `"<alias>" never becomes primary during window "<marker>"`.
- Merge the two distinct alias-distinct steps into one if such an assertion still remains necessary at all.

**Current steps/assertions to throw away entirely rather than rename:**
- `the cluster reaches one stable primary` legacy wording, because all scenarios should label the primary explicitly instead.
- `I record the current pgtm primary and replicas views`, because persistent product-surface snapshots are not part of the new invariant-first model.
- `the node named "..." is not queryable through pgtm connection helpers` / `the node named "..." is not queryable` as a generic cluster-state assertion. Replace with explicit API failure or SQL failure steps only where that surface is what the scenario actually intends to prove.
- Any assertion that decides cluster role by combining `NodeState` with direct SQL fallback when `NodeState` says `Unknown`.
- Hidden arithmetic such as `online_expected_count(...)` derived from mutable `unsampled_nodes` state.
- Any assertion that proves HA behavior by matching log text such as blocker evidence or recovery-path snippets. Logs remain debug artifacts only and are not a pass/fail correctness surface.

**Concrete target feature/scenario inventory after the refactor:**

1. `ha_primary_failover_and_rejoin.feature`
- Scenario: killed primary fails over and later rejoins as replica via follow/start-streaming when its state is still reusable
- Scenario: killed primary under concurrent writes preserves single-primary and data safety
- Scenario: wedged primary loses authority, a new primary is elected, and the old primary never regains leadership

2. `ha_replica_outage_and_recovery.feature`
- Scenario: single replica outage does not replace the current primary
- Scenario: repeated replica flaps never replace the stable primary
- Scenario: losing two replicas leaves the lone node non-writable until one healthy replica returns
- Scenario: full cluster outage followed by two fixed-node returns restores service before the final node rejoins
- Scenario: one healthy return restores service even while another node remains broken

3. `ha_repeated_failovers.feature`
- Scenario: repeated failovers preserve single-primary safety and distinct leaders when topology still allows a new eligible majority-side leader

4. `ha_dcs_quorum_and_fencing.feature`
- Scenario: losing DCS quorum removes authoritative primary visibility and exposes fail-safe behavior
- Scenario: losing DCS quorum fences writes until quorum is restored
- Scenario: mixed DCS loss and observer-API isolation heals back to one healthy primary

5. `ha_majority_minority_partitions.feature`
- Scenario: old primary isolated into the minority loses authority, the majority elects a new primary, and the healed old primary rejoins only as a replica
- Scenario: isolated replica in the minority never self-promotes while the majority preserves one primary
- Scenario: non-primary observer-API isolation does not change authority

6. `ha_replication_degradation_and_catchup.feature`
- Scenario: replication-path isolation delays proof-token visibility and healed replicas catch up
- Scenario: a minority-side or otherwise ineligible degraded replica is not promoted during failover

7. `ha_switchover.feature`
- Scenario: planned switchover moves leadership cleanly to a different primary
- Scenario: planned switchover under concurrent writes preserves single-primary safety
- Scenario: targeted switchover promotes the requested eligible replica and not the other one
- Scenario: targeted switchover to an ineligible or degraded replica is rejected under the exact same eligibility contract as failover, without authority change

8. `ha_rejoin_recovery_paths.feature`
- Scenario: blocked basebackup clone recovers after the blocker is removed when basebackup is actually required
- Scenario: rewind-eligible rejoin attempts `pg_rewind` before any `pg_basebackup` fallback, and only falls back to basebackup after an explicit rewind failure
- Scenario: a broken rejoin attempt does not destabilize quorum or steal leadership

9. `ha_custom_roles.feature`
- Scenario: non-default replicator and rewinder roles survive failover and rejoin

**Explicit current-to-target feature merge plan:**
- Merge current `ha_primary_killed_then_rejoins_as_replica`, `ha_primary_killed_with_concurrent_writes`, and `ha_primary_storage_stalled_then_new_primary_takes_over` into `ha_primary_failover_and_rejoin.feature` as three scenarios.
- The `ha_primary_killed_then_rejoins_as_replica` rewrite must explicitly assert that the restarted old primary returns by follower/start-streaming recovery when the typed state says destructive recovery is unnecessary, rather than merely asserting that it eventually becomes a replica.
- Merge current `ha_replica_stopped_primary_stays_primary`, `ha_replica_flapped_primary_stays_primary`, `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`, `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins`, and `ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken` into `ha_replica_outage_and_recovery.feature`.
- Keep current `ha_repeated_failovers_preserve_single_primary` as `ha_repeated_failovers.feature`, rewritten around semantic aliases and the explicit invariant that an ineligible former leader never regains authority during the second failover window. Do not spin this out into a separate bug task; `make test-long` remains the regression detector for this scenario family.
- Merge current `ha_dcs_quorum_lost_enters_failsafe`, `ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes`, and `ha_dcs_and_api_faults_then_healed_cluster_converges` into `ha_dcs_quorum_and_fencing.feature`.
- Merge current `ha_old_primary_partitioned_from_majority_majority_elects_new_primary` and `ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover` into one stronger old-primary-minority scenario inside `ha_majority_minority_partitions.feature`.
- Keep current `ha_replica_partitioned_from_majority_primary_stays_primary` and `ha_non_primary_api_isolated_primary_stays_primary` as the other two scenarios in `ha_majority_minority_partitions.feature`.
- Merge current `ha_replication_path_isolated_then_healed_replicas_catch_up` and a rewritten replacement for `ha_lagging_replica_is_not_promoted_during_failover` into `ha_replication_degradation_and_catchup.feature`. The current lagging-replica expectation is wrong and must not be preserved as-is; the rewritten scenario must make the non-promoted node actually minority-side or otherwise promotion-ineligible.
- Merge current `ha_planned_switchover_changes_primary_cleanly`, `ha_planned_switchover_with_concurrent_writes`, `ha_targeted_switchover_promotes_requested_replica`, and `ha_targeted_switchover_to_degraded_replica_is_rejected` into `ha_switchover.feature`, with targeted-switchover rejection and acceptance both driven by the exact same typed eligibility predicate used for failover.
- Merge current `ha_basebackup_clone_blocked_then_unblocked_replica_recovers`, `ha_rewind_fails_then_basebackup_rejoins_old_primary`, and `ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum` into `ha_rejoin_recovery_paths.feature`.
- Keep current `ha_primary_killed_custom_roles_survive_rejoin` as `ha_custom_roles.feature`.

**Concrete post-refactor assertion model: what is kept, what is narrowed, what is thrown away:**

Kept and strengthened:
- Single-primary safety checks remain, but are always `NodeState`-based.
- Fail-safe and no-primary checks remain, but are always `NodeState`-based.
- Replica rejoin checks remain, but must fail if `NodeState` still reports `Unknown` where the suite expects a known replica role.
- Recovery-path checks remain, but must now assert typed `ha` / `process` state for follow/start-streaming vs rewind vs basebackup instead of merely checking eventual replica state.
- Proof-token and workload checks remain, but are strictly SQL/workload-based.
- SQL corroboration remains separate and explicit where a scenario claims a node is writable or non-writable; these checks must agree with the published `NodeState` without becoming a fallback truth source.
- Mid-scenario transition-window assertions remain, especially for:
  - no dual-primary
  - no-primary/fail-safe
  - minority node never becoming primary
  - row absence during replication isolation

Narrowed:
- `pgtm primary` and `pgtm replicas` assertions remain only in the scenarios where the product surface is intentionally under test, mainly selected switchover and connection-surface scenarios.
- API reachability assertions remain only where the scenario is specifically about observer/API isolation, not as a general proxy for cluster health.
- Fault-control assertions remain only as black-box behavior checks over the new fault mechanism. No scenario should depend on direct marker-file lifecycle, stopped-container file copying, or log text as the proof that a blocker was applied or cleared.

Thrown away:
- Any role or authority inference that falls back from `NodeState` to SQL or `pgtm` helper behavior.
- Any use of `MemberPostgresView::Unknown(_)` as a reason to probe a second surface and still accept the result as a successful role assertion.
- Any hidden "online expected count" math based on `unsampled_nodes`.
- The idea that `pgtm primary` and `NodeState` independently assert the same cluster truth in the same HA scenario by default.
- Any log-snippet/blocker-evidence assertion as a proof of correctness. The suite must assert current state and external behavior directly instead.

**Expected outcome:**
- The HA suite expresses a smaller, clearer language centered on invariant classes instead of one-off step phrasing.
- `tests/ha/support/steps/` is split into smaller domain modules, and step functions are thin adapters over typed helper APIs.
- `NodeState` is the strict source of truth for cluster-state assertions, with no fallback that masks `Unknown` or incorrect state publication.
- SQL assertions remain end-to-end and explicit, but are separated conceptually from cluster-role assertions.
- `pgtm` product-surface checks are still covered, but in a narrower and more honest way that does not duplicate the main cluster-state oracle.
- The feature set becomes smaller or at least less repetitive, with merged files where appropriate and more semantic participant naming.
- The suite is easier to reason about: a reader can tell which invariant is being tested, which observation surface proves it, and at what point in the scenario that invariant must hold.

</description>

<acceptance_criteria>
- [x] `tests/ha/support/steps/mod.rs` is fully replaced by a split step-module tree, and no replacement step file becomes a new god module with mixed harness/assertion/SQL/polling responsibilities
- [x] The suite-structure cleanup is fully landed in this task: no mixed-concern god module remains in `tests/ha/support`, no hidden fallback-based role inference remains, and no stale mutable bookkeeping like `unsampled_nodes` survives under a new name
- [x] A typed shared topology source exists and removes duplicated hardcoded member/service/config knowledge from `tests/ha/support/faults/mod.rs`, `tests/ha/support/observer/pgtm.rs`, and step files
- [x] The refactor lands the concrete canonical step set described in this task, and each surviving step maps to one typed underlying harness/assertion operation rather than a large mixed-concern branch
- [x] The feature corpus is rewritten to the concrete target feature/scenario inventory described in this task, including the specified merges of current scenario files into scenario-family feature files
- [x] `NodeState` is the sole truth source for cluster-role / authority / quorum / fail-safe assertions; no step or assertion helper falls back to SQL or `pgtm` connection behavior to reinterpret `Unknown` cluster state
- [x] SQL remains the sole truth source for data-plane assertions such as proof-row visibility, replication convergence, write rejection, fencing cutoff, and split-brain evidence
- [x] Where a scenario claims a node is authoritative, non-authoritative, writable, or non-writable, the suite also performs a separate SQL corroboration check that agrees with the published `NodeState`; this corroboration must remain a separate assertion and must never be used as fallback role inference
- [x] `pgtm primary` / `pgtm replicas` checks are reduced to explicit product-surface assertions or dedicated tests and are no longer used as a pervasive co-assertion of cluster authority
- [x] Fault injection is rebuilt around one clean harness-controlled mechanism that works for running nodes, stopped nodes, and repeated restart sequences without direct stopped-container file copying or permission-sensitive cleanup behavior
- [x] Switchover admission and failover promotion use the exact same typed eligibility contract; the suite explicitly covers both accepted and rejected targeted switchover cases against that one shared predicate
- [x] `unsampled_nodes` is removed entirely, and all assertions that previously depended on it are replaced by explicit, typed reachability or scope expectations
- [x] The new feature DSL uses explicit semantic aliases declared near scenario start and no hidden "default alias" behavior; physical node names are used only where the scenario truly depends on fixed identities or configs
- [x] Mid-scenario assertions remain present where they are semantically necessary; the refactor does not collapse the suite into end-only assertions
- [x] Current fallback behavior that masks `NodeState` bugs is removed, including the `MemberPostgresView::Unknown(_)` fallback path in HA assertions unless a scenario explicitly asserts that state should remain unknown
- [x] Recovery-path scenarios assert typed `ha` / `process` state exposed through `NodeState`, including the ordering policy that follower/start-streaming recovery is preferred when valid, `pg_rewind` is tried before `pg_basebackup` when rewind is possible, and `pg_basebackup` is only used when the typed state requires it or after explicit rewind failure with fallback-to-basebackup semantics
- [x] Log-based HA assertions are removed. Compose logs and process logs may be captured for debugging artifacts, but no HA scenario passes or fails based on matching log text
- [x] Repetitive feature files are merged where possible without losing coverage of a distinct invariant or fault class
- [x] The resulting feature set is organized around explicit invariant classes and scenario families, with clear naming and without preserving current file count or wording just for continuity
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Define the post-refactor truth model and invariant catalog
- [x] Create a short HA-test invariant catalog in the repo documentation or task notes and implement against it. The catalog must name:
  - safety invariants
  - liveness invariants
  - which observation surface proves each invariant (`NodeState`, SQL, or explicit `pgtm` product-surface checks)
- [x] Audit all current `.feature` files in `tests/ha/features/` and map each one to one or more invariant classes. Record which scenarios are:
  - unique and must remain
  - repetitive variants that should be merged
  - currently over-asserting the same fact through multiple truth surfaces
- [x] Extend the invariant catalog with explicit cross-surface corroboration rules:
  - `NodeState` remains authoritative for cluster-role assertions
  - SQL corroboration is a separate required check where the scenario claims client-visible writability or non-writability
  - disagreement between `NodeState` and SQL is a test failure, not a cue to reinterpret one surface through the other
- [x] Make the invariant vocabulary explicit in code comments and/or docs for the new assertion layer so a future maintainer understands why some assertions are `always`-style transition checks and others are `eventually`-style convergence checks.

### Phase 2: Introduce typed topology and typed scenario state
- [x] Add a shared topology module, for example `tests/ha/support/topology.rs`, that owns:
  - cluster members
  - service names
  - observer config paths
  - helper iteration over members
- [x] Replace repeated string constants and `all_cluster_members()` / hardcoded config-path matches with that shared topology source.
- [x] Refactor `ScenarioState` in `tests/ha/support/world/mod.rs` into smaller typed structs such as:
  - alias registry
  - workload/proof tracking
  - transition markers and timeline window state
  - expected fault/reachability scope
- [x] Avoid bags of unrelated `BTreeSet<String>` state where a typed enum or dedicated struct would make the semantics explicit.

### Phase 3: Split the support code by domain
- [x] Replace the current monolithic layout with domain-based modules. A target layout like the following is acceptable:
  - `tests/ha/support/world/` for harness and scenario state
  - `tests/ha/support/observe/` for `NodeState`, SQL, and product-surface observation helpers
  - `tests/ha/support/assert/` for polling and invariant assertions
  - `tests/ha/support/steps/` for thin cucumber adapters
- [x] Move repeated polling loops out of step files into a generic polling/assertion layer. There should be one shared poll engine and small typed predicate/check helpers rather than many hand-rolled deadline loops.
- [x] Move DSN resolution, row-fetch logic, and proof-table helpers out of step files into dedicated SQL/data modules.
- [x] Keep `tests/ha/support/faults/mod.rs` typed and focused; move only the higher-level orchestration around it, not the fault ADTs themselves.
- [x] Consider introducing a typed Compose context or wrapper around the repeated Compose command plumbing in `tests/ha/support/docker/cli.rs`.
- [x] Remove log-scraping assertion helpers from the HA pass/fail path. If logs are still collected, they should live only in artifact/debug helpers and not in the assertion DSL.
- [x] Redesign fault injection under `tests/ha/support/faults/` and adjacent harness code so blocker/restart-time faults are expressed through one explicit harness-controlled channel, preferably a dedicated bind-mounted fault-control directory or an equivalently clean mechanism, instead of direct marker-file copying into stopped containers.

### Phase 4: Remove fallback-based role inference
- [x] Delete the current cluster-role fallback behavior where `NodeState` `Unknown` is treated as acceptable if a direct SQL check suggests the node is in recovery.
- [x] Rewrite replica/primary assertions so they fail if `NodeState` is unknown at a point where the suite expects the node to have known state.
- [x] Keep direct SQL checks only for data assertions, and keep `pgtm` helper checks only for explicit product-surface assertions.
- [x] Add explicit paired-but-separate SQL corroboration helpers for the scenarios that claim a node is writable or non-writable, so the suite checks that the data plane agrees with the published `NodeState` without using SQL as fallback role inference.
- [x] Audit all usages of:
  - `assert_member_is_replica_via_member(...)`
  - `sql_target_for_member(...)`
  - `current_primary_target(...)`
  - any other helper that silently substitutes one observation surface for another
- [x] For any remaining fallback that is genuinely needed for a data-plane assertion, document why it is allowed there and why it is not a cluster-role fallback.

### Phase 5: Remove `unsampled_nodes` and replace with explicit reachability scope
- [x] Delete `unsampled_nodes` from `ScenarioState`.
- [x] Replace `online_expected_count(...)` and similar helpers with explicit assertion scopes such as:
  - reachable members from observer API
  - expected healthy SQL targets
  - expected authoritative-members set
- [x] Rewrite affected assertions so their scope is passed in explicitly instead of derived from hidden mutable state.
- [x] Audit every current usage of `unsampled_nodes` and `proof_convergence_blocked_nodes`. If a concept remains necessary, replace it with a better-named, typed structure that describes the actual reason an assertion scope is reduced or delayed.
- [x] Ensure that removing `unsampled_nodes` does not accidentally weaken minority-partition or API-isolation scenarios; those scenarios must still state exactly which observations should or should not work at each step.

### Phase 6: Design a smaller canonical step DSL
- [x] Introduce typed cucumber parameters using `#[derive(Parameter)]` where helpful. This repo's `cucumber = "0.22.1"` supports custom typed parameters.
- [x] Create a small canonical set of setup/action/assertion step families. A target design is:
  - setup: start harness, label participants, create proof/workload context
  - actions: kill/start/restart/isolate/heal/enable-blocker/request-switchover
  - cluster assertions: exactly one primary, no primary, member is replica, member never became primary, no dual-primary during window
  - data assertions: write token, rows converge, row absent during lag, fencing cutoff, workload summary checks
  - product assertions: explicit `pgtm primary` or `pgtm replicas` checks where intentionally covered
- [x] Merge step families only when the merged step corresponds to one underlying typed operation. Examples that should likely merge:
  - enable/disable blocker
  - kill/start/restart with action parameter
  - isolate on `api|dcs|postgres|all`
  - several primary wait/assert variants
- [x] Do not merge steps if the result becomes a large branching function with unrelated world-state side effects.
- [x] Eliminate duplicated current behaviors such as the two distinct "aliases are distinct" steps.

### Phase 7: Rewrite feature files around scenario families and invariants
- [x] Rewrite the `.feature` files under `tests/ha/features/` so they name semantic participants explicitly near the start rather than mixing semantic names and physical member names throughout the scenario.
- [x] Merge feature files when they are materially the same invariant family and differ only by repetitive wording or a thin parameter variation.
- [x] Preserve or improve coverage for these scenario families:
  - primary loss and old-primary rejoin
  - replica outage / flapping replica
  - repeated failovers with distinct leaders when topology still allows them
  - majority restoration after losing two nodes
  - DCS quorum loss and recovery
  - workload fencing under quorum loss
  - minority partition of old primary
  - minority partition of replica
  - replication-path isolation and later convergence
  - degraded-replica failover rejection only when that replica is actually minority-side or otherwise promotion-ineligible
  - planned switchover
  - targeted switchover accepted
  - targeted switchover rejected
  - blocked rejoin / blocked basebackup / blocked rewind
- [x] Preserve or improve coverage for recovery-path policy itself:
  - a reusable replica/old-primary rejoin prefers follower/start-streaming recovery instead of destructive recovery
  - rewind-eligible divergence plans `pg_rewind` before `pg_basebackup`
  - `pg_basebackup` is used only when typed state requires it or after explicit rewind failure marked as fallback-to-basebackup
- [x] For each scenario family, decide which assertions are:
  - immediate post-action assertions
  - transition-window safety assertions
  - eventual convergence assertions after heal/recovery
- [x] Make sure the new feature files remain readable and declarative. The target is a smaller, cleaner DSL, not hidden complexity inside helper wording.
- [x] Rewrite the current lagging-replica failover scenario so it asserts a correct invariant. Do not preserve the current "lagging replica is not promoted" expectation unless the scenario topology actually makes that replica ineligible under the same typed rules used by failover and switchover.

### Phase 8: Narrow and isolate `pgtm` product-surface validation
- [x] Audit every current `pgtm primary points to ...` and `pgtm replicas list ...` assertion in feature files and step code.
- [x] Remove such assertions from scenarios where they merely duplicate the already-established `NodeState` authority result.
- [x] Keep a smaller explicit set of product-surface validations that prove:
  - `pgtm primary` resolves the authoritative primary when one exists
  - `pgtm replicas` resolves the expected replica set when replicas are healthy
  - switchover user-visible behavior returns the correct surface result under the same typed eligibility predicate used by failover
- [x] Where appropriate, move some of this coverage to narrower CLI/integration tests instead of repeating it inside large HA fault scenarios.

### Phase 9: Validation and cleanup
- [x] Run repo-wide searches to ensure stale concepts have actually been removed or narrowed:
  - `rg -n "unsampled_nodes|sampled|debug output|primary history never included|direct_connection_target|sql_target_for_member|emitted blocker evidence|compose logs did not contain|logs for .* contain" tests/ha`
  - keep only the concepts that are still intentionally part of the new design
- [x] Run repo-wide searches to ensure topology duplication is reduced:
  - `rg -n "(node-a|node-b|node-c|observer/node-a.toml|observer/node-b.toml|observer/node-c.toml)" tests/ha/support`
  - remaining fixed names should live in the new topology module or be justified by a given/scenario
- [x] Run `make check`
- [x] Run `make test`
- [x] Run `make lint`
- [x] Run `make test-long`
- [x] Update task status and `<passes>true</passes>` only after all acceptance criteria and implementation-plan checkboxes are complete.

DONE
