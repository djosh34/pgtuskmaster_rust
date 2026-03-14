# Current Tasks Summary

Generated: Sat Mar 14 03:29:49 CET 2026

# Task `.ralph/tasks/story-cert-reload-postgres-sighup/01-task-send-postgres-sighup-after-certificate-reload.md`

```
## Task: Send PostgreSQL `SIGHUP` After Certificate Reload <status>not_started</status> <passes>false</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-config-simplification/01-task-rewrite-config-around-typed-toml-serde-and-remove-the-hand-rolled-parser.md`

```
## Task: Rewrite Config Around Typed TOML + `serde` And Remove The Hand-Rolled Parser <status>not_started</status> <passes>false</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/`</blocked_by>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/09-task-add-a-three-etcd-ha-given-and-design-real-dcs-majority-features.md`

```
## Task: Add A Three-ETCD HA Given And Design Real DCS-Majority Features <status>not_started</status> <passes>false</passes> <priority>low</priority>

<description>
**Goal:** Add a new HA compose given that uses a real three-member `etcd` cluster instead of the current single-`etcd` shortcut, and design the HA feature families that are only valid when DCS majority semantics are real. In this new topology, each `pgtuskmaster` node must talk only to its own colocated `etcd` member, not to a shared list of all `etcd` endpoints. The observer configs for node-specific observations must mirror that same locality so that observing `node-a` means observing the DCS view that `node-a` itself has through its own `etcd`.
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/01-task-find-general-architecture-privacy-and-deduplication-improvements-and-create-follow-up-tasks.md`

```
## Task: Find General Architecture, Privacy, And Deduplication Improvements And Create Follow-Up Tasks <status>not_started</status> <passes>false</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/02-task-shrink-runtime-node-rs-into-a-narrow-composition-root-and-move-startup-logic-into-owning-domains.md`

```
## Task: Shrink `runtime/node.rs` Into A Narrow Composition Root And Move Startup Logic Into Owning Domains <status>not_started</status> <passes>false</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-logging-simplification/01-task-rewrite-logging-around-private-typed-events-and-json-postgres-defaults.md`

```
## Task: Rewrite Logging Around Private Typed Events And JSON Postgres Defaults <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-postgres-role-reconciliation/01-task-make-role-management-config-driven-idempotent-and-reconciling.md`

```
## Task: Make PostgreSQL Role Management Config-Driven, Idempotent, And Reconciling <status>not_started</status> <passes>false</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-config-simplification/`</blocked_by>
```

