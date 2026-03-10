---
name: add-task-as-agent
description: Create a task when the AGENT (Claude) needs to create one. Agents should always use THIS skill, not add-task-as-user.
---

## Purpose

Create an execution-ready Ralph task that acts as the full contract for later work.

The task must not be a vague summary. It must preserve the original user goal, every user request that was agreed to, every suggestion from the conversation that was agreed to, the important research context, and a concrete implementation plan with checkboxes. The person executing the task should not need the original conversation to understand what "done" means.
You should make it verbose!

If you cannot yet write a zero-ambiguity plan, do more research first. Do not create a wishy-washy task.

## Where to create

- Create the task in the same story directory as the research/planning work unless there is a concrete reason to start a new story.
- Use a descriptive slug that reflects the real goal, for example `task-tighten-ha-leader-failover-verification.md`.

## Non-negotiable rules

- The task must include the original user shift/goal when it matters, not just the implementation details.
- Every user request that was discussed and accepted must appear in the task.
- Every suggestion from the conversation that was discussed and accepted must appear in the task.
- Every suggestion that was discussed and explicitly rejected MUST NOT appear in the task.
- Do not silently compress or omit approved details just because they were discussed earlier.
- If the conversation produced an important design choice, invariant, rejection, or tradeoff, include it explicitly so it is not rediscovered later.
- Name exact files, modules, functions, types, commands, and tests whenever they are already known. Do not hide concrete implementation knowledge behind generic wording like "update the HA code".
- If the task removes or replaces an old feature, old flag, old API shape, old type, or old code pattern, the task must explicitly require full cleanup and explicit verification that stale occurrences do not remain accidentally.
- Resolve ambiguity before task creation. If you still cannot resolve it, keep researching while refining the task.
- Acceptance criteria must be real completion criteria, not generic filler.
- The implementation plan must be exhaustive enough that the executor can tick boxes while working without inventing missing steps.
- `<passes>true</passes>` is forbidden until every required checkbox is checked and every required repo gate has actually passed.

## Required creation workflow

### 1. Capture the full approved contract before writing

Before creating the task, make a private checklist for yourself from the conversation and research:

- Original user goal / shift
- Higher-order goal
- Every explicit requirement
- Every explicit constraint
- Every approved suggestion
- Every approved non-goal or rejection
- Every important file/module/function already identified
- Every required verification step

Do not start writing the task until you can account for all of them.
If the discussion already identified concrete code locations, your task should name them directly, for example `src/ha/worker.rs` `reconcile_tick(...)` or `tests/ha_partition_isolation.rs`.

### 2. Write the task as a complete execution handoff

The task must contain, at minimum:

- Clear goal and higher-order goal
- Original user shift / motivation
- Exhaustive scope
- Research context with concrete file/module/function references
- Expected outcome
- Acceptance criteria as a completion contract
- Detailed implementation plan with checkbox steps
- Required verification gates
- A final marker of `TO BE VERIFIED`

If an approved point is intentionally out of scope for this task, still include it in the task and explain where it lands:

- either as an explicit non-goal for this task
- or as required follow-up work that must be split into another named task

Do not leave approved points floating outside the written task.

### 3. Make acceptance criteria strict

Acceptance criteria must answer "what must be true before this task may be marked done?"

They must:

- cover every promised behavior change
- cover every required code area or artifact when that matters
- include docs/workflow/test updates when relevant
- include the repo verification gates
- include explicit cleanup verification when old behavior or old patterns are being removed
- be phrased so an executor cannot honestly mark them complete while required work remains undone

### 4. Make the implementation plan exhaustive

The implementation plan is not a vague outline. It is the execution path.
It must leave ZERO ambiguity

It must:

- be organized into phases or ordered sections when the work is non-trivial
- use checkboxes
- include the concrete code changes to make, with file references and target symbols when known
- include important tests to update or add
- include docs or task-file updates if required
- include cleanup/removal of obsolete code when that is part of the approved plan
- include explicit repo-wide stale-pattern verification steps when removing legacy behavior
- include explicit verification and closeout steps

If the executor would still need to infer major missing steps, the plan is not complete enough.

### 5. Gate pass/fail correctly

The task must make it explicit that:

- all acceptance-criteria checkboxes must be checked before completion
- all implementation-plan checkboxes that represent required work must be checked before completion
- docs must be updated when features/behavior/docs changed
- `make check`
- `make test`
- `make test-long`
- `make lint`
- all must pass before `<passes>true</passes>` may be set

Do not write the task in a way that allows partial completion to be marked as passing.

### 6. Self-audit before saving the task

Before you finish, verify all of the following are true:

- The original user goal is present
- The higher-order goal is present
- Every approved request from the conversation is represented
- Every approved suggestion from the conversation is represented
- Scope is concrete, not hand-wavy
- Context names the relevant files/modules/functions when known
- Cleanup expectations are explicit when removing or replacing old behavior
- Acceptance criteria are specific enough to block premature completion
- The implementation plan is detailed enough to execute without reopening the planning conversation
- The pass gate is explicit and strict
- The task ends with `TO BE VERIFIED`

If any answer is no, rewrite the task before saving it.

## Task file format

Use this structure and adapt it to the task. Expand it when needed; do not shrink it below usefulness.

```markdown
## Task: [Clear Goal Description] <status>not_started</status> <passes>false</passes>

<priority>[low|medium|high]</priority>

<description>
**Goal:** [State the concrete task goal in multiple sentences.]

**Original user shift / motivation:** [Preserve the underlying user intent, why this task exists, and what broader problem it is meant to solve.]

**Higher-order goal:** [State the broader architectural/product/process goal so the executor understands the reason behind the task.]

**Scope:**
- [Exact files/modules/areas involved, for example `src/ha/worker.rs`, `src/ha/lower.rs`, `tests/ha_partition_isolation.rs`]
- [Exact changes expected]
- [Important boundaries]
- [Any required follow-up splitting or out-of-scope items, if relevant]

**Context from research:**
- [Concrete facts learned from code/research]
- [Relevant files/functions/types/tests, for example `src/ha/worker.rs` `reconcile_tick(...)`, `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)`]
- [Important approved design choices, invariants, or rejections]
- [Patterns/examples to follow]

**Expected outcome:**
- [What must be true once the task is done]
- [What the executor/maintainer/operator should be able to rely on afterward]

</description>

<acceptance_criteria>
- [ ] [Behavior/result requirement 1]
- [ ] [Behavior/result requirement 2]
- [ ] [Required code/test/docs area completed, with named file references where the task already knows them]
- [ ] [Any important invariant/tradeoff reflected in shipped code]
- [ ] [If removing/replacing old behavior] old feature/pattern is fully cleaned from the codebase; repo-wide verification confirms no accidental stale occurrences remain outside explicitly allowed locations
- [ ] docs are updated with new/updated/deleted features and stale docs are removed when relevant
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: [Name]
- [ ] Update [path/to/file.rs] `target_function(...)` so [exact behavior change]
- [ ] Update [path/to/other_file.rs] `TargetType` / `target_test_name` so [exact behavior change]
- [ ] Audit adjacent callers in [path/to/caller.rs] and [tests/path/to/test.rs] so the new contract is applied consistently

### Phase 2: [Name]
- [ ] Remove obsolete [old feature / old type / old API / old flag] from [named files]
- [ ] Update or replace tests in [named test files] to assert the new behavior instead of the removed pattern
- [ ] Run repo-wide verification, for example `rg -n "(OldFeatureName|old_pattern_name)" src tests docs` and confirm only explicitly allowed hits remain

### Phase 3: Verification and closeout
- [ ] [Targeted verification / test additions / manual checks]
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Update docs/task notes if required by the task
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence
- [ ] Push with `git push`

TO BE VERIFIED
```

## Example of the right level of detail

```markdown
## Task: Remove Legacy Replica Parse-Back From HA Startup <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove the old managed-config parse-back path from HA startup and force startup intent to be re-derived from DCS authority plus local physical facts.

**Original user shift / motivation:** The user wants the startup architecture to stop depending on wishful reuse of rendered config and to remove a class of bugs where stale managed files silently drive behavior after restart.

**Higher-order goal:** Make the runtime and HA layers converge on one authoritative startup model instead of a mixed model with hidden fallback behavior.

**Scope:**
- Change `src/runtime/node.rs` `select_resume_start_intent(...)`
- Change `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)`
- Remove obsolete helper usage from `src/postgres_managed.rs`
- Update coverage in `src/runtime/node.rs` tests and `src/ha/process_dispatch.rs` tests

**Context from research:**
- `src/runtime/node.rs` still reconstructs startup state from managed artifacts in the resume path
- `src/ha/process_dispatch.rs` still contains the fallback that trusts the old parse-back path
- `src/postgres_managed_conf.rs` exposes the parsing helper that allowed the old pattern to survive
- The approved design choice is that leader-derived DCS state remains the only authoritative replica source

**Expected outcome:**
- Startup intent is derived from DCS plus local facts, not by reparsing rendered managed config
- The old parse-back pattern is gone from production code and cannot silently reappear
</description>

<acceptance_criteria>
- [ ] `src/runtime/node.rs` `select_resume_start_intent(...)` no longer reparses rendered managed config
- [ ] `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)` no longer falls back to the old parse-back path
- [ ] Old parse-back helpers are fully cleaned from production code, and repo-wide verification confirms no accidental stale callers remain outside explicitly allowed historical task text
- [ ] Tests in `src/runtime/node.rs` and `src/ha/process_dispatch.rs` assert the new authoritative-DCS behavior
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Remove runtime and HA use sites
- [ ] Update `src/runtime/node.rs` `select_resume_start_intent(...)` to inspect only minimal local facts and authoritative DCS inputs
- [ ] Update `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)` so replica startup is derived only from leader-backed DCS data
- [ ] Check `src/ha/worker.rs` and `src/ha/lower.rs` for stale parse-back or fallback logic and remove or update any remaining parallel path

### Phase 2: Clean obsolete pattern everywhere
- [ ] Remove obsolete parse-back helpers from `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` if they have no justified remaining boundary
- [ ] Update tests in `src/runtime/node.rs` and `src/ha/process_dispatch.rs` to assert the new behavior instead of the removed parse-back path
- [ ] Run `rg -n "read_existing_replica_start_intent|parse_managed_primary_conninfo|parse_pg_conninfo" src tests docs .ralph/tasks` and confirm any remaining hits are either intentional task history or justified non-production boundaries

### Phase 3: Verification and closeout
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Only after every checkbox above is complete, set `<passes>true</passes>`

TO BE VERIFIED
```

## Quality bar

Bad task:

- vague summary
- missing original user intent
- missing approved details from the conversation
- missing exact file/function/test references even though they were already known
- generic acceptance criteria
- no repo-wide cleanup verification for removed legacy behavior
- no executable checkbox plan
- allows `<passes>true</passes>` while work is still incomplete

Good task:

- captures the full agreed contract
- preserves why the user wants the work
- states the exact behavioral and code changes expected
- names concrete files/functions/tests when known
- explicitly requires cleanup verification for removed features or stale patterns
- gives the executor an explicit checklist to follow
- makes premature passing impossible without visibly lying in the task file
