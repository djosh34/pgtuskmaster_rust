# Current Tasks Summary

Generated: Sat Mar 14 04:49:31 PM CET 2026

# Task `.ralph/tasks/story-dcs-simplification/01-task-rewrite-dcs-as-one-private-async-actor-with-one-public-opaque-view.md`

```
## Task: Rewrite DCS As One Private Async Actor With One Public Opaque `DcsView` <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rewrite the DCS subsystem so it has exactly one owning async loop, exactly one etcd client/session owner, zero `Arc`/`Mutex` inside the production DCS path, and exactly two public concepts for the rest of the codebase: a typed command handle and one public opaque `DcsView`. The higher-order goal is to turn DCS into a small, private coordination domain instead of a collection of storage-shaped types, bridge layers, and leaked implementation details. This is a deliberate simplification task, not a privacy-only wrapper task: the end state must remove code, remove representations, and remove synchronization primitives that only exist because the current design split ownership badly.
```

==============

# Task `.ralph/tasks/story-logging-simplification/02-task-rewrite-logging-around-best-effort-private-traited-domain-events-and-opaque-log-sender.md`

```
## Task: Rewrite Logging Around Best-Effort Private Traited Domain Events And An Opaque LogSender <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Refactor the logging subsystem so that the only outward-facing logging API is an opaque `LogSender` with a single `send(event)` style method, where `event` is a typed enum owned by the emitting domain and accepted through a trait bound. The higher-order goal is to make logging impossible to misuse and impossible to couple to business logic: non-logging code must not know about field maps, records, severities, sinks, tracing APIs, or serialization details, and logging failures must no longer alter worker or runtime control flow except when the send operation itself cannot enqueue because the channel is broken.
```

