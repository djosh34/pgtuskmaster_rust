---
## Task: Setup verbose debug UI and final STOP gate <status>not_started</status> <passes>false</passes> <priority>low</priority>

<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>

<description>
**Goal:** Build a debug UI system that reacts to fine-grained state/action/event changes via a super-verbose debug API endpoint and render those details in a rich static HTML UI; this task runs last.

**Scope:**
- Implement a super-verbose debug API endpoint that streams/exposes all relevant worker state changes, HA actions, events, outputs, and timing/version metadata.
- Ensure payload includes structured sections for pginfo, dcs, process, ha, config, api/debug, and cross-worker timelines.
- Add static HTML/CSS/JS page that fetches debug data and renders visual blocks, figures, timelines, and grouped panels (not plain text dump).
- Ensure UI updates reactively to small incremental state changes.

**Context from research:**
- User requested this task to be run last with explicit priority control and rich visual debug output.

**Expected outcome:**
- Last task provides a practical real-time observability UI and final completion gate for the entire system-harness story.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.

**Note:**
The vm you are in, does not have browser installed nor chrome. But we do expect a working ui.
Please figure out how to validate it is working (by installing something?)
</description>

<acceptance_criteria>
- [ ] Task priority is `low` and the task is blocked by task 15 so it executes last.
- [ ] Debug endpoint exposes super-verbose structured data for all state changes, actions, events, and outputs.
- [ ] Static debug UI renders figures/blocks/panels/timelines and updates on data changes.
- [ ] Debug UI is not text-only; includes visual grouping and state/action emphasis.
- [ ] Perform final validation pass: confirm tests are real (not fake asserts, not tests doing HA logic themselves), all features are present/working/tested, and all suites pass.
- [ ] Run full suite with no exceptions: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] If any validation or suite check fails, do NOT write `.ralph/STOP`; use `$add-bug` skill to create bug task(s).
- [ ] Only when everything above passes, execute `touch .ralph/STOP`.
</acceptance_criteria>
