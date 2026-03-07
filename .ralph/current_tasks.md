# Current Tasks Summary

Generated: Sat Mar  7 08:35:42 PM CET 2026

# Task `.ralph/tasks/bugs/typed-network-endpoints-instead-of-raw-strings.md`

```
## Bug: Type network endpoints instead of carrying raw strings across runtime <status>not_started</status> <passes>false</passes>

<description>
The codebase carries API and DCS endpoint addresses as raw `String` values deep into runtime and harness paths, then parses or binds them at scattered call sites. This was detected during a representation-integrity scan looking for cases where subsystem boundaries retain ad-hoc primitive encodings instead of canonical typed models.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`

```
DO NOT PICK THIS TASK UNLESS ALL OTHER TASKS ARE DONE.
## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes> <priority>very_low</priority>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/task-remove-managed-conf-parseback-and-rederive-start-intent.md`

```
## Task: Remove Managed Conf Parse-Back And Re-Derive Start Intent <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove the current pattern where pgtuskmaster reparses its own managed PostgreSQL startup artifacts from `PGDATA` back into typed startup intent. Replace it with a stricter architecture where typed Rust models are the only authoritative internal model, startup intent is re-derived from DCS plus runtime config plus minimal local physical facts, and managed PostgreSQL files are treated as render outputs only.
```

