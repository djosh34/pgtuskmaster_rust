# Current Tasks Summary

Generated: Sat Mar 14 08:19:21 PM CET 2026

# Task `.ralph/tasks/story-logging-simplification/02-task-rewrite-logging-around-best-effort-private-traited-domain-events-and-opaque-log-sender.md`

```
## Task: Rewrite Logging Around Best-Effort Private Traited Domain Events And An Opaque LogSender <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Refactor the logging subsystem so that the only outward-facing logging API is an opaque `LogSender` with a single `send(event)` style method, where `event` is a typed enum owned by the emitting domain and accepted through a trait bound. The higher-order goal is to make logging impossible to misuse and impossible to couple to business logic: non-logging code must not know about field maps, records, severities, sinks, tracing APIs, or serialization details, and logging failures must no longer alter worker or runtime control flow except when the send operation itself cannot enqueue because the channel is broken.
```

