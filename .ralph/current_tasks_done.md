# Done Tasks Summary

Generated: Sat Mar 14 11:38:20 CET 2026

# Task `.ralph/tasks/bugs/bug-ha-primary-storage-stalled-then-new-primary-takes-over-can-stall-with-no-authoritative-primary.md`

```
## Bug: HA storage-stall failover scenario can stall with no authoritative primary <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
`make test-long` is currently not reliably green because `ha_primary_storage_stalled_then_new_primary_takes_over` can fail waiting for a replacement primary.
```

==============

# Task `.ralph/tasks/story-config-simplification/01-task-rewrite-config-around-typed-toml-serde-and-remove-the-hand-rolled-parser.md`

```
## Task: Rewrite Config Around Typed TOML + `serde` And Remove The Hand-Rolled Parser <status>not_started</status> <passes>true</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/`</blocked_by>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/01-task-rename-the-operator-cli-to-pgtm-and-flatten-the-command-tree.md`

```
## Task: Rename The Operator CLI To `pgtm` And Flatten The Command Tree <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/02-task-add-config-backed-ctl-contexts-and-auto-auth.md`

```
## Task: Add Config-Backed `pgtm` Configuration And Automatic Auth/TLS Discovery <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/03-task-add-cluster-wide-status-topology-and-table-output.md`

```
## Task: Add Cluster-Wide `pgtm status` UX With Topology And Table Output <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/04-task-add-primary-resolution-and-shell-friendly-connection-helpers.md`

```
## Task: Add Primary Resolution And Shell-Friendly Connection Helpers To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/05-task-add-debug-reporting-and-incident-surfaces-to-ctl.md`

```
## Task: Add Debug Reporting And Incident Investigation Surfaces To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/06-task-rewrite-operator-docs-to-prefer-ctl-over-raw-curl.md`

```
## Task: Rewrite Operator Docs To Use `pgtm` Instead Of Raw `curl` <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/07-task-refactor-ha-acceptance-suite-around-node-state-invariants.md`

```
## Task: Refactor The HA Acceptance Suite Around Typed Invariants And `NodeState`-First Assertions <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md`

```
## Task: Replace The Hand-Rolled API Server With `axum` + `axum-server` + `tower` <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/09-task-add-a-three-etcd-ha-given-and-design-real-dcs-majority-features.md`

```
## Task: Add A Three-ETCD HA Given And Design Real DCS-Majority Features <status>done</status> <passes>true</passes> <priority>low</priority>

<description>
**Goal:** Add a new HA compose given that uses a real three-member `etcd` cluster instead of the current single-`etcd` shortcut, and design the HA feature families that are only valid when DCS majority semantics are real. In this new topology, each `pgtuskmaster` node must talk only to its own colocated `etcd` member, not to a shared list of all `etcd` endpoints. The observer configs for node-specific observations must mirror that same locality so that observing `node-a` means observing the DCS view that `node-a` itself has through its own `etcd`.
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`

```
## Task: Collapse DCS Behind A Single Private Component And A Read-Only `DcsView` <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/01-task-find-general-architecture-privacy-and-deduplication-improvements-and-create-follow-up-tasks.md`

```
## Task: Find General Architecture, Privacy, And Deduplication Improvements And Create Follow-Up Tasks <status>completed</status> <passes>true</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/02-task-shrink-runtime-node-rs-into-a-narrow-composition-root-and-move-startup-logic-into-owning-domains.md`

```
## Task: Shrink `runtime/node.rs` Into A Narrow Composition Root And Move Startup Logic Into Owning Domains <status>completed</status> <passes>true</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/03-task-refactor-the-ha-process-boundary-around-a-dedicated-process-intent-adapter-and-remove-secret-bearing-process-defaults-from-ha.md`

```
## Task: Refactor The HA->process Boundary Around A Dedicated Process Intent Adapter And Remove Secret-Bearing Process Defaults From HA <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/04-task-remove-public-test-harness-from-the-production-library-surface-and-move-test-support-behind-a-dev-only-boundary.md`

```
## Task: Remove Public `test_harness` From The Production Library Surface And Move Test Support Behind A Dev-Only Boundary <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/05-task-collapse-duplicated-ha-givens-into-a-typed-topology-fixture-pipeline.md`

```
## Task: Collapse Duplicated HA Givens Into A Typed Topology Fixture Pipeline <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

