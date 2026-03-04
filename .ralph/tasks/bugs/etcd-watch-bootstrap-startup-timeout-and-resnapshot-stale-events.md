---
## Bug: Etcd watch bootstrap can hang startup and resnapshot can replay stale events <status>not_started</status> <passes>false</passes>

<description>
The etcd DCS store watch worker has subtle correctness issues in bootstrap/reconnect handling.

Detected during code audit of `src/dcs/etcd_store.rs`:
- `EtcdDcsStore::connect` waits only `COMMAND_TIMEOUT` for worker startup and then `join()`s the worker thread on timeout. If bootstrap (`connect + get + watch`) takes longer than that timeout, the join can block indefinitely while the worker continues running.
- On watch reconnect/resnapshot, bootstrap snapshot events are appended to the existing queue without clearing/draining stale pre-disconnect events. This can replay stale PUT events that should have been superseded by deletes included in the snapshot state.

Please explore and research existing DCS/watch semantics in the codebase first, then fix implementation and tests.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
