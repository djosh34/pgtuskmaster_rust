## Bug: Ultra-long failsafe integrity scenario flakes during final table verification <status>not_started</status> <passes>false</passes>

<description>
`make test-long` failed during `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` while this task was being verified on March 7, 2026.

Observed failure:
- the first full ultra-long run failed in `tests/ha/support/multi_node.rs` final integrity verification
- `assert_table_key_integrity_strict(...)` could not verify the workload table on any node
- node-1 and node-3 returned `FATAL: could not open file "global/pg_filenode.map": No such file or directory`
- node-2 refused TCP connections
- rerunning the exact test in isolation passed, and rerunning the full `make test-long` gate also passed, so this currently looks like a flaky scenario or verification strategy rather than a deterministic regression

Evidence from the failing run:
- junit: `target/nextest/ultra-long/junit.xml`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_multi_node_failsafe__e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity.log`
- stress artifacts:
  - `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843365646-0-1772843403435.timeline.log`
  - `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843365646-0-1772843403435.summary.json`

Explore and research the codebase first, then fix. Focus on whether the scenario is leaving nodes in a legitimately unreadable state, whether the final verification should wait for a narrower recovery condition, or whether the HA/process path is corrupting the data directory under this no-quorum fencing sequence.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
