## Bug: HA write-convergence table init swallows SQL errors <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/invariant.rs` ignores SQL failures while trying to create the write-convergence table:
`initialize_write_convergence_table` loops over members and uses `.is_ok()` on `sql.execute(...)`, which drops the actual failure details and keeps retrying without surfacing why initialization is failing.

This violates the repo rule against swallowing errors and makes HA invariant startup failures much harder to diagnose.

Explore and research the codebase first, then fix this so create-table failures are preserved and reported with enough context instead of being silently discarded.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
