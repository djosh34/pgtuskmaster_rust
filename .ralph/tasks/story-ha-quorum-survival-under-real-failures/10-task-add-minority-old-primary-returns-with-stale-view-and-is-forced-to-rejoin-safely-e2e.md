## Task: Add Minority Old Primary Returns With Stale View And Is Forced To Rejoin Safely E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add a full-partition recovery scenario where the old primary lived on the minority side, the majority moved on, and the minority old primary later reconnects with a stale view. The higher-order goal is to prove the stale old primary is fenced, rewound, or otherwise forced into a safe follower path instead of continuing to serve as primary.

**Scope:**
- Extend partition or multi-node HA E2E coverage in:
- `tests/ha/support/partition.rs`
- `tests/ha/support/multi_node.rs`
- matching test registration file
- Use a majority/minority split where the old primary is stranded on the 1-side minority and the 2-side majority elects a new primary.
- Then reconnect the minority old primary and require:
- it is no longer primary,
- it is forced through a safe demotion/rejoin path,
- the majority-elected primary remains authoritative.

**Context from research:**
- This is one of the highest-risk real-world split-brain recovery paths.
- It is not enough to prove the majority side survives; the old minority primary must also be shown to rejoin safely when connectivity returns.

**Expected outcome:**
- The suite proves that stale former primaries from the minority side cannot keep serving once they reconnect.
- Recovery behavior after split repair becomes explicit and safety-focused.

</description>

<acceptance_criteria>
- [ ] Add at least one scenario where a full 1:2 split strands the old primary on the minority side and the majority side elects a new primary before heal.
- [ ] Before reconnecting the minority side, the scenario must prove the majority primary is stable and accepts a proof write.
- [ ] After reconnecting the minority old primary, the scenario must prove it does not remain or become primary and instead enters a safe rejoin path as replica.
- [ ] The scenario must verify final proof-row convergence and no dual-primary window across the split and rejoin interval.
- [ ] Timeline artifacts must make the minority old-primary story explicit: split time, majority promotion time, minority reconnect time, safe rejoin time.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
