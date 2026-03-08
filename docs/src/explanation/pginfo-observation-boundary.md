# Why pginfo observes PostgreSQL instead of controlling it

The [pginfo reference](../reference/pginfo.md) describes what the worker polls and publishes. The key architectural point is simpler: pginfo is a sensor, not a controller.

That boundary matters because the runtime needs one component whose job is to report what PostgreSQL looks like right now, even when that report is bad news.

## Observation is still useful when it is degraded

pginfo polls PostgreSQL through `poll_once`. On success it publishes a running, healthy view. On failure it emits a warning and publishes a running state whose SQL status is `Unreachable`.

That can look odd at first. Why not fail hard? Because the system still learns something valuable from degraded observation. An unreachable result is a fact for the rest of the runtime to weigh, not a command that already decided what recovery must happen.

## Why control lives elsewhere

HA consumes pginfo state alongside DCS, process, and config state. The process worker owns subprocess execution. That split keeps responsibilities clean:

- pginfo says what PostgreSQL appears to be doing
- HA says what cluster policy thinks should happen next
- process work says how local commands actually run

If pginfo also controlled PostgreSQL, one layer would be mixing sensing, interpretation, and execution. The current design keeps those concerns separable.

## The tradeoff

Polling adds lag and can misread transient conditions as momentary failures. The runtime accepts that cost because a dedicated observation layer is easier to reason about than a controller that both measures and reacts in one step.

Real tests for pginfo focus on transitions and observed state, which fits that role. The worker's value is not that it acts. Its value is that it reports.
