## Bug: Targeted Switchover Request Can Promote Wrong Node <status>not_started</status> <passes>false</passes>

<description>
An accepted targeted switchover request is not reliably honored in the HA multi-node E2E environment. During work on repeated leadership-churn coverage, a request targeted at `node-2` was accepted through `POST /switchover`, but the cluster later stabilized on `node-3` as primary instead. The failure was reproduced in `e2e_multi_node_repeated_targeted_switchovers_preserve_single_primary`, which observed `node-3` as the only stable promoted primary after the targeted request to `node-2`.

The current behavior contradicts the operator/docs contract for targeted switchovers and strongly suggests the switchover request is being cleared or ignored before the intended successor has actually taken over. Explore and research the HA decision/apply path first, then fix the product behavior and add focused coverage that proves non-target nodes cannot win leadership while a targeted switchover is pending.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
