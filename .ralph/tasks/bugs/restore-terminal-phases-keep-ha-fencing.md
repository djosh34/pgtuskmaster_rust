---
## Bug: Restore terminal phases keep HA in repeated fencing <status>blocked</status> <passes>false</passes>

<blocked_by>05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the backup-removal story and the HA functional rewrite story are both fully complete. The restore control plane is being deleted, and the remaining HA core is being restructured; fixing this in the old design first would likely be throwaway work.

Reassess this bug only after those stories complete through their final tasks. Expected outcomes then:
- most likely the bug is obsolete because restore takeover no longer exists,
- otherwise a much smaller residual HA liveness bug should be filed against the surviving post-removal design.

Current concern recorded here: `src/ha/decide.rs` treats restore as HA-blocking for every phase except `Completed`, even though `Failed`, `Cancelled`, and `Orphaned` are modeled as terminal/non-blocking outcomes. That can keep nodes in repeated restore-specific fencing behavior and can cause the restore takeover e2e to wait forever for `Completed` instead of converging or failing cleanly.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
