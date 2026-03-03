---
## Task: Add Structured File Sink Support (Backlog) <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Extend the unified logging subsystem with optional structured file sink support after the base structured-ingestion task is complete.

**Scope:**
- Implement additional sink modes in the single existing logging config/setup path:
  - structured file output sink(s)
- Keep default behavior unchanged (`stderr` JSONL remains default).
- Ensure sink selection/routing is fully config-driven and composable.

**Context from research:**
- This is intentionally deferred from the base unified logging task:
  - `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md`

**Expected outcome:**
- Operators can route unified structured logs to file without introducing a second logging bootstrap path.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] File sink support exists and is configurable through the single logging config entrypoint
- [ ] Default sink remains JSONL to `stderr`
- [ ] Existing structured log schema and source attribution keys remain stable
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test` — all BDD features pass
</acceptance_criteria>
