## Bug: HA storage-stall failover scenario can stall with no authoritative primary <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
`make test-long` is currently not reliably green because `ha_primary_storage_stalled_then_new_primary_takes_over` can fail waiting for a replacement primary.

This bug was detected during validation for `.ralph/tasks/story-general-architecture-improvement-finding/01-task-find-general-architecture-privacy-and-deduplication-improvements-and-create-follow-up-tasks.md`, which only changed markdown task files. `make check`, `make lint`, and `make test` all passed, but `make test-long` failed twice in the same scenario. One targeted rerun of the single scenario passed in between, which suggests the current failure mode may be timing-sensitive or flaky, but it is still a real repo bug because the required long suite is not stable.

Observed failure details from `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha__ha_primary_storage_stalled_then_new_primary_takes_over.log`:
- feature: `tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature`
- failing step: `Then I wait for a different stable primary than "initial_primary" as "final_primary"`
- captured error: `failover deadline expired; last observed error: cluster has no authoritative primary; authority=no_primary(leaseopen) warnings=authority=no_primary(leaseopen)`

The next agent must explore and research the codebase first, then fix. Do not assume the issue is only in the test. Investigate HA decision/state publication, DCS authority handling, and/or harness timing around the wedged-primary path before choosing a solution.
</description>

<acceptance_criteria>
- [x] Reproduce and understand why `ha_primary_storage_stalled_then_new_primary_takes_over` can end in `authority=no_primary(leaseopen)` instead of converging to a new stable primary.
- [x] Fix the underlying issue in product code and/or the HA harness only after research shows which layer is actually wrong.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Research summary
- The current HA model represents the same coordination facts in multiple places: `GlobalKnowledge` carried `lease`, `observed_lease`, `observed_primary`, and `coordination`, while publication split authority from `fence_cutoff`.
- That allows contradictory states such as “lease is effectively unheld” while a stale observed lease epoch still exists, which then leaks into the decision code as `no_primary(leaseopen)`.
- `build_global_knowledge` currently collapses “DCS says the lease is still held but the holder is not a ready primary” into the same branch as “the lease is actually open”.
- The harness is mainly surfacing the product-state ambiguity. It requires authoritative publication and target resolution to agree; the failing symptom is consistent with product code remaining in an ambiguous no-primary projection rather than a pure harness timing problem.

### Type design completed in this pass
- `src/ha/types.rs` now reshapes the relevant types first, before any compile-fix work:
  - publication is now `PublicationState::Unknown | PublicationState::Projected(AuthorityProjection)`
  - authoritative publication is now `AuthorityProjection::Primary(LeaseEpoch) | AuthorityProjection::NoPrimary(NoPrimaryProjection)`
  - no-primary causes are explicit ADTs, including `LeaseOpen`, `Recovering`, `DcsDegraded`, `StaleObservedLease`, and `SwitchoverRejected`
  - fence state is attached only to the no-primary variants where it is meaningful
  - global coordination no longer has duplicated top-level lease/primary option fields; it now uses `CoordinationState { trust, leadership, primary }`
  - leadership is now an explicit ADT: `Open`, `HeldBySelf`, `HeldByPeer { epoch, state }`, or `StaleObservedLease { epoch, reason }`
- This intentionally breaks compilation. The next turn must adapt the worker, decision, reconcile, API/CLI, and test code to the new type model instead of trying to preserve the old ambiguous state combinations.

### Execution plan
1. Update HA observation building in `src/ha/worker.rs` so DCS leader sampling constructs the new `LeadershipView` and `PrimaryObservation` without reintroducing duplicated coordination state.
2. Refactor `src/ha/decide.rs` so `LeaseOpen` is only emitted when leadership is actually `Open`, and so stale/unready observed leases map to explicit `NoPrimaryProjection` states instead of `LeaseOpen`.
3. Refactor `src/ha/reconcile.rs`, `src/ha/state.rs`, and HA state builders/tests to consume `PublicationState` and `AuthorityProjection` directly.
4. Update API/CLI surfaces and HA harness helpers that currently pattern-match on `AuthorityView` or detached `fence_cutoff` fields.
5. Rework unit and integration expectations so the wedged-primary path asserts the new explicit stale/recovering publication semantics rather than the old ambiguous `no_primary(leaseopen)` state.
6. Run the required validation gates in repo-preferred order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after all checks pass, update docs for any behavior or terminology changes using the `k2-docs-loop` skill, remove stale docs if needed, then complete task closeout (`<passes>true</passes>`, task switch, commit, push).

### Completion notes
- The root HA ambiguity was removed by the new authority/leadership ADTs, and the long-suite regressions uncovered during execution were fixed in product code.
- Replica upstream resolution now uses managed `primary_conninfo`, and local recovery planning now compares PostgreSQL `system_identifier` so mismatched clusters are rebuilt with `basebackup` instead of being treated as safe replica restarts.
- Validation completed successfully with `make check`, `make lint`, `make test`, and `make test-long`.
- Docs were updated in `docs/src/reference/ha-decisions.md` to reflect the new publication shape and the identity-based `basebackup_required` recovery rule.

### Constraints for execution
- Do not revert the type redesign back to the legacy option/duplicate-field model.
- If the new ADT still proves insufficient during implementation, switch this task back to `TO BE VERIFIED`, describe the gap, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

NOW EXECUTE
