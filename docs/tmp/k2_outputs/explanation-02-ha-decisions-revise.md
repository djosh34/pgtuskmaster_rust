# HA Decisions and Actions

The HA worker processes inputs from pginfo, DCS, process, and config subscribers plus a poll interval. It walks each observation through a four-stage pipeline: observe, decide, lower, and apply.

## The Four-Stage Pipeline

Each HA step builds a **world snapshot** from the latest subscriber values, feeds it to the decision engine, **lowers** the chosen decision into an effect plan, publishes the **next HA state** with transition events, and finally **applies** the concrete effects.

### Observe

The observe stage protects the decision logic from transient races by snapshotting all inputs into a single, consistent world view. This isolates policy from the timing of individual subscriber updates.

### Decide

The decide stage clarifies intent. A global trust override runs before phase-specific logic, allowing blanket policy changes without touching every code path. The output is a high-level decision that names what should happen but avoids mentioning concrete mechanisms.

### Lower

The lowering pipeline separates decision selection from implementation. It maps the abstract decision onto a plan of concrete effects: Postgres actions, lease changes, switchover cleanup, replication work, and safety actions. This keeps policy free of implementation details and makes effect substitution testable.

### Apply

The apply stage dispatches the effect plan. If dispatch returns errors, the worker republishes a **faulted state** that retains the chosen phase and decision but marks the worker status as faulted. Explicit logic also skips redundant process dispatch when phase and decision are unchanged for certain waiting or recovery decisions, avoiding duplicate work.

## Tradeoffs of Publishing Intent Early

Publishing the next HA state before all side effects finish makes intent visible immediately. Observers can react to the chosen phase and decision without waiting for lengthy Postgres operations. The downside is that external watchers may see a state that has not yet been fully realized; the faulted marker signals when application failed.

## Implications for Debugging and Evolution

The clear separation between decision and effects means:

- Logs and metrics can surface the decision, the generated plan, and the dispatch result independently.
- New high-level policies require changes only in the decide stage.
- New Postgres behaviors require changes only in the lower and apply stages.
- Faulted states preserve the original decision, simplifying post-mortem analysis.

This architecture lets you reason about "what should happen" and "how it happens" in isolation, making the HA loop easier to test, extend, and operate.
