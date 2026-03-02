---
## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes> <priority>ultra_high</priority>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**.

Every time this task is picked up, the engineer must run a **FRESH verification** from scratch:
- Before starting: ensure `.ralph/model.txt` is exactly `deep_review`; if not, set it and quit immediately to switch model.
- Perform deep skeptical, trust-nothing review across the full codebase quality surface.
- Do not run tests in this specific review task run (test execution belongs to separate tasks).
- Validate test reality and anti-silent-pass guarantees, including real pg16 and real `etcd` binary usage.
- Validate e2e/integration behavior comes from real implementation, not accidental effects.
- Audit all code smells and broader quality concerns with no artificial boundaries.
- Create `$add-bug` tasks for small findings and `$add-task-as-agent` tasks for larger findings.
- Only after the full review/fanout is complete, set `.ralph/model.txt` back to exactly `normal_high`.

**NEVER set this task's passes to anything other than meta-task.**

## Exploration
### YYYY-MM-DD (fresh run)
- Reviewer:
- Preflight model check result:
- Files/modules audited:
- Findings summary:
- Small issues -> bug tasks:
- Large issues -> agent tasks:
- Closeout model reset to `normal_high`:

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] This run is treated as a fresh skeptical verification pass (no assumptions from previous runs).
- [ ] Preflight model gate is enforced (`deep_review`, or set+quit immediately when mismatched).
- [ ] No tests are run in this task run; test execution is deferred to separate tasks.
- [ ] Test reality and silent-pass resistance are verified, including real pg16/etcd binary usage and real implementation behavior in integration/e2e tests.
- [ ] Code smells and broader quality issues are audited across the full codebase.
- [ ] Every small issue is turned into a bug task via `$add-bug`.
- [ ] Every larger issue is turned into a task via `$add-task-as-agent`.
- [ ] Final closeout step sets `.ralph/model.txt` to exactly `normal_high`.
- THIS TASK STAYS AS meta-task FOREVER
</acceptance_criteria>
