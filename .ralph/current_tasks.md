# Current Tasks Summary

Generated: Sat Mar  7 23:54:14 CET 2026

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`

```
## Task: Run How-To Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create the first how-to guides by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`

```
## Task: Run Tutorial Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create the first tutorials by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/06-task-derive-navigation-from-authored-pages.md`

```
## Task: Derive Navigation From Authored Pages <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** After real content exists across multiple Diataxis forms, derive mdBook navigation and any landing pages from that content. This task is for authored structure, not for the final truth-check pass.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/07-task-run-contributor-codemap-codeguide-pages-through-draft-check-edit-revise.md`

```
## Task: Run Contributor Codemap Codeguide Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create a separate contributor chapter for codemap and codeguide material by running the pages through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/08-task-run-final-accuracy-verification-and-create-bugs.md`

```
## Task: Run Final Accuracy Verification And Create Bugs <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Perform the final accuracy-only verification pass after the authoring and navigation tasks are complete. This is the only task in the story that should introduce and use `docs/verifications/`. Its purpose is to check truth, not to do more drafting.
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

