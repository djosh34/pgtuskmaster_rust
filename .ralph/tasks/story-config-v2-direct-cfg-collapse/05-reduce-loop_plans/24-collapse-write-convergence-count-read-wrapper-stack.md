## Plan: Collapse Write Convergence Count Read Wrapper Stack

### Why this reduction target

The stronger reduction target is still local to `tests/ha/support/invariants/write_convergence.rs`, but it is broader than the failed runtime helper:

- `read_monitored_member_counts(...)` fans out into `read_member_count(...)`, which immediately forwards into `read_member_count_via_fresh_connection(...)`.
- `read_member_count_via_fresh_connection(...)` then delegates routing-target resolution to `resolve_observation_routing_target(...)` and delegates the actual fresh read to `read_count_via_fresh_connection_target(...)`.
- `convergence_expectation(...)` repeats the same stack from the other side by calling `authoritative_reconciliation_target(...)`, then `read_count_via_target(...)`, which is only a thin wrapper over `read_count_via_fresh_connection_target(...)`.

That means one local concern, "resolve the right routing target, do one fresh count read, and adapt failures for the caller", is currently split across five small helpers and two separate call chains.

### Current overlap already verified

- `read_member_count(...)` has no behavior beyond forwarding `None` into `read_member_count_via_fresh_connection(...)`.
- `resolve_observation_routing_target(...)` exists only to choose `observer.postgres_routing_target(...)` or clone the already-selected member target.
- `read_count_via_target(...)` exists only to erase the `(&'static str, String)` stage from `read_count_via_fresh_connection_target(...)`.
- `authoritative_reconciliation_target(...)` has a single caller in `convergence_expectation(...)`.
- The real reusable boundary is already `read_count_via_fresh_connection_target(...)`; the extra wrappers above it are mostly call-site shuffling and error reformatting.

### Execution plan

1. Collapse the observation-side wrapper stack.
   - Remove `read_member_count(...)`.
   - Rebuild `read_monitored_member_counts(...)` directly on the one remaining observation helper.
   - Fold `resolve_observation_routing_target(...)` into that observation helper so it owns routing-target refresh and previous-error composition in one place.

2. Collapse the authoritative reconciliation wrapper stack.
   - Inline the single-call-site `authoritative_reconciliation_target(...)` logic into `convergence_expectation(...)`.
   - Call the low-level fresh-read helper directly from `convergence_expectation(...)` instead of routing through `read_count_via_target(...)`.
   - Keep the current error wording stable by preserving the same `WriteConvergenceInvariantError::Failed(...)` strings at the outer boundary.

3. Keep only the boundary that is actually shared.
   - Preserve one low-level helper for "connect to this concrete routing target, read the count once, abort the connection task".
   - Do not introduce any new enum, struct, or cross-file abstraction.
   - Prefer renaming an existing helper over adding a second helper if the rename makes the call sites shorter.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+5061 -7411 diff: -2350` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Keep the refactor local to `tests/ha/support/invariants/write_convergence.rs`.
- Do not reintroduce a generic routing helper; the goal is to remove the wrapper stack, not move it.
- If inlining the authoritative target selection makes `convergence_expectation(...)` materially harder to read, keep one helper there and instead collapse the observation-side wrappers first.
- If the low-level fresh-read helper needs to grow caller-specific formatting logic, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
