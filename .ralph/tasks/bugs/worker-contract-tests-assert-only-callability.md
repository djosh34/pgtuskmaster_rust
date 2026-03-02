---
## Bug: Worker contract tests only assert callability <status>not_started</status> <passes>false</passes>

<description>
[worker_contract_tests.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/worker_contract_tests.rs) primarily asserts that `step_once` functions are callable and return `Ok(())`, without validating resulting state changes or side effects. This means tests can pass even if core worker logic regresses or stops mutating state. Strengthen these tests with minimal behavioral assertions (state version bump, expected phase transitions, or expected publish effects), or split compile-time contract checks into non-test compile gates and add real behavioral tests.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
