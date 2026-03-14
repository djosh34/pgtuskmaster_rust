## Task: Add A Three-ETCD HA Given And Design Real DCS-Majority Features <status>done</status> <passes>true</passes> <priority>low</priority>

<description>
**Goal:** Add a new HA compose given that uses a real three-member `etcd` cluster instead of the current single-`etcd` shortcut, and design the HA feature families that are only valid when DCS majority semantics are real. In this new topology, each `pgtuskmaster` node must talk only to its own colocated `etcd` member, not to a shared list of all `etcd` endpoints. The observer configs for node-specific observations must mirror that same locality so that observing `node-a` means observing the DCS view that `node-a` itself has through its own `etcd`.

**Higher-order goal:** The current `three_node_plain` given is structurally incapable of proving true DCS-majority behavior because it has only one `etcd` container, and all three `pgtuskmaster` nodes point at that same always-happy endpoint. That makes some current and future HA expectations incoherent: a lone survivor can still look like it has full DCS trust, minority behavior cannot be tested honestly, and "stop DCS quorum majority" is not modeling a real quorum loss. This task adds the missing topology so the HA suite can distinguish:
- data-plane minority vs majority
- DCS majority vs DCS minority
- loss of one node vs loss of one node's local DCS
- temporary quorum loss vs total DCS eradication

**Decisions already made from user discussion:**
- The new given is not "three nodes all connect to all three etcd endpoints". Each `pgtuskmaster` node must connect only to its own `etcd` member.
- The purpose of this topology is specifically to enable scenario assertions that are impossible or misleading in the current single-`etcd` harness.
- The resulting feature design must be informed by task 07's `NodeState`-first, invariant-first approach. Do not design new three-`etcd` features around the old fallback-heavy step model.
- Do not mechanically duplicate the whole HA suite under a new given. Only design and add scenario families whose truth actually depends on real DCS majority semantics.
- This task must not stop at "add topology and maybe design some scenarios later". It must name the concrete three-`etcd` feature files or scenario families now and state exactly what they assert.
- Follow the current repo style for feature layout: one scenario per feature, with the feature directory and `.feature` filename matching each other under the existing flat `tests/ha/features/` tree.
- Follow the current repo style for naming: do not give abstract category titles like `minority_by_partition.feature`. Each feature name must describe the fault and the expected result in the same style as existing files such as `ha_old_primary_partitioned_from_majority_majority_elects_new_primary`.
- "Never becomes primary on minority" must be covered in both major classes:
  - minority created by nodes going down
  - minority created by network partition
- "Removed DCS still works" must be treated as a first-class three-`etcd` scenario family, with the exact contract designed explicitly instead of left vague.

**Scope:**
- Add a new HA given, for example `tests/ha/givens/three_node_three_etcd/`, with:
  - three `etcd` services such as `etcd-a`, `etcd-b`, and `etcd-c`
  - one data volume per `etcd` member
  - a real three-member `etcd` peer cluster with correct `--initial-cluster`, peer URLs, client URLs, and health checks
  - `node-a` runtime config pointing only to `etcd-a`
  - `node-b` runtime config pointing only to `etcd-b`
  - `node-c` runtime config pointing only to `etcd-c`
  - observer config for `node-a` pointing only to `etcd-a`
  - observer config for `node-b` pointing only to `etcd-b`
  - observer config for `node-c` pointing only to `etcd-c`
- Generalize HA harness support code so the given can describe multiple DCS services instead of one hardcoded `etcd` service.
- Design the feature/scenario inventory that belongs only to the real-three-`etcd` setup and record it concretely in this task and/or the resulting feature files.
- Ensure the design aligns with task 07's invariant model:
  - `NodeState` is the truth source for authority / role / quorum / fail-safe assertions
  - SQL remains the truth source for writability / proof-row / fencing assertions
  - no fallback from `Unknown` to SQL to reinterpret cluster role

**Out of scope:**
- Do not replace the existing `three_node_plain` given; it still has value for faster or different scenario classes.
- Do not clone all existing feature files onto the new given.
- Do not add a three-`etcd` custom-roles variant in this task unless a concrete three-`etcd`-only scenario really requires it.
- Do not preserve old single-`etcd` expectations in the new topology if they are incoherent under real DCS majority semantics.

**Context from research:**
- The current plain given is `tests/ha/givens/three_node_plain/compose.yml`, and it defines exactly one `etcd` service named `etcd`.
- Current runtime configs under:
  - `tests/ha/givens/three_node_plain/configs/node-a/runtime.toml`
  - `tests/ha/givens/three_node_plain/configs/node-b/runtime.toml`
  - `tests/ha/givens/three_node_plain/configs/node-c/runtime.toml`
  all point to the same endpoint: `endpoints = ["http://etcd:2379"]`.
- Current observer configs under:
  - `tests/ha/givens/three_node_plain/configs/observer/node-a.toml`
  - `tests/ha/givens/three_node_plain/configs/observer/node-b.toml`
  - `tests/ha/givens/three_node_plain/configs/observer/node-c.toml`
  also point to that same single endpoint.
- Current support code still hardcodes the single-service DCS assumption in several places:
  - `tests/ha/support/faults/mod.rs` has a singular `ETCD_SERVICE`
  - `tests/ha/support/steps/mod.rs` maps DCS stop/start steps to service `"etcd"`
  - `tests/ha/support/world/mod.rs` boots only the `"etcd"` service first, waits for `"etcd"` health, and captures artifacts with a fixed `"etcd"` service list
- Task 07 already established the future assertion model: new HA features should be phrased as explicit invariants, use semantic aliases, and rely on `NodeState` rather than mixed truth surfaces.

**Three-ETCD-specific scenario families that must be designed here:**

1. Minority never becomes primary because nodes are down
- Scenario: after healthy startup, two nodes are down and the lone remaining node is in the minority from both DCS and data-plane perspective; it must not remain or become authoritative primary.
- Scenario: after healthy startup, the former primary is stranded with only its own local `etcd` member while the other two nodes and their two `etcd` members still constitute the majority; the minority-side old primary must lose authority, and the majority must converge to one primary.
- Scenario: a minority-side replica left with only its own local `etcd` must never self-promote while the majority side preserves or elects the only authoritative primary.

2. Minority never becomes primary because networks are partitioned
- Scenario: partition the old primary away from the other two nodes so it also loses effective DCS majority through its own local `etcd`; the majority side must elect or preserve exactly one primary, and the minority old primary must never remain authoritative.
- Scenario: partition a replica into the minority with only its own local `etcd`; that replica must never become authoritative primary.
- Scenario: heal the partition and prove the minority-side former leader or replica rejoins according to the typed recovery/role contract rather than regaining authority unexpectedly.

3. Local-ETCD loss scenarios that are only meaningful when each node uses its own ETCD
- Scenario: the current primary loses only its colocated `etcd` member while Postgres/API reachability to the rest of the cluster is otherwise intact; because the primary no longer has a valid DCS path, it must lose authority safely and the majority-side healthy replica set must converge to one primary.
- Scenario: a replica loses only its colocated `etcd` member; it must never become primary solely because the shared data plane is still up.
- Scenario: loss of one non-primary `etcd` member must not destabilize a healthy majority if the remaining two `etcd` members and their attached `pgtuskmaster` nodes still satisfy the contract.

4. Total DCS removal / eradication
- Design and encode the exact contract for "removed DCS still works", where all `etcd` containers are killed and removed and their volumes are deleted.
- This scenario family must not stay vague. The task must define:
  - what "still works" means at the product surface
  - whether the existing primary remains readable, writable, or only locally running
  - whether operator-visible authoritative primary should remain, disappear, or enter explicit fail-safe / degraded state
  - whether any new failover or switchover is allowed after DCS eradication
  - what happens when `etcd` is later recreated from empty volumes
- The resulting assertions must be based on typed state and SQL behavior, not log text.

5. Additional scenario families to include if the design remains coherent
- One-`etcd`-member loss with retained DCS quorum should not be treated the same as DCS quorum loss. The suite should have an explicit scenario proving that losing one `etcd` member does not by itself destroy cluster authority when the other two `etcd` members remain healthy.
- Two-`etcd`-member loss should be tested distinctly from "all DCS removed" so the suite proves the difference between ordinary quorum loss and full DCS eradication.
- If switchover behavior changes meaningfully when the target or current primary loses its own colocated `etcd`, add a three-`etcd` switchover scenario only if it proves a distinct invariant not already covered by task 07's general switchover family.

**Concrete target feature inventory and exact assertion contract:**

1. `ha_lone_old_primary_on_three_etcd_minority_loses_authority`
  - Setup: label `old_primary`, record a transition window, then stop the other two nodes so only `old_primary` and its own local `etcd` member remain.
  - `NodeState` assertions:
    - eventually no authoritative primary exists across 1 reachable member
    - the lone reachable node is not a writable primary
    - `old_primary` does not remain authoritative after the minority window starts
  - SQL assertions:
    - proof writes through `old_primary` are rejected after the fencing cutoff
    - pre-existing proof rows remain readable

2. `ha_lone_replica_on_three_etcd_minority_does_not_become_primary`
  - Setup: label `old_primary` and a replica `minority_replica`, stop `old_primary` and the other replica so only `minority_replica` and its own local `etcd` member remain.
  - `NodeState` assertions:
    - eventually no authoritative primary exists across 1 reachable member
    - `minority_replica` never becomes primary during the transition window
  - SQL assertions:
    - proof writes through `minority_replica` are rejected
    - no split-brain write evidence appears

3. `ha_old_primary_partitioned_on_three_etcd_loses_authority_and_majority_elects_new_primary`
  - Setup: label `old_primary`, isolate it from the other two nodes so the majority side keeps two nodes and two `etcd` members.
  - `NodeState` assertions:
    - eventually exactly one primary exists across 2 reachable majority members as `new_primary`
    - `old_primary` never becomes authoritative primary during the minority window
    - always no dual primary occurs during the transition window
  - SQL assertions:
    - proof writes through `new_primary` succeed
    - proof writes through `old_primary` are rejected once it is minority-side
    - after heal, `old_primary` eventually rejoins as a replica

4. `ha_replica_partitioned_on_three_etcd_does_not_become_primary_and_majority_has_single_primary`
  - Setup: isolate one replica so it is minority-side with only its own local `etcd`.
  - `NodeState` assertions:
    - the healthy majority preserves or elects exactly one primary
    - the isolated replica never becomes authoritative primary
  - SQL assertions:
    - majority-side proof writes succeed
    - the isolated replica does not contain the new proof token until healing
    - after heal, the isolated replica catches up

5. `ha_primary_loses_local_etcd_on_three_etcd_loses_authority_and_majority_elects_new_primary`
  - Setup: kill or remove only the current primary's colocated `etcd` member while keeping normal node-to-node network connectivity.
  - `NodeState` assertions:
    - the former primary eventually loses authority safely
    - the remaining two-node / two-`etcd` majority converges to exactly one primary
    - the former primary never regains authority during that window
  - SQL assertions:
    - writes through the former primary are rejected after authority loss
    - writes through the new authoritative primary succeed

6. `ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary`
  - Setup: kill or remove only a replica's colocated `etcd` member.
  - `NodeState` assertions:
    - the current primary remains the only authoritative primary
    - the affected replica never becomes primary
  - SQL assertions:
    - writes through the primary continue to succeed
    - after restoring the replica's local `etcd`, the replica eventually catches up

7. `ha_all_etcd_removed_on_three_etcd_enters_safe_degraded_mode`
  - Setup: kill and remove all three `etcd` containers and delete all three `etcd` data volumes while leaving the `pgtuskmaster` nodes running.
  - Exact contract for "still works":
    - the system stays process-alive and queryable enough to expose current state and existing committed data
    - it does not continue exposing an authoritative writable primary without DCS
    - it does not allow new failover or switchover authority changes while DCS is absent
  - `NodeState` assertions:
    - eventually no authoritative primary exists across the reachable nodes
    - every reachable node reports fail-safe or an equivalent explicit no-authority / DCS-degraded state
    - no targeted or planned switchover request is accepted while DCS is eradicated
  - SQL assertions:
    - pre-eradication proof rows remain readable
    - a fencing cutoff is established and no later writes commit

8. `ha_three_etcd_recreated_from_empty_volumes_recovers_single_primary`
  - Setup: after the eradication state above, recreate the three-member `etcd` cluster from empty volumes and let the nodes reconnect.
  - `NodeState` assertions:
    - the cluster eventually converges back to exactly one authoritative primary
    - always no dual primary occurs during the recovery window
    - all non-primary nodes eventually report replica state
  - SQL assertions:
    - proof rows committed before eradication remain present
    - new proof writes succeed only after the cluster has regained authoritative primary state

9. Optional three-`etcd`-only extension if behavior is distinct enough:
- `ha_two_etcd_lost_on_three_etcd_enters_safe_degraded_mode`
  - If losing two `etcd` members has a distinct contract worth proving separately from full eradication, keep it as its own feature instead of burying it inside the all-`etcd`-removed case.
  - Assertions must prove the distinction in `NodeState` terms and in SQL writability terms.

**Required design output from this task:**
- A concrete new given layout with service names, config wiring, and harness support for multi-member DCS.
- A concrete feature inventory for the three-`etcd` topology, with final-style feature names, one scenario per feature, and explicit `NodeState` plus SQL assertions rather than a vague "maybe add some minority tests later".
- Explicit statements of which scenarios belong only to the three-`etcd` topology and why they are not honest under the single-`etcd` topology.
- Explicit statements of the authority/writability contract for total DCS eradication.

**Expected outcome:**
- The repo has a reusable three-`etcd` HA given that models real DCS majority.
- The harness can address multiple DCS services cleanly instead of assuming a single service named `etcd`.
- The HA workstream has a clear, self-contained inventory of three-`etcd`-only scenario families.
- Minority assertions that were previously impossible or misleading in `three_node_plain` now have a correct topology to run against.
- Future task-07-style feature rewrites can place the right scenarios on the right given instead of encoding DCS-majority expectations onto a one-`etcd` harness.

</description>

<acceptance_criteria>
- [ ] Add a new given directory under `tests/ha/givens/` for the three-`etcd` topology, with a compose file, node runtime configs, observer configs, TLS material references, and secrets wiring parallel to `three_node_plain`
- [ ] The new compose file defines three distinct `etcd` services with a real three-member peer cluster and one data volume per `etcd` member
- [ ] `node-a`, `node-b`, and `node-c` runtime configs each point only to their own local `etcd` endpoint; no runtime config in the new given lists all three `etcd` endpoints
- [ ] Observer configs in the new given mirror the same node-to-local-`etcd` mapping so observing a node means observing its own DCS view
- [ ] `tests/ha/support/faults/mod.rs`, `tests/ha/support/world/mod.rs`, and `tests/ha/support/steps/mod.rs` are updated so DCS services are no longer hardcoded as one singular `"etcd"` service
- [ ] The harness can start, health-check, stop, start, and collect artifacts for the new multi-`etcd` given without assuming a single DCS container
- [ ] The three-`etcd` feature design follows the current repo layout: one scenario per feature, one directory per feature, under the existing flat `tests/ha/features/` tree
- [ ] The three-`etcd` feature names follow the current repo style by describing the fault and the expected outcome, not abstract grouping titles
- [ ] This task records a concrete three-`etcd` feature inventory that includes, at minimum:
- [ ] minority-never-primary with nodes down
- [ ] minority-never-primary with network partitions
- [ ] local-`etcd` loss scenarios enabled by the one-node-to-one-`etcd` mapping
- [ ] an explicit total-DCS-eradication scenario family with a fully specified expected contract
- [ ] The concrete three-`etcd` feature inventory is the merged scenario list in this task, instead of splitting one fault timeline into separate minority and majority feature files
- [ ] For each three-`etcd` feature family, this task or the resulting feature files explicitly state the `NodeState` assertions and the SQL assertions instead of naming only the high-level theme
- [ ] The task or resulting feature files explicitly explain why those scenarios belong on the three-`etcd` topology and are not honest on `three_node_plain`
- [ ] The designed assertions are consistent with task 07's `NodeState`-first model and do not reintroduce SQL fallback for cluster role
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<execution_plan>
- [x] Replace the singular DCS-service typing with explicit DCS-member identity in the HA topology model so `etcd-a` / `etcd-b` / `etcd-c` can be addressed as typed services instead of a hidden `"etcd"` singleton.
- [x] Lift DCS layout into `ThreeNodeTopologyFixture` and make runtime/observer template descriptors carry typed member-to-DCS bindings rather than inheriting `http://etcd:2379` implicitly.
- [x] Update compose rendering, bootstrap ordering, DCS fault orchestration, and artifact capture to consume the new topology ADTs end to end.
- [x] Materialize the `three_node_three_etcd` given with three-member etcd clustering and node-local runtime/observer DCS wiring.
- [x] Add the three-etcd-only feature files, run `make check`, `make lint`, `make test`, `make test-long`, then update docs with `k2-docs-loop`.
NOW EXECUTE
</execution_plan>
