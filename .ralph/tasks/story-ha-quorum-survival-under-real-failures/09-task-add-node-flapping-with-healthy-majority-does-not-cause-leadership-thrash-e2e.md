## Task: Add Node Flapping With Healthy Majority Does Not Cause Leadership Thrash E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add a real-world instability scenario where one node repeatedly drops and returns while the other two remain healthy, and leadership must not thrash unnecessarily. The higher-order goal is to validate stable HA behavior under noisy infrastructure rather than one clean outage.

**Scope:**
- Extend HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failover.rs`
- relevant harness stop/restart helpers
- Add a scenario where one non-critical node repeatedly flaps while a healthy majority remains available.
- Prefer starting with a replica flap rather than a leader flap so the expected stable outcome is unambiguous.
- Reuse the explicit whole-node outage semantics from task 02. This task does not need to cover both clean-stop and hard-kill flap variants by itself, but whichever flap style it uses must be a real whole-node outage and restart path, not only PostgreSQL stop/start.
- The scenario must verify the cluster remains at exactly one primary, does not trigger unnecessary leadership churn, and continues to accept writes.

**Context from research:**
- Real infrastructure often fails by flap, not by one permanent outage.
- The current suite covers single transitions and some churn, but not repeated nuisance instability with a healthy majority staying up.

**Expected outcome:**
- The suite proves that node flap does not translate into needless leadership movement when a stable majority is intact.
- Operators gain confidence that noisy nodes do not destabilize the cluster unnecessarily.

</description>

<acceptance_criteria>
- [ ] Add at least one scenario where one replica repeatedly goes fully down and comes back while the other two nodes remain online, using one of the explicit whole-node outage variants from task 02.
- [ ] The scenario must prove the primary does not change during the flap window unless an explicitly justified failure forces it.
- [ ] The scenario must include proof writes during the flap window and verify they continue succeeding on the stable primary.
- [ ] The scenario must assert no dual-primary window and final proof-row convergence after the flapping node stabilizes.
- [ ] Timeline artifacts must record each flap event and whether leadership changed; the expected passing case is zero unnecessary leadership changes.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
