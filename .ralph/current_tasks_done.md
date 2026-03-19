# Done Tasks Summary

Generated: Thu Mar 19 03:50:43 AM CET 2026

# Task `.ralph/tasks/bugs/bug-dcs-must-never-report-stale-cluster-data-outside-authoritative-quorum.md`

```
## Bug: DCS must never report stale cluster data outside authoritative quorum <status>done</status> <passes>true</passes>

<description>
The intended product invariant is stricter than the behavior that was reintroduced during `.ralph/tasks/story-general-architecture-improvement-finding/07-task-collapse-duplicate-struct-trees-into-canonical-domain-adts-and-prove-the-struct-count-went-down.md`: DCS has only two meaningful states, and stale or reused cluster data is never allowed outside authoritative quorum.
```

==============

# Task `.ralph/tasks/bugs/bug-ha-primary-storage-stalled-then-new-primary-takes-over-can-stall-with-no-authoritative-primary.md`

```
## Bug: HA storage-stall failover scenario can stall with no authoritative primary <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
`make test-long` is currently not reliably green because `ha_primary_storage_stalled_then_new_primary_takes_over` can fail waiting for a replacement primary.
```

==============

# Task `.ralph/tasks/bugs/bug-ha-stable-primary-gate-allows-non-writable-primary.md`

```
## Bug: HA stable-primary gate allows non-writable primary <status>completed</status> <passes>true</passes>

<blocked_by>Full completion of `.ralph/tasks/story-general-architecture-improvement-finding/06-task-move-ha-scenario-execution-into-a-per-scenario-runner-container-and-remove-docker-daemon-polling.md`</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/bugs/bug-remove-primary-and-replica-api-commands-and-derive-cli-output-from-state.md`

```
## Bug: Remove Primary And Replica API Commands And Derive CLI Output From State <status>done</status> <passes>true</passes>

<description>
`pgtm primary` / `pgtm replicas` can disagree with the cluster `state` view during HA scenarios. This was detected while restoring the older HA harness: ultra-long HA tests exposed cases where the authoritative state view showed one primary expectation while the primary-resolution command returned something different or no authoritative primary at all.
```

==============

# Task `.ralph/tasks/story-cert-reload-postgres-sighup/01-task-send-postgres-sighup-after-certificate-reload.md`

```
## Task: Send PostgreSQL `SIGHUP` After Certificate Reload <status>done</status> <passes>true</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md`</blocked_by>
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

# Task `.ralph/tasks/story-dcs-simplification/01-task-rewrite-dcs-as-one-private-async-actor-with-one-public-opaque-view.md`

```
## Task: Rewrite DCS As One Private Async Actor With One Public `DcsView` <status>completed</status> <passes>true</passes>

<description>
**Goal:** Rewrite the DCS subsystem so it has exactly one owning async loop, exactly one etcd client/session owner, zero `Arc`/`Mutex` inside the production DCS path, one public read-only serde `DcsView`, and one crate-private typed command handle (`DcsHandle`) for mutation. The higher-order goal is to turn DCS into a small, private coordination domain instead of a collection of storage-shaped types, bridge layers, and leaked implementation details. This is a deliberate simplification task, not a privacy-only wrapper task: the end state must remove code, remove representations, and remove synchronization primitives that only exist because the current design split ownership badly.
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

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/06-task-move-ha-scenario-execution-into-a-per-scenario-runner-container-and-remove-docker-daemon-polling.md`

```
## Task: Move HA Scenario Execution Into A Per-Scenario Runner Container And Remove Docker-Daemon Polling <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/07-task-collapse-duplicate-struct-trees-into-canonical-domain-adts-and-prove-the-struct-count-went-down.md`

```
## Task: Collapse Duplicate Struct Trees Into Canonical Domain ADTs And Prove The Struct Count Went Down <status>not_started</status> <passes>true</passes>

<priority>high</priority>
<blocked_by>Full completion of `.ralph/tasks/story-dcs-simplification/02-task-fully-rewrite-etcd-into-a-much-simpler-model-with-derive-support.md`</blocked_by>
```

==============

# Task `.ralph/tasks/story-general-architecture-improvement-finding/08-task-split-process-planning-from-managed-postgres-session-materialization-and-external-tool-execution.md`

```
## Task: Split Process Planning From Managed Postgres Session Materialization And External Tool Execution <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/01-task-delete-implementation-biased-feature-assertions-and-shrink-the-ha-feature-inventory.md`

```
## Task: Delete Implementation-Biased Feature Assertions And Shrink The HA Feature Inventory <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/02-task-delete-scenario-bookkeeping-and-shrink-the-ha-world-and-step-surface.md`

```
## Task: Delete Scenario Bookkeeping And Shrink The HA World And Step Surface <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/03-task-delete-observer-compose-rendering-and-collapse-ha-fixtures-to-two-static-compose-variants.md`

```
## Task: Delete Observer Compose Rendering And Collapse HA Fixtures To Two Static Compose Variants <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/04-task-rebuild-the-ha-observation-surface-around-host-side-pgtm-json-and-minimal-outcomes.md`

```
## Task: Rebuild The HA Observation Surface Around Host-Side `pgtm` JSON And Minimal Outcomes <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/05-task-add-a-perpetual-self-reported-primary-count-invariant-runner.md`

```
## Task: Add A Perpetual Self-Reported Primary-Count Invariant Runner <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ha-acceptance-deletion-first-rewrite/06-task-add-a-perpetual-accepted-write-convergence-invariant-runner.md`

```
## Task: Add A Perpetual Accepted-Write Convergence Invariant Runner <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-logging-simplification/01-task-rewrite-logging-around-private-typed-events-and-json-postgres-defaults.md`

```
## Task: Rewrite Logging Around Private Typed Events And JSON Postgres Defaults <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-logging-simplification/02-task-rewrite-logging-around-best-effort-private-traited-domain-events-and-opaque-log-sender.md`

```
## Task: Rewrite Logging Around Best-Effort Private Traited Domain Events And An Opaque LogSender <status>completed</status> <passes>true</passes>

<description>
**Goal:** Refactor the logging subsystem so that the only outward-facing logging API is an opaque `LogSender` with a single `send(event)` style method, where `event` is a typed enum owned by the emitting domain and accepted through a trait bound. The higher-order goal is to make logging impossible to misuse and impossible to couple to business logic: non-logging code must not know about field maps, records, severities, sinks, tracing APIs, or serialization details, and logging failures must no longer alter worker or runtime control flow except when the send operation itself cannot enqueue because the channel is broken.
```

==============

# Task `.ralph/tasks/story-logging-simplification/03-task-rewrite-logging-around-one-owned-logdto-global-context-and-an-exhaustive-event-set.md`

```
## Task: Rewrite Logging Around A Custom `LoggableEvent` Derive, Logger-Owned Global Context, Flat OTel-Friendly Attributes, And An Exhaustive Event Set <status>completed</status> <passes>true</passes>

<description>
**Goal:** Replace the current logging trait-and-visitor design with a much simpler and stricter model:
- events become loggable through a custom derive and `#[]` tags
```

==============

# Task `.ralph/tasks/story-postgres-role-reconciliation/01-task-make-role-management-config-driven-idempotent-and-reconciling.md`

```
## Task: Make PostgreSQL Role Management Config-Driven, Idempotent, And Reconciling <status>completed</status> <passes>true</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-config-simplification/`</blocked_by>
```

