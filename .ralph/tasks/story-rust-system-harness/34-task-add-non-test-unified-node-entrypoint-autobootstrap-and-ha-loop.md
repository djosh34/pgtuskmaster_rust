---
## Task: Add non-test unified node entrypoint from start through autonomous HA loop <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Provide one production (non-test) entry path that starts a `pgtuskmaster` node from config only and runs it through bootstrap and HA loop without manual orchestration.

**Scope:**
- Add/extend runtime entry code in non-test modules so node startup is performed through a single canonical entrypoint.
- Ensure startup path decides bootstrap mode from existing state:
- no local PGDATA -> initialize as needed,
- existing upstream primary state -> base backup/follow path,
- existing local state -> resume/reconcile path.
- Wire all required channels/subscribers/workers internally (HA, DCS, process, pginfo, API/debug as required by runtime contracts) so callers only pass config.
- Keep decision logic integrated with existing state/coordination components, not test-only helpers.
- Ensure strict `Result`-based error propagation with no unwrap/expect/panic.

**Context from research:**
- Current request reports missing production-grade "main/entry" flow and ad-hoc startup behavior in tests.
- Existing HA/state/coordination logic already exists but must be invoked through one stable runtime entry interface.
- This must be implemented in non-test code and become the canonical startup surface.

**Expected outcome:**
- A node can be started in production using one entrypoint and config, and it autonomously reaches steady HA operation with correct bootstrap behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: runtime entry module(s), startup/bootstrapping decision layer, worker/channel wiring layer, binary/main invocation path, and any required config schema updates
- [ ] Startup behavior validated for key states: empty PGDATA init path, replica/basebackup path, and restart/resume path from existing PGDATA
- [ ] Entry API accepts config-only invocation (no test-only coordination hooks required to bring node online)
- [ ] No unwrap/expect/panic introduced; all new error paths return typed errors
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
