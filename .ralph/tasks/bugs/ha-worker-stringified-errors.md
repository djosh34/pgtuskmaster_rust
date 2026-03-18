## Bug: HA worker still flattens recoverable errors into strings <status>not_started</status> <passes>false</passes>

<description>
`src/ha/startup.rs` and `src/ha/state.rs` already carry `WorkerError` through the HA runtime boundary, but `src/ha/worker.rs` still collapses several distinct failure modes into `WorkerError::Message(format!(...))`.

This loses structured error context for subscriber shutdowns, state publication failures, DCS leadership operations, local role reconciliation, and process dispatch errors. Explore the HA codebase first, confirm the typed-error shape already available in `crate::state::WorkerError` and related modules, then replace the string-only conversions with typed variants or a narrower error enum where appropriate.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
