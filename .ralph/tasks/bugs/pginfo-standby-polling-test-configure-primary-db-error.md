---
## Bug: Pginfo standby polling test fails during primary configure with db error <status>not_started</status> <passes>false</passes>

<description>
`make test` failed in `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` with a runtime panic while preparing the primary postgres fixture.

Repro:
- `make test`
- Failing test:
- `pginfo::worker::tests::step_once_maps_replica_when_polling_standby`

Observed trace excerpt:
- `configure primary failed: db error`

This appears in the test setup path for standby polling, before the HA/worker assertions complete. Please explore and research the codebase first, then implement a robust fix with deterministic setup/teardown and clear error propagation for fixture configuration.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
