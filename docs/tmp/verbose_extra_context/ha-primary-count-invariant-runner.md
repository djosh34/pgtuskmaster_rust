# HA primary-count invariant runner context

This context file exists only to give K2 exhaustive raw facts about the HA test harness change completed in task `05-task-add-a-perpetual-self-reported-primary-count-invariant-runner.md`.

## Task goal

The task adds a scenario-agnostic invariant runner to the HA acceptance harness. The runner continuously queries each node individually and immediately fails the scenario unless the total number of self-reported primaries is in the allowed set `{0, 1}`.

The important intent is:

- remove split-brain / dual-primary assertions from individual feature text
- remove any older transition-history or ad hoc bookkeeping approach
- make the safety property run for the whole scenario lifetime
- count only what each queried node says about itself
- treat command failure as absence of a self-report for that node
- persist direct structured evidence of the violating sample

## Actual implementation location

The new runner lives in `tests/ha/support/invariant.rs`.

It is wired into the harness lifecycle through `tests/ha/support/world/mod.rs`.

The support module now includes `mod invariant;` in `tests/ha/support/mod.rs`.

## How the runner starts and stops

`HarnessShared::initialize()` now starts the invariant runner before cluster bootstrap continues. That means the invariant is active for the full harness lifetime after the run workspace exists, not only after a later feature step.

The runner is stopped in `HarnessShared::cleanup()` before artifact capture and compose teardown continue.

If startup of the invariant runner fails, `initialize()` performs cleanup and returns an error.

If stopping the invariant runner fails or the runner has already recorded a failure, `cleanup()` records that as a scenario cleanup failure.

## How failures surface to step execution

`HaWorld::harness()` now calls `harness.ensure_background_invariants_healthy()?` before returning the harness reference.

This means normal scenario steps and the existing polling helpers fail through the shared harness access path if the invariant runner has recorded a failure.

The runner also still causes the scenario to fail at cleanup time if the violation happens after the final step but before scenario teardown finishes.

## What is counted as a self-reported primary

The runner does not use the operator-visible authority projection in `state.ha.publication` to count primaries.

Instead it uses the local PostgreSQL role from the node-local `NodeState.pg` value returned by `pgtm status --json` through the host-side observer path:

- `PgInfoState::Primary` counts as one self-reported primary
- `PgInfoState::Replica` counts as not primary
- `PgInfoState::Unknown` counts as not primary
- a command failure for that member counts as absence, not as primary

The runner also validates that the local identity returned by `pgtm status --json` matches the member that was queried. If the queried member returns a different local identity, the runner records that as a runner error because the sample would not be trustworthy.

## Observation path and artifact behavior

The runner uses the existing host-side `PgtmObserver`.

That observer already queries each member individually by materializing a host-side observer config for the specific node, then running `pgtm --config <host-config> --json status`.

The runner transforms the full per-member observation into a smaller sample focused only on per-member self-report:

- member name
- whether it self-reported `primary`
- or whether it self-reported `replica` / `unknown`
- or whether the command failed

On violation, the runner writes a structured artifact:

- `artifacts/primary-count-invariant-violation.json`

That artifact stores:

- `observed_at_ms`
- `allowed_primary_counts`
- `primary_count`
- `members`

Each member entry stores the member name and the reduced self-report result.

The error message surfaced back to the harness mentions the structured artifact path explicitly.

## What was already true before this task

Before this task, the harness already:

- queried node state from the host with `pgtm`
- recorded timeline notes and status snapshots
- used scenario steps and polling helpers to check cluster health / authoritative primary behavior

There was already explanatory doc text in `docs/src/explanation/failure-modes.md` that said the harness samples cluster state from the host and fails if more than one primary is observed.

That existing explanation should now be made more precise:

- the invariant is perpetual / always-on for HA scenarios
- the sample is based on per-node self-reported local PostgreSQL role
- command failure is treated as missing self-report, not as inferred state
- the failure evidence is written to a dedicated structured artifact

## What should not be claimed

Please do not claim any production runtime behavior changed. This is HA test harness behavior, not daemon control-plane logic.

Please do not claim the runner uses cluster-wide inference, authority projection, or timeline-based dual-primary history. The task explicitly removes the need for those approaches.

Please do not claim the runner proves more than it does. It only counts self-reported primaries sampled from each node individually over time.

Please do not claim command failures are treated as a failure by themselves. A command failure alone is treated as absence for that member unless some other runner/internal error occurs.
