## Plan: Collapse DCS Service Start/Stop Wrapper Duplication

### Why this reduction target

The next wrong-place duplication is the DCS start/stop wrapper block in `tests/ha/support/world/mod.rs`:

- `tests/ha/support/world/mod.rs` still owns six thin wrappers: `stop_all_dcs_services`, `start_all_dcs_services`, `stop_dcs_quorum_majority`, `start_dcs_quorum_majority`, `stop_member_local_dcs`, and `start_member_local_dcs`.
- Those methods do not own new behavior. They only re-select DCS services through `self.given.dcs_services()`, `self.given.quorum_majority_dcs_services()`, and `self.given.local_dcs_service_for(member)`, then forward into the same `start_service` / `stop_service` orchestration.
- `tests/ha/support/givens/mod.rs` already owns the authoritative DCS topology choices. The world owner should not keep repeating which DCS slice or member-local DCS service to act on when it already knows how to start or stop a service name.

### Current overlap already verified

- `tests/ha/support/givens/mod.rs` already owns `dcs_services`, `quorum_majority_dcs_services`, and `local_dcs_service_for`.
- `tests/ha/support/steps/mod.rs` is the only caller of the six world wrapper methods, and each call site maps directly to one of those existing given-owned service selections.
- `tests/ha/support/world/mod.rs` already has the real behavior in `start_service` and `stop_service`; the DCS-specific wrappers are selection glue only.

### Execution plan

1. Collapse the duplicated DCS world wrappers into generic service-state helpers.
   - Add a small world helper that applies `start_service` or `stop_service` across a DCS service iterator.
   - Add a small world helper for toggling one service by name or `DcsService`.
   - Delete the six duplicated public wrappers once callers use the generic helpers.

2. Let steps choose the given-owned DCS targets directly.
   - Update `tests/ha/support/steps/mod.rs` to call the generic world helpers while choosing the correct DCS slice through `harness.given`.
   - Reuse the existing given-owned selectors instead of introducing a new enum for DCS action scope.

3. Keep coverage where the real behavior lives.
   - Do not add new wrapper tests for deleted DCS methods.
   - If any focused tests are needed, keep them around the generic service-state helpers or existing step behavior, not around semantic aliases we just removed.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4723 -7090 diff: -2367` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Update the reduce-loop task state only after all gates pass.

### Guardrails

- Do not introduce a new DCS scope enum just to replace six wrappers; reuse the existing given-owned selectors directly.
- Keep `tests/ha/support/world/mod.rs` responsible for service start/stop behavior and runtime orchestration, not for restating DCS topology choices.
- If this starts pulling scenario parsing concerns into world helpers or duplicating given-owned DCS selections inside steps, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
