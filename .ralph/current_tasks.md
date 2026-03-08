# Current Tasks Summary

Generated: Sun Mar  8 12:28:05 PM CET 2026

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`

```
## Task: Run How-To Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the how-to chapter through repeated capped runs. Every how-to page must be drafted and revised by K2 under strict Diataxis how-to guidance. The task must provide operational facts and constraints, not write the page prose itself.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`

```
## Task: Run Tutorial Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the tutorial chapter through repeated capped runs. Every tutorial page must be drafted and revised by K2 under strict Diataxis tutorial guidance. The task must provide the learner path, guardrails, and source facts, not hand-write the tutorial prose.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/06-task-derive-navigation-from-authored-pages.md`

```
## Task: Derive Navigation From Authored Pages With K2 Overviews <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Derive mdBook navigation and any needed landing or overview pages from real authored docs. Any new overview prose must be drafted and revised by K2 under Diataxis guidance. The task must provide navigation facts and content relationships, not hand-write the overview prose itself.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/07-task-run-contributor-codemap-codeguide-pages-through-draft-check-edit-revise.md`

```
## Task: Run Contributor Codemap Codeguide Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build contributor-focused codemap and codeguide docs through repeated capped runs. Every contributor page must be drafted and revised by K2 under the correct Diataxis form. The task must provide codebase context, audience needs, and constraints, not hand-write the docs prose itself.
```

==============

# Task `.ralph/tasks/story-build-docs-diataxis-from-zero/08-task-run-final-accuracy-verification-and-create-bugs.md`

```
## Task: Run Final Accuracy Verification And Create Bugs <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Perform the final accuracy-only verification pass after the authoring and navigation tasks are complete. This task verifies K2-authored docs against the repository and creates bug tasks for unsupported or inaccurate claims. It is not a drafting task.
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

