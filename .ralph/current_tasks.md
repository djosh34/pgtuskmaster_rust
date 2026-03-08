# Current Tasks Summary

Generated: Sun Mar  8 06:14:25 PM CET 2026

# Task `.ralph/tasks/story-cluster-startup-friction-improvements/task-smooth-the-local-docker-cluster-startup-experience.md`

```
## Task: Smooth The Local Docker Cluster Startup Experience <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-managed-start-intent-architecture/task-remove-managed-conf-parseback-and-rederive-start-intent.md`

```
## Task: Remove Managed Conf Parse-Back And Re-Derive Start Intent <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove the current pattern where pgtuskmaster reparses its own managed PostgreSQL startup artifacts from `PGDATA` back into typed startup intent. Replace it with a stricter architecture where typed Rust models are the only authoritative internal model, startup intent is re-derived from DCS plus runtime config plus minimal local physical facts, and managed PostgreSQL files are treated as render outputs only.
```

==============

# Task `.ralph/tasks/story-managed-start-intent-architecture/typed-network-endpoints-instead-of-raw-strings.md`

```
## Task: [Improvement] Type network endpoints instead of carrying raw strings across runtime <status>not_started</status> <passes>false</passes>

<description>
The codebase carries API and DCS endpoint addresses as raw `String` values deep into runtime and harness paths, then parses or binds them at scattered call sites. This was detected during a representation-integrity scan looking for cases where subsystem boundaries retain ad-hoc primitive encodings instead of canonical typed models.
```

==============

# Task `.ralph/tasks/story-switchover-operator-model/task-redesign-switchover-request-semantics-for-operators.md`

```
## Task: Redesign Switchover Request Semantics For Operators <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-switchover-operator-model/task-remove-requested-by-and-add-optional-switchover-to.md`

```
## Task: Remove `requested_by` And Add Optional `switchover_to` <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

