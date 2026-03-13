# Current Tasks Summary

Generated: Fri Mar 13 03:40:56 PM CET 2026

# Task `.ralph/tasks/bugs/ha-compose-should-self-bootstrap-with-plain-docker-compose-up.md`

```
## Bug: HA compose should self-bootstrap with plain docker compose up <status>not_started</status> <passes>false</passes>

<description>
The HA docker assets under `tests/ha/givens/three_node_plain/compose.yml` do not currently behave like a self-contained docker-compose environment. A plain `docker compose up` for all services caused the three node containers to start before the seed-primary bootstrap sequence had been established, and each node exited early with DCS startup errors. The stack only became usable when it was started in the same staged order as the Rust HA harness:
- start `etcd`
```

==============

# Task `.ralph/tasks/bugs/ha-total-outage-recovery-can-stall-or-diverge-after-final-rejoin.md`

```
## Bug: HA total-outage recovery can stall leader election or briefly diverge on leader after final rejoin <status>not_started</status> <passes>false</passes>

<description>
The ultra-long HA feature `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins` has at least two real failure modes after a full cluster outage.
```

==============

# Task `.ralph/tasks/bugs/lone-survivor-can-regain-full-quorum-and-stay-writable-after-peers-age-out.md`

```
## Bug: Lone survivor can regain full quorum and stay writable after peers age out <status>not_started</status> <passes>false</passes>

<description>
The HA system currently allows a single surviving node from a multi-node cluster to degrade into a one-member `FullQuorum` cluster after the stopped peers age out of DCS membership.
```

==============

# Task `.ralph/tasks/bugs/restarted-former-primary-rejoins-via-basebackup-instead-of-streaming-or-pg-rewind.md`

```
## Bug: Restarted former primary rejoins via basebackup instead of streaming or pg_rewind <status>not_started</status> <passes>false</passes>

<description>
During a live manual HA exercise against `tests/ha/givens/three_node_plain/compose.yml`, the old primary was killed, a new primary was elected, and then the old primary container was restarted with its volume still present. The restarted node rejoined as a replica, but `pgtm status` showed it going through `follower(Basebackup)` before eventually reaching `follower(StartStreaming)`.
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

