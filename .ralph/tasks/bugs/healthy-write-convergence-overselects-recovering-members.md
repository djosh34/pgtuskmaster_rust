## Bug: Healthy write convergence over-selects recovering members <status>completed</status> <passes>true</passes>

<description>
`make test-long` exposed two remaining failures after moving strong write-convergence checks out of cleanup and into `cluster becomes healthy`:

- `ha_rejoin_and_restart_recovery::blocked_basebackup_recovery_recovers_after_unblock`
- `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority`

In both scenarios, the health step finds a healthy authoritative primary, but the follow-up accepted-write convergence check still selects a member whose Postgres probe is not yet reconnectable (`node-a` / `node-b` connection errors) even though the surviving healthy members already agree on the fixture row count.

Explore the health-step membership boundary first. The strong accepted-write convergence set should be derived from the same successful healthy observation/probe boundary, not from `HaWorld::online_member_ids()` intent state alone.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- `cluster becomes healthy` already uses the tighter health boundary now: `wait_for_authoritative_single_primary()` returns the authoritative writable primary plus the freshly probeable member subset from that same successful poll, and the step passes that exact slice into `ensure_accepted_writes_healthy()`.
- The remaining smell is therefore no longer "healthy step uses `HaWorld::online_member_ids()`". That wrong-place selection boundary has already been flattened without introducing a new wrapper type; the existing `Vec<ClusterMember>` returned from the health poll is enough.
- Focused ultra-long re-execution on the two scenarios named in this bug now passes on the current branch:
  - `ha_rejoin_and_restart_recovery::blocked_basebackup_recovery_recovers_after_unblock`
  - `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority`
- That means the branch may already contain the intended fix, or a nearby refactor may have made the original failure stale. The next execution turn should verify that with the required repo gates before attempting more refactoring.
- If the full suite still finds a failure, the most likely remaining boundary bug is not type shape but a stale caller path that still recomputes strong write-convergence membership from world intent instead of reusing the successful healthy poll's probeable subset.

### Execution plan
1. Start with verification, not more design churn: run the required gates in repo order for this task state, `make check`, `make lint`, `make test`, and `make test-long`.
2. If every gate passes, treat this bug as already fixed by the current code. Tick the acceptance boxes, set `<passes>true</passes>`, switch the task, commit, and push without inventing extra refactors.
3. If any gate fails with this bug's symptom again, keep the current flat boundary and repair only the stale caller path. Reuse the existing `Vec<ClusterMember>` healthy-member slice from the successful health poll; do not add another public struct or duplicate selection helper.
4. If execution does need code changes, keep the strong selected-member derivation colocated with the successful health observation and thread that slice directly into the strong convergence check. Cleanup/liveness paths must stay separate from strong convergence membership.
5. Add or adjust the narrowest regression coverage only if execution reveals a real stale caller path:
   - a unit test proving the healthy poll's probeable subset is the only strong convergence input; or
   - focused ultra-long reruns for the two named scenarios if the full suite reproduces them again.
6. If full-suite execution shows the current boundary is still wrong in a deeper way than this diagnosis covers, switch this task back to `TO BE VERIFIED`, explain the exact remaining gap here, and stop immediately.

### Constraints for execution
- Do not add a new wrapper struct just to name the healthy-member slice. Reuse the existing return values and reduce code.
- Prefer deleting stale member-selection helpers over layering another forwarding function on top.
- Keep liveness checks and strong convergence checks as separate caller boundaries.
- Do not run `cargo test`; use the required `make` targets, and use focused `cargo nextest` only for local repros if a gate actually fails.

NOW EXECUTE
