---
## Bug: Test harness binary checks panic instead of returning errors <status>not_started</status> <passes>false</passes>

<description>
The test harness binary lookup in [src/test_harness/binaries.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/binaries.rs) uses `panic!` to report missing binaries. This conflicts with the project policy of no `panic`/`expect`/`unwrap` and makes tests fail via uncontrolled panics rather than structured errors. Refactor `require_binary` (and callers) to return a typed `HarnessError` instead of panicking, and update callers/tests to propagate or assert errors explicitly.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
