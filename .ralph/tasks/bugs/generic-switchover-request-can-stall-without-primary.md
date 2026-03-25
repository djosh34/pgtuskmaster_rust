## Bug: Switchover requests can stay pending until manual clear and generic requests can strand the cluster without a primary <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
The switchover lifecycle is not reliably self-clearing in the shipped Docker walkthrough.

Observed on 2026-03-24:

- Targeted switchover to `node-b` succeeded and leadership moved, but `status --watch` kept showing `switchover: pending -> node-b` until the operator manually ran `pgtm switchover clear`.
- The generic `pgtm switchover request` flow was worse: it left the shipped Docker cluster in a degraded state with no authoritative primary and a still-pending `switchover: auto` marker until the operator manually cleared it.

Generic reproduction details:

- `pgtm -c docker/pgtm.toml switchover request` returned `accepted=true` while `node-a` was primary.
- `timeout 20 pgtm -c docker/pgtm.toml status --watch` showed repeated degraded states with `warning: seed node does not currently project an authoritative primary` and `switchover: pending -> auto`.
- Alternate seeds confirmed this was not just node-a blindness:
  - `docs/examples/docker-cluster-node-b.toml` showed node-b oscillating between `replica` and `unknown` with an active promote job.
  - `docs/examples/docker-cluster-node-c.toml` reported `no_quorum` and no DCS member slots.
- `pgtm -c docker/pgtm.toml switchover clear` immediately allowed the cluster to converge back to a healthy state with `node-b` primary.

That means the documented switchover path is not yet truthful for the shipped Docker stack. Operators can end up with a lingering pending marker after an otherwise successful targeted handoff, and the generic form can enter a no-primary state that requires manual clear to recover.
</description>

<mandatory_red_green_tdd>
Use Red-Green TDD to solve the problem.
You must make ONE test, and then make ONE test green at the time.

Then verify if bug still holds. If yes, create new Red test, and continue with Red-Green TDD until it does work.
</mandatory_red_green_tdd>

<acceptance_criteria>
- [x] I created a Red unit and/or integration test that captures the bug
- [x] I made the test green by fixing
- [x] I manually verified the bug, and created a new Red test if not working still
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>
- [x] Finish propagating the new switchover ADT split across the codebase:
  `SwitchoverTarget` is operator input only; persisted quorum state is `SwitchoverState::Pending(SwitchoverRequest { requested_from: LeaseEpoch, target })`; remove remaining call sites that still treat persisted switchover state as a raw target enum.
- [x] Add one Red HA/decision test that proves a targeted switchover clears itself after the requested replica becomes leader instead of leaving `switchover: pending -> <target>` behind.
- [x] Make that test green by updating HA decision/reconcile logic so only the originating leader executes a pending request, while the successor that completes the handoff clears the request on the next tick.
- [x] Manually re-verify the targeted Docker walkthrough; if the pending marker still lingers anywhere, add the next Red test before changing more behavior.
- [x] Add one Red HA/integration test for the generic request that proves the successor leader does not immediately reinterpret the old request and demote again.
- [x] Make that test green by centralizing switchover lifecycle helpers on the new typed request boundary and teaching the post-takeover leader to clear stale requests instead of executing them twice.
- [x] Update status rendering, API/CLI serialization, docs/examples, and fixtures/tests that still pattern-match the old enum so they render pending target data from `SwitchoverRequest.target`.
- [x] Run the required validation gates in repo order only after the design still looks correct: `make check`, `make lint`, `make test`, `make test-long`. If execution shows this ADT split is still wrong, switch this task back to `TO BE VERIFIED`, explain the exact mismatch here, and stop immediately.
</plan>
