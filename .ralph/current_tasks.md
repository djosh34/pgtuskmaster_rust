# Current Tasks Summary

Generated: Wed Mar 11 04:29:33 PM CET 2026

# Task `.ralph/tasks/bugs/bug-greenfield-broken-rejoin-can-stay-offline-after-blocker-removal.md`

```
## Bug: Greenfield broken rejoin can stay offline after blocker removal <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_broken_replica_rejoin_does_not_block_healthy_quorum` now reaches a trustworthy product failure after the intended blocker choreography completes: once the broken rejoin blocker is removed and the affected node is restarted, the cluster still never returns to three online nodes.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-clone-failure-can-report-rejoined-replica-before-it-is-queryable.md`

```
## Bug: Greenfield clone failure can report rejoined replica before it is queryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
`ha_clone_failure_recovers_after_blocker_removed` currently reaches a trustworthy failure during the real `make test-long` ultra-long suite.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-concurrent-failover-can-leave-survivor-missing-acknowledged-writes.md`

```
## Bug: Greenfield concurrent failover can leave survivor missing acknowledged writes <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_stress_failover_concurrent_sql` now reaches a trustworthy data-convergence product failure under concurrent writes and primary loss.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-full-cluster-restore-times-out-under-parallel-ultra-long-suite.md`

```
## Bug: Greenfield full-cluster restore times out under parallel ultra-long suite <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-ha-proof-visibility-stalls-on-restarted-replica.md`

```
## Bug: Greenfield HA Proof Visibility Stalls On Restarted Replica <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-lagging-replica-can-still-win-failover.md`

```
## Bug: Greenfield lagging replica can still win failover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_lagging_replica_is_not_promoted` now reaches a trustworthy product failure: the degraded replica still appears in the primary history during failover.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-lone-survivor-remains-primary-after-quorum-loss.md`

```
## Bug: Greenfield lone survivor remains primary after quorum loss <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-majority-partition-can-lose-primary-without-electing-survivor.md`

```
## Bug: Greenfield majority partition can lose primary without electing survivor <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
Two advanced greenfield partition scenarios expose the same trustworthy product failure: after isolating the old primary onto the 1-side minority, the healthy 2-node majority remains observable but never elects a surviving primary.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-mixed-fault-heal-can-end-with-no-resolvable-primary.md`

```
## Bug: Greenfield mixed-fault heal can end with no resolvable primary <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_mixed_network_faults_heal_converges` exposes a distinct trustworthy post-heal recovery failure: after the intended mixed DCS plus API isolation is healed, the cluster can remain in a state where every observer seed rejects primary resolution because sampled members disagree on the leader.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-mixed-fault-heal-can-leave-primary-unqueryable.md`

```
## Bug: Greenfield mixed-fault heal can leave primary unqueryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
`ha_mixed_network_faults_heal_converges` still exposes a trustworthy HA/product failure after the harness cleanup fixes removed the stale-Docker resource noise from `make test-long`.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-mixed-network-fault-can-leave-dcs-cut-primary-authoritative.md`

```
## Bug: Greenfield mixed network fault can leave DCS-cut primary authoritative <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_mixed_network_faults_heal_converges` exposes a trustworthy mixed-fault behavior bug: cutting the current primary off from DCS while isolating a different node on observer API access can leave the original primary retaining authority instead of entering fail-safe or losing authority safely.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-no-quorum-fencing-can-miss-fail-safe-state.md`

```
## Bug: Greenfield no quorum fencing can miss fail-safe state <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_no_quorum_fencing_blocks_post_cutoff_commits` now reaches a deeper trustworthy no-quorum product failure after the operator-visible-primary symptom is avoided: at least one running node still never reports fail-safe state after DCS quorum loss.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-no-quorum-still-exposes-operator-visible-primary.md`

```
## Bug: Greenfield no quorum still exposes operator-visible primary <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
Two advanced greenfield wrappers now reach the same trustworthy no-quorum product failure: after DCS quorum majority loss, `pgtm primary` still returns an operator-visible primary instead of failing closed.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-old-primary-stays-unknown-after-planned-switchover.md`

```
## Bug: Greenfield old primary stays unknown after planned switchover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-old-primary-stays-unknown-after-targeted-switchover.md`

```
## Bug: Greenfield old primary stays unknown after targeted switchover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-replica-flap-can-finish-with-restarted-replica-still-unqueryable.md`

```
## Bug: Greenfield replica flap can finish with restarted replica still unqueryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-restarted-replica-stays-unknown-after-container-return.md`

```
## Bug: Greenfield restarted replica stays unknown after container return <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-storage-stall-does-not-trigger-primary-replacement.md`

```
## Bug: Greenfield storage stall does not trigger primary replacement <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_primary_storage_stall_replaced_by_new_primary` now reaches a trustworthy product failure: wedging the current primary does not cause the cluster to replace it with a different primary.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-targeted-switchover-accepts-isolated-ineligible-target.md`

```
## Bug: Greenfield targeted switchover accepts isolated ineligible target <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_targeted_switchover_rejects_ineligible_member` now reaches a trustworthy product failure: a targeted switchover request is accepted even when the requested replica has been fully isolated from the cluster and observer API.
```

==============

# Task `.ralph/tasks/bugs/bug-greenfield-two-node-quorum-restore-node-exits-before-primary-recovers.md`

```
## Bug: Greenfield two-node quorum restore node exits before primary recovers <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/bug-ha-rejoin-step-allows-unknown-role.md`

```
## Bug: HA rejoin assertion accepts `unknown` role as success <status>not_started</status> <passes>false</passes>

<description>
The HA cucumber assertion for `the node named "<member>" rejoins as a replica` is too weak.
```

==============

# Task `.ralph/tasks/bugs/bug-replica-bootstrap-auth-breaks-when-runtime-roles-reuse-postgres.md`

```
## Bug: Replica Bootstrap Auth Breaks When Runtime Roles Reuse `postgres` <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
```

==============

# Task `.ralph/tasks/bugs/greenfield-repeated-leadership-churn-can-stall-on-stale-leader-lease.md`

```
## Bug: Greenfield Repeated Leadership Churn Can Stall On Stale Leader Lease <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_repeated_leadership_changes_preserve_single_primary` can reach a trustworthy repeated-failover product failure where the third leader is never established because a stale leader lease blocks the remaining healthy node.
```

==============

# Task `.ralph/tasks/bugs/greenfield-rewind-fallback-scenario-never-attempts-pg-rewind.md`

```
## Bug: Greenfield Rewind Fallback Scenario Never Attempts Pg Rewind <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield HA wrapper `ha_rewind_failure_falls_back_to_basebackup` now executes to a trustworthy product outcome, but the product never attempts `pg_rewind`.
```

==============

# Task `.ralph/tasks/story-greenfield-cucumber-ha-harness/05-task-produce-ha-refactor-option-artifacts-email-review-and-stop-ralph.md`

```
## Task: Produce Post-Greenfield HA Refactor Option Artifacts, Email Review, And Stop Ralph <status>completed</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/01-task-make-local-three-node-docker-quickstart-one-command-and-file-based.md`

```
## Task: Make The Local Three-Node Docker Quickstart One Command And File-Based <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/02-task-unify-runtime-and-operator-config-into-one-per-node-with-public-endpoint-overrides.md`

```
## Task: Unify Runtime And Operator Config Into One File Per Node With Public Endpoint Overrides <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/03-task-delete-unnecessary-docker-shell-wrappers-and-shrink-the-makefile-to-gates.md`

```
## Task: Delete Unnecessary Docker Shell Wrappers And Shrink The Makefile To Gates <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/04-task-rewrite-install-and-onboarding-around-the-three-node-operator-journey.md`

```
## Task: Rewrite Install And Onboarding Around The Three-Node Operator Journey <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/05-task-remove-single-node-as-a-shipped-product-path.md`

```
## Task: Remove Single-Node As A Shipped Product Path <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-operator-ergonomics-reset/06-task-flatten-the-shipped-repo-layout-under-docker-and-delete-deep-config-nesting.md`

```
## Task: Flatten The Shipped Repo Layout Under `docker/` And Delete Deep Config Nesting <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

