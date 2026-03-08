# How HA turns shared state into actions

The [HA state machine](../reference/ha-state-machine.md) is easier to understand if you stop thinking about it as "the failover code" and instead treat it as a pipeline: observe, decide, lower, apply.

That separation is not ceremony. It is the reason HA policy can stay readable while the runtime still performs concrete work such as process actions, lease changes, switchover cleanup, replication work, and safety actions.

## Observe first

Each HA step begins by building a world snapshot from the latest pginfo, DCS, process, and config subscriber values, plus the worker's poll cadence. That snapshot gives one frozen input set for the rest of the step.

The point is not perfect global synchrony. The point is that one HA decision should be derived from one coherent view, instead of racing against inputs that change halfway through the calculation.

## Decide intent before mechanics

The decision stage chooses the next phase and the next HA decision. It also starts with a global trust override, so coordination safety can short-circuit phase-specific logic before the worker reasons about normal transitions.

This keeps policy legible. The decision layer answers "what should the cluster try to do next?" without yet committing to the mechanics of doing it.

## Lower intent into effects

The lowering stage translates the chosen decision into an effect plan. That plan separates the action buckets that downstream code knows how to dispatch: Postgres work, lease work, switchover cleanup, replication work, and safety work.

This is where the architecture pays off. The state machine does not need to know subprocess details, filesystem details, or DCS write details at the same level where it reasons about leadership and recovery. Decision logic stays about cluster policy; execution logic stays about execution.

## Publish, then apply

After deciding and lowering, the worker publishes the next HA state, emits transition events, and then applies the effect plan unless the worker can safely skip redundant dispatch for certain waiting or recovery decisions.

Publishing before all side effects finish is a deliberate tradeoff. It means observers can see the chosen intent immediately, and it preserves the record of what HA believed should happen next. If dispatch fails, HA republishes a faulted state that keeps the chosen phase and decision but marks the worker as faulted.

The cost is that published intent is not the same thing as fully completed execution. The system accepts that distinction because it is easier to debug and reason about than trying to hide partial progress behind a fake all-or-nothing abstraction.

## Why this split matters

This pipeline makes the runtime easier to extend and easier to inspect:

- New policy work mostly belongs in the decision layer.
- New execution behavior mostly belongs in lowering or apply paths.
- Fault analysis can ask separate questions: what inputs were observed, what decision was selected, what effect plan was built, and what dispatch failed.

The result is an HA loop that can evolve without collapsing policy, coordination, and subprocess execution into one opaque control path.
