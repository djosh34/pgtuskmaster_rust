## Bug: Greenfield storage stall does not trigger primary replacement <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_primary_storage_stall_replaced_by_new_primary` now reaches a trustworthy product failure: wedging the current primary does not cause the cluster to replace it with a different primary.

Observed on March 10, 2026 from:
- `make test-long`
- failing log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_primary_storage_stall_replaced_by_new_primary__primary_storage_stall_replaced_by_new_primary.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary as `initial_primary`
- created the proof table
- inserted `1:before-storage-stall`
- recorded the `storage_stall` marker
- injected the wedge fault into `initial_primary`

The scenario then failed on the failover assertion:
- step failure: `Then I wait for a different stable primary than "initial_primary" as "final_primary"`
- observed error: `failover deadline expired; last observed error: expected a different primary than \`node-b\`, observed \`node-b\``

This is trustworthy product evidence because the harness applied the intended wedged-primary fault and the failure happened only when the product failed to move authority away from the stalled primary. Explore and research the codebase first, then fix the wedged-primary handling so storage or WAL stall conditions cause a safe primary replacement instead of leaving the wedged node authoritative.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
