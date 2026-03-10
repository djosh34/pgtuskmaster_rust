# Current Tasks Summary

Generated: Tue Mar 10 01:26:47 AM CET 2026

# Task `.ralph/tasks/bugs/ha-authoritative-startup-redesign-still-has-legacy-phase-machine-and-incomplete-offline-election.md`

```
## Bug: HA authoritative startup redesign still has legacy phase machine and incomplete offline election <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
The task `.ralph/tasks/story-managed-start-intent-architecture/task-redesign-ha-startup-bootstrap-and-rejoin-around-authoritative-dcs-reconciliation.md`
is marked completed, but source audit shows major required redesign pieces are still not implemented.
```

==============

# Task `.ralph/tasks/bugs/ha-replica-must-not-follow-non-authoritative-primary.md`

```
## Bug: HA replica must not follow non-authoritative primary <status>not_started</status> <passes>false</passes>

<description>
In the HA decision and startup-rejoin paths, source selection falls back from the authoritative
leader lease to any healthy primary member record in DCS. This lets a stale former primary, or a
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/02-task-add-whole-node-kill-and-partial-recovery-ha-e2e.md`

```
## Task: Add Whole Node Kill And Partial Recovery HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/03-task-add-full-1-to-2-network-partition-quorum-survival-ha-e2e.md`

```
## Task: Add Full 1 To 2 Network Partition Quorum Survival HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/04-task-add-primary-storage-stall-and-wal-full-failover-e2e.md`

```
## Task: Add Primary Storage Stall And WAL Full Failover E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/05-task-add-broken-returning-node-and-single-good-recovery-ha-e2e.md`

```
## Task: Add Broken Returning Node And Single Good Recovery HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/06-task-add-full-failsafe-recovery-when-quorum-returns-ha-e2e.md`

```
## Task: Add Full FailSafe Recovery When Quorum Returns HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/07-task-add-old-primary-returns-as-replica-only-after-majority-failover-e2e.md`

```
## Task: Add Old Primary Returns As Replica Only After Majority Failover E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/08-task-add-lagging-or-stale-replica-is-never-promoted-over-healthier-candidate-e2e.md`

```
## Task: Add Lagging Or Stale Replica Is Never Promoted Over Healthier Candidate E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/09-task-add-node-flapping-with-healthy-majority-does-not-cause-leadership-thrash-e2e.md`

```
## Task: Add Node Flapping With Healthy Majority Does Not Cause Leadership Thrash E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/10-task-add-minority-old-primary-returns-with-stale-view-and-is-forced-to-rejoin-safely-e2e.md`

```
## Task: Add Minority Old Primary Returns With Stale View And Is Forced To Rejoin Safely E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-quorum-survival-under-real-failures/11-task-add-broken-replica-rejoin-does-not-block-healthy-quorum-availability-e2e.md`

```
## Task: Add Broken Replica Rejoin Does Not Block Healthy Quorum Availability E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-managed-start-intent-architecture/task-redesign-ha-startup-bootstrap-and-rejoin-around-authoritative-dcs-reconciliation.md`

```
## Task: Redesign HA Startup Bootstrap And Rejoin Around Authoritative DCS Reconciliation <status>retry-tests</status> <passes>false</passes> <priority>ultra high</priority>


<description>
**Goal:** Replace the current split startup/rejoin/follow-leader architecture with one authoritative reconciliation model that derives node behavior from DCS authority plus local physical facts, rather than from mixed local heuristics and phase-specific patches. The higher-order goal is to guarantee that ephemeral node restarts, cold restarts, preserved-PGDATA rejoins, and leader-loss reactions all converge through the same control rules and therefore produce the same safe behavior.
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

