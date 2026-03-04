---
## Bug: HA e2e false-pass via best-effort polling and timestamp fallback <status>not_started</status> <passes>false</passes>

<description>
HA e2e assertions can pass without reliable cluster-wide observations during unstable windows.

Detected during review of plan section 2.4 against:
- src/ha/e2e_multi_node.rs:1395-1410 (`assert_no_dual_primary_window` ignores polling errors and returns success if every poll fails)
- src/ha/e2e_multi_node.rs:1412-1458 (`wait_for_all_failsafe` is explicitly best-effort but is used with all-node language in scenario logs)
- src/ha/e2e_multi_node.rs:2099-2100 (matrix scenario logs "fail-safe observed on all nodes" after calling `wait_for_all_failsafe`)
- src/ha/e2e_multi_node.rs:2484-2500 (post-failsafe verification path tolerates per-node poll errors and still can proceed)
- src/ha/e2e_multi_node.rs:1718-1721 (`unix_now()` failure mapped to `0` for commit timestamps)
- src/ha/e2e_multi_node.rs:2619-2629 (fencing cutoff assertion depends on commit timestamps; `0` fallback can undercount post-cutoff commits)

Ask the fixing agent to explore and research the codebase first, then implement a minimal, deterministic fix set.

Expected direction:
- Make dual-primary window assertions fail closed if sample reliability is insufficient (e.g. zero successful samples, excessive poll errors, or missing-node coverage).
- Align no-quorum helper semantics and scenario claim text: either require strict all-node observation or explicitly downgrade assertion wording + criteria.
- Remove/replace `unix_now()` fallback-to-0 in fencing-sensitive commit timestamp collection; propagate hard error or mark sample invalid and fail the assertion.
- Add/adjust tests to ensure these paths cannot silently pass under repeated API transport failures.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
