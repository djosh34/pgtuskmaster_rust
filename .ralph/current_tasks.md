# Current Tasks Summary

Generated: Sun Mar 15 09:33:35 PM CET 2026

# Task `.ralph/tasks/story-logging-simplification/03-task-rewrite-logging-around-one-owned-logdto-global-context-and-an-exhaustive-event-set.md`

```
## Task: Rewrite Logging Around One Owned LogDto, Logger-Owned Global Context, And An Exhaustive Event Set <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace the current logging trait-and-visitor design with a much simpler boundary: each domain owns typed log event enums, each event converts itself in one step into one owned logging DTO, and the logger itself injects global node context such as hostname, cluster name, scope, and member id. The higher-order goal is to make logging compiler-driven and minimal: no emitter should know about logger-global context, no emitter should know how fields are encoded, no generic field visitor should exist, no logging trait should be shaped by process-specific concepts such as `job_id`, and no log-event types should exist outside the exhaustive set defined in this task.
```

