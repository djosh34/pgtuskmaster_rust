## Bug: HA rejoin assertion accepts `unknown` role as success <status>not_started</status> <passes>false</passes>

<description>
The HA cucumber assertion for `the node named "<member>" rejoins as a replica` is too weak.

Research first, then fix.

Current behavior:
- `assert_member_is_replica_via_member(...)` in `cucumber_tests/ha/support/steps/mod.rs` accepts `member.role == "unknown"` as success if `connection_target_for_member(...)` returns a target for that member.
- `connection_target_for_member(...)` always falls back to `direct_connection_target(member_id)` when the member is not returned by `pgtm` helper discovery, so this check is effectively always satisfied for any sampled `unknown` member.
- This means a restarted node can still be in an API-reported `unknown` HA role and the cucumber step can still pass.

Relevant context:
- `src/cli/status.rs` `observed_role(...)` intentionally reports `unknown` for several non-steady HA phases, including `init`, `candidate_leader`, `rewinding`, `bootstrapping`, `fencing`, and `fail_safe`, and only reports `replica` in steady-state cases.
- A real run of `ha_replica_stopped_primary_stays_primary` showed `status.replica.node-c` with `sampled: true` and `role: "unknown"` immediately after restart, while the step still passed.

The fix should make the test contract honest: either require an explicit stable `replica` role before passing, or split the semantics so a weaker transitional-state assertion is named differently and the current `rejoins as a replica` wording remains strict.
</description>

<acceptance_criteria>
- [ ] `the node named "<member>" rejoins as a replica` no longer passes when the sampled member role is `unknown`
- [ ] `cucumber_tests/ha/support/steps/mod.rs` no longer uses the unconditional direct-DSN fallback as proof that an `unknown` node has rejoined as a replica
- [ ] Any needed replacement assertion logic distinguishes transitional API reachability from steady replica state
- [ ] HA ultra-long tests that rely on replica rejoin semantics are updated to reflect the corrected contract
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

TO BE VERIFIED
