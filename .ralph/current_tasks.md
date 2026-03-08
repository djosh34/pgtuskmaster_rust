# Current Tasks Summary

Generated: Sun Mar  8 05:04:29 PM CET 2026

# Task `.ralph/tasks/story-cluster-startup-friction-improvements/task-smooth-the-local-docker-cluster-startup-experience.md`

```
## Task: Smooth The Local Docker Cluster Startup Experience <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/01-task-run-k2-docs-loop-in-five-way-parallel-batches.md`

```
## Task: Run K2 Docs Loop In Five-Way Parallel Batches Until All Diataxis Sections Have Enough Pages <status>in_progress</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/02-task-write-chapter-overviews-and-landing-pages-with-ask-k2.md`

```
## Task: Write Chapter Overviews, Introductions, README, And Landing Page With Ask-K2 <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/03-task-run-truth-only-docs-verification.md`

```
## Task: Run Truth-Only Verification For Documentation Accuracy, Mermaid Diagrams, And Navigation <status>not_started</status> <passes>false</passes>

<priority>high</priority>

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

# Task `.ralph/tasks/story-secure-explicit-node-config/07-task-remove-phantom-config-versioning-and-restore-single-config-contract.md`

```
## Task: Remove phantom config versioning and restore a single as-is config contract <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Fully remove the hallucinated runtime-config versioning model from this repository. There is one config contract only. There is no `config_version` field, there never was a `v1` config, there never was a `v2` config, and no code, test, doc, fixture, or generated doc artifact may describe or enforce such a split.
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

