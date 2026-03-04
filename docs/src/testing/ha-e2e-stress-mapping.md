# HA no-quorum e2e stress mapping

This document maps the previous ultra-long “no quorum fencing under load” HA scenario to the current short, parallel-safe real-binary tests.

## What changed

**Old ultra-long test (removed):**

- `ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql`

This scenario was evidence-backed as the only consistently 3min+ HA e2e test and was the primary blocker for parallel developer loops.

**New short real-binary tests (regular `make test`):**

- `ha::e2e_multi_node::e2e_no_quorum_enters_failsafe_strict_all_nodes`
- `ha::e2e_multi_node::e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`

## Assertion coverage mapping

| Prior assertion / intent | Covered by new test(s) | Notes |
|---|---|---|
| Cluster boots and reaches a stable primary | `e2e_no_quorum_enters_failsafe_strict_all_nodes`, `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Both tests begin with stable-primary bootstrap. |
| Etcd majority loss stimulus | Both | Both tests stop an etcd majority. |
| Fail-safe is observed after quorum loss | Both | Uses a strict helper requiring **all nodes** to report `FailSafe` and no primary. |
| HA sample window does not observe dual primary | Both | Both tests sample `/ha/state` during/after the transition and assert no dual-primary in the window. |
| Concurrent SQL workload under quorum loss | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Workload runs during the majority-loss transition. |
| Workload commits occur (>0) | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Required to avoid “all writes rejected” false positives. |
| Write rejections occur (>0) during fail-safe window | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Ensures fencing/transient failures are exercised. |
| After fail-safe + grace, commits are near-zero (<= tolerance) | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Maintains the post-cutoff commit tolerance check. |
| No split-brain write evidence (no duplicate committed keys; no hard SQL failures) | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | Keeps the existing workload evidence assertions. |
| Best-effort key integrity probe (when a Postgres is reachable) | `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` | If any node remains queryable, validates no duplicate `(worker_id,seq)` rows and at least 1 row. |
| Artifacts written + cluster shutdown even on failure | Both | Both tests write stress artifacts and always shut down the fixture. |

## Thresholds and timing notes

The new tests intentionally reduce fixed wall-clock sleeps and rely on explicit per-step timeouts to stay under regular `make test` runtime policy:

- The “stable primary” wait is bounded (60s in both tests).
- The “failsafe observed” wait is bounded (60s in the strict-all-nodes test).
- HA sampling windows are short (4s in the strict-all-nodes test; 2s in the fencing+workload test).
- The fencing cutoff window uses a 7s grace and an 8s tail sleep before measuring post-cutoff commits.
- Post-cutoff commit tolerance remains `allowed_post_cutoff_commits = 10`.
