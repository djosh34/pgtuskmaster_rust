## Task: Add A Perpetual Self-Reported Primary-Count Invariant Runner <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Add a scenario-agnostic invariant runner that continuously queries each node individually and fails immediately unless the sum of self-reported primaries is either 0 or 1. The higher-order goal is to move "no dual primary" logic completely out of feature files and out of ad hoc transition-history bookkeeping. This invariant must run independently of scenario text and must be stronger than the current step-based assertions because it applies for the whole scenario lifetime.

**Context from research and PO direction:**
- The current suite encodes this concern indirectly through feature steps like `there is no dual-primary evidence during the transition window` plus timeline/history helpers such as `assert_no_dual_primary_evidence`.
- The PO explicitly wants this as one of the two stronger continuous expectations:
  - query each node individually
  - look only at how each node reports its own state
  - the total number of primary self-reports must be 0 or 1
  - fail directly when violated
- After task 04, the suite should already have the direct per-node JSON observation surface needed to implement this cleanly.

**Scope:**
- Add the invariant runner under `tests/ha/support/` in whatever minimal module shape best fits the reduced harness.
- Wire the invariant runner into scenario startup/shutdown so it runs automatically for every HA scenario.
- Delete the old dual-primary transition-history bookkeeping once the perpetual runner replaces it.

**Required behavior:**
- Poll every node individually through the direct host-side `pgtm` observation surface.
- Use only each node's self-report to count primaries.
- Treat command failure as absence of a self-report for that node, not as an excuse to infer state from another node.
- Fail immediately on any sample where the primary count is greater than 1 or otherwise outside the allowed `0 | 1` set.
- Emit an artifact or timeline snapshot that makes the failure obvious without log-scraping.

**Expected outcome:**
- The suite has one always-on primary-count safety gate instead of repeated feature-local "no dual primary evidence" steps.
- The world/step layer no longer needs timeline-based dual-primary bookkeeping.
- Scenarios become simpler because this safety property is enforced globally.

</description>

<acceptance_criteria>
- [x] Add a perpetual HA invariant runner that continuously samples all nodes' self-reported status and enforces the allowed primary-count set `{0, 1}`.
- [x] Start this runner automatically for every HA scenario and stop/clean it up deterministically at scenario end.
- [x] Fail the scenario immediately when the invariant is violated; do not wait for a later step to notice.
- [x] Delete `assert_no_dual_primary_evidence`, transition-window primary-history dependence, and any equivalent feature-local dual-primary assertion machinery after the runner replaces it.
- [x] Remove all feature text that explicitly asserts "no dual primary evidence"; this property must now live only in the perpetual runner.
- [x] Persist enough structured failure evidence that the violating sample can be inspected without grepping logs.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Build the perpetual invariant runner on top of the per-node observation surface from task 04.
2. Wire it into the HA scenario lifecycle so it starts automatically and owns its own failure channel/artifacts.
3. Delete the old dual-primary step/history machinery after the runner proves the same property continuously.
4. Run the required repo validation gates after the invariant runner is active in the reduced suite.

### Constraints for execution
- Use only node-local self-reports for this invariant. Do not reintroduce cluster-wide inference or seed-selection heuristics.
- Do not preserve the old timeline/history mechanism as a parallel implementation.
- Keep the failure payload structured and small; the point is direct evidence, not more log scraping.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
