---
## Bug: Real-binary tests fail when port allocation is blocked <status>not_started</status> <passes>false</passes>

<description>
`make test` is not passing in the current environment because multiple tests panic when `allocate_ports(...)` returns `io error: Operation not permitted (os error 1)`.

Detected on 2026-03-02 with:
- `make test` (failed/terminated after reporting multiple failures and a long-running test)
- `cargo test test_harness::ports::tests::allocate_ports_returns_unique_ports -- --nocapture`
- `cargo test test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns -- --nocapture`
- `cargo test pginfo::worker::tests::step_once_maps_replica_when_polling_standby -- --nocapture`
- `cargo test test_harness::ports::tests::concurrent_allocations_do_not_collide_while_reserved -- --nocapture`

Representative failures:
- `src/test_harness/ports.rs:76` and `src/test_harness/ports.rs:103` panic on `Operation not permitted (os error 1)`.
- `src/test_harness/etcd3.rs:214` fails with `allocate ports failed`.
- `src/pginfo/worker.rs:239` fails with `port allocation failed`.
- Process worker real-binary tests (`real_demote/promote/restart/start-stop/fencing`) fail in the same gate because they depend on port allocation.

Please explore and research the test harness and real-binary test code paths first, then implement a fix so tests fail deterministically for product regressions rather than environment socket-policy artifacts.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
