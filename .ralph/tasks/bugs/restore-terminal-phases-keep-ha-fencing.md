---
## Bug: Restore terminal phases keep HA in repeated fencing <status>not_started</status> <passes>false</passes>

<description>
`src/ha/decide.rs` treats restore as HA-blocking for every phase except `Completed`, even though `Failed` and `Cancelled` are already modeled as terminal and `Orphaned` is documented as the state that should stop blocking HA forever. That leaves non-executors in the restore guard fencing path and keeps the executor in restore-specific fencing branches on every tick. The real-binary test `ha::e2e_multi_node::e2e_multi_node_restore_takeover_external_repo_converges_cluster` then waits only for `Completed`, so failure/orphan paths can degenerate into a long fencing loop until timeout. Explore the restore guard flow and fixture polling first, then fix the liveness bug.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
