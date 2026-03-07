## Bug: No-quorum integrity check does not verify committed keys <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/multi_node.rs` records `committed_writes`, `committed_keys`, and commit timestamps during `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`, but the final verification only calls `assert_table_key_integrity_strict(...)`, which succeeds when any reachable node reports `COUNT(*) >= min_rows` and no duplicate `(worker_id, seq)` rows.

That means the scenario can pass without proving that the recovered table contains exactly the keys that were acknowledged before fencing. It does not detect lost committed rows, unexpected extra rows, or divergence between the in-memory workload ledger and the post-recovery table contents.

Explore and research the codebase first, then fix. Focus on restoring a real post-recovery path for this scenario and adding an assertion that compares the recovered table contents against the expected committed key set captured by the workload.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
