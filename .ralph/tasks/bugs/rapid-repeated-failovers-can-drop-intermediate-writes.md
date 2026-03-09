## Bug: Rapid Repeated Failovers Can Drop Intermediate Writes <status>done</status> <passes>true</passes>

<description>
The original `e2e_multi_node_repeated_leadership_changes_preserve_single_primary` scenario exposed a write-survival problem that is separate from the scenario's single-primary contract.

Observed failure chain:
- node A starts as primary
- node A fails and node B becomes primary
- a proof row written on node B after the first failover does not reach node C
- node B then fails and node C becomes primary
- the cluster later converges without the intermediate proof row, meaning the later winner did not contain the earlier acknowledged write

This needs source-level investigation and a dedicated fix. Explore the failover sequencing, candidate eligibility, and promotion safety rules first. In particular, verify whether rapid successive promotions can choose a replica that is still behind the current primary's committed writes, and whether the HA model needs an explicit freshness/catch-up requirement before a node is eligible to become the next primary.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### Current source findings
- The failing churn scenario lives in `tests/ha/support/multi_node.rs` inside `e2e_multi_node_repeated_leadership_changes_preserve_single_primary(...)`. Today it only proves there is no long-lived dual primary; it does not prove that a write acknowledged by the first successor survives the second failover.
- The HA decision engine currently builds candidate facts in `src/ha/decision.rs`. `DecisionFacts::from_world(...)` records WAL metadata from DCS member records, but leader availability and candidate eligibility only use freshness, role, SQL health, and readiness.
- Promotion attempts are emitted from `src/ha/decide.rs` in `decide_replica(...)` and `decide_candidate_leader(...)`. When there is no followable leader, both paths can return `HaDecision::AttemptLeadership` without checking whether the local replica has caught up to the most recent committed WAL position known in DCS.
- Switchover eligibility also currently ignores catch-up state: `eligible_switchover_targets(...)` and `switchover_target_is_eligible_member(...)` only require a fresh healthy ready replica record.
- DCS member records already carry the data needed for a safety gate:
  - primaries publish `write_lsn`
  - replicas publish `replay_lsn`
  - both publish `timeline`
  That means the missing behavior is a source-level eligibility rule, not a missing telemetry path.
- A stricter review changes one key assumption from the first draft: comparing only against a fresh primary record is not sufficient. After a crash, the freshest surviving WAL evidence may only remain visible on a replica record, so the promotion gate needs to use the highest fresh WAL evidence on the candidate timeline, regardless of member role.

### Implementation strategy
1. Add a dedicated regression proof for write survival across repeated failovers.
   - Keep the existing single-primary churn scenario focused on split-brain safety.
   - Add a new HA E2E scenario in `tests/ha/support/multi_node.rs` and wire it through `tests/ha_multi_node_failover.rs`.
   - Scenario shape:
     - bootstrap a 3-node cluster and wait for a stable primary,
     - create a proof table and verify the initial row reaches every node,
     - force the first failover,
     - write an intermediate proof row on the first successor,
     - deliberately trigger the second failover while the remaining replica is the only possible successor,
     - assert the final primary either already contains the intermediate row before promotion or never becomes primary until it can preserve that row,
     - finish by checking all surviving/rejoined nodes converge on the full proof set.
   - Reuse the existing phase-history and timeline-artifact helpers so a failure still leaves precise evidence.

2. Introduce an explicit promotion-safety gate derived from DCS WAL metadata.
   - Extend `src/ha/decision.rs` with a helper that computes whether the local node is promotion-safe.
   - The gate must require all of the following before the local node is eligible to take leadership:
     - local PostgreSQL is a healthy replica,
     - local `replay_lsn` is known,
     - the local timeline is known,
     - the node is on the highest fresh timeline visible in DCS that it can prove, and
     - the node is caught up to the highest fresh WAL position advertised on that same timeline, whether that evidence comes from a primary `write_lsn` or another replica `replay_lsn`.
   - Treat unknown WAL position or unknown timeline as ineligible for promotion. Safety should win over availability here.
   - Keep the rule centralized in `src/ha/decision.rs` so failover and switchover eligibility cannot drift apart.
   - Surface enough diagnostic detail from the helper for tests and operator visibility, so blocked promotion can be attributed to unknown timeline, unknown replay LSN, lower timeline, or lagging WAL.

3. Apply that gate everywhere leadership eligibility matters.
   - Update `DecisionFacts::from_world(...)` to derive a reusable promotion-safety fact for the local node, plus whatever supporting details are needed for tests and diagnostics.
   - Update `decide_replica(...)` and `decide_candidate_leader(...)` in `src/ha/decide.rs` so a lagging node no longer returns `AttemptLeadership`.
   - Update `eligible_switchover_targets(...)` / `switchover_target_is_eligible_member(...)` in `src/ha/decision.rs` so operator-initiated switchovers also reject lagging replicas. This bug was exposed by failover, but the same safety rule should protect the manual path too.
   - If the current decision vocabulary cannot express this clearly, add a dedicated wait-style decision variant instead of overloading `wait_for_dcs_trust`. The runtime is greenfield, so explicit operator-visible state is preferable to ambiguous reuse.

4. Add focused unit coverage before relying on the E2E proof.
   - In `src/ha/decision.rs` tests:
     - add cases where a fresh healthy replica is excluded from failover/switchover eligibility because its `replay_lsn` trails the freshest primary `write_lsn`,
     - add cases where no fresh primary record exists but another fresh replica still advertises a higher `replay_lsn`, and the lagging candidate remains ineligible,
     - add cases where a caught-up replica remains eligible,
     - add cases where a higher fresh timeline exists elsewhere in DCS and a lower-timeline candidate is ineligible,
     - add cases for unknown timeline / unknown LSN being treated as ineligible.
   - In `src/ha/decide.rs` tests:
     - assert a lagging replica in `Replica` phase does not emit `AttemptLeadership`,
     - assert a lagging node already in `CandidateLeader` also remains blocked,
     - assert a caught-up node can still proceed to `AttemptLeadership` and `BecomePrimary`,
     - if a new decision variant is introduced, cover its lowering/API mapping as well.

5. Update operator-facing docs during execution with the `k2-docs-loop` skill.
   - Refresh the HA decision documentation in:
     - `docs/src/reference/ha-decisions.md`
     - `docs/src/explanation/ha-decision-engine.md`
   - Document the new promotion-safety rule in terms of freshness plus WAL catch-up, and remove any stale wording that implies any healthy ready replica can always be promoted.
   - If execution adds a new wait-style decision variant, make sure the docs and any surfaced API examples describe it explicitly.

### Execution order
1. Add the unit tests that capture the missing eligibility rule.
2. Implement the shared promotion-safety helper and thread it through decision facts and switchover eligibility.
3. Update the HA phase logic so lagging replicas cannot acquire leadership.
4. Add the dedicated repeated-failover write-survival E2E scenario.
5. Update docs with `k2-docs-loop`.
6. Run the full required gates and only then mark the task as passing.

### Verification plan
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Confirm the new repeated-failover proof passes reliably and the docs reflect the final behavior with no stale election wording left behind.

NOW EXECUTE
