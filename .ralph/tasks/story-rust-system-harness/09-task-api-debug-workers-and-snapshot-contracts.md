---
## Task: Implement API and Debug API workers with typed contracts <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>05-task-dcs-worker-trust-cache-watch-member-publish,08-task-ha-worker-select-loop-and-action-dispatch</blocked_by>

<description>
**Goal:** Implement typed API endpoints and debug snapshot visibility without bypassing system ownership rules.

**Scope:**
- Implement `src/api/controller.rs`, `src/api/fallback.rs`, `src/api/worker.rs`, `src/api/mod.rs`.
- Implement `src/debug_api/snapshot.rs`, `src/debug_api/worker.rs`, `src/debug_api/mod.rs`.
- Add `post_switchover`, `get_fallback_cluster`, `post_fallback_heartbeat`, `build_snapshot`, and worker loops.

**Context from research:**
- API must write switchover requests through DCS adapter, not direct HA mutation.

**Expected outcome:**
- Controller/fallback endpoints and debug snapshots are typed, tested, and aligned to worker ownership boundaries.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] API request/response models are typed and validated.
- [ ] Debug snapshot includes app/config/pg/dcs/process/ha versioned states.
- [ ] Integration tests verify API requests influence HA only via DCS state changes.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] If failing, create `$add-bug` tasks with endpoint payload and response evidence.
</acceptance_criteria>
