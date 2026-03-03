---
## Bug: Remove writable HA leader API control path and enforce HA-loop-only leadership transitions <status>not_started</status> <passes>false</passes>

<description>
Investigation found that writable `/ha/leader` was introduced by task `22-task-ha-admin-api-read-write-surface` as part of a "full HA admin API read and write surface". In runtime code, `src/api/worker.rs` routes `POST /ha/leader` and `DELETE /ha/leader` to controller handlers in `src/api/controller.rs` that call `DcsHaWriter::write_leader_lease` / `delete_leader`, so external callers can directly mutate the leader key outside autonomous HA-loop decision flow.

This conflicts with lease/autonomous leadership expectations and enables direct DCS steering through API, including in e2e scenario code.

Research first, then fix end-to-end:
- Remove writable `POST /ha/leader` and `DELETE /ha/leader` runtime routes and associated controller handlers.
- Keep read surfaces (`/ha/state`, debug reads) and switchover request semantics unless separately deprecated by another decision.
- Remove CLI commands/client methods that call writable `/ha/leader` APIs.
- Migrate tests and e2e scenarios that currently depend on `/ha/leader` mutation to HA-loop-driven transitions (failure injection + switchover + convergence observation).
- Update policy guard tests so e2e cannot reintroduce writable `/ha/leader` steering.
- Update published API endpoint contracts/lists so writable `/ha/leader` is no longer advertised.

Directly impacted usages discovered during investigation:
- `tests/bdd_api_http.rs` route/auth mutation checks for `/ha/leader`
- `src/api/worker.rs` tests that exercise `/ha/leader` routing/auth behavior
- `src/api/controller.rs` tests for `post_set_leader` and `delete_leader`
- `src/cli/mod.rs` and `src/cli/client.rs` `ha leader set/clear` command paths/tests
- `src/ha/e2e_multi_node.rs` scenario matrix uses `POST /ha/leader` and `DELETE /ha/leader` to inject leader conflicts and force failover steering
</description>

<acceptance_criteria>
- [ ] Writable `/ha/leader` API surface is removed from runtime routing, controller handlers, and published debug API endpoint listing.
- [ ] CLI no longer exposes leader set/clear commands that mutate DCS leader key directly.
- [ ] HA e2e scenario(s) prove switchover/failover/fencing behavior without `/ha/leader` manual writes; transitions are driven by HA loop and external failure/switchover stimuli only.
- [ ] Tests previously validating `/ha/leader` writes/deletes are replaced with tests validating forbidden/absent route behavior and HA-loop outcomes.
- [ ] Policy guard coverage fails if e2e code reintroduces `/ha/leader` write/delete steering.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
