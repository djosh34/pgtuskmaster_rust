## Plan: Collapse Write Convergence Connect Member Duplication

### Why this reduction target

The next wrong-place overlap is the split TLS vs non-TLS connection path in `tests/ha/support/invariants/write_convergence.rs`:

- `connect_member` duplicates the same connect timeout mapping, connection-task spawn, `SELECT 1` probe, abort-on-probe-failure handling, and identical error rendering in both branches.
- The real variation is only connector selection: `build_tls_connector(&routing_target.conninfo)` versus `NoTls`.
- This is one shared connection shape living in two branches inside the same local helper.

### Current overlap already verified

- Both branches render the same `connect_dsn` from `connectable_conninfo`.
- Both branches use the same member-specific timeout and failure messages for connect and probe failures.
- Both branches wrap the `Client` in `Arc`, spawn the connection task, and abort it on failed probe.

### Execution plan

1. Extract one local helper for the shared connect-and-probe flow.
   - Keep the helper inside `tests/ha/support/invariants/write_convergence.rs`.
   - Let `connect_member` choose only the connector and pass it into the shared path.

2. Preserve behavior exactly while reducing the branch duplication.
   - Keep the current error strings for connect timeout, connect failure, and probe timeout/failure.
   - Keep the current `SELECT 1` probe and connection-task abort behavior unchanged.

3. Keep the helper narrow.
   - Do not introduce a new public connection type or move conninfo logic into another module.
   - Do not widen this into a generic rewrite of all write-convergence connection helpers unless that directly removes more code without new abstraction weight.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4929 -7339 diff: -2410` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Keep the refactor local to `write_convergence.rs`.
- Keep connector choice as the only branch-specific concern.
- If this starts requiring a new connection enum or pushing conninfo/TLS policy across modules, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
