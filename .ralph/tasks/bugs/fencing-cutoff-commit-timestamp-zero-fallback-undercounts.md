---
## Bug: Fencing cutoff commit timestamp fallback undercounts post-cutoff commits <status>not_started</status> <passes>false</passes>

<description>
In `src/ha/e2e_multi_node.rs`, successful SQL commits record `committed_at_unix_ms` using `ha_e2e::util::unix_now()`, but on error the code falls back to `0` (`Err(_) => 0`).

The no-quorum fencing assertion computes post-cutoff commits using `timestamp > cutoff_ms`. Any commit with fallback timestamp `0` is silently excluded, which can undercount post-cutoff commits and weaken (or falsely pass) the safety assertion.

Please explore and research the codebase first, then implement a fail-closed fix that does not use unwrap/panic/expect:
- avoid sentinel `0` timestamps for committed writes,
- either propagate timestamp capture failures or explicitly fail sampling/assertion when timestamps are incomplete,
- consider a monotonic-time based alternative for cutoff comparisons,
- add/update focused tests for regression coverage.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
