# Current Tasks Summary

Generated: Sun Mar  8 02:56:37 PM CET 2026

# Task `.ralph/tasks/story-docs-k2-expansion/task-run-k2-docs-loop-in-five-way-parallel-batches.md`

```
## Task: Run K2 Docs Loop In Five-Way Parallel Batches Until All Diataxis Sections Have Enough Pages <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Create a docs-only execution task that follows the `k2-docs-loop` skill precisely, but accelerates throughput by running five fully independent page-selection and draft-preparation lanes in parallel. The higher-order goal is to expand the real docs corpus quickly without inventing structure by hand and without collapsing independent K2 choices into one blended workflow.
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/task-run-truth-only-docs-verification.md`

```
## Task: Run Truth-Only Verification For Documentation Accuracy, Mermaid Diagrams, And Navigation <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Perform a final docs verification pass that checks only factual truth against the repository, mermaid-diagram correctness and clarity, and navigation/linking quality. The higher-order goal is to keep the final docs set honest and usable without injecting subjective style edits.
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/task-write-chapter-overviews-and-landing-pages-with-ask-k2.md`

```
## Task: Write Chapter Overviews, Introductions, README, And Landing Page With Ask-K2 <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Use the `ask-k2` skill, in the same raw-prompt style used by the `k2-docs-loop` workflow, to author the docs navigation and orientation pages that tie the generated content together. The higher-order goal is to turn a set of isolated pages into a coherent documentation experience with chapter entry points, a clear first landing page, and a repository-facing README.
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

