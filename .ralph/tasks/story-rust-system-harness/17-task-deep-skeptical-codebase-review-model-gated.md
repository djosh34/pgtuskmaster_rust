---
## Task: Deep skeptical codebase review with strict model gate and task fanout <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<description>
**Goal:** Run a trust-nothing deep skeptical audit of the entire codebase quality and produce follow-up tasks for every finding, while enforcing model-profile switching rules.

**Scope:**
- Before any review work, check `.ralph/model.txt` and enforce `deep_review` preflight.
- If `.ralph/model.txt` is not exactly `deep_review`, set it to `deep_review` and quit immediately so the next run uses the correct model.
- Perform deep research and a deep skeptical check of code and tests; trust nothing and assume nothing.
- Do not run tests in this task run. Test execution is explicitly out of scope and handled by another task.
- Audit test reality and anti-silent-pass properties:
- Verify tests are real and meaningful.
- Verify tests cannot pass silently when there was an actual error.
- Verify tests use real PostgreSQL 16 and real `etcd` binaries where required.
- Verify e2e/integration tests exercise real implementation behavior, not accidental behavior.
- Audit all code smells and broader code quality issues; nothing is out of scope.
- For each small issue, create a bug task via `$add-bug`.
- For each larger issue/refactor, create a task via `$add-task-as-agent`.
- When the review is fully complete, set `.ralph/model.txt` back to exactly `normal_high`.

**Context from research:**
- Existing deep-review work already produced multiple bug tasks and a larger integration task; this run must re-check independently and skeptically.
- Story path: `.ralph/tasks/story-rust-system-harness/`.

**Expected outcome:**
- A fresh skeptical review record exists.
- Every issue found has an explicit follow-up task (`$add-bug` for small fixes, `$add-task-as-agent` for larger work).
- `.ralph/model.txt` is switched to `normal_high` only after the full review/fanout is done.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Preflight check verifies whether `.ralph/model.txt` is exactly `deep_review`.
- [ ] If `.ralph/model.txt` is missing or not `deep_review`, write exactly `deep_review` and quit immediately (no further work in that run).
- [ ] If `.ralph/model.txt` is already exactly `deep_review`, proceed with deep skeptical review.
- [ ] No tests are executed in this task run (`make check`, `make test`, `make lint`, `make test-bdd` are out of scope here).
- [ ] Review explicitly verifies test reality, silent-pass prevention, real pg16/etcd usage, and real implementation coverage in e2e/integration flows.
- [ ] Review explicitly audits code smells and general code quality across the whole codebase with no out-of-scope areas.
- [ ] Every small finding gets a dedicated bug task via `$add-bug`.
- [ ] Every larger finding/refactor gets a dedicated task via `$add-task-as-agent`.
- [ ] Final step after full completion sets `.ralph/model.txt` to exactly `normal_high`.
</acceptance_criteria>
