## Task: Produce Post-Greenfield HA Refactor Option Artifacts, Email Review, And Stop Ralph <status>completed</status> <passes>false</passes>

<priority>high</priority>

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
- Use the user-provided workflow requirement that the executor must find the on-host mail reply helper, use `reply.sh` to send the email, and then write `.ralph/STOP`. At task-creation time the known helper directory is `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/` and the known script path is `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`; re-check that path during execution in case it moves.
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
- [x] The task markdown remains blocked by all four tasks in `story-greenfield-cucumber-ha-harness`, so it cannot honestly be started before the full greenfield HA test creation story is done.
- [x] The executor investigates the then-current greenfield failing set only when this task actually starts, not during task creation, and records that evidence in a story-local artifact file.
- [x] A story-local artifact subdirectory exists at `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/` and contains:
- [x] one evidence/inventory artifact for the then-current failing set and implicated code paths
- [x] multiple executor-named option artifacts representing materially different solution directions
- [x] one comparison artifact that maps the failing set across those options
- [x] Every option document explicitly explains how it fixes all then-current greenfield HA failures as one coherent design, not as an itemized bug patch list.
- [x] The option set spans meaningfully different sizes of change, including at least one relatively conservative refactor and at least one much larger structural refactor.
- [x] Every option preserves the `ha_loop` and functional `decide` direction, avoids `mut`-driven architecture, rejects hacks and edge-case spray, and explains how `src/ha/worker.rs` deduplication gets deleted rather than moved elsewhere.
- [x] Every option explicitly addresses all of the user-named design problems: true majority quorum semantics, less blunt trust-versus-election policy, deterministic durability-based leader ranking, startup versus steady-state authority alignment, and a less coarse uncertainty-versus-availability model.
- [x] The comparison artifact explicitly maps each then-current failing test or failing scenario class to every option, so it is clear how each plan claims to solve the entire failing set.
- [x] If existing bug tasks from the greenfield runs are present, the artifacts reference them only as evidence and do not fix them during this task.
- [x] The executor independently checks the artifacts against the then-current failing greenfield test files and the implicated HA source files, and records that cross-check in the story artifacts.
- [x] An email is sent with the discovered on-host `reply.sh` helper from the receive_mail or receive_email directory. The known path to verify first is `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`, and the email contains a natural-language summary without bullet points, the best and second-best options with reasons, and substantive architecture questions for the user.
- [x] `.ralph/STOP` is written after the email is sent and before any further Ralph loop work continues.
- [x] Docs are updated if this task changes any documented workflow around the design-review stop; otherwise the story artifacts and task markdown are the required written record.
- [x] `make check` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set and `<passes>true</passes>` remains false.
- [x] `make test` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set
- [x] `make test-long` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set
- [x] `make lint` is run and its then-current result is captured in the story artifacts as evidence for planning; if it fails, the failure is mapped into the option set and `<passes>true</passes>` remains false.
- [x] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete; if the planning stop intentionally lands before downstream bug fixing, leave `<passes>false</passes>` and say so explicitly in the artifacts instead of pretending the repo is passing.
</acceptance_criteria>

## Detailed implementation plan

### Phase 0: Honor sequencing and preserve the planning-only boundary
- [x] Do not start this task until all four tasks in `story-greenfield-cucumber-ha-harness` are complete.
- [x] Re-read this task before starting and keep the boundary clear: this task exists to investigate, compare, recommend, email, and stop, not to implement the chosen refactor.
- [x] Create the story-local artifact directory `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/`.
- [x] Create an artifact note that states this task was intentionally created without inspecting the then-current `src/` and `tests/` state because the migration was still underway at task-creation time.
- [x] If existing bug tasks from the greenfield story are already present, list them in the artifacts as evidence inputs only and explicitly mark them out of scope for implementation in this task.

### Phase 1: Investigate the then-current failing set after the blockers clear
- [x] Run the then-relevant greenfield HA commands and repo gates needed to expose the current failing set without changing the product to “make progress.”
- [x] Record the failing scenarios, failing commands, relevant logs, and observed symptoms in an evidence artifact whose exact filename is chosen during execution.
- [x] Inspect the then-current greenfield HA test files and wrappers under `cucumber_tests/ha/` that correspond to the failing set, and map each failing scenario to the behavior it expects.
- [x] Inspect the then-current HA source areas implicated by those failures, including at minimum `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/runtime/node.rs`, and any directly adjacent modules that the failing tests actually touch.
- [x] Record where the current behavior appears to come from, including decision boundaries, state-loop interactions, startup authority paths, election ordering behavior, quorum behavior, and any deduplication or side-path logic that affects the failures.
- [x] Cross-check the observed failures against any existing greenfield bug tasks and note where the bug tasks do or do not describe the full design issue.

### Phase 2: Generate multiple independent refactor-option artifacts
- [x] Derive multiple genuinely different solution directions from the actual failing evidence rather than from predetermined labels in this task.
- [x] Name the option artifacts during execution based on their real design centers after the evidence review is complete.
- [x] Ensure the option set includes both smaller and larger refactor directions, but let the actual number of options and their names follow from the real design space discovered at execution time.
- [x] For every option document, include the exact design goal, the expected behavior changes, the concrete code areas that would likely need to change after implementation starts, the likely test files or scenario classes it satisfies, the tradeoffs, the risks, the migration size, and the reasons it remains compatible with the current `ha_loop` plus functional `decide` direction.
- [x] For every option document, explain explicitly how quorum becomes a real majority rule, how trust-versus-election policy changes, how leader ranking becomes deterministic and durability-driven, how startup authority aligns with steady-state HA authority, how uncertainty is handled without a blunt collapse into `FailSafe`, and how `src/ha/worker.rs` deduplication is removed cleanly.
- [x] For every option document, explain explicitly how the option avoids `mut`-heavy architecture, avoids unwrap/panic/expect-style design shortcuts, and avoids edge-case spray.

### Phase 3: Compare the options against the actual failing evidence
- [x] Write one comparison artifact whose exact filename is chosen during execution.
- [x] In that comparison artifact, list each then-current failing greenfield scenario or failure class and show how each option resolves it.
- [x] In that comparison artifact, compare implementation size, conceptual cleanliness, risk of hidden edge cases, likely impact on `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/runtime/node.rs`, and alignment with the user’s functional-programming constraints.
- [x] In that comparison artifact, state clearly which option is best and which is second-best, but keep a higher-level natural-language version of that recommendation for the email.
- [x] Re-read every artifact against the then-current failing tests and source files and add a final verification note that the plans were independently checked against real repo evidence instead of being generic architecture essays.

### Phase 4: Send the email review
- [x] Find the on-host mail reply helper by searching for a `reply.sh` script under a receive-mail or receive-email directory. Check `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh` first. If that location still exists, use it; if not, locate the new path and record it in the artifacts.
- [x] Draft the email summary in natural language without bullet points.
- [x] In the email, explain that this is an intentionally weird stop between greenfield test creation and bug fixing, and that the purpose is to choose a whole-system direction before touching the exposed bugs.
- [x] In the email, summarize the option set, name the best and second-best options with reasons, link or quote the story-local artifact paths, and ask substantive questions that would materially help choose the implementation path.
- [x] Send the email with `reply.sh` and record the exact command shape or invocation notes in the story artifacts without leaking secrets.
- [x] Use the reply helper with the same heredoc shape documented in the mail workflow so multiline prose is safe, for example:
  `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh "<sender_email>" "<original_subject>" << 'EOF_REPLY_EMAIL'`
  `Natural-language body goes here.`
  `EOF_REPLY_EMAIL`
- [x] Note in the artifacts that `reply.sh` adds the `Re:` prefix automatically when needed, accepts the recipient as the first argument, the original subject as the second argument, and either a third inline body argument or stdin from the heredoc body.

### Phase 5: Stop the Ralph loop and close out honestly
- [x] Write `.ralph/STOP` after the email is sent.
- [x] Run `/bin/bash .ralph/task_switch.sh` before writing `.ralph/STOP` so Ralph history/current-task state is advanced without violating the stop marker boundary.
- [x] Update this task markdown and the story artifacts so they accurately describe what was produced, what was recommended, what questions were sent, and whether the repo gates were passing or intentionally left failing.
- [x] If the worktree has task-artifact changes that should be preserved, commit them, including `.ralph/STOP`, with a message that makes the intentional design-review stop explicit.
- [x] Push with `git push` if the normal Ralph workflow still expects the story artifacts and stop marker to be shared.
- [x] For this execution, treat `make check`, `make test`, `make test-long`, and `make lint` as required for final closeout in addition to the planning artifacts. If any gate fails, record that truthfully in the artifacts and task file, but do not pretend the task fully passes.

## Skeptical plan review notes

- Verified that the four prerequisite tasks in `story-greenfield-cucumber-ha-harness` are already marked complete with `<passes>true</passes>`, so this task is no longer sequencing-blocked.
- Resolved a closeout-order conflict between this task and the higher-level Ralph operator instructions by requiring `/bin/bash .ralph/task_switch.sh` before `.ralph/STOP`, not after it.
- Tightened the final-closeout interpretation so the planning-only nature of the task does not override the stricter repo-gate requirement for claiming completion in this run.

NOW EXECUTE

## Execution report

- Produced the full story-local artifact set under `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/`, including the task-creation note, evidence inventory, three materially different refactor-option documents, the comparison matrix, the email draft, and email send notes.
- Verified the on-host reply helper at `/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh`, sent the natural-language review email, ran `/bin/bash .ralph/task_switch.sh`, and then wrote `.ralph/STOP`.
- Gate results for this execution:
  - `make check`: pass
  - `make test`: pass
  - `make test-long`: fail (`26 tests run: 11 passed, 15 failed, 0 skipped`)
  - `make lint`: pass
- Preserved the planning-stop state in git and pushed it after the review artifacts, email, `task_switch`, and `.ralph/STOP` were in place.
- Because `make test-long` is still failing, this task is intentionally closed with `<passes>false</passes>`. The review artifacts are complete, but the repo is not green, and this markdown does not pretend otherwise.
