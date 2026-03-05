---
## Task: Add distributed backup scheduling and runtime packaging for pgBackRest <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Integrate scheduled backups with tokio-cron-scheduler under DCS coordination, and ensure pgBackRest is available by default in runtime/container environments.

**PO Directive (2026-03-05):** Use pgBackRest config-method ownership only. Do not rely on repo-local wrapper/hack paths; use minimal CLI flags, with behavior/config sourced from managed config surfaces first.

**Scope:**
- Add scheduler worker based on tokio-cron-scheduler for cron-like backup schedules.
- Coordinate scheduling ownership with DCS so only one eligible node executes each scheduled run.
- Define overlap/missed-run policy (single-flight, catch-up, jitter, and timeout behavior).
- Add runtime/container/tooling packaging so pgBackRest binary is available where pgtuskmaster runs by default.
- Keep manual and API-triggered backup paths consistent with scheduled execution path.

**Context from research:**
- pgBackRest intentionally does not ship a built-in scheduler, so an external scheduler is required: https://pgbackrest.org/user-guide.html
- Existing system already has DCS membership/leadership and can host scheduler lease ownership.
- Existing `process.binaries` model is explicit and should include pgBackRest similarly to other required binaries.

**Expected outcome:**
- Backup schedules are deterministic and cluster-safe (no duplicate concurrent backups from multiple nodes).
- Operators can configure schedule ownership policy via config, with clear failover behavior.
- Default runtime images/tooling include pgBackRest out of the box.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add scheduler dependency/wiring:
- [ ] update `Cargo.toml` and lockfile for `tokio-cron-scheduler` (and any needed companion crates)
- [ ] add scheduler worker module(s), e.g. `src/backup/scheduler.rs`
- [ ] integrate scheduler lifecycle into runtime startup/shutdown (`src/runtime/node.rs` and related worker wiring)
- [ ] Extend config schema/parser/docs for scheduling block:
- [ ] cron expression(s), timezone policy, enable/disable flags
- [ ] ownership policy (leader-only vs lease-holder)
- [ ] overlap policy and command timeout controls
- [ ] Extend DCS model/store if needed for scheduler ownership/lease records:
- [ ] strict single owner at a time
- [ ] resilient behavior across reconnect/resnapshot
- [ ] no stale queued events causing duplicate scheduling
- [ ] Add scheduling tests:
- [ ] unit tests for cron parsing/next-run calculations and overlap guard behavior
- [ ] integration tests for DCS ownership failover and single-flight backup execution
- [ ] real e2e test proving schedule continues after node role/leader changes
- [ ] Add packaging/install updates:
- [ ] add pgBackRest install script under `tools/` (or extend existing tooling)
- [ ] ensure container/runtime image definitions include pgBackRest binary by default
- [ ] update quick-start/prereq docs to include pgBackRest default availability expectations
- [ ] Add/extend command-line/admin trigger docs to clarify scheduled vs manual backup behavior
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
