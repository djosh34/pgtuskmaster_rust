# HA invariant runners context

This note exists only as raw context for the docs drafting workflow. It summarizes the current HA acceptance harness behavior after the accepted-write convergence task.

## Current background invariants

The HA harness now runs two background invariant runners for every HA scenario:

1. `PrimaryCountInvariantRunner`
2. `WriteConvergenceInvariantRunner`

They both live in `tests/ha/support/invariant.rs`.

## Primary-count invariant

The primary-count runner starts during `HarnessShared::initialize()` before cluster bootstrap begins.

It continuously samples every member through the host-side `pgtm status --json` observation surface.

It counts only each member's local self-report from `NodeState.pg`:

- `PgInfoState::Primary` counts as primary
- `PgInfoState::Replica` counts as not-primary
- `PgInfoState::Unknown` counts as not-primary
- a command failure to a member is recorded as `command_failed` and does not count as a primary

The only allowed total self-reported primary counts are `{0, 1}`.

If the sampled total is outside `{0, 1}`, the runner immediately persists `artifacts/primary-count-invariant-violation.json` and the scenario fails.

## Write-convergence invariant

The write-convergence runner starts after bootstrap succeeds. It is also wired from `HarnessShared::initialize()`, but only after `bootstrap_cluster().await` returns successfully.

The write-convergence runner is intended to enforce this rule:

- every write that the cluster accepted as committed must eventually be visible on all nodes

The runner does not treat every successful direct SQL write on every node as "cluster accepted". That would be too broad during failure scenarios. Instead it uses the current HA authority projection from `NodeState.ha.publication` to decide which member is the authoritative primary for the current sample.

## Activation behavior

Before the write-convergence runner starts issuing writes, it waits for the invariant table to become visible on all members.

The table name is:

- `public.ha_write_convergence_invariant`

The runner first issues `CREATE TABLE IF NOT EXISTS ...` against the member Postgres endpoints until one target accepts the DDL.

After that it waits until every member reports that `to_regclass('public.ha_write_convergence_invariant')` resolves to the table name. This means the runner does not start counting writes during the initial cluster bootstrap window before the relation exists everywhere.

If scenario cleanup starts before the table becomes visible on all members, the runner exits cleanly instead of turning cleanup into a failure.

## Accepted writes versus rejected writes

Each loop iteration can record both an accepted-write probe and a rejected-write probe.

Accepted-write probe:

- the runner observes the cluster state
- it extracts the single authoritative primary from `PublicationState::Projected(AuthorityProjection::Primary(...))`
- it writes only through that authoritative primary's Postgres endpoint
- if that SQL succeeds, the write is recorded as an accepted write
- if that SQL fails, the attempt is recorded as a rejection, not an accepted write

Rejected-write probe:

- the runner round-robins over non-authoritative members
- if a target self-reports primary but is not the authoritative primary, the runner records a rejection without counting that target as cluster-accepted
- if a non-authoritative target accepts the SQL write anyway, the runner treats that as a runner failure because a non-authoritative target unexpectedly accepted the invariant write

This split is important. The runner is modeling "cluster accepted as committed", not merely "some node accepted a direct SQL connection".

## Convergence tracking

Accepted writes are tracked until they are visible on every member.

The runner persists and updates:

- `artifacts/write-convergence-invariant-events.jsonl`
- `artifacts/write-convergence-invariant-summary.json`
- `artifacts/write-convergence-invariant-violation.json` on timeout/failure

The summary includes:

- accepted count
- rejected count
- converged count
- pending accepted writes
- per-member visibility state for each pending write

Per-member visibility states are:

- `visible`
- `missing`
- `query_failed`

The runner treats `relation does not exist` as `missing` instead of a transport failure. Transport-level SQL failures such as connection refusal remain `query_failed`.

## Convergence timeout

The harness derives a dedicated convergence window in `tests/ha/support/timeouts/mod.rs`.

The field is:

- `TimeoutModel::write_convergence_deadline`

It is derived as:

- `failover_deadline + recovery_deadline`

This window is intentionally longer than a single poll or failover loop because accepted writes may need both a failover interval and a node recovery interval before they can become visible on every member again.

## Cleanup and failure surfacing

`HarnessShared::ensure_background_invariants_healthy()` checks both runners.

That means normal HA steps fail through the shared harness access path if either background invariant has already recorded a failure.

During cleanup:

- the primary-count runner is stopped
- the write-convergence runner is stopped
- artifact capture runs afterward

If either runner fails during stop or has already persisted a failure, cleanup reports that failure.

## Documentation implications

The failure-modes explanation page currently mentions only the primary-count invariant.

It should now explain that the HA harness enforces two always-on safety checks:

1. self-reported primary count must stay in `{0, 1}`
2. writes accepted through the authoritative primary must eventually converge to all members

It should also mention the concrete write-convergence artifacts and the fact that accepted writes are defined by authoritative-primary routing, not by arbitrary direct SQL success on any node.
