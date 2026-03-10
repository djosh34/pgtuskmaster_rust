## Task: Produce Post-Greenfield HA Refactor Option Artifacts, Email Review, And Stop Ralph <status>not_started</status> <passes>false</passes>

<priority>high</priority>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
**Goal:** This is intentionally a weird task. After the full greenfield HA cucumber story has finished creating the new tests, but before anyone starts fixing the newly exposed bugs, inspect the then-current failing greenfield HA tests and the then-current HA implementation, and produce multiple different refactor-plan artifacts that each aim to fix the entire failing set at once instead of doing bug-by-bug patching. After the plans are written, send the user an email with a natural-language summary, a recommendation for the best and second-best options, and deep questions, then stop the Ralph loop by writing `.ralph/STOP`.

**Original user shift / motivation:** The user explicitly does not want this moment in the process to turn into scattered reactive bug fixing against an earlier HA design. The greenfield tests are intentionally raising expectations against an older design shape, so the next step must be a coherent design review stop, not immediate local repairs. This task exists to force a pause between “we now have much better tests” and “we start fixing whatever they found,” so the codebase can choose a whole-system refactor direction instead of accumulating hacks.

**Higher-order goal:** Keep the HA architecture centered on the current `ha_loop` plus functional `decide` style, while generating credible whole-system refactor options that improve correctness, determinism, and maintainability together. The broader goal is to make later implementation follow a deliberate architectural choice, not an accident of whichever failing test somebody notices first.

HARD REQUIREMENT: DO NOT FIX/ALTER ANY CODE WHATSOEVER! ONLY PLAN AND WRITE PLANS!
This means that if make test and/or make test-long fails, ARE NOT BLOCKING FOR THIS TASK!

**Scope:**
- This task is planning-only. It must not implement the HA refactor itself, and it must not “fix forward” production code or test code just to get through failing scenarios.
- The task must begin only after all four tasks in `story-greenfield-cucumber-ha-harness` are complete, because its input is the full then-current greenfield failing set after the migration work has landed.
- Task creation itself intentionally does not inspect the current `src/` and `tests/` state because the migration is still in flight right now. The actual executor of this task must do that investigation later, after the blockers clear.
- Create one story-local artifact subdirectory under `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/` and put all plan outputs there. Use that directory for the failure inventory, option documents, comparison matrix, email draft notes if needed, and any command-output evidence.
- The plans must cover both smaller and larger refactor directions, but the actual option shapes, groupings, and names must be discovered from the then-current failing tests and code rather than predetermined here.
- Every option must preserve the current `ha_loop` and functional `decide` pattern. Do not propose an imperative rewrite, a `mut`-heavy controller, or a design that spreads weird edge-case branches everywhere.
- Every option must explicitly remove the deduplication path in `src/ha/worker.rs` and instead explain how the state loop becomes authoritative enough that duplicate suppression is no longer needed there.
- Every option must move the system toward stronger state-loop integration rather than keeping a pile of differing internal code paths.
- Use the user-provided workflow requirement that the executor must find the on-host mail reply helper, use `reply.sh` to send the email, and then write `.ralph/STOP`.
- If bug tasks already exist from the greenfield work, use them as evidence only. Do not start fixing them inside this task.

**Context from research:**
- Prior Ralph notes already identified likely HA design pressure points to revisit during execution of this task: `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, and `src/runtime/node.rs`.
- Prior Ralph notes already identified a quorum shortcut in `src/dcs/state.rs` that behaves like “one fresh member if the view is size one, otherwise two fresh members,” which is not a general majority-of-configured-membership rule.
- Prior Ralph notes already identified a trust-policy boundary in `src/ha/decide.rs` that routes states outside `FullQuorum` into `FailSafe`, which suggests trust and election are still coupled too bluntly.
- Prior Ralph notes already identified deduplication in `src/ha/worker.rs`, which the user explicitly wants removed in favor of cleaner state-loop integration.
- Prior Ralph notes already identified startup authority/path-selection concerns in `src/runtime/node.rs`, where startup source selection may still reason differently from the steady-state HA authority checks.
- The user’s current design concerns that every option must address are: quorum must become a real majority rule over configured cluster membership; trust reduction must not be treated as a blanket reason to stop participating when a healthy majority can still make progress; leader selection must become deterministic and durability-based rather than mostly timing-based; startup authority checks must align with steady-state HA authority checks; and the availability policy under uncertainty must be modeled more precisely than “confidence fell, therefore fail-safe.”
- Explicit design constraints from the user: keep the current `ha_loop` plus functional `decide` style, prefer functional programming over `mut`, do not use hacks or edge-case spray, integrate more into the state loop instead of adding more side paths, remove `src/ha/worker.rs` deduplication, and show both small and large refactor options.
- The user also explicitly wants the final email summary written in natural language without bullet points, and wants real questions that go deep into the architecture and tradeoffs.

**Expected outcome:**
- The greenfield story contains a concrete evidence file for the then-current failing greenfield HA scenarios and the implicated HA code paths.
- The greenfield story contains multiple materially different refactor-option documents, named by the executor based on the real findings at execution time, and each one is capable of fixing the full then-current failing set as a coherent whole rather than as isolated bug patches.
- The greenfield story contains a comparison artifact that makes tradeoffs clear across small, medium, and large refactor directions.
- The user receives an email that explains the options in natural language, recommends the best and second-best directions with reasons, and asks substantive questions that will help choose an implementation path.
- `.ralph/STOP` exists at the end so Ralph does not keep rolling into bug-fixing work before the design pause has been reviewed by the user.
</description>

<acceptance_criteria>
- [ ] The task markdown remains blocked by all four tasks in `story-greenfield-cucumber-ha-harness`, so it cannot honestly be started before the full greenfield HA test creation story is done.
- [ ] The executor investigates the then-current greenfield failing set only when this task actually starts, not during task creation, and records that evidence in a story-local artifact file.
- [ ] A story-local artifact subdirectory exists at `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/` and contains:
- [ ] one evidence/inventory artifact for the then-current failing set and implicated code paths
- [ ] multiple executor-named option artifacts representing materially different solution directions
- [ ] one comparison artifact that maps the failing set across those options
- [ ] Every option document explicitly explains how it fixes all then-current greenfield HA failures as one coherent design, not as an itemized bug patch list.
- [ ] The option set spans meaningfully different sizes of change, including at least one relatively conservative refactor and at least one much larger structural refactor.
- [ ] Every option preserves the `ha_loop` and functional `decide` direction, avoids `mut`-driven architecture, rejects hacks and edge-case spray, and explains how `src/ha/worker.rs` deduplication gets deleted rather than moved elsewhere.
- [ ] Every option explicitly addresses all of the user-named design problems: true majority quorum semantics, less blunt trust-versus-election policy, deterministic durability-based leader ranking, startup versus steady-state authority alignment, and a less coarse uncertainty-versus-availability model.
- [ ] The comparison artifact explicitly maps each then-current failing test or failing scenario class to every option, so it is clear how each plan claims to solve the entire failing set.
- [ ] If existing bug tasks from the greenfield runs are present, the artifacts reference them only as evidence and do not fix them during this task.
- [ ] The executor independently checks the artifacts against the then-current failing greenfield test files and the implicated HA source files, and records that cross-check in the story artifacts.
- [ ] An email is sent with the discovered on-host `reply.sh` helper from the receive_mail or receive_email directory, and the email contains a natural-language summary without bullet points, the best and second-best options with reasons, and substantive architecture questions for the user.
- [ ] `.ralph/STOP` is written after the email is sent and before any further Ralph loop work continues.
- [ ] Docs are updated if this task changes any documented workflow around the design-review stop; otherwise the story artifacts and task markdown are the required written record.
- [ ] `make check` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set and `<passes>true</passes>` remains false.
- [ ] `make test` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set
- [ ] `make test-long` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set
- [ ] `make lint` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set and `<passes>true</passes>` remains false.
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete; if the planning stop intentionally lands before downstream bug fixing, leave `<passes>false</passes>` and say so explicitly in the artifacts instead of pretending the repo is passing.
</acceptance_criteria>

## Detailed implementation plan

### Phase 0: Honor sequencing and preserve the planning-only boundary
- [ ] Do not start this task until all four tasks in `story-greenfield-cucumber-ha-harness` are complete.
- [ ] Re-read this task before starting and keep the boundary clear: this task exists to investigate, compare, recommend, email, and stop, not to implement the chosen refactor.
- [ ] Create the story-local artifact directory `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/`.
- [ ] Create an artifact note that states this task was intentionally created without inspecting the then-current `src/` and `tests/` state because the migration was still underway at task-creation time.
- [ ] If existing bug tasks from the greenfield story are already present, list them in the artifacts as evidence inputs only and explicitly mark them out of scope for implementation in this task.

### Phase 1: Investigate the then-current failing set after the blockers clear
- [ ] Run the then-relevant greenfield HA commands and repo gates needed to expose the current failing set without changing the product to “make progress.”
- [ ] Record the failing scenarios, failing commands, relevant logs, and observed symptoms in an evidence artifact whose exact filename is chosen during execution.
- [ ] Inspect the then-current greenfield HA test files and wrappers under `cucumber_tests/ha/` that correspond to the failing set, and map each failing scenario to the behavior it expects.
- [ ] Inspect the then-current HA source areas implicated by those failures, including at minimum `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/runtime/node.rs`, and any directly adjacent modules that the failing tests actually touch.
- [ ] Record where the current behavior appears to come from, including decision boundaries, state-loop interactions, startup authority paths, election ordering behavior, quorum behavior, and any deduplication or side-path logic that affects the failures.
- [ ] Cross-check the observed failures against any existing greenfield bug tasks and note where the bug tasks do or do not describe the full design issue.

### Phase 2: Generate multiple independent refactor-option artifacts
- [ ] Derive multiple genuinely different solution directions from the actual failing evidence rather than from predetermined labels in this task.
- [ ] Name the option artifacts during execution based on their real design centers after the evidence review is complete.
- [ ] Ensure the option set includes both smaller and larger refactor directions, but let the actual number of options and their names follow from the real design space discovered at execution time.
- [ ] For every option document, include the exact design goal, the expected behavior changes, the concrete code areas that would likely need to change after implementation starts, the likely test files or scenario classes it satisfies, the tradeoffs, the risks, the migration size, and the reasons it remains compatible with the current `ha_loop` plus functional `decide` direction.
- [ ] For every option document, explain explicitly how quorum becomes a real majority rule, how trust-versus-election policy changes, how leader ranking becomes deterministic and durability-driven, how startup authority aligns with steady-state HA authority, how uncertainty is handled without a blunt collapse into `FailSafe`, and how `src/ha/worker.rs` deduplication is removed cleanly.
- [ ] For every option document, explain explicitly how the option avoids `mut`-heavy architecture, avoids unwrap/panic/expect-style design shortcuts, and avoids edge-case spray.

### Phase 3: Compare the options against the actual failing evidence
- [ ] Write one comparison artifact whose exact filename is chosen during execution.
- [ ] In that comparison artifact, list each then-current failing greenfield scenario or failure class and show how each option resolves it.
- [ ] In that comparison artifact, compare implementation size, conceptual cleanliness, risk of hidden edge cases, likely impact on `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/runtime/node.rs`, and alignment with the user’s functional-programming constraints.
- [ ] In that comparison artifact, state clearly which option is best and which is second-best, but keep a higher-level natural-language version of that recommendation for the email.
- [ ] Re-read every artifact against the then-current failing tests and source files and add a final verification note that the plans were independently checked against real repo evidence instead of being generic architecture essays.

### Phase 4: Send the email review
- [ ] Find the on-host mail reply helper by searching for a `reply.sh` script under a receive-mail or receive-email directory. If the previously seen location still exists, use it; if not, locate the new path and record it in the artifacts.
- [ ] Draft the email summary in natural language without bullet points.
- [ ] In the email, explain that this is an intentionally weird stop between greenfield test creation and bug fixing, and that the purpose is to choose a whole-system direction before touching the exposed bugs.
- [ ] In the email, summarize the option set, name the best and second-best options with reasons, link or quote the story-local artifact paths, and ask substantive questions that would materially help choose the implementation path.
- [ ] Send the email with `reply.sh` and record the exact command shape or invocation notes in the story artifacts without leaking secrets.

### Phase 5: Stop the Ralph loop and close out honestly
- [ ] Write `.ralph/STOP` after the email is sent.
- [ ] Do not run `/bin/bash .ralph/task_switch.sh` after writing `.ralph/STOP`.
- [ ] Update this task markdown and the story artifacts so they accurately describe what was produced, what was recommended, what questions were sent, and whether the repo gates were passing or intentionally left failing.
- [ ] If the worktree has task-artifact changes that should be preserved, commit them, including `.ralph/STOP`, with a message that makes the intentional design-review stop explicit.
- [ ] Push with `git push` if the normal Ralph workflow still expects the story artifacts and stop marker to be shared.
- [ ] Only after every required checkbox above is complete and the repo gates are genuinely passing should `<passes>true</passes>` ever be set. If this task stops before downstream remediation, keep `<passes>false</passes>` and leave a truthful note in the task instead of pretending the repo is green.

TO BE VERIFIED
