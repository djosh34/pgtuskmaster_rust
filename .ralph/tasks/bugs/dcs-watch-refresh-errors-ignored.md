---
## Bug: DCS watch refresh errors are tracked but ignored <status>not_started</status> <passes>false</passes>

<description>
`refresh_from_etcd_watch` in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) records `had_errors` (for unknown keys or decode failures) but no caller uses it. In [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), `step_once` only checks for `Err`, so unknown/malformed watch events can be silently ignored while the worker still reports healthy state. Decide on the correct behavior (e.g., mark store unhealthy, emit faulted state, or log/telemetry), and wire `had_errors` into worker health so errors do not pass silently.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
